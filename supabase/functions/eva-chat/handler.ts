// eva-chat · Supabase Edge Function (Deno). Same behaviour as the public site's Vercel function
// (public-sites/api/eva/chat.ts): the app's topic guard (space travel, health and medicine only;
// injection stripping; length limit; treatment and shock requests refused; emergencies pointed to
// local emergency services / the crew lead) in front of Groq.
//
// Differences from the Vercel function, on purpose:
//   - a valid Supabase JWT is required (any role: crew_lead, mission_control, team, viewer);
//   - the rate limit is per user and persistent (eva.eva_chat_consume, table eva.rate_limits),
//     not per IP in memory.
//
// Request:  POST { "text" | "message": string, "history"?: [{role, content}], "lang"?: "en" | "es" }
//           Authorization: Bearer <user access token>
// Response: 200 { text, source: "guard", reason } | { text, source: "model", model }
//           401 unauthenticated · 403 no_role · 429 rate_limited (retry-after) · 503 eva_offline
//
// EVA_GROQ_API_KEY is a function secret (eva-prefixed so it never clashes with the host app's). It is never logged or returned; logs carry a hash of the user
// id, lengths and the guard's decision, never message text.
import { asUserData, guardReply, MAX_HISTORY_TURNS, MAX_INPUT_CHARS, sanitize, screen, SYSTEM_PROMPT } from "./guard.ts";
import { chat, CHAT_MODEL, redact, type Turn } from "./groq.ts";

const ALLOWED_ORIGINS = new Set([
  "https://eva-wearmeds.vercel.app",
  "http://localhost:8080",
  "http://127.0.0.1:8080",
  ...(Deno.env.get("EVA_ALLOWED_ORIGINS") ?? "").split(",").map((s) => s.trim()).filter(Boolean),
]);

function corsHeaders(origin: string | null): Record<string, string> {
  const h: Record<string, string> = {
    "access-control-allow-headers": "authorization, x-client-info, apikey, content-type",
    "access-control-allow-methods": "POST, OPTIONS",
    "access-control-max-age": "600",
    vary: "Origin",
  };
  if (origin && ALLOWED_ORIGINS.has(origin)) h["access-control-allow-origin"] = origin;
  return h;
}

function json(status: number, body: unknown, origin: string | null, extra: Record<string, string> = {}): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json; charset=utf-8", "cache-control": "no-store", "x-content-type-options": "nosniff", ...corsHeaders(origin), ...extra },
  });
}

async function uidTag(uid: string): Promise<string> {
  const d = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(uid));
  return Array.from(new Uint8Array(d).slice(0, 6), (b) => b.toString(16).padStart(2, "0")).join("");
}

function cleanHistory(raw: unknown): Turn[] {
  if (!Array.isArray(raw)) return [];
  return raw
    .slice(-MAX_HISTORY_TURNS)
    .filter((t) => t && typeof t === "object" && (t.role === "user" || t.role === "assistant") && typeof t.content === "string")
    .map((t) =>
      t.role === "user"
        ? { role: "user" as const, content: asUserData(sanitize(String(t.content)).text.slice(0, MAX_INPUT_CHARS)) }
        : { role: "assistant" as const, content: String(t.content).slice(0, 1200) },
    );
}

interface Consume {
  role: string | null;
  allowed: boolean;
  retry_after: number;
}

/** Validate the caller's JWT with Supabase Auth, then count the message against their bucket. */
async function authorize(token: string): Promise<{ uid: string; consume: Consume } | { status: number; error: string }> {
  const base = Deno.env.get("SUPABASE_URL");
  const anon = Deno.env.get("SUPABASE_ANON_KEY");
  if (!base || !anon) return { status: 503, error: "eva_offline" };
  const headers = { apikey: anon, authorization: `Bearer ${token}` };
  const u = await fetch(`${base}/auth/v1/user`, { headers });
  if (!u.ok) return { status: 401, error: "unauthenticated" };
  const user = await u.json().catch(() => null) as { id?: string } | null;
  if (!user?.id) return { status: 401, error: "unauthenticated" };
  // Eva lives in the `eva` schema (shared project): PostgREST needs the Content-Profile header.
  const rpc = (fn: string) =>
    fetch(`${base}/rest/v1/rpc/${fn}`, {
      method: "POST",
      headers: { ...headers, "content-type": "application/json", "content-profile": "eva", "accept-profile": "eva" },
      body: "{}",
    });
  // First use creates the caller's Eva profile (role viewer); users of the host app get nothing in
  // `eva` until they talk to Eva.
  const p = await rpc("ensure_profile");
  if (!p.ok) return { status: p.status === 401 ? 401 : 503, error: p.status === 401 ? "unauthenticated" : "eva_offline" };
  const r = await rpc("eva_chat_consume");
  if (!r.ok) return { status: r.status === 401 ? 401 : 503, error: r.status === 401 ? "unauthenticated" : "eva_offline" };
  const rows = await r.json().catch(() => []) as Consume[];
  const c = rows?.[0];
  if (!c) return { status: 503, error: "eva_offline" };
  return { uid: user.id, consume: c };
}

export async function handler(req: Request): Promise<Response> {
  const origin = req.headers.get("origin");
  if (req.method === "OPTIONS") return new Response(null, { status: 204, headers: corsHeaders(origin) });
  if (req.method !== "POST") return json(405, { error: "method_not_allowed" }, origin, { allow: "POST, OPTIONS" });
  if (origin && !ALLOWED_ORIGINS.has(origin)) return json(403, { error: "cross_origin" }, origin);

  const token = (req.headers.get("authorization") ?? "").replace(/^Bearer\s+/i, "").trim();
  if (!token) return json(401, { error: "unauthenticated", text: "Sign in to talk to Eva." }, origin);

  const started = Date.now();
  const auth = await authorize(token);
  if ("error" in auth) {
    const text = auth.status === 401 ? "Sign in to talk to Eva." : "Eva is offline right now. Try again in a minute.";
    return json(auth.status, { error: auth.error, text }, origin);
  }
  const tag = await uidTag(auth.uid);
  if (!auth.consume.role) {
    return json(403, { error: "no_role", text: "Your account has no Eva role yet. Ask the team." }, origin);
  }
  if (!auth.consume.allowed) {
    const retry = Math.max(1, auth.consume.retry_after);
    console.log(JSON.stringify({ ev: "eva-chat", uid: tag, decision: "rate_limited", retry }));
    return json(429, { error: "rate_limited", text: `Too many messages. Try again in ${retry} s.`, retryAfter: retry }, origin, { "retry-after": String(retry) });
  }

  let body: Record<string, unknown> = {};
  try {
    body = await req.json();
  } catch {
    body = {};
  }
  const verdict = screen(body?.text ?? body?.message);
  if (verdict.action === "reply") {
    console.log(JSON.stringify({ ev: "eva-chat", uid: tag, decision: "guard", reason: verdict.reason, stripped: verdict.stripped, ms: Date.now() - started }));
    return json(200, { text: verdict.text, source: "guard", reason: verdict.reason }, origin);
  }

  const key = Deno.env.get("EVA_GROQ_API_KEY"); // eva-prefixed: the project may be shared
  if (!key) return json(503, { error: "eva_offline", text: "Eva is offline right now. Try again in a minute." }, origin);

  const lang = body?.lang === "es" ? "Answer in Spanish (Rioplatense, using 'vos')." : "Answer in English.";
  const turns: Turn[] = [...cleanHistory(body?.history), { role: "user", content: asUserData(verdict.text) }];
  try {
    const raw = await chat(key, `${SYSTEM_PROMPT}\n${lang}`, turns);
    const text = guardReply(raw) || "Sorry, I have no answer for that. Ask me about space or health.";
    console.log(JSON.stringify({ ev: "eva-chat", uid: tag, decision: "model", topic: verdict.topic, role: auth.consume.role, inLen: verdict.text.length, outLen: text.length, ms: Date.now() - started }));
    return json(200, { text, source: "model", model: CHAT_MODEL }, origin);
  } catch (e) {
    console.error(JSON.stringify({ ev: "eva-chat", uid: tag, decision: "upstream_failed", error: redact(e instanceof Error ? e.message : "unknown") }));
    return json(503, { error: "eva_offline", text: "Eva is offline right now. Try again in a minute." }, origin);
  }
}
