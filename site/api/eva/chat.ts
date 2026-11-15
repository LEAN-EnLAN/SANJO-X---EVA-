// Eva chat for the public demo (Vercel Function, Node). Same topic guard as the ship app and the
// Firebase kit: space, spaceflight, health and medicine only; injection stripping; length limit;
// treatment requests refused; emergencies always pointed to the crew lead / Help button.
// The Groq key comes only from the GROQ_API_KEY environment variable and is never logged or returned.
import { screen, guardReply, SYSTEM_PROMPT, asUserData } from "../_lib/guard";
import { chat, redact, CHAT_MODEL, type Turn } from "../_lib/groq";

const WINDOW_MS = 60_000;
const MAX_PER_WINDOW = 12;
const hits = new Map<string, number[]>();

function limited(ip: string): number {
  const now = Date.now();
  const list = (hits.get(ip) ?? []).filter((t) => now - t < WINDOW_MS);
  if (list.length >= MAX_PER_WINDOW) {
    hits.set(ip, list);
    return Math.ceil((WINDOW_MS - (now - list[0])) / 1000);
  }
  list.push(now);
  hits.set(ip, list);
  if (hits.size > 5000) hits.clear();
  return 0;
}

function cleanHistory(raw: unknown): Turn[] {
  if (!Array.isArray(raw)) return [];
  return raw
    .slice(-6)
    .filter((t: any) => t && (t.role === "user" || t.role === "assistant") && typeof t.content === "string")
    .map((t: any) => ({ role: t.role, content: String(t.content).slice(0, 600) }));
}

export default async function handler(req: any, res: any) {
  res.setHeader("cache-control", "no-store");
  res.setHeader("x-content-type-options", "nosniff");
  // The Android app's bundled offline demo is served from this origin and may call Eva live.
  const APP_ORIGIN = "https://appassets.androidplatform.net";
  const reqOrigin = req.headers.origin as string | undefined;
  if (reqOrigin === APP_ORIGIN) {
    res.setHeader("access-control-allow-origin", APP_ORIGIN);
    res.setHeader("vary", "origin");
  }
  if (req.method === "OPTIONS") {
    if (reqOrigin !== APP_ORIGIN) return res.status(403).end();
    res.setHeader("access-control-allow-methods", "POST");
    res.setHeader("access-control-allow-headers", "content-type");
    res.setHeader("access-control-max-age", "600");
    return res.status(204).end();
  }
  if (req.method !== "POST") {
    res.setHeader("allow", "POST");
    return res.status(405).json({ error: "method_not_allowed" });
  }
  const origin = req.headers.origin as string | undefined;
  const host = req.headers.host as string | undefined;
  if (origin && host && origin !== APP_ORIGIN && new URL(origin).host !== host) return res.status(403).json({ error: "cross_origin" });

  const ip = String(req.headers["x-forwarded-for"] ?? "").split(",")[0].trim() || "anon";
  const retry = limited(ip);
  if (retry > 0) {
    res.setHeader("retry-after", String(retry));
    return res.status(429).json({ error: "rate_limited", text: `Too many messages. Try again in ${retry} s.` });
  }

  let body: any = req.body;
  if (typeof body === "string") {
    try { body = JSON.parse(body); } catch { body = {}; }
  }
  const verdict = screen(body?.text ?? body?.message);
  if (verdict.action === "reply") return res.status(200).json({ text: verdict.text, source: "guard", reason: verdict.reason });

  const key = process.env.GROQ_API_KEY;
  if (!key) return res.status(503).json({ error: "eva_offline" });

  const lang = body?.lang === "es" ? "Answer in Spanish (Rioplatense, using 'vos')." : "Answer in English.";
  const turns: Turn[] = [...cleanHistory(body?.history), { role: "user", content: asUserData(verdict.text) }];
  try {
    const raw = await chat(key, `${SYSTEM_PROMPT}\n${lang}`, turns);
    let text = guardReply(raw) || "Sorry, I have no answer for that. Ask me about space or health.";
    text = body?.lang === "es" ? text.replace(/Super Intelligence/g, "Super Inteligencia") : text.replace(/Super Inteligencia/g, "Super Intelligence");
    return res.status(200).json({ text, source: "model", model: CHAT_MODEL });
  } catch (e) {
    console.error("eva chat upstream failed", redact(e instanceof Error ? e.message : "unknown"));
    return res.status(503).json({ error: "eva_offline", text: "Eva is offline right now. Try again in a minute." });
  }
}
