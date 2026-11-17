// Text to speech for the public demo: Groq Orpheus (English). Spanish, or any failure, returns 204
// and the client speaks with the browser's own speechSynthesis. Text is capped and passed through
// the same copy rules as chat replies.
import { preflight, limited, jsonBody } from "../_lib/http";
import { GROQ_BASE_URL, redact } from "../_lib/groq";
import { copyRules } from "../_lib/guard";

const VOICE = process.env.EVA_TTS_VOICE ?? "hannah";

export default async function handler(req: any, res: any) {
  if (preflight(req, res)) return;
  const wait = limited(req, "tts", 40);
  if (wait) { res.setHeader("retry-after", String(wait)); return res.status(429).json({ error: "rate_limited" }); }
  const body = jsonBody(req);
  const text = copyRules(String(body.text ?? "")).replace(/\s+/g, " ").trim().slice(0, 400);
  if (!text) return res.status(400).json({ error: "empty" });
  if (body.lang === "es") return res.status(204).end();
  const key = process.env.GROQ_API_KEY;
  if (!key) return res.status(204).end();
  const ctrl = new AbortController();
  const t = setTimeout(() => ctrl.abort(), 20_000);
  try {
    const r = await fetch(`${GROQ_BASE_URL}/audio/speech`, {
      method: "POST", signal: ctrl.signal,
      headers: { "content-type": "application/json", authorization: `Bearer ${key}` },
      body: JSON.stringify({ model: "canopylabs/orpheus-v1-english", input: text, voice: VOICE, response_format: "wav" }),
    });
    if (!r.ok) {
      const j: any = await r.json().catch(() => ({}));
      console.error("tts upstream", r.status, redact(String(j?.error?.code ?? j?.error?.message ?? "")));
      return res.status(204).end();
    }
    const audio = Buffer.from(await r.arrayBuffer());
    res.setHeader("content-type", "audio/wav");
    res.setHeader("content-length", String(audio.length));
    return res.status(200).send(audio);
  } catch (e) {
    console.error("tts failed", redact(e instanceof Error ? e.message : "unknown"));
    return res.status(204).end();
  } finally { clearTimeout(t); }
}
