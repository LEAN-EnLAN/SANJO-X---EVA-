package tech.sanjox.eva;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertTrue;

import org.junit.Test;

/** Mode selection, the demo allow-lists and where trust-on-first-use applies. */
public class AppModeTest {
    @Test
    public void savedIdsRoundTripAndUnknownFallsBack() {
        for (AppMode m : AppMode.values()) assertEquals(m, AppMode.fromId(m.id, null));
        assertNull(AppMode.fromId("mars", null));
        assertEquals(AppMode.HUB, AppMode.fromId(null, AppMode.HUB));
    }

    @Test
    public void onlineDemoFallsBackToTheBundleWithoutNetwork() {
        assertEquals(AppMode.ONLINE, AppMode.effective(AppMode.ONLINE, true));
        assertEquals(AppMode.OFFLINE, AppMode.effective(AppMode.ONLINE, false));
        assertEquals(AppMode.OFFLINE, AppMode.effective(AppMode.OFFLINE, true));
        assertEquals("a hub is never swapped for a demo", AppMode.HUB, AppMode.effective(AppMode.HUB, false));
    }

    @Test
    public void onlyTheHubIsTrustedOnFirstUse() {
        assertTrue(AppMode.HUB.allowsTofu());
        assertFalse(AppMode.ONLINE.allowsTofu());
        assertFalse(AppMode.OFFLINE.allowsTofu());
    }

    @Test
    public void startPages() {
        assertEquals("https://eva-wearmeds.vercel.app/app/", AppMode.ONLINE.demoStartUrl());
        assertEquals("https://appassets.androidplatform.net/assets/replay/index.html", AppMode.OFFLINE.demoStartUrl());
    }

    @Test(expected = IllegalStateException.class)
    public void theHubStartPageComesFromItsConfig() {
        AppMode.HUB.demoStartUrl();
    }

    @Test
    public void onlineAllowListIsTheAppAndTheApiOnTheDemoHost() {
        AppMode m = AppMode.ONLINE;
        assertTrue(m.allowsInWebView("https://eva-wearmeds.vercel.app/app/"));
        assertTrue(m.allowsInWebView("https://eva-wearmeds.vercel.app/app"));
        assertTrue(m.allowsInWebView("https://eva-wearmeds.vercel.app/app/v/s1-4B-home.html#x"));
        assertTrue(m.allowsInWebView("https://EVA-WEARMEDS.vercel.app/api/eva/chat"));
        assertTrue(m.allowsInWebView("https://eva-wearmeds.vercel.app:443/app/"));
        assertFalse("the landing opens in the browser", m.allowsInWebView("https://eva-wearmeds.vercel.app/"));
        assertFalse(m.allowsInWebView("https://eva-wearmeds.vercel.app/hub/"));
        assertFalse(m.allowsInWebView("https://eva-wearmeds.vercel.app/application"));
        assertFalse(m.allowsInWebView("https://eva-wearmeds.vercel.app/app/../hub/"));
        assertFalse(m.allowsInWebView("https://eva-wearmeds.vercel.app/app/%2e%2e/hub/"));
        assertFalse("cleartext", m.allowsInWebView("http://eva-wearmeds.vercel.app/app/"));
        assertFalse("other port", m.allowsInWebView("https://eva-wearmeds.vercel.app:8443/app/"));
        assertFalse("look-alike", m.allowsInWebView("https://eva-wearmeds.vercel.app.evil.example/app/"));
        assertFalse("user info", m.allowsInWebView("https://x@eva-wearmeds.vercel.app/app/"));
        assertFalse(m.allowsInWebView("https://other.vercel.app/app/"));
        assertFalse("the bundle is offline-only", m.allowsInWebView(AppMode.OFFLINE_START));
        assertFalse(m.allowsInWebView(null));
        assertFalse(m.allowsInWebView("intent://x#Intent;end"));
    }

    @Test
    public void offlineAllowListIsTheBundleOnTheAssetHost() {
        AppMode m = AppMode.OFFLINE;
        assertTrue(m.allowsInWebView(AppMode.OFFLINE_START));
        assertTrue(m.allowsInWebView("https://appassets.androidplatform.net/assets/replay/v/s1-4B-eva.html"));
        assertFalse(m.allowsInWebView("https://appassets.androidplatform.net/assets/other.html"));
        assertFalse(m.allowsInWebView("https://appassets.androidplatform.net/res/x"));
        assertFalse(m.allowsInWebView("http://appassets.androidplatform.net/assets/replay/index.html"));
        assertFalse(m.allowsInWebView("file:///android_asset/replay/index.html"));
        assertFalse(m.allowsInWebView(AppMode.ONLINE_START));
    }

    @Test
    public void theHubNeverUsesTheDemoAllowList() {
        assertFalse(AppMode.HUB.allowsInWebView(AppMode.ONLINE_START));
        assertFalse(AppMode.HUB.allowsInWebView(AppMode.OFFLINE_START));
    }

    @Test
    public void demoHostCheckIsExact() {
        assertTrue(AppMode.isOnlineDemoHost("https://eva-wearmeds.vercel.app/app/"));
        assertFalse(AppMode.isOnlineDemoHost("https://eva-wearmeds.vercel.app.evil.example/"));
        assertFalse(AppMode.isOnlineDemoHost("https://192.168.0.10:8080/app"));
    }
}
