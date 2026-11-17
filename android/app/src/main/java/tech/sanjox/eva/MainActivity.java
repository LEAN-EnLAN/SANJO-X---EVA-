package tech.sanjox.eva;

import android.Manifest;
import android.annotation.SuppressLint;
import android.app.Activity;
import android.app.AlertDialog;
import android.content.ActivityNotFoundException;
import android.content.ClipData;
import android.content.Intent;
import android.content.pm.ActivityInfo;
import android.content.pm.ApplicationInfo;
import android.content.pm.PackageManager;
import android.graphics.Bitmap;
import android.graphics.Color;
import android.net.ConnectivityManager;
import android.net.NetworkCapabilities;
import android.net.Uri;
import android.net.http.SslCertificate;
import android.net.http.SslError;
import android.os.Build;
import android.os.Bundle;
import android.provider.MediaStore;
import android.view.Display;
import android.view.View;
import android.view.WindowManager;
import android.webkit.PermissionRequest;
import android.webkit.RenderProcessGoneDetail;
import android.webkit.SslErrorHandler;
import android.webkit.ValueCallback;
import android.webkit.WebChromeClient;
import android.webkit.WebResourceError;
import android.webkit.WebResourceRequest;
import android.webkit.WebResourceResponse;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.widget.TextView;
import android.widget.Toast;
import android.window.OnBackInvokedCallback;
import android.window.OnBackInvokedDispatcher;

import androidx.webkit.WebViewAssetLoader;

import java.io.ByteArrayInputStream;
import java.io.File;
import java.security.cert.CertificateEncodingException;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

/**
 * Full-screen WebView on one of three {@link AppMode}s: the hosted demo (live Eva chat), the demo bundled in the APK
 * (served by WebViewAssetLoader, works with no network) or a ship hub on the LAN. Native pieces: mic/camera grants
 * for the hub origin only, file chooser with camera capture, trust-on-first-use certificate pinning for the hub
 * only, a native error screen, keep-screen-on, back navigation through WebView history.
 */
public class MainActivity extends Activity {
    private static final int REQ_MEDIA = 10;
    private static final int REQ_CAMERA_FOR_CAPTURE = 11;
    private static final int REQ_FILE = 20;

    private HubPrefs prefs;
    /** The mode the user chose (remembered) and the one actually showing (online falls back to offline). */
    private AppMode chosenMode;
    private AppMode mode;
    /** The hub, in {@link AppMode#HUB} only; null in the demo modes. */
    private HubConfig config;
    private WebViewAssetLoader assetLoader;
    private WebView web;
    private View errorView;
    private TextView errorTitle;
    private TextView errorDetail;
    private TextView errorCode;

    /** Set when the main frame failed in the current load; cleared on the next page start. */
    private boolean mainFrameFailed;
    /** Set when this app cancelled a certificate for the hub and already shows why; the WebView's follow-up
     *  network error for that load must not replace the explanation. */
    private boolean certErrorShown;

    private PermissionRequest pendingPermission;
    private ValueCallback<Uri[]> fileCallback;
    private WebChromeClient.FileChooserParams pendingFileParams;
    private Uri captureUri;
    private File captureFile;

    /** Certificate errors waiting for the user's Trust / Don't trust answer, all for the same fingerprint. */
    private final List<SslErrorHandler> pendingSsl = new ArrayList<>();
    private String pendingSslFingerprint;
    private AlertDialog trustDialog;

    private Object backCallback;
    /** The landing's black: window, WebView and system bars, so nothing flashes around the page. */
    static final int BACKGROUND = 0xFF000000;
    /** Whether the Eva status toast was shown for this load of the hub. */
    private boolean statusShown;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        prefs = new HubPrefs(this);
        if (!readMode()) {
            startActivity(new Intent(this, SetupActivity.class));
            finish();
            return;
        }
        getWindow().setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(BACKGROUND));
        applyOrientation();
        preferHighRefreshRate();
        if (Build.VERSION.SDK_INT >= 29) getWindow().setNavigationBarContrastEnforced(false);
        setContentView(R.layout.activity_main);
        Insets.applySystemBarPadding(findViewById(R.id.root));

        web = findViewById(R.id.web);
        errorView = findViewById(R.id.error);
        errorTitle = findViewById(R.id.error_title);
        errorDetail = findViewById(R.id.error_detail);
        errorCode = findViewById(R.id.error_code);
        findViewById(R.id.retry).setOnClickListener(v -> load());
        findViewById(R.id.change_hub).setOnClickListener(v -> openSettings());

        CaptureProvider.prune(this);
        setUpWebView();
        setUpBack();
        announceFallback();
        load();
    }

    @Override
    protected void onNewIntent(Intent intent) {
        super.onNewIntent(intent);
        setIntent(intent);
    }

    @Override
    protected void onResume() {
        super.onResume();
        if (web == null) return;
        web.onResume();
        if (prefs.keepScreenOn()) {
            getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
        } else {
            getWindow().clearFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
        }
        // Back from settings: reload only if the mode, the hub or the seat changed.
        AppMode wasChosen = chosenMode, wasMode = mode;
        HubConfig wasConfig = config;
        if (!readMode()) {
            openSettings();
        } else if (chosenMode != wasChosen || (config != null && !config.equals(wasConfig))) {
            if (wasConfig == null || config == null || !config.host.equals(wasConfig.host)) web.clearSslPreferences();
            applyOrientation();
            web.clearHistory();
            announceFallback();
            load();
        } else {
            mode = wasMode; // same choice: keep what is showing (an online demo that fell back stays on the bundle)
        }
    }

    @Override
    protected void onPause() {
        if (web != null) web.onPause(); // pauses media and animations; JavaScript (live alerts) keeps running
        super.onPause();
    }

    @Override
    protected void onDestroy() {
        if (Build.VERSION.SDK_INT >= 33 && backCallback != null) {
            getOnBackInvokedDispatcher().unregisterOnBackInvokedCallback((OnBackInvokedCallback) backCallback);
        }
        cancelPendingSsl();
        if (web != null) {
            web.destroy();
            web = null;
        }
        super.onDestroy();
    }

    /**
     * Reads the remembered mode (and the hub for {@link AppMode#HUB}). The online demo opens the bundle when there is
     * no network. False when there is nothing to open yet (first run, or a hub that no longer parses).
     */
    private boolean readMode() {
        AppMode chosen = prefs.mode();
        HubConfig hub = chosen == AppMode.HUB ? prefs.load() : null;
        if (chosen == null || (chosen == AppMode.HUB && hub == null)) return false;
        chosenMode = chosen;
        config = hub;
        mode = AppMode.effective(chosen, isOnline());
        return true;
    }

    private void announceFallback() {
        if (mode != chosenMode) Toast.makeText(this, R.string.offline_fallback_toast, Toast.LENGTH_LONG).show();
    }

    /** The current mode (for tests and the settings screen). */
    AppMode mode() {
        return mode;
    }

    /** Whether the current mode keeps this URL in the WebView: the hub's origin, or the demo's allow-list. */
    private boolean inApp(String url) {
        return mode == AppMode.HUB ? config.isSameOrigin(url) : mode.allowsInWebView(url);
    }

    private String startUrl() {
        return mode == AppMode.HUB ? config.startUrl() : mode.demoStartUrl();
    }

    /** The hosted demo did not load (no route, DNS, server down): open the bundled copy and say so. */
    private void fallBackToOffline() {
        mode = AppMode.OFFLINE;
        Toast.makeText(this, R.string.offline_fallback_toast, Toast.LENGTH_LONG).show();
        web.clearHistory();
        load();
    }

    // ---------------------------------------------------------------- WebView

    // JavaScript is the app: Eva is an htmx PWA. Only the configured hub origin is loaded in the WebView
    // (other links open in the browser) and no JavaScript interface is exposed.
    @SuppressLint("SetJavaScriptEnabled")
    private void setUpWebView() {
        boolean debuggable = (getApplicationInfo().flags & ApplicationInfo.FLAG_DEBUGGABLE) != 0;
        WebView.setWebContentsDebuggingEnabled(debuggable);
        WebSettings s = web.getSettings();
        s.setJavaScriptEnabled(true);
        s.setDomStorageEnabled(true);
        s.setMediaPlaybackRequiresUserGesture(false);
        s.setMixedContentMode(WebSettings.MIXED_CONTENT_NEVER_ALLOW);
        s.setAllowFileAccess(false);
        s.setAllowContentAccess(true); // uploads of content:// photos picked or captured
        s.setSupportMultipleWindows(false);
        s.setUserAgentString(s.getUserAgentString() + " EvaAndroid/0.3.6");
        // The demo replay is plain static pages: no file:// or content:// beyond uploads, and its player frames and
        // fetches stay same-origin (the asset loader serves the bundle on an https origin, not file://).
        s.setAllowFileAccessFromFileURLs(false);
        s.setAllowUniversalAccessFromFileURLs(false);
        // Smoothness (APP-POLISH §8): the page owns overscroll (pull to refresh) and its own type scale; the
        // WebView is always full screen, so pre-rastering off-screen tiles avoids checkerboarding on fast scroll.
        // The default hardware layer is kept (no setLayerType anywhere).
        s.setOffscreenPreRaster(true);
        s.setTextZoom(100);
        web.setOverScrollMode(View.OVER_SCROLL_NEVER);
        web.setVerticalScrollBarEnabled(false);
        web.setHorizontalScrollBarEnabled(false);
        web.setRendererPriorityPolicy(WebView.RENDERER_PRIORITY_IMPORTANT, false);
        web.setBackgroundColor(BACKGROUND);
        assetLoader = new WebViewAssetLoader.Builder()
                .setDomain(AppMode.ASSET_HOST)
                .addPathHandler(AppMode.ASSET_PREFIX, new WebViewAssetLoader.AssetsPathHandler(this))
                .build();
        web.setWebViewClient(new HubClient());
        web.setWebChromeClient(new HubChrome());
        web.setDownloadListener((url, ua, cd, mime, len) -> openExternally(url));
        // Long-press on the page (not on a link, an image or a field) opens the menu: settings, reload, leave.
        web.setOnLongClickListener(v -> {
            WebView.HitTestResult r = web.getHitTestResult();
            if (r != null && !menuOnLongPress(r.getType())) return false;
            showMenu(false);
            return true;
        });
        // Service workers are on by default in WebView; Eva's /sw.js registers itself when the origin is trusted.
    }

    /** A long-press on a link, an image or a field keeps the WebView's own behaviour; anywhere else opens the menu. */
    static boolean menuOnLongPress(int hitType) {
        switch (hitType) {
            case WebView.HitTestResult.SRC_ANCHOR_TYPE:
            case WebView.HitTestResult.SRC_IMAGE_ANCHOR_TYPE:
            case WebView.HitTestResult.IMAGE_TYPE:
            case WebView.HitTestResult.EDIT_TEXT_TYPE:
            case WebView.HitTestResult.PHONE_TYPE:
            case WebView.HitTestResult.EMAIL_TYPE:
            case WebView.HitTestResult.GEO_TYPE:
                return false;
            default:
                return true;
        }
    }

    /** Wristband (landscape) locks landscape; everything else follows the phone (auto-rotate flips the PWA). */
    private void applyOrientation() {
        setRequestedOrientation(mode == AppMode.HUB && config != null && config.seat == Seat.WRIST
                ? ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE
                : ActivityInfo.SCREEN_ORIENTATION_FULL_USER);
    }

    /** 90/120 Hz phones run the springs at their native rate (the spring loop is time-based). */
    private void preferHighRefreshRate() {
        Display d = getWindowManager().getDefaultDisplay();
        if (d == null) return;
        Display.Mode cur = d.getMode(), best = cur;
        for (Display.Mode m : d.getSupportedModes()) {
            if (m.getPhysicalWidth() == cur.getPhysicalWidth() && m.getPhysicalHeight() == cur.getPhysicalHeight()
                    && m.getRefreshRate() > best.getRefreshRate()) best = m;
        }
        if (best.getModeId() != cur.getModeId()) {
            WindowManager.LayoutParams lp = getWindow().getAttributes();
            lp.preferredDisplayModeId = best.getModeId();
            getWindow().setAttributes(lp);
        }
    }

    /** After the hub answers once: ask it whether Eva is online and say so for 3 seconds (same origin, same pin). */
    void showEvaStatus() {
        if (statusShown || web == null || mode != AppMode.HUB) return;
        statusShown = true;
        web.evaluateJavascript(EvaStatus.FETCH_JS, value -> {
            String msg = EvaStatus.toastFor(EvaStatus.unquote(value));
            if (msg != null) Toast.makeText(this, msg, Toast.LENGTH_LONG).show();
        });
    }

    private void load() {
        statusShown = false;
        hideError();
        mainFrameFailed = false;
        certErrorShown = false;
        web.loadUrl(startUrl());
    }

    private void openSettings() {
        startActivity(new Intent(this, SetupActivity.class));
    }

    private void openExternally(String url) {
        try {
            startActivity(new Intent(Intent.ACTION_VIEW, Uri.parse(url)));
        } catch (ActivityNotFoundException e) {
            Toast.makeText(this, url, Toast.LENGTH_LONG).show();
        }
    }

    private void showCertError(String title, String detail, String code) {
        certErrorShown = true;
        showError(title, detail, code);
    }

    private void showError(String title, String detail, String code) {
        mainFrameFailed = true;
        errorTitle.setText(title);
        errorDetail.setText(detail);
        errorCode.setText(code == null ? "" : code);
        errorCode.setVisibility(code == null ? View.GONE : View.VISIBLE);
        errorView.setVisibility(View.VISIBLE);
        web.setVisibility(View.INVISIBLE);
    }

    private void hideError() {
        errorView.setVisibility(View.GONE);
        web.setVisibility(View.VISIBLE);
    }

    private boolean isOnline() {
        ConnectivityManager cm = getSystemService(ConnectivityManager.class);
        if (cm == null) return true;
        NetworkCapabilities nc = cm.getNetworkCapabilities(cm.getActiveNetwork());
        return nc != null;
    }

    private void showUnreachable(String description, int code) {
        if (!isOnline()) {
            showError(getString(R.string.error_offline_title),
                    "This phone has no network. Join the same Wi-Fi (or the laptop's hotspot) as the ship hub, then Retry.",
                    config.origin());
            return;
        }
        String hint = config.isHttps()
                ? "Check that the hub is running with --tls (eva demo --tls) and that this phone is on the same Wi-Fi. If the hub was started without --tls, switch to http in Settings."
                : "Check that the hub is running (eva demo) and that this phone is on the same Wi-Fi. If the hub was started with --tls, switch to https in Settings.";
        showError(getString(R.string.error_unreachable_title),
                "Eva could not open " + config.address() + ".\n" + hint,
                config.origin() + " · " + description + " (" + code + ")");
    }

    /** The hub's origin (or the demo's allow-list) stays in the WebView; other links open in the browser. */
    private final class HubClient extends WebViewClient {
        @Override
        public boolean shouldOverrideUrlLoading(WebView view, WebResourceRequest request) {
            String url = request.getUrl().toString();
            if (inApp(url)) return false;
            String scheme = request.getUrl().getScheme();
            if ("http".equalsIgnoreCase(scheme) || "https".equalsIgnoreCase(scheme)) {
                if (!HubConfig.isCleartextAllowed(url)) return true; // blocked: http to a public host
                openExternally(url);
                return true;
            }
            if ("tel".equalsIgnoreCase(scheme) || "mailto".equalsIgnoreCase(scheme)) {
                openExternally(url);
            }
            return true; // anything else (intent:, javascript:, file:) is dropped
        }

        @Override
        public WebResourceResponse shouldInterceptRequest(WebView view, WebResourceRequest request) {
            String url = request.getUrl().toString();
            if (AppMode.ASSET_HOST.equals(HubConfig.hostOf(url))) {
                // the bundled replay: only in the offline mode, only from the APK's assets
                WebResourceResponse r = mode == AppMode.OFFLINE && assetLoader != null
                        ? assetLoader.shouldInterceptRequest(request.getUrl()) : null;
                return r != null ? r : new WebResourceResponse("text/plain", "utf-8", 404, "Not found",
                        Collections.emptyMap(), new ByteArrayInputStream(new byte[0]));
            }
            if (HubConfig.isCleartextAllowed(url)) return null;
            return new WebResourceResponse("text/plain", "utf-8", 403, "Cleartext blocked",
                    Collections.emptyMap(), new ByteArrayInputStream(new byte[0]));
        }

        @Override
        public void onPageStarted(WebView view, String url, Bitmap favicon) {
            if (inApp(url)) mainFrameFailed = false;
        }

        @Override
        public void onPageFinished(WebView view, String url) {
            if (!mainFrameFailed && inApp(url)) {
                hideError();
                showEvaStatus();
            }
        }

        @Override
        public void onReceivedError(WebView view, WebResourceRequest request, WebResourceError error) {
            if (!request.isForMainFrame()) return;
            if (certErrorShown) return; // the follow-up of our own certificate cancel: keep its explanation
            if (mode == AppMode.ONLINE) {
                fallBackToOffline();
                return;
            }
            if (mode == AppMode.OFFLINE) {
                showError(getString(R.string.error_demo_title), getString(R.string.error_demo_detail),
                        String.valueOf(error.getDescription()) + " (" + error.getErrorCode() + ")");
                return;
            }
            showUnreachable(String.valueOf(error.getDescription()), error.getErrorCode());
        }

        @Override
        public void onReceivedSslError(WebView view, SslErrorHandler handler, SslError error) {
            handleSslError(handler, error);
        }

        @Override
        public boolean onRenderProcessGone(WebView view, RenderProcessGoneDetail detail) {
            // The renderer crashed or was killed for memory: rebuild the activity instead of crashing the app.
            if (web != null) {
                ((android.view.ViewGroup) web.getParent()).removeView(web);
                web.destroy();
                web = null;
            }
            recreate();
            return true;
        }
    }

    // ---------------------------------------------------------------- TLS: trust on first use

    private void handleSslError(SslErrorHandler handler, SslError error) {
        if (!mode.allowsTofu()) {
            // Demo modes: the hosted demo has a public CA certificate, so the system already verified it. Any error
            // here is a real failure (or an interception): cancel, never prompt, never pin.
            handler.cancel();
            if (mode == AppMode.ONLINE && AppMode.isOnlineDemoHost(error.getUrl())) {
                showCertError(getString(R.string.error_cert_invalid_title),
                        "The demo site's certificate did not verify (" + describe(error) + "). Check the phone's date and "
                                + "time, or open the Offline demo in settings.", AppMode.ONLINE_HOST);
            }
            return;
        }
        String errorHost = HubConfig.hostOf(error.getUrl());
        String fp = Fingerprints.sha256(derOf(error.getCertificate()));
        boolean onlyTrustErrors = !error.hasError(SslError.SSL_DATE_INVALID)
                && !error.hasError(SslError.SSL_EXPIRED)
                && !error.hasError(SslError.SSL_NOTYETVALID)
                && !error.hasError(SslError.SSL_INVALID);
        PinPolicy.Decision d = PinPolicy.decide(config.host, errorHost, onlyTrustErrors, prefs.pin(config.host), fp);
        boolean forHub = config.host.equals(errorHost);
        switch (d) {
            case PROCEED:
                handler.proceed();
                return;
            case ASK_USER:
                askTrust(handler, fp);
                return;
            case REJECT_CHANGED:
                handler.cancel();
                showCertError(getString(R.string.error_cert_changed_title),
                        "The hub at " + config.host + " presented a different certificate from the one you trusted. "
                                + "Eva did not connect.\n\nIf the hub's certificate was recreated (new data folder, new IP), "
                                + "open Settings, tap Forget next to the trusted certificate, and connect again. "
                                + "Otherwise, someone else may be answering at this address.",
                        "pinned  " + prefs.pin(config.host) + "\nnow     " + fp);
                return;
            case REJECT_INVALID:
                handler.cancel();
                if (forHub) {
                    showCertError(getString(R.string.error_cert_invalid_title),
                            "The hub's certificate is expired, not yet valid or malformed. Check the phone's date and time, "
                                    + "or recreate the hub certificate (delete data/certs/ and restart with --tls).",
                            describe(error));
                }
                return;
            case REJECT_OTHER_HOST:
            default:
                handler.cancel(); // never prompt for, or proceed to, anything but the configured hub
        }
    }

    private static String describe(SslError e) {
        StringBuilder sb = new StringBuilder();
        if (e.hasError(SslError.SSL_UNTRUSTED)) sb.append("untrusted ");
        if (e.hasError(SslError.SSL_IDMISMATCH)) sb.append("name-mismatch ");
        if (e.hasError(SslError.SSL_EXPIRED)) sb.append("expired ");
        if (e.hasError(SslError.SSL_NOTYETVALID)) sb.append("not-yet-valid ");
        if (e.hasError(SslError.SSL_DATE_INVALID)) sb.append("date-invalid ");
        if (e.hasError(SslError.SSL_INVALID)) sb.append("invalid ");
        return sb.toString().trim();
    }

    /** DER bytes of the presented certificate (public API on 29+, the saved-state bundle before). */
    static byte[] derOf(SslCertificate cert) {
        if (cert == null) return null;
        try {
            if (Build.VERSION.SDK_INT >= 29) {
                java.security.cert.X509Certificate x = cert.getX509Certificate();
                return x == null ? null : x.getEncoded();
            }
            Bundle b = SslCertificate.saveState(cert);
            return b == null ? null : b.getByteArray("x509-certificate");
        } catch (CertificateEncodingException e) {
            return null;
        }
    }

    private void askTrust(SslErrorHandler handler, String fp) {
        if (trustDialog != null && trustDialog.isShowing()) {
            if (fp.equals(pendingSslFingerprint)) {
                pendingSsl.add(handler);
            } else {
                handler.cancel();
            }
            return;
        }
        pendingSsl.clear();
        pendingSsl.add(handler);
        pendingSslFingerprint = fp;
        String msg = "The hub at " + config.host + " uses its own certificate (eva --tls). Trust it only if this "
                + "fingerprint matches the one on the hub laptop:\n\n"
                + "openssl x509 -noout -fingerprint -sha256 -in data/certs/eva-cert.pem\n\n"
                + "SHA-256\n" + Fingerprints.pretty(fp) + "\n\n"
                + "Eva will remember it for " + config.host + " and refuse any other certificate there.";
        trustDialog = new AlertDialog.Builder(this, android.R.style.Theme_Material_Dialog_Alert)
                .setTitle(R.string.trust_title)
                .setMessage(msg)
                .setCancelable(false)
                .setPositiveButton(R.string.trust_button, (dlg, w) -> {
                    prefs.savePin(config.host, pendingSslFingerprint);
                    for (SslErrorHandler h : pendingSsl) h.proceed();
                    pendingSsl.clear();
                })
                .setNegativeButton(R.string.dont_trust_button, (dlg, w) -> {
                    cancelPendingSsl();
                    showCertError(getString(R.string.error_cert_rejected_title),
                            "Eva did not connect to " + config.host + ". Retry to see the fingerprint again, or use "
                                    + "Settings to pick another address or plain http.",
                            "SHA-256 " + fp);
                })
                .create();
        trustDialog.show();
        TextView tv = trustDialog.findViewById(android.R.id.message);
        if (tv != null) {
            tv.setTextIsSelectable(true);
            tv.setTypeface(android.graphics.Typeface.MONOSPACE);
            tv.setTextSize(13);
        }
    }

    private void cancelPendingSsl() {
        for (SslErrorHandler h : pendingSsl) h.cancel();
        pendingSsl.clear();
        if (trustDialog != null && trustDialog.isShowing()) trustDialog.dismiss();
    }

    // ---------------------------------------------------------------- mic, camera, file chooser

    private final class HubChrome extends WebChromeClient {
        @Override
        public void onPermissionRequest(PermissionRequest request) {
            String[] wanted = allowedMedia(request);
            if (wanted.length == 0) {
                request.deny();
                return;
            }
            List<String> missing = new ArrayList<>();
            for (String p : MediaPolicy.androidPermissionsFor(wanted)) {
                if (checkSelfPermission(p) != PackageManager.PERMISSION_GRANTED) missing.add(p);
            }
            if (missing.isEmpty()) {
                grantOrDeny(request);
                return;
            }
            if (pendingPermission != null) pendingPermission.deny();
            pendingPermission = request;
            requestPermissions(missing.toArray(new String[0]), REQ_MEDIA);
        }

        @Override
        public void onPermissionRequestCanceled(PermissionRequest request) {
            if (pendingPermission == request) pendingPermission = null;
        }

        @Override
        public boolean onShowFileChooser(WebView view, ValueCallback<Uri[]> callback, FileChooserParams params) {
            if (fileCallback != null) fileCallback.onReceiveValue(null);
            fileCallback = callback;
            pendingFileParams = params;
            if (checkSelfPermission(Manifest.permission.CAMERA) != PackageManager.PERMISSION_GRANTED) {
                // CAMERA is declared in the manifest, so camera apps refuse capture intents until it is granted.
                requestPermissions(new String[] {Manifest.permission.CAMERA}, REQ_CAMERA_FOR_CAPTURE);
            } else {
                launchChooser(true);
            }
            return true;
        }
    }

    /**
     * What this page may ask for: mic and camera for the ship hub's origin; the mic only for the demo's own origin
     * (Vercel online, the asset host offline); nothing for anyone else.
     */
    private String[] allowedMedia(PermissionRequest request) {
        String origin = request.getOrigin() == null ? null : request.getOrigin().toString();
        if (mode == AppMode.HUB) return config.isSameOrigin(origin) ? MediaPolicy.supported(request.getResources()) : new String[0];
        return mode.isDemoMediaOrigin(origin) ? MediaPolicy.audioOnly(request.getResources()) : new String[0];
    }

    private void grantOrDeny(PermissionRequest request) {
        String[] allowed = allowedMedia(request);
        String[] grant = MediaPolicy.toGrant(allowed, allowed.length > 0,
                checkSelfPermission(Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED,
                checkSelfPermission(Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED);
        if (grant.length > 0) {
            request.grant(grant);
        } else {
            request.deny();
            Toast.makeText(this, R.string.permission_denied, Toast.LENGTH_LONG).show();
        }
    }

    private void launchChooser(boolean cameraAllowed) {
        WebChromeClient.FileChooserParams params = pendingFileParams;
        if (params == null || fileCallback == null) return;
        String[] accept = params.getAcceptTypes();
        String acceptJoined = accept == null ? "" : String.join(",", accept).toLowerCase(java.util.Locale.ROOT);
        boolean video = acceptJoined.contains("video");
        boolean image = acceptJoined.contains("image") || acceptJoined.isEmpty();

        Intent pick = params.createIntent();
        List<Intent> extras = new ArrayList<>();
        captureUri = null;
        captureFile = null;
        if (cameraAllowed && (image || video)) {
            String ext = video && !image ? ".mp4" : ".jpg";
            captureFile = new File(CaptureProvider.dir(this), "eva-" + System.currentTimeMillis() + ext);
            captureUri = CaptureProvider.uriFor(this, captureFile);
            Intent cam = new Intent(video && !image ? MediaStore.ACTION_VIDEO_CAPTURE : MediaStore.ACTION_IMAGE_CAPTURE);
            cam.putExtra(MediaStore.EXTRA_OUTPUT, captureUri);
            if (video && !image) cam.putExtra(MediaStore.EXTRA_DURATION_LIMIT, 60);
            cam.setClipData(ClipData.newRawUri("", captureUri));
            cam.addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION | Intent.FLAG_GRANT_READ_URI_PERMISSION);
            extras.add(cam);
        }
        Intent chooser = Intent.createChooser(pick, getString(R.string.file_chooser_title));
        if (!extras.isEmpty()) chooser.putExtra(Intent.EXTRA_INITIAL_INTENTS, extras.toArray(new Intent[0]));
        try {
            startActivityForResult(chooser, REQ_FILE);
        } catch (ActivityNotFoundException e) {
            fileCallback.onReceiveValue(null);
            fileCallback = null;
        }
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        if (requestCode != REQ_FILE || fileCallback == null) return;
        Uri[] result = null;
        if (resultCode == RESULT_OK) {
            if (data != null && data.getClipData() != null && data.getClipData().getItemCount() > 0
                    && !(captureUri != null && captureUri.equals(data.getClipData().getItemAt(0).getUri()))) {
                ClipData cd = data.getClipData();
                result = new Uri[cd.getItemCount()];
                for (int i = 0; i < cd.getItemCount(); i++) result[i] = cd.getItemAt(i).getUri();
            } else if (data != null && data.getData() != null) {
                result = new Uri[] {data.getData()};
            } else if (captureFile != null && captureFile.length() > 0) {
                result = new Uri[] {captureUri};
            }
        }
        fileCallback.onReceiveValue(result);
        fileCallback = null;
        pendingFileParams = null;
    }

    @Override
    public void onRequestPermissionsResult(int requestCode, String[] permissions, int[] grantResults) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults);
        if (requestCode == REQ_MEDIA && pendingPermission != null) {
            PermissionRequest r = pendingPermission;
            pendingPermission = null;
            grantOrDeny(r);
        } else if (requestCode == REQ_CAMERA_FOR_CAPTURE) {
            boolean granted = grantResults.length > 0 && grantResults[0] == PackageManager.PERMISSION_GRANTED;
            launchChooser(granted);
        }
    }

    // ---------------------------------------------------------------- back

    private void setUpBack() {
        if (Build.VERSION.SDK_INT >= 33) {
            OnBackInvokedCallback cb = this::handleBack;
            getOnBackInvokedDispatcher().registerOnBackInvokedCallback(OnBackInvokedDispatcher.PRIORITY_DEFAULT, cb);
            backCallback = cb;
        }
    }

    // API 26-32 only: on 33+ the OnBackInvokedCallback above handles back (enableOnBackInvokedCallback="true"),
    // and no AndroidX is used, so there is no OnBackPressedDispatcher to migrate to.
    @SuppressLint("GestureBackNavigation")
    @SuppressWarnings("deprecation")
    @Override
    public void onBackPressed() {
        handleBack(); // API 26-32; 33+ goes through the OnBackInvokedCallback
    }

    private void handleBack() {
        if (web != null && errorView.getVisibility() != View.VISIBLE && web.canGoBack()) {
            web.goBack();
            return;
        }
        showMenu(true);
    }

    /** Back at the root, or a long-press on the page: leave, or go back to settings (mode, hub, seat). */
    void showMenu(boolean fromBack) {
        new AlertDialog.Builder(this, android.R.style.Theme_Material_Dialog_Alert)
                .setTitle(fromBack ? R.string.leave_title : R.string.menu_title)
                .setPositiveButton(fromBack ? R.string.leave : R.string.reload, (d, w) -> {
                    if (fromBack) finish(); else load();
                })
                .setNeutralButton(R.string.change_hub, (d, w) -> openSettings())
                .setNegativeButton(R.string.stay, null)
                .show();
    }
}
