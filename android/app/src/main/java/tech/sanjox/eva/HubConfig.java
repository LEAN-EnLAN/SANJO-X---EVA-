package tech.sanjox.eva;

import java.net.URI;
import java.net.URISyntaxException;
import java.util.Locale;
import java.util.regex.Pattern;

/**
 * Which ship hub to open and where: scheme, host, port and seat. Pure Java (no Android types) so the
 * URL rules are unit-tested on the JVM.
 *
 * <p>Plain http is only accepted for private / local network hosts; see {@link #isPrivateHost}.
 */
public final class HubConfig {
    public static final int DEFAULT_PORT = 8080;
    public static final String HTTP = "http";
    public static final String HTTPS = "https";

    private static final Pattern IPV4 = Pattern.compile("^(\\d{1,3})\\.(\\d{1,3})\\.(\\d{1,3})\\.(\\d{1,3})$");
    private static final Pattern HOSTNAME =
            Pattern.compile("^(?=.{1,253}$)([a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)(\\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)*$");
    private static final Pattern IPV6 = Pattern.compile("^[0-9a-f:.]+(%[0-9a-z]+)?$");

    public final String scheme;
    /** Lower-case host; IPv6 without brackets. */
    public final String host;
    public final int port;
    public final Seat seat;

    public HubConfig(String scheme, String host, int port, Seat seat) {
        this.scheme = scheme;
        this.host = host;
        this.port = port;
        this.seat = seat;
    }

    /**
     * Parses what the user typed in the hub field.
     *
     * <ul>
     *   <li>{@code 192.168.0.10}, {@code 192.168.0.10:8080}, {@code eva-hub.local:8443}, {@code [fe80::1]:8080}:
     *       uses {@code scheme} and {@code seat} as chosen in the form.
     *   <li>A full URL ({@code https://192.168.0.10:8080/app?seat=3B}, {@code http://10.0.2.2:9280/dash}): its scheme
     *       and seat win over the form, so a URL copied from a QR code or a chat just works.
     * </ul>
     *
     * @throws IllegalArgumentException with a message fit to show the user.
     */
    public static HubConfig parse(String input, String scheme, Seat seat) {
        if (input == null || input.trim().isEmpty()) {
            throw new IllegalArgumentException("Enter the hub address, for example 192.168.0.10:8080.");
        }
        String s = input.trim();
        if (s.contains(" ")) throw new IllegalArgumentException("The hub address cannot contain spaces.");
        String chosenScheme = normalizeScheme(scheme);
        Seat chosenSeat = seat == null ? Seat.DEFAULT : seat;

        String lower = s.toLowerCase(Locale.ROOT);
        if (!lower.startsWith("http://") && !lower.startsWith("https://")) {
            if (lower.contains("://")) {
                throw new IllegalArgumentException("Only http:// or https:// addresses are supported.");
            }
            s = chosenScheme + "://" + s;
        }
        URI uri;
        try {
            uri = new URI(s);
        } catch (URISyntaxException e) {
            throw new IllegalArgumentException("That does not look like an address: " + input.trim());
        }
        String sch = uri.getScheme().toLowerCase(Locale.ROOT);
        String rawHost = uri.getHost();
        if (rawHost == null) {
            // java.net.URI leaves host null for some invalid authorities (e.g. "a_b:80"): retry by hand.
            throw new IllegalArgumentException("That does not look like a host name or IP address: " + input.trim());
        }
        if (uri.getUserInfo() != null) throw new IllegalArgumentException("User names in the address are not supported.");
        String host = normalizeHost(rawHost);
        if (!isValidHost(host)) {
            throw new IllegalArgumentException("That does not look like a host name or IP address: " + rawHost);
        }
        int port = uri.getPort();
        if (port == -1) {
            // A bare host means the Eva default port, not the scheme default.
            port = input.trim().toLowerCase(Locale.ROOT).contains("://") ? defaultPortFor(sch) : DEFAULT_PORT;
        }
        if (port < 1 || port > 65535) throw new IllegalArgumentException("The port must be between 1 and 65535.");

        Seat finalSeat = chosenSeat;
        String path = uri.getRawPath();
        if (path != null && !path.isEmpty() && !path.equals("/")) {
            finalSeat = Seat.fromPath(path, uri.getRawQuery(), chosenSeat);
        } else if (uri.getRawQuery() != null) {
            finalSeat = Seat.fromPath("/app", uri.getRawQuery(), chosenSeat);
        }
        if (HTTP.equals(sch) && !isPrivateHost(host)) {
            throw new IllegalArgumentException(
                    "Plain http is only allowed on a local network address (192.168.x.x, 10.x.x.x, 172.16-31.x.x, "
                            + ".local). Use https for " + host + ".");
        }
        return new HubConfig(sch, host, port, finalSeat);
    }

    static String normalizeScheme(String scheme) {
        return HTTP.equalsIgnoreCase(scheme == null ? "" : scheme.trim()) ? HTTP : HTTPS;
    }

    static String normalizeHost(String host) {
        String h = host.trim().toLowerCase(Locale.ROOT);
        if (h.startsWith("[") && h.endsWith("]")) h = h.substring(1, h.length() - 1);
        if (h.endsWith(".")) h = h.substring(0, h.length() - 1);
        return h;
    }

    static boolean isValidHost(String host) {
        if (host.isEmpty()) return false;
        if (host.contains(":")) return IPV6.matcher(host).matches();
        if (IPV4.matcher(host).matches()) return parseIpv4(host) != null;
        if (host.matches("^[0-9.]+$")) return false; // "192.168.1" or "1.2.3.4.5"
        return HOSTNAME.matcher(host).matches();
    }

    static int defaultPortFor(String scheme) {
        return HTTPS.equals(scheme) ? 443 : 80;
    }

    /** "host:port" as shown in the form, with brackets for IPv6. */
    public String address() {
        return hostForUrl() + ":" + port;
    }

    private String hostForUrl() {
        return host.contains(":") ? "[" + host + "]" : host;
    }

    /** Origin, e.g. {@code https://192.168.0.10:8080} (port always explicit). */
    public String origin() {
        return scheme + "://" + hostForUrl() + ":" + port;
    }

    /** The page to open: origin plus the seat's path. */
    public String startUrl() {
        return origin() + seat.path;
    }

    public boolean isHttps() {
        return HTTPS.equals(scheme);
    }

    /**
     * True when {@code url} has exactly this hub's scheme, host and port (default ports filled in). Used to
     * grant mic/camera only to the configured origin and to keep navigation inside the app.
     */
    public boolean isSameOrigin(String url) {
        if (url == null) return false;
        try {
            URI u = new URI(url.trim());
            if (u.getScheme() == null || u.getHost() == null) return false;
            String sch = u.getScheme().toLowerCase(Locale.ROOT);
            int p = u.getPort() == -1 ? defaultPortFor(sch) : u.getPort();
            return sch.equals(scheme) && normalizeHost(u.getHost()).equals(host) && p == port;
        } catch (URISyntaxException e) {
            return false;
        }
    }

    /** Host of a URL, lower-case and without IPv6 brackets, or null. */
    public static String hostOf(String url) {
        if (url == null) return null;
        try {
            String h = new URI(url.trim()).getHost();
            return h == null ? null : normalizeHost(h);
        } catch (URISyntaxException e) {
            return null;
        }
    }

    /**
     * Cleartext policy for every request the WebView makes: https (and non-network schemes) always pass; http only
     * to a private / local host. The network security config has to permit cleartext globally because it cannot
     * express IP ranges, so this is where the range check lives.
     */
    public static boolean isCleartextAllowed(String url) {
        if (url == null) return false;
        String lower = url.trim().toLowerCase(Locale.ROOT);
        if (!lower.startsWith("http://") && !lower.startsWith("ws://")) return true;
        String host = hostOf(url);
        return host != null && isPrivateHost(host);
    }

    /**
     * Private or local network host: RFC 1918 (10/8, 172.16/12, 192.168/16), loopback, link-local (169.254/16),
     * CGNAT / tailnets (100.64/10), IPv6 loopback, link-local (fe80::/10) and unique-local (fc00::/7), plus
     * {@code localhost} and mDNS / home names ({@code .local}, {@code .lan}, {@code .home.arpa}). The Android
     * emulator's host alias 10.0.2.2 is inside 10/8. No DNS lookup is made.
     */
    public static boolean isPrivateHost(String rawHost) {
        if (rawHost == null) return false;
        String host = normalizeHost(rawHost);
        if (host.isEmpty()) return false;
        int[] v4 = parseIpv4(host);
        if (v4 != null) {
            int a = v4[0], b = v4[1];
            return a == 10
                    || (a == 172 && b >= 16 && b <= 31)
                    || (a == 192 && b == 168)
                    || a == 127
                    || (a == 169 && b == 254)
                    || (a == 100 && b >= 64 && b <= 127);
        }
        if (host.contains(":")) {
            String h = host;
            int pct = h.indexOf('%');
            if (pct >= 0) h = h.substring(0, pct);
            if (h.equals("::1") || h.equals("0:0:0:0:0:0:0:1")) return true;
            if (h.startsWith("::ffff:")) return isPrivateHost(h.substring(7));
            String first = h.split(":", -1)[0];
            if (first.isEmpty()) return false;
            int word;
            try {
                word = Integer.parseInt(first, 16);
            } catch (NumberFormatException e) {
                return false;
            }
            return (word & 0xffc0) == 0xfe80 || (word & 0xfe00) == 0xfc00;
        }
        return host.equals("localhost")
                || host.endsWith(".localhost")
                || host.endsWith(".local")
                || host.endsWith(".lan")
                || host.endsWith(".home.arpa");
    }

    private static int[] parseIpv4(String host) {
        java.util.regex.Matcher m = IPV4.matcher(host);
        if (!m.matches()) return null;
        int[] out = new int[4];
        for (int i = 0; i < 4; i++) {
            String part = m.group(i + 1);
            if (part.length() > 1 && part.startsWith("0")) return null; // no octal-looking octets
            int v = Integer.parseInt(part);
            if (v > 255) return null;
            out[i] = v;
        }
        return out;
    }

    @Override
    public boolean equals(Object o) {
        if (!(o instanceof HubConfig)) return false;
        HubConfig c = (HubConfig) o;
        return scheme.equals(c.scheme) && host.equals(c.host) && port == c.port && seat == c.seat;
    }

    @Override
    public int hashCode() {
        return startUrl().hashCode();
    }

    @Override
    public String toString() {
        return startUrl();
    }
}
