# Eva app · architecture

One Rust binary (`eva`) serves everything: the astronaut PWA, the crew/ground dashboard, the JSON/CBOR API, the Eva Link WebSocket, the exports and the embedded static files. The same binary runs on the ship (hub) and on the ground; a flag picks the role.

## Big picture

```text
                         ┌──────────────────────── SHIP (eva serve --role ship) ────────────────────────┐
  WearMeds bands         │                                                                              │
  (sensor base +         │   sim.rs ──baseline──┐                                                       │
   interface)            │   (or `eva sim` over │  overlay (scenario, dev panel)                       │
  ──Eva Link vitals────▶ │    /link, role Band) ▼                                                       │
                         │                 state.rs  World { passengers, alerts, cases, journal, … }    │
                         │                    │  ▲                                                      │
                         │   eva_agent ◀──────┘  │ tools.rs (13 tools, validated, logged)               │
                         │    L1 reflex.rs  (every tick, instant, offline)                              │
                         │    L2 chat.rs · speech.rs · vision.rs  (Groq, key from env only)             │
                         │    L3 chain.rs   (SHA-256 hash chain, JSONL, signed → sent → acked)          │
                         │                    │                                                         │
                         │   web/  axum router ── /app /dash (askama + htmx) ── /sse/* (fragments)      │
                         │         /api/* (JSON | CBOR) ── /export/* ── /static (rust-embed)            │
                         │   link.rs  /link WebSocket ── mesh.rs (presence, relay, RTT per client)      │
                         │                    │                                                         │
                         │   uplink.rs ── Endpoint (queue, ACK/NACK, retransmit) ── LinkSim(profile) ───┼──┐
                         └──────────────────────────────────────────────────────────────────────────────┘  │
     PWA phones / tablets ◀── one WebSocket each to the hub (/link) + SSE for the page ──▶                   │
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
| `main.rs` | CLI: `serve`, `sim`, `demo`, `bench`, `verify`; starts tasks and the HTTP(S) server |
| `lib.rs` | module list (the crate is a library so tests and benches can reach everything) |
| `protocol/` | **Eva Link**: `wire` (varints, CRC-16), `frame` (header, types, priorities), `payload`, `delta` (vitals), `queue` (store-and-forward, ACK/NACK, RTT), `profile` (link simulator), `stats` (bytes), `Endpoint` |
| `state.rs` | `AppState` (world, chain, clock, mesh, agent config, UI event bus), `World`, `LinkStatus`, demo reset |
| `model.rs` | passengers, statuses, alerts, cases, journal, meals, reminders, chat, Eva moods |
| `seed.rs` | demo data matching the wireframes (Ana 4B, Leo 3B, Tomás 1B, Priya, Diego, Kwame…) |
| `clock.rs` | mission clock (day 96, 13:58 at start) with fast-forward |
| `sim.rs` | simulated bands: healthy baseline + server-side scenario overlay; `eva sim` client |
| `eva_agent/` | `reflex` (layer 1), `tools` (registry), `chat`, `speech` (STT/TTS), `vision`, `prompt`, `config` (key handling), emergency flow and case timeline in `mod.rs` |
| `chain.rs` | hash-chained record log, replay from JSONL, verify, sync states |
| `mesh.rs` | mesh registry: nodes, relay, hub heartbeat (ping + presence every 2 s) |
| `link.rs` | `/link` WebSocket handler for every role; ground-side chain `Mirror` |
| `uplink.rs` | ship → ground client: records + vitals through a simulated profile, acks back |
| `tls.rs` | self-signed certificate for `--tls` (rcgen) |
| `web/` | `mod.rs` router and embedded assets, `pages` (screens), `views` (askama view models), `sse`, `api`, `export`, `ui` (class lookups, own icons) |

Templates live in `app/templates/` (`app/`, `dash/`, `dev/`, `export/`); static files in `app/static/` (built CSS, vendored htmx, our JS, images, manifest, service worker). Everything in `static/` is compiled into the binary with rust-embed (in debug builds it is read from disk, so edits show without recompiling).

## Data flow of the cardiac case (the demo story)

1. Dev panel → "Cardiac arrest" sets Ana's scenario. The overlay drops her pulse and motion; after 20 s the band reports *no answer*.
2. `eva_agent::on_tick` → `reflex::evaluate` sees **no pulse + no movement + no answer** → `open_emergency`: nearest trained passenger for CPR (Leo, 3 m), nearest to the AED station for the AED (Tomás). No model involved.
3. Layer 3 logs `alarm` and `case` records; the uplink queues the alarm in the highest priority class.
4. SSE: helpers' apps receive `go-emergency` and switch screens; the dashboard re-renders roster, KPIs and the case card.
5. Leo taps **I'm going** → CPR screen. The case timeline advances (crew lead notified, CPR started, AED attached and *shock advised by the AED*).
6. The ground receives each record seconds later (near-space profile), verifies the hash chain, acknowledges; the ship's sync card turns them **Acknowledged**.

## Seams for the next agents

* **Screens**: add a template in `templates/app/` + a view model in `web/views.rs` + a route in `web/mod.rs`. Live parts: give them an `sse-swap` name and render them in `web/sse.rs`.
* **Eva**: new tool = one `ToolDef` + one match arm in `eva_agent/tools.rs` (logged automatically). New pipeline backend = implement the trait in `chat.rs`, `speech.rs` or `vision.rs`.
* **Telemetry**: `POST /api/ingest/vitals` (JSON) and Eva Link `Vitals` frames are the two ways in; NASA dictionaries / Hermes go behind them (ADR-007).
* **Storage**: the chain is the durable store; SQLite can replace the in-memory working set behind `state.rs` (ADR-005).
* **Tests / benchmarks**: `cargo test`, `eva bench`, `protocol::message_size_table()`, `GET /api/protocol/sizes`.
