package tech.sanjox.eva;

/**
 * Eva's status from the hub's {@code GET /api/eva/status}, for a 3-second toast after the hub first answers.
 * Pure Java (no Android types) so it is unit-tested. The request runs inside the WebView, so it uses the same
 * origin and the same certificate pin as the page; the body never holds a secret.
 */
public final class EvaStatus {
    private EvaStatus() {}

    /** Synchronous same-origin fetch evaluated in the page; returns the JSON text (or "" on failure). */
    public static final String FETCH_JS =
            "(function(){try{var x=new XMLHttpRequest();x.open('GET','/api/eva/status',false);x.send();"
                    + "return x.status===200?x.responseText:''}catch(e){return ''}})()";

    /** evaluateJavascript hands back a JSON-encoded string: strip the quotes and unescape it. */
    public static String unquote(String v) {
        if (v == null || v.equals("null")) return "";
        if (v.length() < 2 || v.charAt(0) != '"') return v;
        StringBuilder out = new StringBuilder();
        for (int i = 1; i < v.length() - 1; i++) {
            char c = v.charAt(i);
            if (c == '\\' && i + 1 < v.length() - 1) {
                char n = v.charAt(++i);
                if (n == 'u' && i + 4 < v.length() - 1) {
                    out.append((char) Integer.parseInt(v.substring(i + 1, i + 5), 16));
                    i += 4;
                } else {
                    out.append(n == 'n' ? '\n' : n == 't' ? '\t' : n);
                }
            } else {
                out.append(c);
            }
        }
        return out.toString();
    }

    /** The toast for a status body: "Eva online", "Eva: on-board rules (…)", or null if the hub did not say. */
    public static String toastFor(String json) {
        if (json == null) return null;
        String mode = field(json, "mode");
        if ("online".equals(mode)) return "Eva online";
        if (!"rules".equals(mode)) return null;
        String reason = field(json, "reason");
        if ("no_key".equals(reason)) return "Eva: on-board rules (the hub has no key)";
        if ("bad_key".equals(reason)) return "Eva: on-board rules (the hub's key was refused)";
        if ("unreachable".equals(reason)) return "Eva: on-board rules (no internet from the hub)";
        if ("offline_flag".equals(reason)) return "Eva: on-board rules (the hub runs offline)";
        return "Eva: on-board rules";
    }

    /** A flat string field from a small JSON object (enough for this body; no JSON library in the APK). */
    static String field(String json, String key) {
        String k = "\"" + key + "\"";
        int i = json.indexOf(k);
        if (i < 0) return null;
        int colon = json.indexOf(':', i + k.length());
        if (colon < 0) return null;
        int q1 = json.indexOf('"', colon + 1);
        if (q1 < 0) return null;
        int q2 = json.indexOf('"', q1 + 1);
        return q2 < 0 ? null : json.substring(q1 + 1, q2);
    }
}
