package tech.sanjox.eva;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNull;

import org.junit.Test;

public class SeatTest {
    @Test
    public void fromId() {
        assertEquals(Seat.ANA, Seat.fromId("4B", null));
        assertEquals(Seat.LEO, Seat.fromId("3b", null));
        assertEquals(Seat.TOMAS, Seat.fromId(" 1B ", null));
        assertEquals(Seat.DASH, Seat.fromId("dash", null));
        assertEquals(Seat.STATION, Seat.fromId("STATION", null));
        assertNull(Seat.fromId("9Z", null));
        assertEquals(Seat.ANA, Seat.fromId(null, Seat.ANA));
    }

    @Test
    public void fromPath() {
        assertEquals(Seat.LEO, Seat.fromPath("/app", "seat=3B", Seat.ANA));
        assertEquals(Seat.TOMAS, Seat.fromPath("/app", "x=1&seat=1B", Seat.ANA));
        assertEquals(Seat.ANA, Seat.fromPath("/app", "seat=7C", Seat.LEO));
        assertEquals("?seat=dash is not a passenger seat", Seat.ANA, Seat.fromPath("/app", "seat=dash", Seat.LEO));
        assertEquals(Seat.DASH, Seat.fromPath("/dash", null, Seat.ANA));
        assertEquals(Seat.DASH, Seat.fromPath("/DASH/", "", Seat.ANA));
        assertEquals(Seat.STATION, Seat.fromPath("/station", null, Seat.ANA));
        assertEquals(Seat.LEO, Seat.fromPath("/other", null, Seat.LEO));
        assertEquals(Seat.LEO, Seat.fromPath(null, null, Seat.LEO));
    }

    @Test
    public void idsAreUniqueAndPathsAbsolute() {
        java.util.Set<String> ids = new java.util.HashSet<>();
        for (Seat s : Seat.values()) {
            assertEquals(true, ids.add(s.id.toLowerCase()));
            assertEquals('/', s.path.charAt(0));
        }
    }
}
