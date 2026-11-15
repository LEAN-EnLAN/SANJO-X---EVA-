// deno test supabase/functions/eva-chat/handler_test.ts
// Offline tests: Supabase Auth, the rate-limit RPC and Groq are replaced by a fake fetch.
import { handler } from "./handler.ts";
import { classifyTopic, isEmergency, OFF_TOPIC_REPLY, REFUSAL, screen } from "./guard.ts";

function assert(cond: unknown, msg: string): asserts cond {
  if (!cond) throw new Error(msg);
}

Deno.env.set("SUPABASE_URL", "https://example.supabase.co");
Deno.env.set("SUPABASE_ANON_KEY", "anon-test");
Deno.env.set("GROQ_API_KEY", "test-key-not-real");

let modelCalls = 0;
let lastSystem = "";
let consume = { role: "viewer" as string | null, allowed: true, retry_after: 0 };

globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
  const url = String(input instanceof Request ? input.url : input);
  const auth = new Headers(init?.headers).get("authorization") ?? "";
  if (url.endsWith("/auth/v1/user")) {
    return auth === "Bearer good" ? Response.json({ id: "u-1" }) : new Response("{}", { status: 401 });
  }
  if (url.endsWith("/rest/v1/rpc/eva_chat_consume")) return Response.json([consume]);
  if (url.includes("/chat/completions")) {
    modelCalls++;
    lastSystem = JSON.parse(String(init?.body)).messages[0].content;
    return Response.json({ choices: [{ message: { content: "Now: drink water.\nWhy: microgravity shifts fluids." } }] });
  }
  return new Response("not found", { status: 404 });
}) as typeof fetch;

const post = (body: unknown, token = "good", origin = "https://eva-wearmeds.vercel.app") =>
  handler(new Request("https://example.supabase.co/functions/v1/eva-chat", {
    method: "POST",
    headers: { "content-type": "application/json", authorization: `Bearer ${token}`, origin },
    body: JSON.stringify(body),
  }));

Deno.test("no token → 401", async () => {
  const r = await handler(new Request("https://x/functions/v1/eva-chat", { method: "POST", body: "{}" }));
  assert(r.status === 401, `got ${r.status}`);
});

Deno.test("invalid token → 401", async () => {
  const r = await post({ text: "hydration in orbit?" }, "bad");
  assert(r.status === 401, `got ${r.status}`);
});

Deno.test("in-scope question → model reply, CORS for the site", async () => {
  consume = { role: "viewer", allowed: true, retry_after: 0 };
  const before = modelCalls;
  const r = await post({ text: "How should I stay hydrated in microgravity?" });
  const j = await r.json();
  assert(r.status === 200 && j.source === "model", JSON.stringify(j));
  assert(modelCalls === before + 1, "model not called");
  assert(r.headers.get("access-control-allow-origin") === "https://eva-wearmeds.vercel.app", "cors");
});

Deno.test("off-topic → guard redirect, no model call", async () => {
  const before = modelCalls;
  const j = await (await post({ text: "Who won the football match last night?" })).json();
  assert(j.source === "guard" && j.reason === "off_topic" && j.text === OFF_TOPIC_REPLY, JSON.stringify(j));
  assert(modelCalls === before, "model called for off-topic");
});

Deno.test("Spanish in-scope → model, asked to answer in Spanish", async () => {
  const j = await (await post({ text: "¿Cómo duermo mejor en la nave espacial?", lang: "es" })).json();
  assert(j.source === "model", JSON.stringify(j));
  assert(lastSystem.includes("Answer in Spanish"), "lang hint missing");
});

Deno.test("treatment request → refusal", async () => {
  const j = await (await post({ text: "Give me some adrenaline for my heart" })).json();
  assert(j.reason === "treatment" && j.text === REFUSAL, JSON.stringify(j));
});

Deno.test("rate limited → 429 with retry-after", async () => {
  consume = { role: "viewer", allowed: false, retry_after: 42 };
  const r = await post({ text: "hydration in orbit?" });
  assert(r.status === 429 && r.headers.get("retry-after") === "42", `got ${r.status}`);
  consume = { role: "viewer", allowed: true, retry_after: 0 };
});

Deno.test("no profile / role → 403", async () => {
  consume = { role: null, allowed: false, retry_after: 0 };
  const r = await post({ text: "hydration in orbit?" });
  assert(r.status === 403, `got ${r.status}`);
  consume = { role: "viewer", allowed: true, retry_after: 0 };
});

Deno.test("foreign origin → 403", async () => {
  const r = await post({ text: "hydration in orbit?" }, "good", "https://evil.example");
  assert(r.status === 403, `got ${r.status}`);
});

Deno.test("guard: 'help me' rule and Spanish vocabulary", () => {
  assert(!isEmergency("help me with my homework"), "homework is not an emergency");
  assert(isEmergency("help me"), "bare help me is an emergency");
  assert(isEmergency("help me, i can't feel a pulse"), "help me + body word");
  assert(classifyTopic("me duele la cabeza") === "health", "es health");
  assert(classifyTopic("¿cuánto dura el viaje a marte?") === "space", "es space");
  assert(screen("help me write my essay about football").action === "reply", "off-topic help");
});
