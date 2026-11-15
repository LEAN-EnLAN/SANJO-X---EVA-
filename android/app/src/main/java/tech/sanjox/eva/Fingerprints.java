package tech.sanjox.eva;

import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.Locale;

/** SHA-256 certificate fingerprints in the usual "AB:CD:…" form (as printed by openssl / browsers). */
public final class Fingerprints {
    private Fingerprints() {}

    /** SHA-256 of the DER-encoded certificate, upper-case hex pairs joined by ':'. */
    public static String sha256(byte[] der) {
        if (der == null || der.length == 0) return null;
        try {
            byte[] d = MessageDigest.getInstance("SHA-256").digest(der);
            StringBuilder sb = new StringBuilder(d.length * 3);
            for (int i = 0; i < d.length; i++) {
                if (i > 0) sb.append(':');
                sb.append(String.format(Locale.ROOT, "%02X", d[i] & 0xff));
            }
            return sb.toString();
        } catch (NoSuchAlgorithmException e) {
            throw new IllegalStateException(e);
        }
    }

    /**
     * Canonical form for comparison: 64 upper-case hex digits in colon-separated pairs. Accepts colons, spaces or
     * nothing between pairs and an optional "SHA256 Fingerprint=" prefix (openssl output). Null if not a SHA-256.
     */
    public static String normalize(String fp) {
        if (fp == null) return null;
        String s = fp.trim();
        int eq = s.lastIndexOf('=');
        if (eq >= 0) s = s.substring(eq + 1);
        String hex = s.replaceAll("[\\s:]", "").toUpperCase(Locale.ROOT);
        if (!hex.matches("^[0-9A-F]{64}$")) return null;
        StringBuilder sb = new StringBuilder(95);
        for (int i = 0; i < 64; i += 2) {
            if (i > 0) sb.append(':');
            sb.append(hex, i, i + 2);
        }
        return sb.toString();
    }

    /** Four lines of eight pairs, easier to compare by eye in a dialog. */
    public static String pretty(String fp) {
        String n = normalize(fp);
        if (n == null) return String.valueOf(fp);
        String[] pairs = n.split(":");
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < pairs.length; i++) {
            if (i > 0) sb.append(i % 8 == 0 ? "\n" : ":");
            sb.append(pairs[i]);
        }
        return sb.toString();
    }
}
