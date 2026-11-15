# ADR-001 · Stack: one Rust binary, server-rendered HTML, htmx

- **Status:** accepted · 15/11/2026 · Claude (for Sanjo-X)
- **Context:** the team rule is *Rust + htmx + HTML, one binary, 100% own code*. Nobody on the team writes Rust daily, so the code must be plain and documented. The demo runs on one laptop, offline, and phones on the same Wi-Fi install the app.

## Decision

| Need | Choice | Why |
|---|---|---|
| HTTP, WebSocket, SSE | **axum 0.8 + tokio** | mature, small API, WebSocket and SSE built in |
| Templates | **askama 0.14** | HTML files compiled into the binary, checked at compile time; people who know HTML can edit them |
| Interactivity | **htmx 2.0.4 + htmx-ext-sse 2.2.2**, vendored in `static/js/` | no front-end build, server owns state, live updates by SSE fragments |
| Styling | **Tailwind v4 CLI at build time** from a copy of the wireframes' `theme.css` | the templates use the *same class names* as the wireframes, so porting a screen is mostly copy-adapting markup |
| Static files | **rust-embed** | CSS, JS, images, manifest and service worker live inside the binary |
| Binary encoding | own Eva Link codec (ADR-003); **ciborium** for CBOR on `/api/*` | the link format must be tiny and documented byte by byte; CBOR is a cheap option for API clients on slow links |
| Storage | append-only JSONL hash chain (ADR-005) | durable, inspectable, no C dependency in the build |
| Hashing | **sha2** | SHA-256 chain |
| HTTPS | **axum-server + rustls (ring) + rcgen** | `--tls` with a self-signed certificate for LAN installs (ADR-004) |
| LLM / speech / vision | **reqwest** (rustls, native roots) to Groq | OpenAI-compatible API; key only from the environment (ADR-006) |

No aws-lc/cmake in the tree: rustls uses *ring*, so `cargo build` needs only a Rust toolchain.

## Consequences

- `cargo build --release` produces one file (`target/release/eva`); see RUNNING.md for size.
- CSS is generated: after changing templates or `styles/app.css`, run `scripts/build-css.sh`. The generated `static/css/app.css` is committed so a Rust-only machine can build.
- Askama passes call arguments by reference; numeric template helpers take `impl Val<T>` (see `web/ui.rs`).
- Libraries are dependencies only; no code was copied from other projects (team rule).
