package tech.sanjox.eva;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertTrue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.robolectric.Shadows.shadowOf;

import android.app.AlertDialog;
import android.app.Application;
import android.content.Intent;
import android.net.ConnectivityManager;
import android.net.Uri;
import android.net.http.SslCertificate;
import android.net.http.SslError;
import android.os.Looper;
import android.view.View;
import android.webkit.PermissionRequest;
import android.webkit.SslErrorHandler;
import android.webkit.WebResourceError;
import android.webkit.WebResourceRequest;
import android.webkit.WebResourceResponse;
import android.webkit.WebView;
import android.webkit.WebViewClient;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.robolectric.Robolectric;
import org.robolectric.RobolectricTestRunner;
import org.robolectric.RuntimeEnvironment;
import org.robolectric.annotation.Config;
import org.robolectric.shadows.ShadowAlertDialog;
import org.robolectric.shadows.ShadowToast;

import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.security.cert.CertificateFactory;
import java.security.cert.X509Certificate;
import java.util.Map;

/**
 * 0.3.4: the APK works without a ship hub. The three modes on the setup screen, the online demo on Vercel (system
 * TLS, never trust-on-first-use, a strict allow-list), the replay bundled in the APK (WebViewAssetLoader, not
 * file://) and the fallback from online to offline.
 */
@RunWith(RobolectricTestRunner.class)
@Config(sdk = 34)
public class DemoModeRobolectricTest {
    private Application app;
    private HubPrefs prefs;

    @Before
    public void setUp() {
        app = RuntimeEnvironment.getApplication();
        prefs = new HubPrefs(app);
    }

    private MainActivity launch(AppMode m) {
        prefs.setMode(m);
        return Robolectric.buildActivity(MainActivity.class).setup().get();
    }

    private static WebView web(MainActivity a) {
        return a.findViewById(R.id.web);
    }

    private static WebViewClient client(MainActivity a) {
        return shadowOf(web(a)).getWebViewClient();
    }

    private static void idle() {
        shadowOf(Looper.getMainLooper()).idle();
    }

    private static WebResourceRequest request(String url, boolean mainFrame) {
        Uri u = Uri.parse(url);
        return new WebResourceRequest() {
            @Override public Uri getUrl() { return u; }
            @Override public boolean isForMainFrame() { return mainFrame; }
            @Override public boolean isRedirect() { return false; }
            @Override public boolean hasGesture() { return true; }
            @Override public String getMethod() { return "GET"; }
            @Override public Map<String, String> getRequestHeaders() { return java.util.Collections.emptyMap(); }
        };
    }

    private static String body(WebResourceResponse r) throws Exception {
        try (InputStream in = r.getData()) {
            return new String(in.readAllBytes(), StandardCharsets.UTF_8);
        }
    }

    @SuppressWarnings("deprecation")
    private SslError sslError(String url) throws Exception {
        X509Certificate c;
        try (InputStream in = getClass().getResourceAsStream("/certs/hub-a.pem")) {
            c = (X509Certificate) CertificateFactory.getInstance("X.509").generateCertificate(in);
        }
        return new SslError(SslError.SSL_UNTRUSTED, new SslCertificate(c), url);
    }

    // ------------------------------------------------------------ prefs

    @Test
    public void rememberedModeAndUpgradeFromAHubOnlyInstall() {
        assertNull("first run", prefs.mode());
        prefs.save(HubConfig.parse("192.168.0.10:8080", "https", Seat.ANA));
        assertEquals("0.3.3 installs saved only a hub: they stay on it", AppMode.HUB, prefs.mode());
        prefs.setMode(AppMode.OFFLINE);
        assertEquals(AppMode.OFFLINE, prefs.mode());
        assertNotNull("the hub is kept for later", prefs.load());
    }

    // ------------------------------------------------------------ setup screen

    @Test
    public void setupOffersThreeModesWithTheHubFormFolded() {
        SetupActivity s = Robolectric.buildActivity(SetupActivity.class).setup().get();
        assertEquals(View.VISIBLE, s.findViewById(R.id.mode_online).getVisibility());
        assertEquals(View.VISIBLE, s.findViewById(R.id.mode_offline).getVisibility());
        assertEquals(View.VISIBLE, s.findViewById(R.id.mode_hub).getVisibility());
        assertEquals(View.VISIBLE, s.findViewById(R.id.recommended).getVisibility());
        assertEquals(View.GONE, s.findViewById(R.id.hub_form).getVisibility());
        s.findViewById(R.id.mode_hub).performClick();
        assertEquals(View.VISIBLE, s.findViewById(R.id.hub_form).getVisibility());
        assertTrue(s.findViewById(R.id.mode_hub).isSelected());
    }

    @Test
    public void tryTheDemoIsRememberedAndOpensTheApp() {
        SetupActivity s = Robolectric.buildActivity(SetupActivity.class).setup().get();
        s.findViewById(R.id.mode_online).performClick();
        assertEquals(AppMode.ONLINE, prefs.mode());
        assertEquals(MainActivity.class.getName(), shadowOf(s).getNextStartedActivity().getComponent().getClassName());
        assertTrue(s.isFinishing());
        // back in settings later: the remembered mode is marked, the hub form stays folded
        SetupActivity again = Robolectric.buildActivity(SetupActivity.class).setup().get();
        assertTrue(again.findViewById(R.id.mode_online).isSelected());
        assertEquals(View.GONE, again.findViewById(R.id.hub_form).getVisibility());
    }

    @Test
    public void offlineDemoIsRemembered() {
        SetupActivity s = Robolectric.buildActivity(SetupActivity.class).setup().get();
        s.findViewById(R.id.mode_offline).performClick();
        assertEquals(AppMode.OFFLINE, prefs.mode());
    }

    @Test
    public void aSavedHubOpensSettingsWithItsForm() {
        prefs.save(HubConfig.parse("192.168.0.10:8080", "https", Seat.ANA));
        SetupActivity s = Robolectric.buildActivity(SetupActivity.class).setup().get();
        assertEquals(View.VISIBLE, s.findViewById(R.id.hub_form).getVisibility());
        assertTrue(s.findViewById(R.id.mode_hub).isSelected());
    }

    // ------------------------------------------------------------ online demo

    @Test
    public void onlineDemoLoadsTheHostedReplayWithJsAndStorage() {
        MainActivity a = launch(AppMode.ONLINE);
        assertEquals(AppMode.ONLINE, a.mode());
        assertEquals("https://eva-wearmeds.vercel.app/app/", shadowOf(web(a)).getLastLoadedUrl());
        assertTrue(web(a).getSettings().getJavaScriptEnabled());
        assertTrue(web(a).getSettings().getDomStorageEnabled());
        assertEquals(android.content.pm.ActivityInfo.SCREEN_ORIENTATION_FULL_USER, a.getRequestedOrientation());
    }

    @Test
    public void onlineDemoKeepsTheAppAndApiInsideAndSendsTheRestOut() {
        MainActivity a = launch(AppMode.ONLINE);
        WebViewClient c = client(a);
        assertFalse(c.shouldOverrideUrlLoading(web(a), request("https://eva-wearmeds.vercel.app/app/v/s1-4B-eva.html", true)));
        assertFalse(c.shouldOverrideUrlLoading(web(a), request("https://eva-wearmeds.vercel.app/api/eva/chat", true)));
        assertTrue(c.shouldOverrideUrlLoading(web(a), request("https://eva-wearmeds.vercel.app/", true)));
        Intent out = shadowOf(a).getNextStartedActivity();
        assertEquals(Intent.ACTION_VIEW, out.getAction());
        assertEquals("https://eva-wearmeds.vercel.app/", out.getDataString());
        assertTrue(c.shouldOverrideUrlLoading(web(a), request("https://example.com/", true)));
        assertEquals("https://example.com/", shadowOf(a).getNextStartedActivity().getDataString());
        assertTrue("dropped", c.shouldOverrideUrlLoading(web(a), request("intent://x#Intent;end", true)));
        // the bundle is not served while online
        WebResourceResponse r = c.shouldInterceptRequest(web(a), request(AppMode.OFFLINE_START, true));
        assertEquals(404, r.getStatusCode());
        // ordinary https subresources go to the network
        assertNull(c.shouldInterceptRequest(web(a), request("https://eva-wearmeds.vercel.app/app/static/css/app.css", false)));
    }

    @Test
    public void onlineDemoNeverAsksToTrustACertificate() throws Exception {
        MainActivity a = launch(AppMode.ONLINE);
        SslErrorHandler h = mock(SslErrorHandler.class);
        client(a).onReceivedSslError(web(a), h, sslError("https://eva-wearmeds.vercel.app/app/"));
        idle();
        verify(h).cancel();
        verify(h, never()).proceed();
        assertNull("no Trust dialog", ShadowAlertDialog.getLatestAlertDialog());
        assertNull("nothing pinned", prefs.pin(AppMode.ONLINE_HOST));
        assertEquals(View.VISIBLE, a.findViewById(R.id.error).getVisibility());
    }

    @Test
    public void theHubStillAsksOnFirstUse() throws Exception {
        prefs.save(HubConfig.parse("192.168.0.10:8080", "https", Seat.ANA));
        MainActivity a = Robolectric.buildActivity(MainActivity.class).setup().get();
        SslErrorHandler h = mock(SslErrorHandler.class);
        client(a).onReceivedSslError(web(a), h, sslError("https://192.168.0.10:8080/app"));
        idle();
        assertNotNull("TOFU prompt for the hub", ShadowAlertDialog.getLatestAlertDialog());
        verify(h, never()).proceed();
    }

    @Test
    public void onlineDemoFallsBackToTheBundleWhenTheSiteDoesNotLoad() {
        MainActivity a = launch(AppMode.ONLINE);
        WebResourceError err = mock(WebResourceError.class);
        when(err.getErrorCode()).thenReturn(WebViewClient.ERROR_HOST_LOOKUP);
        when(err.getDescription()).thenReturn("net::ERR_NAME_NOT_RESOLVED");
        client(a).onReceivedError(web(a), request("https://eva-wearmeds.vercel.app/app/static/x.png", false), err);
        assertEquals("subresource errors do not switch", AppMode.ONLINE, a.mode());
        client(a).onReceivedError(web(a), request("https://eva-wearmeds.vercel.app/app/", true), err);
        assertEquals(AppMode.OFFLINE, a.mode());
        assertEquals(AppMode.OFFLINE_START, shadowOf(web(a)).getLastLoadedUrl());
        assertEquals(a.getString(R.string.offline_fallback_toast), ShadowToast.getTextOfLatestToast());
        assertEquals(View.GONE, a.findViewById(R.id.error).getVisibility());
        assertEquals("the choice stays online for next time", AppMode.ONLINE, prefs.mode());
    }

    @Test
    public void noNetworkAtLaunchOpensTheBundleWithAToast() {
        ConnectivityManager cm = app.getSystemService(ConnectivityManager.class);
        shadowOf(cm).setDefaultNetworkActive(false);
        shadowOf(cm).setActiveNetworkInfo(null);
        MainActivity a = launch(AppMode.ONLINE);
        assertEquals(AppMode.OFFLINE, a.mode());
        assertEquals(AppMode.OFFLINE_START, shadowOf(web(a)).getLastLoadedUrl());
        assertEquals(a.getString(R.string.offline_fallback_toast), ShadowToast.getTextOfLatestToast());
    }

    @Test
    public void demoModesNeverGrantMicOrCamera() {
        MainActivity a = launch(AppMode.ONLINE);
        PermissionRequest req = mock(PermissionRequest.class);
        when(req.getOrigin()).thenReturn(Uri.parse("https://eva-wearmeds.vercel.app/"));
        when(req.getResources()).thenReturn(new String[] {PermissionRequest.RESOURCE_AUDIO_CAPTURE});
        shadowOf(web(a)).getWebChromeClient().onPermissionRequest(req);
        verify(req).deny();
        verify(req, never()).grant(org.mockito.ArgumentMatchers.any());
    }

    // ------------------------------------------------------------ offline demo (bundled)

    @Test
    public void offlineDemoIsServedFromTheApkAssets() throws Exception {
        MainActivity a = launch(AppMode.OFFLINE);
        assertEquals(AppMode.OFFLINE_START, shadowOf(web(a)).getLastLoadedUrl());
        WebViewClient c = client(a);
        WebResourceResponse index = c.shouldInterceptRequest(web(a), request(AppMode.OFFLINE_START, true));
        assertNotNull(index);
        assertEquals("text/html", index.getMimeType());
        assertTrue(body(index).contains("Eva"));
        WebResourceResponse css = c.shouldInterceptRequest(web(a),
                request("https://appassets.androidplatform.net/assets/replay/static/css/app.css", false));
        assertEquals("text/css", css.getMimeType());
        WebResourceResponse js = c.shouldInterceptRequest(web(a),
                request("https://appassets.androidplatform.net/assets/replay/static/js/replay.js", false));
        String replay = body(js);
        assertTrue("captured answers only under the asset loader", replay.contains("appassets.androidplatform.net"));
        WebResourceResponse missing = c.shouldInterceptRequest(web(a),
                request("https://appassets.androidplatform.net/assets/replay/nope.html", false));
        assertTrue(missing == null || missing.getStatusCode() == 404 || missing.getData() == null);
        assertFalse(c.shouldOverrideUrlLoading(web(a), request("https://appassets.androidplatform.net/assets/replay/v/s1-4B-eva.html", true)));
        assertTrue(c.shouldOverrideUrlLoading(web(a), request(AppMode.ONLINE_START, true)));
    }

    // ------------------------------------------------------------ the way back to settings

    @Test
    public void longPressOpensTheMenuWithSettings() {
        MainActivity a = launch(AppMode.OFFLINE);
        assertTrue(MainActivity.menuOnLongPress(WebView.HitTestResult.UNKNOWN_TYPE));
        assertFalse("links keep their own long-press", MainActivity.menuOnLongPress(WebView.HitTestResult.SRC_ANCHOR_TYPE));
        assertFalse(MainActivity.menuOnLongPress(WebView.HitTestResult.EDIT_TEXT_TYPE));
        // Robolectric's WebView has no Chromium gesture layer, so call the listener the long-press reaches on a device
        View.OnLongClickListener l = shadowOf(web(a)).getOnLongClickListener();
        assertNotNull(l);
        assertTrue(l.onLongClick(web(a)));
        idle();
        AlertDialog d = ShadowAlertDialog.getLatestAlertDialog();
        assertNotNull(d);
        assertEquals(a.getString(R.string.menu_title), shadowOf(d).getTitle().toString());
        d.getButton(AlertDialog.BUTTON_NEUTRAL).performClick();
        idle();
        assertEquals(SetupActivity.class.getName(), shadowOf(a).getNextStartedActivity().getComponent().getClassName());
    }
}
