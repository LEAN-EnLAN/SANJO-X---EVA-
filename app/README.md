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
- Images generated with Super Intelligence are declared in this README (below), in the repo README and in the release notes, not on screen.

## Images generated with Super Intelligence

Every illustration in the app is concept art generated with Super Intelligence, made for team Sanjo-X (NASA Space Apps Challenge 2026, Rosario): the 3D icons, Eva's mascot states, the app icon, the launch screen, the large detail photos and the wristband v2 renders (`static/img/icons`, `eva`, `hero`, `band`, `splash*`, `static/icons`). The wristband renders in `static/img/band` are the home card, the parts and suit-mode views on `/app/band` (the screen part also illustrates the ECG until a one-hand ECG image exists), the two emergency thumbnails and the 8 frames of the 360° turn, cropped and resized from the Codex v2 kit. The team wrote the prompts with Claude, and the images were generated with an OpenAI image model through Codex. They are illustrative, show no real person or device, and contain no NASA insignia or branding. This notice replaces on-screen labels: the app itself shows none.


## Fonts

The type stack is the landing's ("PP Neue Montreal", Inter, Arial). PP Neue Montreal is not shipped, so Inter renders: it is self-hosted in `static/fonts/` (SIL Open Font License 1.1, from Google Fonts, latin subset) so the app works offline. The colours are the landing's palette (pure black, off-white, grey, #2a2a2a hairlines, teal for Eva, orange accents, red for alarms); the tokens live in `styles/app.css`.
