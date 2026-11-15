# Running Eva

## Build

Needs only a Rust toolchain (1.85+; tested with cargo 1.97). On a busy machine add `-j 2`.

```bash
cd app
cargo build --release          # → target/release/eva (single binary, assets embedded)
cargo test                     # 46 tests: protocol, chain, rules, tools, HTTP flows
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

Without a key Eva runs on offline rules (`--offline` forces it). The key is read only from `GROQ_API_KEY`; never commit it (`.env*` is ignored).

### The demo story

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

Other commands: `eva bench` (bytes per message + 10-minute run per link profile), `eva verify --data data` (check chain files), `eva --help`.

Options: `--host 0.0.0.0` (default, LAN-visible), `--data <dir>` (chain files, certificates), `--name <node>`, `--tls`, `--tls-host <name>`, `--offline`. Env: `GROQ_API_KEY`, `EVA_CHAT_MODEL`, `EVA_STT_MODEL`, `EVA_TTS_MODEL`, `EVA_TTS_VOICE`, `EVA_VISION_MODEL`, `EVA_OFFLINE=1`, `RUST_LOG=eva=debug`.

## Phones on the LAN (install the PWA)

`localhost` is a secure context, other devices are not: service workers and "Add to Home Screen" need HTTPS there.

```bash
eva demo --tls                 # self-signed cert in data/certs/ for localhost + this machine's LAN IP
# open https://<lan-ip>:8080/app on the phone, trust the certificate, then Share → Add to Home Screen
```

Smoother: [mkcert](https://github.com/FiloSottile/mkcert) — `mkcert -install`, create `data/certs/eva-cert.pem` and `eva-key.pem` for `localhost 127.0.0.1 <lan-ip>`, install mkcert's root CA on the phones; `--tls` reuses those files. Details: ADR-004.

## API quick reference

`GET /api/state` (add `Accept: application/cbor` for CBOR) · `GET /api/passengers/{seat}` · `GET /api/chain` · `GET /api/chain/verify` · `GET /api/mesh` · `GET /api/link` · `GET /api/protocol/sizes` · `GET /api/tools` · `POST /api/eva/chat|tool|stt|tts|vision` · `POST /api/journal` · `POST /api/ingest/vitals` · `GET /export/records.csv|records.json|vitals.csv|journal.csv|report` · `GET /link` (WebSocket, Eva Link) · `GET /sse/app?seat=`, `/sse/dash`.
