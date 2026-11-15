package tech.sanjox.eva;

import android.content.Context;
import android.content.SharedPreferences;

/** The remembered hub choice, the keep-screen-on setting and the pinned certificate fingerprints. */
public final class HubPrefs {
    private static final String FILE = "eva_hub";
    private static final String K_ADDRESS = "address";
    private static final String K_SCHEME = "scheme";
    private static final String K_SEAT = "seat";
    private static final String K_KEEP_ON = "keep_screen_on";
    private static final String K_PIN = "pin.";

    private final SharedPreferences sp;

    public HubPrefs(Context ctx) {
        sp = ctx.getApplicationContext().getSharedPreferences(FILE, Context.MODE_PRIVATE);
    }

    /** The saved hub, or null on first run (or if what was saved no longer parses). */
    public HubConfig load() {
        String address = sp.getString(K_ADDRESS, null);
        if (address == null) return null;
        try {
            return HubConfig.parse(address, sp.getString(K_SCHEME, HubConfig.HTTPS),
                    Seat.fromId(sp.getString(K_SEAT, null), Seat.DEFAULT));
        } catch (IllegalArgumentException e) {
            return null;
        }
    }

    public void save(HubConfig c) {
        sp.edit().putString(K_ADDRESS, c.address()).putString(K_SCHEME, c.scheme).putString(K_SEAT, c.seat.id).apply();
    }

    public String lastAddress() {
        return sp.getString(K_ADDRESS, "");
    }

    public String lastScheme() {
        return sp.getString(K_SCHEME, HubConfig.HTTPS);
    }

    public Seat lastSeat() {
        return Seat.fromId(sp.getString(K_SEAT, null), Seat.DEFAULT);
    }

    public boolean keepScreenOn() {
        return sp.getBoolean(K_KEEP_ON, true);
    }

    public void setKeepScreenOn(boolean on) {
        sp.edit().putBoolean(K_KEEP_ON, on).apply();
    }

    public String pin(String host) {
        return host == null ? null : sp.getString(K_PIN + HubConfig.normalizeHost(host), null);
    }

    // commit(): the pin must be on disk before the WebView is told to proceed.
    @android.annotation.SuppressLint("ApplySharedPref")
    public void savePin(String host, String fingerprint) {
        sp.edit().putString(K_PIN + HubConfig.normalizeHost(host), Fingerprints.normalize(fingerprint)).commit();
    }

    @android.annotation.SuppressLint("ApplySharedPref")
    public void forgetPin(String host) {
        sp.edit().remove(K_PIN + HubConfig.normalizeHost(host)).commit();
    }
}
