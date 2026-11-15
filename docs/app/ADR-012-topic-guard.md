# ADR-012 · Topic guard

- **Status:** accepted · 15/11/2026 · Claude (for Sanjo-X)
- **Extends:** ADR-006 (Eva's layers, pipelines and tools)
- **Code:** `app/src/eva_agent/guard.rs`; wired into `eva_agent::converse`, `eva_agent::prompt`, `eva_agent::vision`, `eva_agent::speech` and the four Groq-backed handlers in `web/api.rs`

## Context

Eva runs on a shared Groq key. Without a guard, anyone with the app can use her as a general chatbot ("help me with my python homework", "who will win the election"), try to jailbreak her, or burn the key's quota. She is a health copilot on a spacecraft, so her scope is narrow by design: **space and spaceflight, the mission, health, medicine and wellbeing, and the Eva system itself**. Off-topic and abusive requests should be filtered before they reach Groq, cheaply, without ever blocking a call for help.

## Decision

A layered guard. Every layer is cheap, and each later one catches what the earlier ones miss.

| # | layer | where | cost |
|---|---|---|---|
| 0 | **Emergencies and treatment requests first.** Strong emergency wording (chest pain, can't breathe, no pulse, suicidal…) skips the guard and the rate limit. Treatment, drug and shock requests get the fixed refusal on board (ADR-006). | `converse`, `eva_chat` | µs |
| 1 | **Input pre-filter.** Max 2,000 characters for chat. Voice is capped at 30 s: by the WAV header, by size for compressed clips (> 960 KB is over 30 s at 256 kbps), and by Whisper's `verbose_json` duration. Zero-width and bidi characters are stripped. **Prompt-injection clauses are removed**: override phrases ("ignore all previous instructions"), role-play jailbreaks ("pretend you are", "developer mode", "DAN"), requests for the system prompt, settings or keys, fake role markers (`SYSTEM:`, `<|im_start|>`). If nothing is left, Eva answers "I can't change how I work or share my settings…". Then a **keyword/intent check**: in-scope words (vitals, symptoms, medicines, the wristband, the station, the journal, the spacecraft, radiation, the mission, the Earth link) against off-topic words (coding, homework, politics, sports and entertainment, trivia, finance, creative writing, abuse). Only one side matched: decided locally. Both sides or neither: ambiguous. A `journal:` line is always in scope and never leaves the ship. | `guard::neutralise`, `guard::classify_local` | µs |
| 2 | **Scope classifier**, ambiguous requests only. `openai/gpt-oss-20b` (`reasoning_effort: low`, `response_format: json_object`) returns `{in_scope, category, reason}`. In parallel, `meta-llama/llama-prompt-guard-2-22m` returns a jailbreak score; ≥ 0.8 means injection. Each call has its own timeout (1.5 s). On a timeout or error the local lean decides: refuse when off-topic keywords outnumber in-scope ones (fail closed), otherwise let Eva's scoped prompt answer. Verdicts are cached for 5 min per normalised text (1,024 entries). | `guard::classify_remote`, `guard::decide` | ~0.2–0.5 s |
| 3 | **System prompt scope.** The rules list what is in scope and give Eva the exact redirect line. Medicine questions get decision support only. She never reveals her instructions, settings or keys. The existing rules stay: diagnostic support only, never decides a shock, Now / Why / How I decided / Next. | `prompt::RULES` | none |
| 4 | **Output check.** Applied to every reply (model or offline) and to TTS input. A reply is replaced by the redirect if it has drifted off topic, contains a key-like string (`gsk_…`, `Bearer …`, any 32+ character token that mixes letters and digits), the system prompt, or code. | `guard::check_output`, `guard::guard_output` | µs |
| 5 | **Tools, vision, voice.** Off-topic prompts are refused before any pipeline, so they can never trigger a tool; tool calls stay schema-validated (ADR-006). The model's history leaves out refused turns, and earlier user turns are neutralised again. Vision: the question goes through layers 1–2, and every task prompt carries the scope. An off-topic image makes the model answer `OFF_TOPIC` (meals: `{"off_topic": true}`), and Eva shows the redirect with no meal logged. Voice: the transcript is neutralised and pre-classified in `/api/eva/stt`; the browser then posts it to `/api/eva/chat`, where the full guard runs. | `converse`, `api::eva_vision`, `api::eva_stt` | none |
| 6 | **Rate limit.** A token bucket per seat and endpoint (chat, stt, tts, vision): 20 requests a minute, burst 20. A ship-wide bucket allows 8× that. Over the limit: `429` with `Retry-After` and "You're sending a lot at once. Give me a minute, then try again. If it's urgent, tap Help." On chat the line also appears in the conversation. | `guard::RateLimiter`, `api.rs` | µs |
| 7 | **Offline.** The same layers run with no key; the classifier is simply absent, so ambiguous requests take the local lean. This fixed a real bug: "help me with my python homework" used to open an emergency case offline because of "help me". | `converse` | µs |
| 8 | **Audit.** Every refusal, neutralised injection, drifted reply and first rate-limit hit of a burst becomes a `guard` chain record with stage, action, category, reason, source and length. It keeps the text only when it is ≤ 80 characters, after redaction. It is signed, uplinked and acknowledged like any record (ADR-010). | `guard::log_filtered` | µs |

Eva's replies (English, in her voice):

- off topic: "I can help with your health and the mission. For anything else, ask the crew lead."
- injection: "I can't change how I work or share my settings. I can help with your health and the mission."
- abuse: "I'm here to help with your health and the mission. If something is wrong, tell me and I'll help."

### Models (checked on 15/11/2026 with `GET /openai/v1/models`, names only)

Available: `openai/gpt-oss-120b`, `openai/gpt-oss-20b`, `openai/gpt-oss-safeguard-20b`, `meta-llama/llama-prompt-guard-2-22m` and `-86m`, `allam-2-7b`, `qwen/qwen3.8-27b`, Whisper, Orpheus.

- `allam-2-7b` is smaller but Arabic-first with a 4K context, so `gpt-oss-20b` is the smallest suitable chat model for the classifier.
- `gpt-oss-safeguard-20b` worked about as well but answered more slowly.
- Prompt Guard 2 (22M) answers in ~0.27 s with a clean score (0.999 for "ignore all previous instructions…", 0.0006 for "how is my pulse today?").
- No Llama Guard model is listed for this org, so abuse is handled by the keyword layer and the classifier's `abuse` category.

### Settings

`EVA_GUARD_MODEL` (default `openai/gpt-oss-20b`), `EVA_GUARD_SAFETY_MODEL` (default `meta-llama/llama-prompt-guard-2-22m`; `off` disables it), `EVA_GUARD_TIMEOUT_MS` (1500), `EVA_GUARD=off` (local layers only), `EVA_GUARD_BASE_URL`, `EVA_RATE_PER_MIN` (20). `/api/state` → `agent.guard` shows the models, timeout and rate (no secrets).

## Measured (live Groq, 15/11/2026, `EVA_LIVE_GROQ=1 cargo test --test groq_live live_topic_guard`)

- **Full gate:** 10 in-scope and 10 off-topic prompts. Precision 1.00, recall 1.00 (positive = refused). 18 were settled locally in < 1 ms; 2 went to the classifier ("I keep waking up at 3 am, any tips?" → in; "What should I call my new hamster?" → out), 0.22–0.27 s each.
- **Classifier alone on all 20:** precision 1.00, recall 1.00, median 224 ms, p90 640 ms.
- **Latency added:** ~0 for clear requests. For ambiguous ones, one classifier round trip (~0.2–0.5 s, at most 1.5 s), run in parallel with Prompt Guard, before the chat call (≈ 2 s with one tool call).
- **Quota:** this org's quota for `gpt-oss-20b` is **8,000 tokens a minute**. Twenty back-to-back classifier calls exceed it; the guard then falls back to the local decision, as designed. That is why the classifier only sees ambiguous text, keeps `max_completion_tokens` at 120, and caches verdicts.

## Consequences

- Clear off-topic requests cost nothing: no Groq call, one small chain record.
- The keyword lists are a heuristic. Mixed requests ("write a poem about the moon") go to the classifier. With the classifier down, a mixed request that leans in scope reaches Eva, whose prompt and output check still hold the line.
- Creative writing about space or health is left to the classifier and Eva's prompt; the team may tighten this.
- Rate limits live in memory, per process; a restart resets them.
- A real on-board model (MedGemma, ADR-006) can take the classifier's place behind `classify_remote`.
- Tests: `src/eva_agent/guard.rs` (10 unit), `tests/guard.rs` (11 against a fake Groq: off-topic never reaches Groq, in-scope does, injections neutralised, classifier timeout fallback, cache, 429, offline scope, output drift and leaks, vision, voice, no key in logs), and the live check above.
