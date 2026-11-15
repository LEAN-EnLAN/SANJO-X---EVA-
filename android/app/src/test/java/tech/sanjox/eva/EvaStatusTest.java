package tech.sanjox.eva;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNull;

import org.junit.Test;

public class EvaStatusTest {
    private static final String RULES = "{\"mode\":\"rules\",\"reason\":\"no_key\",\"label\":\"On-board rules\",\"hint\":\"Set GROQ_API_KEY and restart\",\"key_source\":\"none\",\"version\":\"0.3.2\"}";

    @Test
    public void onlineAndRulesToasts() {
        assertEquals("Eva online", EvaStatus.toastFor("{\"mode\":\"online\",\"reason\":\"ok\"}"));
        assertEquals("Eva: on-board rules (the hub has no key)", EvaStatus.toastFor(RULES));
        assertEquals("Eva: on-board rules (the hub's key was refused)", EvaStatus.toastFor("{\"mode\":\"rules\",\"reason\":\"bad_key\"}"));
        assertEquals("Eva: on-board rules (no internet from the hub)", EvaStatus.toastFor("{\"mode\":\"rules\",\"reason\":\"unreachable\"}"));
    }

    @Test
    public void nothingWhenTheHubDidNotSay() {
        assertNull(EvaStatus.toastFor(""));
        assertNull(EvaStatus.toastFor(null));
        assertNull(EvaStatus.toastFor("<html>"));
    }

    @Test
    public void unquotesWhatEvaluateJavascriptReturns() {
        String encoded = "\"{\\\"mode\\\":\\\"rules\\\",\\\"reason\\\":\\\"no_key\\\",\\\"hint\\\":\\\"a\\\\u00e9\\\"}\"";
        String json = EvaStatus.unquote(encoded);
        assertEquals("rules", EvaStatus.field(json, "mode"));
        assertEquals("Eva: on-board rules (the hub has no key)", EvaStatus.toastFor(json));
        assertEquals("", EvaStatus.unquote("null"));
    }
}
