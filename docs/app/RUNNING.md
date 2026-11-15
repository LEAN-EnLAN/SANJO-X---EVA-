# Running Eva

## Build

Needs only a Rust toolchain (1.85+; tested with cargo 1.97). On a busy machine add `-j 2`.

```bash
cd app
cargo build --release          # → target/release/eva: one 12.5 MB binary (stripped, thin LTO), all assets embedded
cargo test                     # 148 tests: protocol, chain + signatures, rules, tools, agent, HTTP flows, medical station, env file + Eva status, break tests (TESTING.md)
```

CSS is generated from `app/styles/app.css` with the Tailwind v4 CLI and committed. Only after changing templates, styles or class strings in Rust/JS:

```bash
scripts/build-css.sh           # npm install (once) + tailwindcss --minify
cd app && npm run css:watch    # while editing templates
```

## Demo mode (one process, ship + ground)

```bash
cd app
./target/release/eva demo                                 # Eva online if a key file is found (see below)
./target/release/eva --env-file /path/outside/the/repo/eva.env demo   # or name the file
```

### Turning Eva online: the env file (`--env-file`, `.env`, `eva.env`)

Eva needs `GROQ_API_KEY` for open conversation, voice and photos. Without it she runs on her **on-board rules** (she still reads the band, logs, reminds and warns). The binary reads the key from the environment, or from a file, so nobody has to export it in a shell:

1. `--env-file <path>` (or `EVA_ENV_FILE=<path>`): this file, and it must exist (otherwise `eva` exits with code 2 and says why);
2. otherwise the first that exists: `./.env` (the working directory) → `.env` next to the `eva` program → `eva.env` next to the `eva` program.

Easiest on any computer: copy `eva.env.example` (in the release zip as `.env.example`) to **`eva.env`** next to the `eva` program, put the key after `GROQ_API_KEY=`, and start Eva again. Windows hides or renames dot-files (Explorer happily saves `.env.txt`), which is why the instructions say `eva.env`.

Rules: one `KEY=VALUE` per line; `#` comments and blank lines are ignored; `export ` is allowed; one pair of matching quotes is stripped; a UTF-8 BOM and Windows line ends are fine. **A variable already set in the environment always wins.** The file is read before any thread starts (`main` is a plain `fn` that builds the runtime afterwards).

The startup log says which mode Eva is in, never the key: `Eva: online · key from eva.env next to the binary`, or `Eva: on-board rules · no GROQ_API_KEY found (looked in ./.env, <dir>/.env, <dir>/eva.env) · …`, or `Eva: on-board rules · --offline`.

`GET /api/eva/status` answers what the UI shows, with no secrets and `Cache-Control: no-store`:

```json
{"mode":"rules","reason":"no_key","label":"On-board rules","hint":"Set GROQ_API_KEY and restart",
 "chat":false,"voice_in":false,"voice_out":"browser","vision":false,"key_source":"none",
 "checked_at":"2026-11-15T14:32:10Z","version":"0.3.2"}
```

`reason` is `ok`, `no_key`, `offline_flag`, `bad_key` (the provider refused the key at the start-up probe or on a call) or `unreachable` (no internet from the hub; Eva re-probes every 60 s and comes back online by herself). Changes go out as the `eva-status` SSE event to every app and dashboard. In the app: the Eva tab shows a chip (Online / On-board rules) and a banner with the reason-specific fix; the dashboard header has a pill; the dev panel shows `reason`, `key_source` and `checked_at`.

| URL | what |
|---|---|
| http://localhost:8080/app | astronaut app (Ana, seat 4B; pick another seat in the header) |
| http://localhost:8080/app?seat=3B | Leo, the nearest trained passenger to Ana |
| http://localhost:8080/dash | crew dashboard (Mara, crew lead) · the medical station tile |
| http://localhost:8080/station | the ship's **medical station** screen (full screen, landscape): put it on a tablet or a second laptop |
| http://localhost:8081/dash | ground view (flight surgeon), fed by the near-space uplink |

Without a key Eva runs on her on-board rules (`--offline` forces it). The key is read only from `GROQ_API_KEY` (environment or env file); never commit it (`.env`, `.env.*`, `*.env` and `eva.env` are ignored, `*.env.example` is not). Logs and error messages scrub it (`gsk_***`), and a test fails if it ever shows up in a log line, an error, the chain or `/api/state`.

### Eva's voice: Groq TTS needs the Orpheus terms accepted

Groq's text-to-speech model (`canopylabs/orpheus-v1-english`) answers **"requires terms acceptance"** until an **admin of the Groq organisation** that owns the key accepts the model's terms once: sign in at [console.groq.com](https://console.groq.com), open the Playground, pick *canopylabs/orpheus-v1-english* and accept the terms (only org admins see the button). Nothing in the app or the key changes.

Until then the fallback is explicit, not a silent error:

- `POST /api/eva/tts` answers `503` with `x-eva-voice: browser` and `{"fallback":"speechSynthesis","reason":"terms_not_accepted","message":"…"}`; the browser speaks with its own `speechSynthesis`.
- The Eva badge (app Eva screen, dashboard case card, dev panel) reads **"Groq online · browser voice (TTS terms pending)"**; `/api/state` → `agent.voice.engine` is `browser_terms`.
- The server remembers the refusal for 10 minutes, so replies do not wait for a failed call; after that it tries Groq again and switches back by itself once the terms are accepted.
- No key or `--offline`: same fallback with reason `no_key` and the badge "offline rules · browser voice". `EVA_TTS=browser` forces the browser voice even with a key.

### Pipelines and their settings

Each Eva pipeline has its own settings (model, base URL, on/off); all share `GROQ_API_KEY`.

| pipeline | model env (default) | base URL env | off switch |
|---|---|---|---|
| chat + tools | `EVA_CHAT_MODEL` (`openai/gpt-oss-120b`) | `EVA_CHAT_BASE_URL` | `EVA_CHAT=off` |
| speech-to-text | `EVA_STT_MODEL` (`whisper-large-v3-turbo`) | `EVA_STT_BASE_URL` | `EVA_STT=off` |
| text-to-speech | `EVA_TTS_MODEL` (`canopylabs/orpheus-v1-english`), `EVA_TTS_VOICE` (`hannah`) | `EVA_TTS_BASE_URL` | `EVA_TTS=browser` |
| vision | `EVA_VISION_MODEL` (`qwen/qwen3.8-27b`) | `EVA_VISION_BASE_URL` | `EVA_VISION=off` |
| topic guard classifier | `EVA_GUARD_MODEL` (`openai/gpt-oss-20b`), `EVA_GUARD_SAFETY_MODEL` (`meta-llama/llama-prompt-guard-2-22m`, `off` to skip), `EVA_GUARD_TIMEOUT_MS` (`1500`) | `EVA_GUARD_BASE_URL` | `EVA_GUARD=off` (local layers only) |

### Topic guard (ADR-012)

Eva only talks about space and spaceflight, the mission, health, medicine and wellbeing, and the Eva system. Clear off-topic or abusive requests are refused on board with a short redirect ("I can help with your health and the mission. For anything else, ask the crew lead."). Prompt-injection clauses are removed before anything reaches Groq. Only ambiguous text goes to the small classifier (~0.2–0.5 s, 1.5 s timeout, cached 5 min). Chat is capped at 2,000 characters and voice at 30 s. Each seat may make `EVA_RATE_PER_MIN` (20) requests a minute per Groq-backed endpoint; over that, the answer is `429` with `Retry-After` and a friendly line, and a message with emergency wording is never limited. Every filtered request is a `guard` record on the chain (`GET /api/chain`), with no long text and no key. Offline, the same rules run without the classifier.

`GROQ_BASE_URL` changes the default for every pipeline. Live check (prints models and timings, never the key): `EVA_LIVE_GROQ=1 cargo test --test groq_live -- --nocapture` with the key in the environment.

### The demo story

The stage version, minute by minute, with what to say and what to do if the Wi-Fi dies: [DEMO.md](DEMO.md). The same story runs headless as a test: `scripts/e2e/run.sh` (TESTING.md).

1. Open `/dash` and `/app?seat=3B` side by side, and `/station` on a tablet or another window (and `:8081/dash` if you have room).
2. Press **↑ ↑ ↓ ↓ ← → ← → B A** (on a phone: tap the clock / logo 5 times) → **dev panel**.
3. Passenger 4B · **Cardiac arrest**. About 20 s later the band reports *no pulse, no movement, no answer*; Eva's reflex rule opens the case. Leo's phone turns red: "Ana needs help · seat 4B · 3 m". Tomás gets "Get the AED". On `/station`, compartment 01 glows teal and the Eva screen says "Take the AED · Ana, seat 4B".
4. Leo taps **I'm going** → CPR screen (110 a minute, "AED decides shocks"). Tap compartment 01 on `/station` (door opened, AED taken), or let Tomás tap **I have it**: the case timeline shows "AED taken at the station · mm:ss" and the dashboard's station tile shows the report.
5. The dashboard shows Ana in *Needs help now*, the case with *now / why / how Eva decided / next*, helpers, and the ground sync queue going *Saved → Sent → Acknowledged* a few seconds later.
6. Dev panel → uplink profile **Link outage**: records queue (store-and-forward). Back to **Near-space uplink**: they flow and get acknowledged. **Solar storm** sends every app to the shelter screen. **+10 min** fast-forwards the clock. **Reset demo** starts over (the station restocks too).
7. Dev panel → **Hub down**: the hub stops relaying. After 5 s the station says **Standalone**. Start a cardiac arrest: the band's own LoRa alarm reaches the station, which lights 01 and shows "Emergency · Take the AED" by itself. Its red **Emergency** button reaches every phone over the air. Tap **Hub down** again: the station uploads what it buffered, every report becomes a signed record, the case appears, and the chain still verifies.
8. Exports: dashboard header → Report (print to PDF), CSV, JSON; the app's Records tab exports one person.

## Wristband mode (testing)

The same PWA turns into the wristband's own UI when the screen has the band's shape: landscape, at most 480 px high and at least 8:5 wide (or a tiny screen under 340 × 340). It switches by itself on rotation, without a reload; the SSE connection stays open.

- **Chrome devtools**: open `http://localhost:8080/app?seat=4B`, toggle the device toolbar, choose *Responsive* and type **480 × 270**, **640 × 360** (the reference) or **800 × 450**. The page moves to `/app/wrist` and `<html data-mode="wristband">`. Swipe or use ←/→ for the five pages (face · vitals · Eva · ECG · Help), `h` jumps to Help, `Esc` back to the face. Back to a phone size and it returns to `/app`.
- **Phone**: rotate it (auto-rotate on). The APK has an "Open as: Wristband (landscape)" option that locks landscape.
- **Overrides** (saved in the browser): `?mode=wristband` forces the band UI at any size (a letterboxed 16:9 stage), `?mode=phone` keeps the phone UI in landscape, `?mode=auto` clears the override. On the band, 5 taps on the time open the phone UI.
- **Takeovers**: from the dev panel (on a phone-sized window or `/dash`), Cardiac arrest on 4B: a band-sized 3B shows "Ana needs help" → **I'm going** → the CPR metronome (a vibration on every compression), 1B shows "Take the AED · 01", 4B "Help is coming"; Solar storm shows the shelter route. Help on the band needs a **2 s hold**.

## Separate processes

```bash
eva serve --role ground --port 8081
eva serve --role ship --port 8080 --peer http://127.0.0.1:8081             # 15 simulated bands in-process
eva serve --role ship --port 8080 --peer http://127.0.0.1:8081 --station   # … plus the simulated medical station (/station)
eva serve --role ship --port 8080 --sim 0 & eva sim --target ws://127.0.0.1:8080/link --n 15   # bands over Eva Link
```

Other commands: `eva bench` (see [BENCHMARKS.md](BENCHMARKS.md)), `eva verify --data data` (hash chain + Ed25519 signature of every record in every chain file, against the node's pinned public key, and the ground's mirror; exits 1 on any tampering), `eva --help`.

Keys: on first run each node creates `data/keys/<node>.ed25519` (private, mode 0600) and `data/keys/<node>.pub`. The ground pins the ship's public key the first time the ship connects (`data/keys/peer-<ship>.pub`); delete that file to accept a new ship key. Back up the `.ed25519` file if you want to keep signing as the same node.

Options: `--station` (ship role: one simulated medical station as a client of this hub; `eva demo` always has it), `--host 0.0.0.0` (default, LAN-visible), `--data <dir>` (chain files, node keys in `keys/`, the ground's mirror, certificates), `--name <node>`, `--tls`, `--tls-host <name>`, `--offline`, `--env-file <path>` (any subcommand). Env: `GROQ_API_KEY`, `EVA_ENV_FILE`, the pipeline settings above, `EVA_RATE_PER_MIN`, `EVA_OFFLINE=1`, `RUST_LOG=eva=debug`.

## Phones on the LAN (install the PWA)

`localhost` is a secure context, other devices are not: service workers and "Add to Home Screen" need HTTPS there.

```bash
eva demo --tls                 # self-signed cert in data/certs/ for localhost + this machine's LAN IP
# open https://<lan-ip>:8080/app on the phone, trust the certificate, then Share → Add to Home Screen
```

Android phones can also use the native **Eva APK** instead of the PWA (hub address + seat picker, certificate pinning, mic/camera): [APK.md](APK.md).

Smoother: [mkcert](https://github.com/FiloSottile/mkcert) — `mkcert -install`, create `data/certs/eva-cert.pem` and `eva-key.pem` for `localhost 127.0.0.1 <lan-ip>`, install mkcert's root CA on the phones; `--tls` reuses those files. Details: ADR-004.

## API quick reference

`GET /api/state` (add `Accept: application/cbor` for CBOR) · `GET /api/passengers/{seat}` · `GET /api/chain` · `GET /api/chain/verify` · `GET /api/mesh` · `GET /api/link` · `GET /api/protocol/sizes` · `GET /api/tools` · `GET /api/eva/status` · `POST /api/eva/chat|tool|stt|tts|vision` · `POST /api/journal` · `POST /api/ingest/vitals` · `GET /app/wrist`, `/app/wrist/eva|ecg|emergency|storm`, `/app/wrist/live?part=face|vitals` (wristband mode) · `GET /station` (medical station screen) · `POST /api/station/tap` (`n=1…12`: door opened, item taken) · `POST /api/station/button` (red button) · `POST /api/dev/hub_down` (toggle the simulated hub failure) · `GET /export/records.csv|records.json|vitals.csv|journal.csv|report` · `GET /link` (WebSocket, Eva Link) · `GET /sse/app?seat=`, `/sse/dash`, `/sse/station`.

Earth / cloud side (optional, never used by the ship): Supabase, `eva` schema inside the shared project `changuito`, with the records mirror, roles, storage and the `eva-chat` Edge Function; setup, tests and provisioning in [supabase/README.md](../../supabase/README.md), decision in ADR-014.
