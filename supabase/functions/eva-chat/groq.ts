// Minimal Groq chat client (OpenAI-compatible endpoint), same defaults as the
// ship app's chat pipeline (docs/app/RUNNING.md: EVA_CHAT_MODEL, temperature 0.3).
// Deno port for the eva-chat Edge Function. The key comes from the EVA_GROQ_API_KEY function secret; it is never logged,
// never returned and never written anywhere.

export const GROQ_BASE_URL = Deno.env.get("GROQ_BASE_URL") ?? "https://api.groq.com/openai/v1";
export const CHAT_MODEL = Deno.env.get("EVA_CHAT_MODEL") ?? "openai/gpt-oss-120b";
const TIMEOUT_MS = 20_000;

export interface Turn {
  role: "user" | "assistant";
  content: string;
}

export class UpstreamError extends Error {
  constructor(
    public readonly status: number,
    message: string,
  ) {
    super(message);
  }
}

/** Remove anything that looks like a Groq key from a string before it can reach a log. */
export function redact(s: string): string {
  return s.replace(/gsk_[A-Za-z0-9]+/g, "gsk_***");
}

export async function chat(key: string, system: string, turns: Turn[]): Promise<string> {
  const ctrl = new AbortController();
  const timer = setTimeout(() => ctrl.abort(), TIMEOUT_MS);
  try {
    const resp = await fetch(`${GROQ_BASE_URL}/chat/completions`, {
      method: "POST",
      signal: ctrl.signal,
      headers: { "content-type": "application/json", authorization: `Bearer ${key}` },
      body: JSON.stringify({
        model: CHAT_MODEL,
        messages: [{ role: "system", content: system }, ...turns],
        temperature: 0.3,
        max_completion_tokens: 1200,
        reasoning_effort: "low",
      }),
    });
    const body = (await resp.json().catch(() => ({}))) as {
      choices?: { message?: { content?: string } }[];
      error?: { message?: string };
    };
    if (!resp.ok) {
      throw new UpstreamError(resp.status, redact(body.error?.message ?? `groq ${resp.status}`));
    }
    return body.choices?.[0]?.message?.content?.trim() ?? "";
  } finally {
    clearTimeout(timer);
  }
}
