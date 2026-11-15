package tech.sanjox.eva;

/** Where the app opens on the hub: a passenger seat, the crew dashboard or the AED station. */
public enum Seat {
    ANA("4B", "Ana", "Seat 4B", "/app?seat=4B"),
    LEO("3B", "Leo", "Seat 3B", "/app?seat=3B"),
    TOMAS("1B", "Tomás", "Seat 1B", "/app?seat=1B"),
    DASH("dash", "Crew dashboard", "Mara, crew lead", "/dash"),
    STATION("station", "Station", "AED station screen", "/station"),
    /** The phone strapped to a forearm stands in for the wristband: locked landscape, wristband UI. */
    WRIST("wrist", "Wristband (landscape)", "Ana · 4B, the band's own screen", "/app/wrist?seat=4B&mode=wristband");

    /** Stable id stored in preferences and accepted in a pasted URL (?seat=). */
    public final String id;
    public final String title;
    public final String subtitle;
    /** Path and query on the hub, always starting with "/". */
    public final String path;

    Seat(String id, String title, String subtitle, String path) {
        this.id = id;
        this.title = title;
        this.subtitle = subtitle;
        this.path = path;
    }

    public static final Seat DEFAULT = ANA;

    /** Seat for a stored id (case-insensitive), or {@code fallback}. */
    public static Seat fromId(String id, Seat fallback) {
        if (id == null) return fallback;
        for (Seat s : values()) {
            if (s.id.equalsIgnoreCase(id.trim())) return s;
        }
        return fallback;
    }

    /**
     * Seat for a hub URL path and query, e.g. "/app" + "seat=3B", "/dash", "/station".
     * Returns {@code fallback} when the URL names none.
     */
    public static Seat fromPath(String path, String query, Seat fallback) {
        String p = path == null ? "" : path.toLowerCase(java.util.Locale.ROOT);
        if (p.startsWith("/dash")) return DASH;
        if (p.startsWith("/station")) return STATION;
        if (p.startsWith("/app/wrist") || (query != null && query.toLowerCase(java.util.Locale.ROOT).contains("mode=wristband"))) return WRIST;
        if (query != null) {
            for (String kv : query.split("&")) {
                int eq = kv.indexOf('=');
                if (eq > 0 && kv.substring(0, eq).equalsIgnoreCase("seat")) {
                    Seat s = fromId(kv.substring(eq + 1), null);
                    if (s != null && s != DASH && s != STATION && s != WRIST) return s;
                }
            }
        }
        if (p.startsWith("/app")) return ANA;
        return fallback;
    }
}
