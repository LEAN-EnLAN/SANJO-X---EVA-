// Eva chat for the public demo (Vercel Function, Node). Same topic guard as the ship app and the
// Firebase kit: space, spaceflight, health and medicine only; injection stripping; length limit;
// treatment requests refused; emergencies always pointed to the crew lead / Help button.
// The Groq key comes only from the GROQ_API_KEY environment variable and is never logged or returned.
//
// Request:  POST { text, lang?: "en" | "es", history?, style?: "simple", context? }
//   style "simple" (sent by the public demo): warm, 2 or 3 short everyday sentences and one suggestion, no
//   "Now: / Why:" labels. Any other value, or none, keeps the app's labelled lines, so other callers are unchanged.
//   context (public demo only): the demo character's simulated wristband readings; see demoContextLine().
// GET, or POST { ping: true }: { ok, live } so the demo can say "Eva is online" without a model call and without
// counting against the rate limit. (The demo sends the POST form: an older deploy answers it with a plain 200 guard
// reply instead of a 405, so no browser console error either way.)
import { screen, guardReply, systemPrompt, replyStyle, demoContextLine, simplifyReply, asUserData, OFF_TOPIC_REPLY } from "../_lib/guard";
import { chat, redact, CHAT_MODEL, UpstreamError, type Turn } from "../_lib/groq";

const WINDOW_MS = 60_000;
// Per IP, per instance. 20 a minute covers a guided tour plus a few questions, and a shared venue network.
const MAX_PER_WINDOW = 20;
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
    res.setHeader("access-control-allow-methods", "GET, POST");
    res.setHeader("access-control-allow-headers", "content-type");
    res.setHeader("access-control-max-age", "600");
    return res.status(204).end();
  }
  if (req.method === "GET" || req.method === "HEAD") {
    // Status only: is Eva's key configured? No model call, nothing about the key itself.
    return res.status(200).json({ ok: true, live: Boolean(process.env.GROQ_API_KEY), styles: ["default", "simple"] });
  }
  if (req.method !== "POST") {
    res.setHeader("allow", "GET, POST");
    return res.status(405).json({ error: "method_not_allowed" });
  }
  const origin = req.headers.origin as string | undefined;
  const host = req.headers.host as string | undefined;
  if (origin && host && origin !== APP_ORIGIN && new URL(origin).host !== host) return res.status(403).json({ error: "cross_origin" });

  let body: any = req.body;
  if (typeof body === "string") {
    try { body = JSON.parse(body); } catch { body = {}; }
  }
  if (body?.ping === true) return res.status(200).json({ ok: true, live: Boolean(process.env.GROQ_API_KEY) });

  const ip = String(req.headers["x-forwarded-for"] ?? "").split(",")[0].trim() || "anon";
  const retry = limited(ip);
  if (retry > 0) {
    res.setHeader("retry-after", String(retry));
    return res.status(429).json({ error: "rate_limited", retryAfter: retry, text: `Too many messages. Try again in ${retry} s.` });
  }

  const style = replyStyle(body?.style);
  const verdict = screen(body?.text ?? body?.message, style);
  if (verdict.action === "reply") return res.status(200).json({ text: verdict.text, source: "guard", reason: verdict.reason });

  const key = process.env.GROQ_API_KEY;
  if (!key) return res.status(503).json({ error: "eva_offline" });

  const lang = body?.lang === "es" ? "Answer in Spanish (Rioplatense, using 'vos')." : "Answer in English.";
  const turns: Turn[] = [...cleanHistory(body?.history), { role: "user", content: asUserData(verdict.text) }];
  try {
    const raw = await chat(key, [systemPrompt(style), demoContextLine(body?.context), lang].filter(Boolean).join("\n"), turns);
    let text = guardReply(raw) || "Sorry, I have no answer for that. Ask me about space or health.";
    // The guard already said this is a health question; if the model still answers with the off-topic line, ask once
    // more and say so plainly (one extra call, only in that case).
    if (verdict.topic === "health" && text.includes(OFF_TOPIC_REPLY)) {
      try {
        const again = guardReply(await chat(key, [systemPrompt(style), demoContextLine(body?.context), "The next message IS a health question: answer it helpfully.", lang].filter(Boolean).join("\n"), turns));
        if (again && !again.includes(OFF_TOPIC_REPLY)) text = again;
      } catch { /* keep the first answer */ }
    }
    if (style === "simple") text = simplifyReply(text);
    text = body?.lang === "es" ? text.replace(/Super Intelligence/g, "Super Inteligencia") : text.replace(/Super Inteligencia/g, "Super Intelligence");
    return res.status(200).json({ text, source: "model", model: CHAT_MODEL });
  } catch (e) {
    console.error("eva chat upstream failed", redact(e instanceof Error ? e.message : "unknown"));
    // Groq busy: tell the client when a retry makes sense (the demo retries once).
    if (e instanceof UpstreamError && (e.status === 429 || e.status >= 500)) res.setHeader("retry-after", "2");
    return res.status(503).json({ error: "eva_offline", text: "Eva is offline right now. Try again in a minute." });
  }
}
