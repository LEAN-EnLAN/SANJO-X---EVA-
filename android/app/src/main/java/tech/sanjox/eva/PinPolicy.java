package tech.sanjox.eva;

/**
 * Trust-on-first-use decision for the hub's self-signed certificate. Pure Java, unit-tested.
 *
 * <p>The WebView reports a certificate it cannot verify; the app proceeds only when the certificate's SHA-256
 * fingerprint equals the one the user approved for that host. Nothing ever proceeds without a pin match or an
 * explicit "Trust" from the user, and never for a host other than the configured hub.
 */
public final class PinPolicy {
    private PinPolicy() {}

    public enum Decision {
        /** Fingerprint matches the pin for this host: proceed. */
        PROCEED,
        /** No pin yet for the hub: show the fingerprint and ask the user. Cancel unless they accept. */
        ASK_USER,
        /** A pin exists and the certificate differs: cancel and explain (possible impostor, or a recreated cert). */
        REJECT_CHANGED,
        /** The error is for a host that is not the configured hub: cancel, never ask. */
        REJECT_OTHER_HOST,
        /** Expired, not yet valid, malformed, or no certificate to fingerprint: cancel. */
        REJECT_INVALID
    }

    /**
     * @param hubHost the configured hub host (lower case)
     * @param errorHost host of the URL that raised the SSL error
     * @param onlyTrustErrors true when the only problems are "untrusted issuer" and/or "name mismatch" (what a
     *     self-signed hub certificate produces); false for date or validity errors
     * @param pinned stored fingerprint for this host, or null
     * @param presented fingerprint of the certificate just presented, or null when it could not be read
     */
    public static Decision decide(
            String hubHost, String errorHost, boolean onlyTrustErrors, String pinned, String presented) {
        if (hubHost == null || errorHost == null || !HubConfig.normalizeHost(errorHost).equals(HubConfig.normalizeHost(hubHost))) {
            return Decision.REJECT_OTHER_HOST;
        }
        String now = Fingerprints.normalize(presented);
        if (now == null || !onlyTrustErrors) return Decision.REJECT_INVALID;
        String pin = Fingerprints.normalize(pinned);
        if (pin == null) return Decision.ASK_USER;
        return pin.equals(now) ? Decision.PROCEED : Decision.REJECT_CHANGED;
    }
}
