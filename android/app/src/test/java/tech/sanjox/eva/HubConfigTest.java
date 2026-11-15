package tech.sanjox.eva;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNotEquals;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

import org.junit.Test;

public class HubConfigTest {

    @Test
    public void bareIpUsesFormSchemeSeatAndEvaDefaultPort() {
        HubConfig c = HubConfig.parse("192.168.0.10", "https", Seat.LEO);
        assertEquals("https", c.scheme);
        assertEquals("192.168.0.10", c.host);
        assertEquals(8080, c.port);
        assertEquals(Seat.LEO, c.seat);
        assertEquals("https://192.168.0.10:8080/app?seat=3B", c.startUrl());
    }

    @Test
    public void hostAndPortTrimmedAndLowerCased() {
        HubConfig c = HubConfig.parse("  Eva-Hub.LOCAL:9280  ", "http", Seat.ANA);
        assertEquals("eva-hub.local", c.host);
        assertEquals(9280, c.port);
        assertEquals("http://eva-hub.local:9280/app?seat=4B", c.startUrl());
        assertEquals("eva-hub.local:9280", c.address());
    }

    @Test
    public void everySeatHasItsPath() {
        assertEquals("http://10.0.2.2:9280/app?seat=4B", HubConfig.parse("10.0.2.2:9280", "http", Seat.ANA).startUrl());
        assertEquals("http://10.0.2.2:9280/app?seat=3B", HubConfig.parse("10.0.2.2:9280", "http", Seat.LEO).startUrl());
        assertEquals("http://10.0.2.2:9280/app?seat=1B", HubConfig.parse("10.0.2.2:9280", "http", Seat.TOMAS).startUrl());
        assertEquals("http://10.0.2.2:9280/dash", HubConfig.parse("10.0.2.2:9280", "http", Seat.DASH).startUrl());
        assertEquals("http://10.0.2.2:9280/station", HubConfig.parse("10.0.2.2:9280", "http", Seat.STATION).startUrl());
    }

    @Test
    public void nullSchemeOrSeatFallsBackToHttpsAndAna() {
        HubConfig c = HubConfig.parse("192.168.1.5:8080", null, null);
        assertEquals("https", c.scheme);
        assertEquals(Seat.ANA, c.seat);
    }

    @Test
    public void fullUrlSchemeAndSeatWinOverTheForm() {
        HubConfig c = HubConfig.parse("https://192.168.0.10:8080/app?seat=1B", "http", Seat.ANA);
        assertEquals("https", c.scheme);
        assertEquals(Seat.TOMAS, c.seat);
        assertEquals("https://192.168.0.10:8080/app?seat=1B", c.startUrl());

        assertEquals(Seat.DASH, HubConfig.parse("http://10.0.2.2:9280/dash", "https", Seat.LEO).seat);
        assertEquals(Seat.STATION, HubConfig.parse("http://10.0.2.2:9280/station", "https", Seat.LEO).seat);
        assertEquals(Seat.ANA, HubConfig.parse("http://10.0.2.2:9280/app", "https", Seat.LEO).seat);
        assertEquals("seat=3b lower case", Seat.LEO, HubConfig.parse("http://10.0.2.2:9280/app?seat=3b", "https", Seat.ANA).seat);
        // URL without path keeps the seat chosen in the form
        assertEquals(Seat.LEO, HubConfig.parse("http://10.0.2.2:9280/", "https", Seat.LEO).seat);
    }

    @Test
    public void fullUrlWithoutPortUsesSchemeDefault() {
        assertEquals(443, HubConfig.parse("https://eva.local/app", "http", Seat.ANA).port);
        assertEquals(80, HubConfig.parse("http://eva.local/app", "http", Seat.ANA).port);
    }

    @Test
    public void ipv6InBrackets() {
        HubConfig c = HubConfig.parse("[fe80::1]:8080", "http", Seat.ANA);
        assertEquals("fe80::1", c.host);
        assertEquals("http://[fe80::1]:8080/app?seat=4B", c.startUrl());
        assertEquals("[fe80::1]:8080", c.address());
    }

    @Test
    public void rejectsBadInput() {
        String[] bad = {
            "", "   ", "192.168.0.10:0", "192.168.0.10:70000", "192.168.0.10:abc", "192.168 .0.10",
            "ftp://192.168.0.10", "192.168.1", "999.1.1.1", "1.2.3.4.5", "user@192.168.0.10:8080",
            "-bad-.local", "under_score:8080", "http://"
        };
        for (String b : bad) {
            assertThrows("should reject [" + b + "]", IllegalArgumentException.class, () -> HubConfig.parse(b, "http", Seat.ANA));
        }
        assertThrows(IllegalArgumentException.class, () -> HubConfig.parse(null, "http", Seat.ANA));
    }

    @Test
    public void plainHttpOnlyToPrivateHosts() {
        IllegalArgumentException e = assertThrows(IllegalArgumentException.class,
                () -> HubConfig.parse("8.8.8.8:8080", "http", Seat.ANA));
        assertTrue(e.getMessage().contains("https"));
        assertThrows(IllegalArgumentException.class, () -> HubConfig.parse("http://example.com/app", "https", Seat.ANA));
        // https to a public name is fine
        assertEquals("example.com", HubConfig.parse("example.com:8443", "https", Seat.ANA).host);
    }

    @Test
    public void privateHostRanges() {
        String[] priv = {
            "10.0.2.2", "10.255.255.255", "172.16.0.1", "172.31.255.254", "192.168.0.10", "192.168.137.1",
            "127.0.0.1", "169.254.10.20", "100.64.0.1", "100.127.255.255", "localhost", "eva-hub.local", "hub.lan",
            "laptop.home.arpa", "::1", "fe80::1", "FE80::abcd%wlan0", "fd12:3456::1", "fc00::1", "::ffff:192.168.1.2",
            "[fe80::1]"
        };
        for (String h : priv) assertTrue(h, HubConfig.isPrivateHost(h));
        String[] pub = {
            "8.8.8.8", "172.15.255.255", "172.32.0.1", "192.169.0.1", "11.0.0.1", "100.63.255.255", "100.128.0.1",
            "example.com", "local.example.com", "localhost.example.com", "2001:4860::8888", "::ffff:8.8.8.8", "",
            "256.1.1.1"
        };
        for (String h : pub) assertFalse(h, HubConfig.isPrivateHost(h));
        assertFalse(HubConfig.isPrivateHost(null));
    }

    @Test
    public void cleartextPolicy() {
        assertTrue(HubConfig.isCleartextAllowed("https://example.com/x"));
        assertTrue(HubConfig.isCleartextAllowed("http://192.168.0.10:8080/static/app.css"));
        assertTrue(HubConfig.isCleartextAllowed("ws://10.0.2.2:9280/link"));
        assertTrue(HubConfig.isCleartextAllowed("data:image/png;base64,AAAA"));
        assertTrue(HubConfig.isCleartextAllowed("blob:https://192.168.0.10:8080/abc"));
        assertFalse(HubConfig.isCleartextAllowed("http://example.com/tracker.js"));
        assertFalse(HubConfig.isCleartextAllowed("HTTP://8.8.8.8/"));
        assertFalse(HubConfig.isCleartextAllowed("ws://example.com/socket"));
        assertFalse(HubConfig.isCleartextAllowed(null));
    }

    @Test
    public void sameOrigin() {
        HubConfig https = HubConfig.parse("192.168.0.10:8080", "https", Seat.ANA);
        assertTrue(https.isSameOrigin("https://192.168.0.10:8080/"));
        assertTrue(https.isSameOrigin("https://192.168.0.10:8080"));
        assertTrue(https.isSameOrigin("https://192.168.0.10:8080/app?seat=4B#eva"));
        assertTrue(https.isSameOrigin("HTTPS://192.168.0.10:8080/dash"));
        assertFalse(https.isSameOrigin("http://192.168.0.10:8080/"));
        assertFalse(https.isSameOrigin("https://192.168.0.10:8081/"));
        assertFalse(https.isSameOrigin("https://192.168.0.11:8080/"));
        assertFalse(https.isSameOrigin("https://192.168.0.10.evil.com:8080/"));
        assertFalse(https.isSameOrigin("https://192.168.0.10/"));
        assertFalse(https.isSameOrigin("not a url"));
        assertFalse(https.isSameOrigin(""));
        assertFalse(https.isSameOrigin(null));

        HubConfig on443 = HubConfig.parse("https://eva.local/app", "https", Seat.ANA);
        assertTrue("default port filled in", on443.isSameOrigin("https://eva.local/"));
        assertTrue(on443.isSameOrigin("https://eva.local:443/"));
        assertEquals("https://eva.local:443", on443.origin());
    }

    @Test
    public void hostOf() {
        assertEquals("192.168.0.10", HubConfig.hostOf("https://192.168.0.10:8080/app"));
        assertEquals("fe80::1", HubConfig.hostOf("https://[fe80::1]:8080/"));
        assertEquals("eva.local", HubConfig.hostOf("https://EVA.local./"));
        assertNull(HubConfig.hostOf("nonsense"));
        assertNull(HubConfig.hostOf(null));
    }

    @Test
    public void equalityCoversSeat() {
        HubConfig a = HubConfig.parse("192.168.0.10:8080", "https", Seat.ANA);
        assertEquals(a, HubConfig.parse("https://192.168.0.10:8080/app?seat=4B", "http", Seat.LEO));
        assertNotEquals(a, HubConfig.parse("192.168.0.10:8080", "https", Seat.LEO));
        assertNotEquals(a, HubConfig.parse("192.168.0.10:8080", "http", Seat.ANA));
    }

    @Test
    public void roundTripThroughStoredAddress() {
        // HubPrefs stores address() + scheme + seat id and parses them back.
        for (String in : new String[] {"192.168.0.10:8080", "[fd00::5]:9000", "eva.local:8443", "https://eva.local/dash"}) {
            HubConfig c = HubConfig.parse(in, "https", Seat.TOMAS);
            HubConfig back = HubConfig.parse(c.address(), c.scheme, Seat.fromId(c.seat.id, Seat.ANA));
            assertEquals(in, c, back);
        }
    }
}
