# ADR-006 · Eva in three layers, separate pipelines, one tool registry

- **Status:** accepted · 15/11/2026 · Claude · extends the team's 3-layer AI architecture (HANDOFF.md)

## Context

Eva is the product: an AI health copilot that reads the WearMeds band live, keeps logs and the mood journal, gives **diagnostic support** (never a diagnosis on its own), guides emergencies and records everything. It must never wait for a model to react to a dying passenger, and **it never decides a shock: the ship's AED does.**

## Decision

### Layer 1 · Reflex rules (`eva_agent/reflex.rs`)
Pure functions over each passenger's latest sample, run on every tick (1 Hz), microseconds each, offline. Only this layer opens an emergency, and only with all three signs: **no pulse + no movement + no answer** to the band's "are you OK?" prompt. Fewer signs → watch list (band loose, weak signal). Other rules: SpO₂ < 90%, wrist ECG flagged for review, sensor base battery < 20%, skin ≥ 1 °C above the person's usual. Each rule writes one plain line ("Sensor base at 14%, about 1 h left · spare base in the med bay, drawer B2"). The emergency flow assigns the nearest trained passenger (not in a suit, not the crew lead) for CPR and the nearest to the AED station to fetch the AED, then advances a case timeline (*what to do now / why / how Eva decided / what comes next*, Russell et al. 2023).

### Layer 2 · Pipelines, each with its own module, trait and config
| pipeline | module · trait | backend (checked Nov 2026 at console.groq.com/docs) | offline fallback |
|---|---|---|---|
| Chat / reasoning + tool calling | `chat.rs` · `ChatPipeline` | `openai/gpt-oss-120b` (`EVA_CHAT_MODEL`) via `/chat/completions`, `tools` + `tool_choice:auto`, up to 3 tool rounds | `OfflineChat`: keyword intents → same tools |
| Speech-to-text | `speech.rs` · `SttPipeline` | `whisper-large-v3-turbo` (`EVA_STT_MODEL`) via `/audio/transcriptions`; browser records with `MediaRecorder`, uploads to `POST /api/eva/stt` | 503 → typing or browser dictation |
| Text-to-speech | `speech.rs` · `TtsPipeline` | `canopylabs/orpheus-v1-english`, voice `hannah` (`EVA_TTS_MODEL`, `EVA_TTS_VOICE`) via `/audio/speech`, WAV streamed by `POST /api/eva/tts` | 503 `{"fallback":"speechSynthesis"}` → browser voice |
| Vision (image, short video) | `vision.rs` · `VisionPipeline` | `qwen/qwen3.8-27b` (`EVA_VISION_MODEL`, preview, ≤ 3 images/request, base64 data URIs) via `POST /api/eva/vision`; **video = 3 frames sampled in the browser** | placeholder text, meal logged as an estimate |

Config is read once from the environment (`config.rs`). **The key is read only from `GROQ_API_KEY`**, held in an `ApiKey` type whose `Debug`/`Display` print `***`, never logged, never put on the chain, never sent to the browser. `--offline` / `EVA_OFFLINE=1` forces the offline pipelines. Any online failure falls back to offline rules, so a person always gets an answer.

Note (15/11): Groq returns *"requires terms acceptance"* for the Orpheus TTS model on the team's org until an admin accepts the terms in the Groq console; until then TTS uses the browser voice. Chat (with tools), STT and vision were verified live.

### Tools (`eva_agent/tools.rs`): one registry for Eva and for people
13 tools with JSON-schema parameters, sent to the model as OpenAI `tools`, and callable from the app (buttons, chips, voice): `get_vitals`, `log_mood`, `add_journal_entry`, `log_meal` (text or photo via vision), `log_water`, `start_ecg_check`, `request_help` (assigns the nearest trained passenger), `call_aed_runner`, `daily_summary`, `export_report`, `send_to_ground`, `set_reminder`, `show_screen` (navigates the person's app over SSE). Every call is validated on the server and **logged to the chain** with its origin (`eva`, `user`, `reflex`), including rejected calls. **Safety: no tool delivers a shock, a drug or any treatment**; a test fails if a tool name looks like one. `call_aed_runner` only sends a person to fetch the AED.

### Layer 3 · Ledger
Findings, case steps, tool calls and answers (model name, tools used, not the private text) go to the hash chain (ADR-005) and up to the ground.

### Eva's state for the UI
`EvaMood` (idle, listening, thinking, speaking, happy, concerned, alert) is set by the agent and pushed over SSE; the mascot animates it with springs (ADR-009, MOTION.md).

## Consequences

- Swapping Groq for an on-board model (MedGemma, per the team plan) = new struct implementing `ChatPipeline` / `VisionPipeline`; `SttPipeline` for MedASR.
- Measured on 15/11 with the demo key: chat with one tool call ≈ 1.9 s; STT ≈ 0.2 s for 1 s of audio; vision (meal photo) ≈ 1–2 s.
- The LLM sees a short live context (vitals, status, open case) and the last 8 turns, nothing from the private journal.
