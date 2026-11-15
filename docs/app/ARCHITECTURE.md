# Eva app · architecture

One Rust binary (`eva`) serves everything: the astronaut PWA, the crew/ground dashboard, the JSON/CBOR API, the Eva Link WebSocket, the exports and the embedded static files. The same binary runs on the ship (hub) and on the ground; a flag picks the role.

## Big picture

```text
                         ┌──────────────────────── SHIP (eva serve --role ship) ────────────────────────┐
  WearMeds bands         │                                                                              │
  (sensor base +         │   sim.rs ──baseline──┐                                                       │
   interface)            │   (or `eva sim` over │  overlay (scenario, dev panel)                       │
  ──Eva Link vitals────▶ │    /link, role Band) ▼                                                       │
                         │                 state.rs  World { passengers, alerts, cases, station, … }    │
                         │                    │  ▲                                                      │
                         │   eva_agent ◀──────┘  │ tools.rs (13 tools, validated, logged)               │
                         │    L1 reflex.rs  (every tick, instant, offline)                              │
                         │    L2 chat.rs · speech.rs · vision.rs  (Groq, key from env only)             │
                         │    L3 chain.rs   (SHA-256 hash chain + Ed25519, JSONL, signed → sent → acked)│
                         │                    │                                                         │
                         │   web/  axum router ── /app /dash /station (askama + htmx) ── /sse/*         │
                         │         /api/* (JSON | CBOR) ── /export/* ── /static (rust-embed)            │
                         │   link.rs  /link sessions ── mesh.rs (presence, relay, RTT per client)       │
                         │      │  station.rs: hub side (StationView, commands, report → signed record) │
                         │      │                                                                       │
                         │   uplink.rs ── Endpoint (queue, ACK/NACK, retransmit) ── LinkSim(profile) ───┼──┐
                         └──────┼───────────────────────────────────────────────────────────────────────┘  │
     PWA phones / tablets ◀──── one WebSocket each to the hub (/link) + SSE for the page ──▶                 │
                                │                                                                            │
  MEDICAL STATION (client) ◀────┘ Eva Link session, role Station: StationCmd ▼  ▲ StationEvent / Status      │
  12 compartments · Eva screen · door + item sensors · red button · backup battery                           │
  station.rs device side (in `eva demo`: in-process, screen = /station on a tablet or second laptop)         │
        ▲  LoRa broadcast (AppState::air): band no-pulse alarm → station lights 01 by itself;                │
        └─ red button → every wristband. Used when the hub is down; the station buffers and uploads later.   │
                                                                                                             │
                         ┌──────────────────────── GROUND (eva serve --role ground) ───────────────────┐    │
                         │   link.rs (role Ship session) ── Endpoint ── Mirror (verify hash chain) ◀───┼────┘
                         │   acks each record · shows cases with ship time + ground received time       │
                         │   /dash ground view · "Message the ship" (reliable Text frames)             │
                         └──────────────────────────────────────────────────────────────────────────────┘
```

## Module map (`app/src`)

| Module | What it owns |
|---|---|
| `main.rs` | plain `fn main`: parses the CLI, loads the env file (`envfile.rs`: `--env-file`, `./.env`, `.env`/`eva.env` next to the binary; real env vars win) while single-threaded, then builds the tokio runtime. CLI: `serve` (`--station` adds one simulated medical station), `sim`, `demo` (always with the station), `bench`, `verify`; starts tasks and the HTTP(S) server |
| `lib.rs` | module list (the crate is a library so tests and benches can reach everything) |
| `protocol/` | **Eva Link**: `wire` (varints, CRC-16), `frame` (header, types, priorities), `payload` (incl. `StationCmd`, `StationEvent`, `StationStatus`), `delta` (vitals), `queue` (store-and-forward, ACK/NACK, RTT, `retransmit_now`), `profile` (link simulator), `stats` (bytes), `Endpoint` |
| `state.rs` | `AppState` (world, chain, clock, mesh, agent config, UI event bus, `hub_down`, the simulated LoRa `air`, station command outbox), `World` (incl. the hub's `StationView`), `LinkStatus`, demo reset |
| `station.rs` | **the ship's medical station** (ADR-011). Device side: `StationDevice` (12 compartments idle / lit / open / taken with item counts, the Eva screen's message, standalone detection, its send queue as the event buffer, LoRa in and out) and `run_device` over any datagram pipe. Hub side: `StationView`, `light` / `clear` / `restock`, `on_frame` (each report → one signed chain record, "AED taken at the station · mm:ss" on the case, red button → alarm to every app), `band_lora_alarms` while the hub is down |
| `model.rs` | passengers, statuses, alerts, cases, journal, meals, reminders, chat, Eva moods |
| `seed.rs` | demo data matching the wireframes (Ana 4B, Leo 3B, Tomás 1B, Priya, Diego, Kwame…) |
| `clock.rs` | mission clock (day 96, 13:58 at start) with fast-forward |
| `sim.rs` | simulated bands: healthy baseline + server-side scenario overlay; `eva sim` client |
| `eva_agent/` | `reflex` (layer 1), `guard` (topic guard, ADR-012: injection removal, keyword scope, Groq scope classifier + Prompt Guard, output check, per-seat rate limit), `tools` (registry, refusal guard), `schema` (JSON-schema validator for tool args), `chat`, `offline` (intents + Now/Why/How/Next answers), `speech` (STT/TTS, browser-voice fallback), `vision`, `prompt`, `config` (one config per pipeline, key redaction), `status` (`/api/eva/status`: online or on-board rules and why, start-up probe, health from every call, `eva-status` SSE), emergency flow and case timeline in `mod.rs` |
| `chain.rs` | hash-chained, Ed25519-signed record log, node keys, replay from JSONL, verify (records and files), sync states (ADR-010) |
| `mesh.rs` | mesh registry: nodes, relay, hub heartbeat (ping + presence every 2 s; silent while "Hub down") |
| `link.rs` | `/link`: one transport-agnostic session per client (`session`; WebSocket adapter `handle`, in-process `local_session`), every role incl. Station; drops everything while "Hub down"; forwards the simulated LoRa air to apps; ground-side chain `Mirror` (verifies hash + signature, pins ship keys, persisted to `mirror-<node>.jsonl`) |
| `bench.rs` | `eva bench`: message sizes vs JSON/CBOR, throughput, link profiles, live server run (BENCHMARKS.md) |
| `uplink.rs` | ship → ground client: records + vitals through a simulated profile, acks back |
| `tls.rs` | self-signed certificate for `--tls` (rcgen) |
| `web/` | `mod.rs` router and embedded assets, `pages` (screens, `/station`, wristband mode `/app/wrist*`), `views` (askama view models, incl. `StationStrip`, `StationGrid`, `StationTile`), `sse` (`/sse/station`: strip and grid), `api` (`/api/station/tap`, `/api/station/button`, dev `hub_down`), `export`, `ui` (class lookups, own icons) |

Templates live in `app/templates/` (`app/`, `dash/`, `dev/`, `export/`, `station/`); static files in `app/static/` (built CSS, vendored htmx, our JS, images, manifest, service worker). Everything in `static/` is compiled into the binary with rust-embed (in debug builds it is read from disk, so edits show without recompiling).

## Data flow of the cardiac case (the demo story)

1. Dev panel → "Cardiac arrest" sets Ana's scenario. The overlay drops her pulse and motion; after 20 s the band reports *no answer*.
2. `eva_agent::on_tick` → `reflex::evaluate` sees **no pulse + no movement + no answer** → `open_emergency`: nearest trained passenger for CPR (Leo, 3 m), nearest to the AED station for the AED (Tomás). No model involved.
3. Layer 3 logs `alarm` and `case` records; the uplink queues the alarm in the highest priority class.
4. SSE: helpers' apps receive `go-emergency` and switch screens; the dashboard re-renders roster, KPIs and the case card. `station::on_case_opened` queues `StationCmd` *light 01 · "Take the AED · Ana, seat 4B"*: the medical station's compartment 01 glows teal on `/station`.
5. Leo taps **I'm going** → CPR screen. Someone opens 01 (a tap on `/station`): the station reports *door opened* and *item taken*; each is a signed `station` record and the case gets **"AED taken at the station · mm:ss"** (or the same line when Tomás taps **I have it**, whichever comes first). The case timeline advances (crew lead notified, CPR started, AED attached and *shock advised by the AED*).
6. The ground receives each record seconds later (near-space profile), verifies the hash chain, acknowledges; the ship's sync card turns them **Acknowledged**. Closing the case clears 01.

## Hub down (the fallback path)

Dev panel → **Hub down** sets `AppState::hub_down`: link sessions drop every datagram, the heartbeat stops, the hub's rules stop. The station hears nothing for 5 s and goes **standalone**. A band whose own rule sees the three signs broadcasts a no-pulse `Alarm` on the simulated air; the station lights 01 and shows "Emergency · Take the AED" by itself; its red button goes on the air to every app. Its reports wait in its send queue. When the hub comes back, its first ping wakes the station, which sends `online` (with the number of buffered events) and retransmits everything at once; the hub signs each report into the chain in order, opens the case dated from when the station heard the band, and the chain still verifies on board and on the ground (`tests/station.rs`).

## Seams for the next agents

* **Screens**: add a template in `templates/app/` + a view model in `web/views.rs` + a route in `web/mod.rs`. Live parts: give them an `sse-swap` name and render them in `web/sse.rs`.
* **Station**: a real cabinet replaces `StationDevice` behind the same Eva Link session (`station::ws_pipe` + `run_device` already run it over a WebSocket); new compartment behaviour = a `StationCmd` op or `StationEvent` code (PROTOCOL.md §11).
* **Eva**: new tool = one `ToolDef` + one match arm in `eva_agent/tools.rs` (logged automatically). New pipeline backend = implement the trait in `chat.rs`, `speech.rs` or `vision.rs`.
* **Telemetry**: `POST /api/ingest/vitals` (JSON) and Eva Link `Vitals` frames are the two ways in; NASA dictionaries / Hermes go behind them (ADR-007).
* **Storage**: the chain is the durable store; SQLite can replace the in-memory working set behind `state.rs` (ADR-005).
* **Tests / benchmarks**: `cargo test` (TESTING.md), `eva bench` (BENCHMARKS.md), `scripts/e2e/run.sh` (Playwright happy path), `protocol::message_size_table()`, `GET /api/protocol/sizes`.
