package tech.sanjox.eva;

import static org.junit.Assert.assertEquals;
import static tech.sanjox.eva.PinPolicy.Decision.ASK_USER;
import static tech.sanjox.eva.PinPolicy.Decision.PROCEED;
import static tech.sanjox.eva.PinPolicy.Decision.REJECT_CHANGED;
import static tech.sanjox.eva.PinPolicy.Decision.REJECT_INVALID;
import static tech.sanjox.eva.PinPolicy.Decision.REJECT_OTHER_HOST;

import org.junit.Test;

public class PinPolicyTest {
    static final String HUB = "192.168.0.10";
    static final String FP_A = Fingerprints.sha256("hub certificate A".getBytes());
    static final String FP_B = Fingerprints.sha256("impostor certificate B".getBytes());

    @Test
    public void firstConnectAsksTheUser() {
        assertEquals(ASK_USER, PinPolicy.decide(HUB, HUB, true, null, FP_A));
        assertEquals("empty stored value counts as no pin", ASK_USER, PinPolicy.decide(HUB, HUB, true, "", FP_A));
    }

    @Test
    public void matchingPinProceeds() {
        assertEquals(PROCEED, PinPolicy.decide(HUB, HUB, true, FP_A, FP_A));
        // stored in another format (lower case, no colons) still matches
        String raw = FP_A.replace(":", "").toLowerCase();
        assertEquals(PROCEED, PinPolicy.decide(HUB, HUB, true, raw, FP_A));
        assertEquals("host compare ignores case", PROCEED, PinPolicy.decide("eva.local", "EVA.LOCAL", true, FP_A, FP_A));
    }

    @Test
    public void changedCertificateIsRejectedNeverAsked() {
        assertEquals(REJECT_CHANGED, PinPolicy.decide(HUB, HUB, true, FP_A, FP_B));
    }

    @Test
    public void otherHostsAreNeverTrusted() {
        assertEquals(REJECT_OTHER_HOST, PinPolicy.decide(HUB, "192.168.0.11", true, null, FP_A));
        assertEquals(REJECT_OTHER_HOST, PinPolicy.decide(HUB, "evil.example.com", true, FP_A, FP_A));
        assertEquals(REJECT_OTHER_HOST, PinPolicy.decide(HUB, null, true, FP_A, FP_A));
        assertEquals(REJECT_OTHER_HOST, PinPolicy.decide(null, HUB, true, FP_A, FP_A));
    }

    @Test
    public void dateOrValidityErrorsAreRejectedEvenWhenPinned() {
        assertEquals(REJECT_INVALID, PinPolicy.decide(HUB, HUB, false, FP_A, FP_A));
        assertEquals(REJECT_INVALID, PinPolicy.decide(HUB, HUB, false, null, FP_A));
    }

    @Test
    public void unreadableCertificateIsRejected() {
        assertEquals(REJECT_INVALID, PinPolicy.decide(HUB, HUB, true, FP_A, null));
        assertEquals(REJECT_INVALID, PinPolicy.decide(HUB, HUB, true, null, "not-a-fingerprint"));
    }

    @Test
    public void corruptStoredPinDoesNotProceed() {
        // a garbage stored value normalizes to null → treated as no pin → the user is asked again, never auto-proceed
        assertEquals(ASK_USER, PinPolicy.decide(HUB, HUB, true, "garbage", FP_A));
    }

    @Test
    public void onlyProceedIsProceed() {
        // exhaustive over the inputs: PROCEED only when host matches, errors are trust-only and pin equals presented
        String[] hosts = {HUB, "other", null};
        boolean[] trust = {true, false};
        String[] pins = {null, FP_A, FP_B, "junk"};
        String[] presented = {FP_A, FP_B, null};
        for (String h : hosts)
            for (boolean t : trust)
                for (String p : pins)
                    for (String fp : presented) {
                        PinPolicy.Decision d = PinPolicy.decide(HUB, h, t, p, fp);
                        boolean expectProceed = HUB.equals(h) && t && fp != null && fp.equals(p);
                        assertEquals(h + " " + t + " " + p + " " + fp, expectProceed, d == PROCEED);
                    }
    }
}
