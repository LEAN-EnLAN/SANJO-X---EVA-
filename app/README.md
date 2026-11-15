# app · Eva by WearMeds (Rust + htmx, one binary)

The real Eva app: an astronaut/passenger **PWA** (`/app`), a **crew / ground dashboard** (`/dash`), a JSON/CBOR **API** (`/api/*`), **exports** (`/export/*`) and **Eva Link**, our super-light protocol for **near space** (`/link`). One Rust binary, assets embedded, works offline on the ship. 100% own code (libraries as dependencies only).

```bash
cargo build --release && ./target/release/eva demo
# http://localhost:8080/app · http://localhost:8080/dash · http://localhost:8081/dash (ground)
# ↑↑↓↓←→←→BA opens the dev panel (cardiac arrest, solar storm, suit mode, link profiles, clock, reset)
```

| Read | For |
|---|---|
| [`../docs/app/RUNNING.md`](../docs/app/RUNNING.md) | build, run, demo story, phones on the LAN, API list |
| [`../docs/app/ARCHITECTURE.md`](../docs/app/ARCHITECTURE.md) | diagram, module map, data flow, seams for the next agents |
| [`../docs/app/PROTOCOL.md`](../docs/app/PROTOCOL.md) | Eva Link byte by byte |
| [`../docs/app/MOTION.md`](../docs/app/MOTION.md) | spring tokens, Eva's mascot states |
| ADRs in [`../docs/app/`](../docs/app/) | 001 stack · 002 near space, not deep space · 003 protocol · 004 PWA and TLS · 005 data and chain · 006 Eva layers, pipelines, tools · 007 Hermes seam · 008 mesh hub · 009 motion |

## Layout

```text
app/
  Cargo.toml            crate `eva` (lib + bin)
  src/                  Rust (see ARCHITECTURE.md for the module map)
  templates/            askama HTML: app/ (phone screens), dash/ (dashboard), dev/, export/
  styles/app.css        Tailwind v4 input: the wireframe tokens + motion tokens
  static/               embedded: css/app.css (built), js/ (htmx vendored, motion, eva, link), img/, icons/, sw.js, manifest
  tests/flows.rs        end-to-end flows (reflex → case, tools → chain, HTTP)
  package.json          build-time only: Tailwind CLI (scripts/build-css.sh)
```

Design source of truth: the app wireframes (`design/app-wireframes`, tokens in `src/theme.css`). Templates reuse their Tailwind classes, so porting a screen is mostly adapting markup.

## Rules that apply here

- Product copy in **English**. Eva **guides, records and warns**, gives *diagnostic support*, never decides a shock: **the AED decides**. No harness, no injector.
- The Groq key is read **only** from `GROQ_API_KEY` at runtime; never in the repo, logs, docs or browser code.
- AI-generated images carry a visible "AI-generated" note: large photos (detail heroes, splash) carry their own mark plus a readable label; the line at the foot of each screen, the About & credits screen (`/app/about`) and the dashboard footer cover the small 3D icons and Eva's mascot states (Codex kit, `static/img/icons`, `eva`, `hero`).
