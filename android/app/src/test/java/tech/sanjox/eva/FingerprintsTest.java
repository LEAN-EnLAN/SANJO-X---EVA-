package tech.sanjox.eva;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNull;

import org.junit.Test;

public class FingerprintsTest {
    // SHA-256("abc"), FIPS 180-2 test vector
    static final String ABC = "BA:78:16:BF:8F:01:CF:EA:41:41:40:DE:5D:AE:22:23:B0:03:61:A3:96:17:7A:9C:B4:10:FF:61:F2:00:15:AD";

    @Test
    public void sha256OfDer() {
        assertEquals(ABC, Fingerprints.sha256("abc".getBytes()));
        assertNull(Fingerprints.sha256(null));
        assertNull(Fingerprints.sha256(new byte[0]));
    }

    @Test
    public void normalizeAcceptsCommonFormats() {
        assertEquals(ABC, Fingerprints.normalize(ABC));
        assertEquals(ABC, Fingerprints.normalize(ABC.toLowerCase()));
        assertEquals(ABC, Fingerprints.normalize(ABC.replace(":", "")));
        assertEquals(ABC, Fingerprints.normalize(ABC.replace(":", " ")));
        assertEquals("openssl output", ABC, Fingerprints.normalize("sha256 Fingerprint=" + ABC));
        assertNull(Fingerprints.normalize("AB:CD"));
        assertNull(Fingerprints.normalize(ABC + ":00"));
        assertNull(Fingerprints.normalize(ABC.replace('B', 'G')));
        assertNull(Fingerprints.normalize(null));
    }

    @Test
    public void prettyIsFourLinesOfEight() {
        String p = Fingerprints.pretty(ABC);
        String[] lines = p.split("\n");
        assertEquals(4, lines.length);
        assertEquals("BA:78:16:BF:8F:01:CF:EA", lines[0]);
        assertEquals(ABC, Fingerprints.normalize(p.replace("\n", ":")));
    }
}
