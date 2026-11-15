# Running Eva

## Build

Needs only a Rust toolchain (1.85+; tested with cargo 1.97). On a busy machine add `-j 2`.

```bash
cd app
cargo build --release          # → target/release/eva: one 12.5 MB binary (stripped, thin LTO), all assets embedded
cargo test                     # 101 tests: protocol, chain + signatures, rules, tools, agent, HTTP flows, break tests (TESTING.md)
```

CSS is generated from `app/styles/app.css` with the Tailwind v4 CLI and committed. Only after changing templates, styles or class strings in Rust/JS:

```bash
scripts/build-css.sh           # npm install (once) + tailwindcss --minify
cd app && npm run css:watch    # while editing templates
```

## Demo mode (one process, ship + ground)

```bash
cd app
set -a; . /path/outside/the/repo/.eva-demo.env; set +a   # optional: GROQ_API_KEY for online Eva
./target/release/eva demo
```

| URL | what |
|---|---|
| http://localhost:8080/app | astronaut app (Ana, seat 4B; pick another seat in the header) |
| http://localhost:8080/app?seat=3B | Leo, the nearest trained passenger to Ana |
| http://localhost:8080/dash | crew dashboard (Mara, crew lead) |
| http://localhost:8081/dash | ground view (flight surgeon), fed by the near-space uplink |

Without a key Eva runs on offline rules (`--offline` forces it). The key is read only from `GROQ_API_KEY`; never commit it (`.env*` is ignored). Logs and error messages scrub it (`gsk_***`), and a test fails if it ever shows up in a log line, an error, the chain or `/api/state`.

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

`GROQ_BASE_URL` changes the default for all four. Live check (prints models and timings, never the key): `EVA_LIVE_GROQ=1 cargo test --test groq_live -- --nocapture` with the key in the environment.

### The demo story

The stage version, minute by minute, with what to say and what to do if the Wi-Fi dies: [DEMO.md](DEMO.md). The same story runs headless as a test: `scripts/e2e/run.sh` (TESTING.md).

1. Open `/dash` and `/app?seat=3B` side by side (and `:8081/dash` if you have room).
2. Press **↑ ↑ ↓ ↓ ← → ← → B A** (on a phone: tap the clock / logo 5 times) → **dev panel**.
3. Passenger 4B · **Cardiac arrest**. About 20 s later the band reports *no pulse, no movement, no answer*; Eva's reflex rule opens the case. Leo's phone turns red: "Ana needs help · seat 4B · 3 m". Tomás gets "Get the AED".
4. Leo taps **I'm going** → CPR screen (110 a minute, "AED decides shocks").
5. The dashboard shows Ana in *Needs help now*, the case with *now / why / how Eva decided / next*, helpers, and the ground sync queue going *Saved → Sent → Acknowledged* a few seconds later.
6. Dev panel → uplink profile **Link outage**: records queue (store-and-forward). Back to **Near-space uplink**: they flow and get acknowledged. **Solar storm** sends every app to the shelter screen. **+10 min** fast-forwards the clock. **Reset demo** starts over.
7. Exports: dashboard header → Report (print to PDF), CSV, JSON; the app's Records tab exports one person.

## Separate processes

```bash
eva serve --role ground --port 8081
eva serve --role ship --port 8080 --peer http://127.0.0.1:8081             # 15 simulated bands in-process
eva serve --role ship --port 8080 --sim 0 & eva sim --target ws://127.0.0.1:8080/link --n 15   # bands over Eva Link
```

Other commands: `eva bench` (see [BENCHMARKS.md](BENCHMARKS.md)), `eva verify --data data` (hash chain + Ed25519 signature of every record in every chain file, against the node's pinned public key, and the ground's mirror; exits 1 on any tampering), `eva --help`.

Keys: on first run each node creates `data/keys/<node>.ed25519` (private, mode 0600) and `data/keys/<node>.pub`. The ground pins the ship's public key the first time the ship connects (`data/keys/peer-<ship>.pub`); delete that file to accept a new ship key. Back up the `.ed25519` file if you want to keep signing as the same node.

Options: `--host 0.0.0.0` (default, LAN-visible), `--data <dir>` (chain files, node keys in `keys/`, the ground's mirror, certificates), `--name <node>`, `--tls`, `--tls-host <name>`, `--offline`. Env: `GROQ_API_KEY`, the pipeline settings above, `EVA_OFFLINE=1`, `RUST_LOG=eva=debug`.

## Phones on the LAN (install the PWA)

`localhost` is a secure context, other devices are not: service workers and "Add to Home Screen" need HTTPS there.

```bash
eva demo --tls                 # self-signed cert in data/certs/ for localhost + this machine's LAN IP
# open https://<lan-ip>:8080/app on the phone, trust the certificate, then Share → Add to Home Screen
```

Smoother: [mkcert](https://github.com/FiloSottile/mkcert) — `mkcert -install`, create `data/certs/eva-cert.pem` and `eva-key.pem` for `localhost 127.0.0.1 <lan-ip>`, install mkcert's root CA on the phones; `--tls` reuses those files. Details: ADR-004.

## API quick reference

`GET /api/state` (add `Accept: application/cbor` for CBOR) · `GET /api/passengers/{seat}` · `GET /api/chain` · `GET /api/chain/verify` · `GET /api/mesh` · `GET /api/link` · `GET /api/protocol/sizes` · `GET /api/tools` · `POST /api/eva/chat|tool|stt|tts|vision` · `POST /api/journal` · `POST /api/ingest/vitals` · `GET /export/records.csv|records.json|vitals.csv|journal.csv|report` · `GET /link` (WebSocket, Eva Link) · `GET /sse/app?seat=`, `/sse/dash`.
