package tech.sanjox.eva;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.robolectric.Shadows.shadowOf;

import android.Manifest;
import android.app.AlertDialog;
import android.app.Application;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.net.Uri;
import android.net.http.SslCertificate;
import android.net.http.SslError;
import android.os.Looper;
import android.view.View;
import android.view.WindowManager;
import android.webkit.PermissionRequest;
import android.webkit.SslErrorHandler;
import android.webkit.WebChromeClient;
import android.webkit.WebResourceError;
import android.webkit.WebResourceRequest;
import android.webkit.WebResourceResponse;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.widget.Button;
import android.widget.EditText;
import android.widget.TextView;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.ArgumentCaptor;
import org.robolectric.Robolectric;
import org.robolectric.RobolectricTestRunner;
import org.robolectric.RuntimeEnvironment;
import org.robolectric.android.controller.ActivityController;
import org.robolectric.annotation.Config;
import org.robolectric.shadows.ShadowAlertDialog;

import java.io.InputStream;
import java.security.cert.CertificateFactory;
import java.security.cert.X509Certificate;
import java.util.Arrays;
import java.util.HashSet;
import java.util.Map;

/**
 * The native shell on a Robolectric Android (API 34): first run routing, the setup form, the WebViewClient
 * (cleartext policy, navigation, error screen, trust-on-first-use pinning with real certificates) and the
 * WebChromeClient (mic/camera grants by origin).
 */
@RunWith(RobolectricTestRunner.class)
@Config(sdk = 34)
public class MainActivityRobolectricTest {
    // Fingerprints printed by `openssl x509 -noout -fingerprint -sha256 -in <file>` for the test certificates.
    static final String FP_A = "79:49:DF:74:E0:06:05:C8:1B:A6:7D:90:23:B7:23:22:9C:D0:FE:BF:B1:07:B7:85:D6:E9:D0:83:F0:EA:8C:32";
    static final String FP_B = "59:C6:15:F3:2B:E2:82:C1:29:CD:4A:35:16:5F:89:D0:C2:1E:9B:B4:EC:93:42:28:02:CF:5C:6A:11:75:78:CE";

    private Application app;
    private HubPrefs prefs;

    @Before
    public void setUp() {
        app = RuntimeEnvironment.getApplication();
        prefs = new HubPrefs(app);
    }

    // ------------------------------------------------------------ helpers

    private MainActivity launchWith(String address, String scheme, Seat seat) {
        prefs.save(HubConfig.parse(address, scheme, seat));
        ActivityController<MainActivity> c = Robolectric.buildActivity(MainActivity.class).setup();
        return c.get();
    }

    private static WebView web(MainActivity a) {
        return a.findViewById(R.id.web);
    }

    private static WebViewClient client(MainActivity a) {
        return shadowOf(web(a)).getWebViewClient();
    }

    private static WebChromeClient chrome(MainActivity a) {
        return shadowOf(web(a)).getWebChromeClient();
    }

    private static boolean errorShown(MainActivity a) {
        return a.findViewById(R.id.error).getVisibility() == View.VISIBLE;
    }

    private static String errorTitle(MainActivity a) {
        return ((TextView) a.findViewById(R.id.error_title)).getText().toString();
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

    private X509Certificate cert(String name) throws Exception {
        try (InputStream in = getClass().getResourceAsStream("/certs/" + name + ".pem")) {
            assertNotNull(name, in);
            return (X509Certificate) CertificateFactory.getInstance("X.509").generateCertificate(in);
        }
    }

    @SuppressWarnings("deprecation")
    private SslError sslError(String certName, String url, int... errors) throws Exception {
        SslError e = new SslError(errors[0], new SslCertificate(cert(certName)), url);
        for (int i = 1; i < errors.length; i++) e.addError(errors[i]);
        return e;
    }

    // ------------------------------------------------------------ first run and setup

    @Test
    public void firstRunOpensSetup() {
        MainActivity a = Robolectric.buildActivity(MainActivity.class).setup().get();
        assertTrue(a.isFinishing());
        Intent next = shadowOf(a).getNextStartedActivity();
        assertEquals(SetupActivity.class.getName(), next.getComponent().getClassName());
    }

    @Test
    public void setupSavesChoiceAndOpensTheHub() {
        SetupActivity s = Robolectric.buildActivity(SetupActivity.class).setup().get();
        ((EditText) s.findViewById(R.id.address)).setText("10.0.2.2:9280");
        s.findViewById(R.id.scheme_http).performClick();
        // pick "Crew dashboard"
        android.widget.RadioGroup seats = s.findViewById(R.id.seats);
        for (int i = 0; i < seats.getChildCount(); i++) {
            if (seats.getChildAt(i).getTag() == Seat.DASH) seats.getChildAt(i).performClick();
        }
        ((Button) s.findViewById(R.id.connect)).performClick();

        HubConfig saved = prefs.load();
        assertEquals("http://10.0.2.2:9280/dash", saved.startUrl());
        Intent next = shadowOf(s).getNextStartedActivity();
        assertEquals(MainActivity.class.getName(), next.getComponent().getClassName());
        assertTrue(s.isFinishing());
    }

    @Test
    public void setupRejectsPublicHttpAndShowsWhy() {
        SetupActivity s = Robolectric.buildActivity(SetupActivity.class).setup().get();
        ((EditText) s.findViewById(R.id.address)).setText("8.8.8.8:8080");
        s.findViewById(R.id.scheme_http).performClick();
        ((Button) s.findViewById(R.id.connect)).performClick();
        TextView err = s.findViewById(R.id.form_error);
        assertEquals(View.VISIBLE, err.getVisibility());
        assertTrue(err.getText().toString().contains("https"));
        assertNull(prefs.load());
        assertNull(shadowOf(s).getNextStartedActivity());
    }

    @Test
    public void setupShowsAndForgetsThePinnedCertificate() {
        prefs.save(HubConfig.parse("192.168.0.10:8080", "https", Seat.ANA));
        prefs.savePin("192.168.0.10", FP_A);
        SetupActivity s = Robolectric.buildActivity(SetupActivity.class).setup().get();
        assertEquals(View.VISIBLE, s.findViewById(R.id.pin_row).getVisibility());
        assertTrue(((TextView) s.findViewById(R.id.pin_text)).getText().toString().contains("79:49:DF:74"));
        s.findViewById(R.id.forget_pin).performClick();
        assertNull(prefs.pin("192.168.0.10"));
        assertEquals(View.GONE, s.findViewById(R.id.pin_row).getVisibility());
    }

    // ------------------------------------------------------------ WebView basics

    @Test
    public void wristbandOpensTheBandUiLockedLandscape() {
        MainActivity a = launchWith("10.0.2.2:9280", "http", Seat.WRIST);
        assertEquals("http://10.0.2.2:9280/app/wrist?seat=4B&mode=wristband", shadowOf(web(a)).getLastLoadedUrl());
        assertEquals(android.content.pm.ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE, a.getRequestedOrientation());
    }

    @Test
    public void asksTheHubForEvaStatusOnceAfterTheFirstPage() {
        MainActivity a = launchWith("10.0.2.2:9280", "http", Seat.ANA);
        a.showEvaStatus();
        String js = shadowOf(web(a)).getLastEvaluatedJavascript();
        assertNotNull(js);
        assertTrue(js.contains("/api/eva/status"));
    }

    @Test
    public void loadsTheSeatUrlAndKeepsTheScreenOn() {
        MainActivity a = launchWith("10.0.2.2:9280", "http", Seat.LEO);
        assertEquals("http://10.0.2.2:9280/app?seat=3B", shadowOf(web(a)).getLastLoadedUrl());
        assertTrue(web(a).getSettings().getJavaScriptEnabled());
        assertTrue(web(a).getSettings().getDomStorageEnabled());
        assertFalse(web(a).getSettings().getMediaPlaybackRequiresUserGesture());
        // smoothness settings (APP-POLISH §8) and the user agent
        assertTrue(web(a).getSettings().getUserAgentString().endsWith(" EvaAndroid/0.3.4"));
        assertEquals(100, web(a).getSettings().getTextZoom());
        assertEquals(View.OVER_SCROLL_NEVER, web(a).getOverScrollMode());
        assertFalse(web(a).isVerticalScrollBarEnabled());
        assertEquals(android.content.pm.ActivityInfo.SCREEN_ORIENTATION_FULL_USER, a.getRequestedOrientation());
        int flags = a.getWindow().getAttributes().flags;
        assertTrue((flags & WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON) != 0);
    }

    @Test
    public void keepScreenOnCanBeTurnedOff() {
        prefs.setKeepScreenOn(false);
        MainActivity a = launchWith("10.0.2.2:9280", "http", Seat.ANA);
        int flags = a.getWindow().getAttributes().flags;
        assertEquals(0, flags & WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
    }

    @Test
    public void cleartextToPublicHostsIsBlocked() {
        MainActivity a = launchWith("10.0.2.2:9280", "http", Seat.ANA);
        WebViewClient c = client(a);
        assertNull(c.shouldInterceptRequest(web(a), request("http://10.0.2.2:9280/static/css/app.css", false)));
        assertNull(c.shouldInterceptRequest(web(a), request("https://example.com/x.js", false)));
        WebResourceResponse blocked = c.shouldInterceptRequest(web(a), request("http://example.com/x.js", false));
        assertNotNull(blocked);
        assertEquals(403, blocked.getStatusCode());
    }

    @Test
    public void hubLinksStayInsideOthersLeave() {
        MainActivity a = launchWith("10.0.2.2:9280", "http", Seat.ANA);
        WebViewClient c = client(a);
        assertFalse(c.shouldOverrideUrlLoading(web(a), request("http://10.0.2.2:9280/dash", true)));
        assertTrue(c.shouldOverrideUrlLoading(web(a), request("https://example.com/", true)));
        Intent external = shadowOf(a).getNextStartedActivity();
        assertEquals(Intent.ACTION_VIEW, external.getAction());
        assertEquals("https://example.com/", external.getDataString());
        assertTrue("other port = other origin", c.shouldOverrideUrlLoading(web(a), request("http://10.0.2.2:9281/dash", true)));
        assertTrue("intent: dropped", c.shouldOverrideUrlLoading(web(a), request("intent://x#Intent;end", true)));
    }

    @Test
    public void unreachableHubShowsNativeErrorWithRetry() {
        MainActivity a = launchWith("10.0.2.2:9280", "http", Seat.ANA);
        WebResourceError err = mock(WebResourceError.class);
        when(err.getErrorCode()).thenReturn(WebViewClient.ERROR_CONNECT);
        when(err.getDescription()).thenReturn("net::ERR_CONNECTION_REFUSED");

        client(a).onReceivedError(web(a), request("http://10.0.2.2:9280/static/x.png", false), err);
        assertFalse("subresource errors are left to the page", errorShown(a));

        client(a).onReceivedError(web(a), request("http://10.0.2.2:9280/app?seat=4B", true), err);
        assertTrue(errorShown(a));
        assertEquals(a.getString(R.string.error_unreachable_title), errorTitle(a));
        assertEquals(View.INVISIBLE, web(a).getVisibility());
        assertTrue(((TextView) a.findViewById(R.id.error_code)).getText().toString().contains("ERR_CONNECTION_REFUSED"));

        a.findViewById(R.id.retry).performClick();
        assertFalse(errorShown(a));
        assertEquals("http://10.0.2.2:9280/app?seat=4B", shadowOf(web(a)).getLastLoadedUrl());

        client(a).onReceivedError(web(a), request("http://10.0.2.2:9280/app?seat=4B", true), err);
        a.findViewById(R.id.change_hub).performClick();
        assertEquals(SetupActivity.class.getName(), shadowOf(a).getNextStartedActivity().getComponent().getClassName());
    }

    // ------------------------------------------------------------ TLS trust on first use

    @Test
    public void fingerprintOfTheRealCertificateMatchesOpenssl() throws Exception {
        assertEquals(FP_A, Fingerprints.sha256(MainActivity.derOf(new SslCertificate(cert("hub-a")))));
        assertEquals(FP_B, Fingerprints.sha256(MainActivity.derOf(new SslCertificate(cert("hub-b")))));
    }

    @Test
    public void firstConnectAsksThenPinsOnTrust() throws Exception {
        MainActivity a = launchWith("192.168.0.10:8080", "https", Seat.ANA);
        SslErrorHandler h1 = mock(SslErrorHandler.class);
        SslErrorHandler h2 = mock(SslErrorHandler.class);
        client(a).onReceivedSslError(web(a), h1, sslError("hub-a", "https://192.168.0.10:8080/app?seat=4B", SslError.SSL_UNTRUSTED));
        // a subresource with the same certificate while the dialog is open joins the same decision
        client(a).onReceivedSslError(web(a), h2, sslError("hub-a", "https://192.168.0.10:8080/static/js/eva.js", SslError.SSL_UNTRUSTED));
        idle();
        verify(h1, never()).proceed();
        verify(h1, never()).cancel();

        AlertDialog d = ShadowAlertDialog.getLatestAlertDialog();
        assertNotNull("trust dialog shown", d);
        assertTrue(d.isShowing());
        String msg = shadowOf(d).getMessage().toString();
        assertTrue(msg, msg.contains("79:49:DF:74:E0:06:05:C8"));
        assertTrue(msg.contains("192.168.0.10"));

        d.getButton(AlertDialog.BUTTON_POSITIVE).performClick();
        idle();
        verify(h1).proceed();
        verify(h2).proceed();
        assertEquals(FP_A, prefs.pin("192.168.0.10"));
    }

    @Test
    public void dontTrustCancelsAndExplains() throws Exception {
        MainActivity a = launchWith("192.168.0.10:8080", "https", Seat.ANA);
        SslErrorHandler h = mock(SslErrorHandler.class);
        client(a).onReceivedSslError(web(a), h, sslError("hub-a", "https://192.168.0.10:8080/app?seat=4B", SslError.SSL_UNTRUSTED));
        idle();
        AlertDialog d = ShadowAlertDialog.getLatestAlertDialog();
        d.getButton(AlertDialog.BUTTON_NEGATIVE).performClick();
        idle();
        verify(h).cancel();
        verify(h, never()).proceed();
        assertNull(prefs.pin("192.168.0.10"));
        assertTrue(errorShown(a));
        assertEquals(a.getString(R.string.error_cert_rejected_title), errorTitle(a));

        // the WebView's follow-up network error must not replace the certificate explanation
        WebResourceError err = mock(WebResourceError.class);
        when(err.getErrorCode()).thenReturn(WebViewClient.ERROR_FAILED_SSL_HANDSHAKE);
        when(err.getDescription()).thenReturn("net::ERR_FAILED");
        client(a).onReceivedError(web(a), request("https://192.168.0.10:8080/app?seat=4B", true), err);
        assertEquals(a.getString(R.string.error_cert_rejected_title), errorTitle(a));
    }

    @Test
    public void pinnedCertificateProceedsWithoutAsking() throws Exception {
        prefs.savePin("192.168.0.10", FP_A);
        MainActivity a = launchWith("192.168.0.10:8080", "https", Seat.ANA);
        SslErrorHandler h = mock(SslErrorHandler.class);
        // name mismatch too (hub reached by a new IP that is not in the certificate): the pin is what counts
        client(a).onReceivedSslError(web(a), h,
                sslError("hub-a", "https://192.168.0.10:8080/app", SslError.SSL_UNTRUSTED, SslError.SSL_IDMISMATCH));
        idle();
        verify(h).proceed();
        assertNull("no dialog", ShadowAlertDialog.getLatestAlertDialog());
    }

    @Test
    public void changedCertificateIsRefused() throws Exception {
        prefs.savePin("192.168.0.10", FP_A);
        MainActivity a = launchWith("192.168.0.10:8080", "https", Seat.ANA);
        SslErrorHandler h = mock(SslErrorHandler.class);
        client(a).onReceivedSslError(web(a), h, sslError("hub-b", "https://192.168.0.10:8080/app", SslError.SSL_UNTRUSTED));
        idle();
        verify(h).cancel();
        verify(h, never()).proceed();
        assertNull("never asks to replace a pin", ShadowAlertDialog.getLatestAlertDialog());
        assertTrue(errorShown(a));
        assertEquals(a.getString(R.string.error_cert_changed_title), errorTitle(a));
        String code = ((TextView) a.findViewById(R.id.error_code)).getText().toString();
        assertTrue(code, code.contains(FP_A) && code.contains(FP_B));
        assertEquals("pin unchanged", FP_A, prefs.pin("192.168.0.10"));
    }

    @Test
    public void otherHostsAndExpiredCertificatesAreRefused() throws Exception {
        prefs.savePin("192.168.0.10", FP_A);
        MainActivity a = launchWith("192.168.0.10:8080", "https", Seat.ANA);
        SslErrorHandler other = mock(SslErrorHandler.class);
        client(a).onReceivedSslError(web(a), other, sslError("hub-a", "https://evil.example.com/x.js", SslError.SSL_UNTRUSTED));
        SslErrorHandler expired = mock(SslErrorHandler.class);
        client(a).onReceivedSslError(web(a), expired,
                sslError("hub-a", "https://192.168.0.10:8080/app", SslError.SSL_UNTRUSTED, SslError.SSL_EXPIRED));
        idle();
        verify(other).cancel();
        verify(other, never()).proceed();
        verify(expired).cancel();
        verify(expired, never()).proceed();
        assertNull(ShadowAlertDialog.getLatestAlertDialog());
        assertEquals(a.getString(R.string.error_cert_invalid_title), errorTitle(a));
    }

    // ------------------------------------------------------------ mic and camera

    private static PermissionRequest mediaRequest(String origin) {
        PermissionRequest r = mock(PermissionRequest.class);
        when(r.getOrigin()).thenReturn(Uri.parse(origin));
        when(r.getResources()).thenReturn(new String[] {
            PermissionRequest.RESOURCE_AUDIO_CAPTURE, PermissionRequest.RESOURCE_VIDEO_CAPTURE,
            PermissionRequest.RESOURCE_PROTECTED_MEDIA_ID});
        return r;
    }

    @Test
    public void micAndCameraGrantedToTheHubOnly() {
        shadowOf(app).grantPermissions(Manifest.permission.RECORD_AUDIO, Manifest.permission.CAMERA);
        MainActivity a = launchWith("192.168.0.10:8080", "https", Seat.ANA);

        PermissionRequest hub = mediaRequest("https://192.168.0.10:8080/");
        chrome(a).onPermissionRequest(hub);
        ArgumentCaptor<String[]> granted = ArgumentCaptor.forClass(String[].class);
        verify(hub).grant(granted.capture());
        assertEquals(new HashSet<>(Arrays.asList(PermissionRequest.RESOURCE_AUDIO_CAPTURE, PermissionRequest.RESOURCE_VIDEO_CAPTURE)),
                new HashSet<>(Arrays.asList(granted.getValue())));

        PermissionRequest otherPort = mediaRequest("https://192.168.0.10:8081/");
        chrome(a).onPermissionRequest(otherPort);
        verify(otherPort).deny();
        verify(otherPort, never()).grant(any());

        PermissionRequest plainHttp = mediaRequest("http://192.168.0.10:8080/");
        chrome(a).onPermissionRequest(plainHttp);
        verify(plainHttp).deny();
    }

    @Test
    public void runtimePermissionIsRequestedFirst() {
        MainActivity a = launchWith("192.168.0.10:8080", "https", Seat.ANA);
        PermissionRequest hub = mock(PermissionRequest.class);
        when(hub.getOrigin()).thenReturn(Uri.parse("https://192.168.0.10:8080/"));
        when(hub.getResources()).thenReturn(new String[] {PermissionRequest.RESOURCE_AUDIO_CAPTURE});
        chrome(a).onPermissionRequest(hub);
        verify(hub, never()).grant(any());
        verify(hub, never()).deny();
        assertEquals(Manifest.permission.RECORD_AUDIO, shadowOf(a).getLastRequestedPermission().requestedPermissions[0]);

        shadowOf(app).grantPermissions(Manifest.permission.RECORD_AUDIO);
        a.onRequestPermissionsResult(shadowOf(a).getLastRequestedPermission().requestCode,
                new String[] {Manifest.permission.RECORD_AUDIO}, new int[] {PackageManager.PERMISSION_GRANTED});
        verify(hub).grant(new String[] {PermissionRequest.RESOURCE_AUDIO_CAPTURE});
    }

    // ------------------------------------------------------------ back

    @Test
    public void backAtTheRootOffersLeaveOrChangeHub() {
        MainActivity a = launchWith("10.0.2.2:9280", "http", Seat.ANA);
        a.onBackPressed();
        idle();
        AlertDialog d = ShadowAlertDialog.getLatestAlertDialog();
        assertNotNull(d);
        assertEquals(a.getString(R.string.leave_title), shadowOf(d).getTitle().toString());
        assertEquals(a.getString(R.string.leave), d.getButton(AlertDialog.BUTTON_POSITIVE).getText().toString());
        d.getButton(AlertDialog.BUTTON_NEUTRAL).performClick();
        idle(); // dialog button listeners run from the main looper
        assertEquals(SetupActivity.class.getName(), shadowOf(a).getNextStartedActivity().getComponent().getClassName());
    }
}
