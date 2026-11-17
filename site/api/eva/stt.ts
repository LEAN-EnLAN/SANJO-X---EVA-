// Speech to text for the public demo: Groq whisper-large-v3-turbo. Audio arrives as base64 JSON,
// at most 2.5 MB (about a minute of opus). Returns { text } only; the client then calls /api/eva/chat,
// which runs the topic guard. The key comes from GROQ_API_KEY and is never logged or returned.
import { preflight, limited, jsonBody } from "../_lib/http";
import { GROQ_BASE_URL, redact } from "../_lib/groq";

const MAX_BYTES = 2_500_000;
const TYPES: Record<string, string> = { "audio/webm": "webm", "audio/ogg": "ogg", "audio/mp4": "m4a", "audio/mpeg": "mp3", "audio/wav": "wav", "audio/x-wav": "wav", "audio/aac": "m4a" };

export default async function handler(req: any, res: any) {
  if (preflight(req, res)) return;
  const wait = limited(req, "stt", 15);
  if (wait) { res.setHeader("retry-after", String(wait)); return res.status(429).json({ error: "rate_limited" }); }
  const body = jsonBody(req);
  const mime = String(body.mime ?? "").split(";")[0].trim().toLowerCase();
  const ext = TYPES[mime];
  if (!ext || typeof body.audio !== "string") return res.status(400).json({ error: "bad_audio" });
  const buf = Buffer.from(body.audio, "base64");
  if (buf.length < 200 || buf.length > MAX_BYTES) return res.status(413).json({ error: "audio_size" });
  const key = process.env.GROQ_API_KEY;
  if (!key) return res.status(503).json({ error: "eva_offline" });
  const form = new FormData();
  form.append("model", "whisper-large-v3-turbo");
  form.append("response_format", "json");
  if (body.lang === "es" || body.lang === "en") form.append("language", body.lang);
  form.append("file", new Blob([buf], { type: mime }), `voice.${ext}`);
  const ctrl = new AbortController();
  const t = setTimeout(() => ctrl.abort(), 20_000);
  try {
    const r = await fetch(`${GROQ_BASE_URL}/audio/transcriptions`, { method: "POST", headers: { authorization: `Bearer ${key}` }, body: form, signal: ctrl.signal });
    const j: any = await r.json().catch(() => ({}));
    if (!r.ok) { console.error("stt upstream", r.status, redact(String(j?.error?.message ?? ""))); return res.status(502).json({ error: "stt_failed" }); }
    return res.status(200).json({ text: String(j.text ?? "").trim().slice(0, 1000) });
  } catch (e) {
    console.error("stt failed", redact(e instanceof Error ? e.message : "unknown"));
    return res.status(503).json({ error: "eva_offline" });
  } finally { clearTimeout(t); }
}
