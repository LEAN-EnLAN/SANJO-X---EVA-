package tech.sanjox.eva;

import java.net.URI;
import java.net.URISyntaxException;
import java.util.Locale;

/**
 * What the app opens. Pure Java, unit-tested.
 *
 * <ul>
 *   <li>{@link #ONLINE}: the hosted replay of the real app with live Eva chat, {@value #ONLINE_START}. Normal system
 *       TLS, no trust-on-first-use (a public CA certificate). Navigation stays inside {@code /app/} and {@code /api/}.
 *   <li>{@link #OFFLINE}: the same replay bundled in the APK assets, served by WebViewAssetLoader from
 *       {@value #OFFLINE_START} (never {@code file://}). Eva answers with the captured replies.
 *   <li>{@link #HUB}: a ship hub on the local network (the {@code eva} binary), with TOFU certificate pinning.
 * </ul>
 */
public enum AppMode {
    ONLINE("online"),
    OFFLINE("offline"),
    HUB("hub");

    public static final String ONLINE_HOST = "eva-wearmeds.vercel.app";
    public static final String ONLINE_START = "https://" + ONLINE_HOST + "/app/";
    /** WebViewAssetLoader's default domain; the bundle lives in {@code assets/replay/}. */
    public static final String ASSET_HOST = "appassets.androidplatform.net";
    public static final String ASSET_PREFIX = "/assets/";
    public static final String OFFLINE_ROOT = ASSET_PREFIX + "replay/";
    public static final String OFFLINE_START = "https://" + ASSET_HOST + OFFLINE_ROOT + "index.html";

    public final String id;

    AppMode(String id) {
        this.id = id;
    }

    /** The saved mode, or {@code fallback} for null or an unknown value. */
    public static AppMode fromId(String id, AppMode fallback) {
        if (id != null) {
            for (AppMode m : values()) {
                if (m.id.equals(id)) return m;
            }
        }
        return fallback;
    }

    /** The online demo needs a network; without one it opens the bundled replay instead. */
    public static AppMode effective(AppMode chosen, boolean online) {
        return chosen == ONLINE && !online ? OFFLINE : chosen;
    }

    /** Only a ship hub on the LAN may be trusted on first use; the demo hosts never prompt and never pin. */
    public boolean allowsTofu() {
        return this == HUB;
    }

    /** Start page for the demo modes (the hub's comes from {@link HubConfig#startUrl()}). */
    public String demoStartUrl() {
        switch (this) {
            case ONLINE:
                return ONLINE_START;
            case OFFLINE:
                return OFFLINE_START;
            default:
                throw new IllegalStateException("the hub's start URL comes from HubConfig");
        }
    }

    /**
     * Whether a demo mode keeps {@code url} inside the WebView: https on the exact demo host, under {@code /app/}
     * or {@code /api/} online, under the bundle's folder offline. Anything else opens in the browser (or is dropped).
     * Always false for {@link #HUB}, which uses {@link HubConfig#isSameOrigin(String)}.
     */
    public boolean allowsInWebView(String url) {
        if (url == null || this == HUB) return false;
        URI u;
        try {
            u = new URI(url.trim());
        } catch (URISyntaxException e) {
            return false;
        }
        if (u.getScheme() == null || u.getHost() == null || !"https".equals(u.getScheme().toLowerCase(Locale.ROOT))) return false;
        if (u.getPort() != -1 && u.getPort() != 443) return false;
        if (u.getRawUserInfo() != null) return false;
        String host = u.getHost().toLowerCase(Locale.ROOT);
        String path = u.getRawPath() == null || u.getRawPath().isEmpty() ? "/" : u.getRawPath();
        if (path.contains("/../") || path.endsWith("/..") || path.contains("%2e%2e") || path.contains("%2E%2E")) return false;
        if (this == ONLINE) {
            return ONLINE_HOST.equals(host) && (path.equals("/app") || path.startsWith("/app/") || path.startsWith("/api/"));
        }
        return ASSET_HOST.equals(host) && path.startsWith(OFFLINE_ROOT);
    }

    /** True for a URL on the hosted demo's host (the only host the online mode talks to). */
    public static boolean isOnlineDemoHost(String url) {
        return ONLINE_HOST.equals(HubConfig.hostOf(url));
    }
}
