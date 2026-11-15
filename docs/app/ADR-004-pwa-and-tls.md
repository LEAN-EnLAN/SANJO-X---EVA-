# ADR-004 · PWA, offline shell and HTTPS on the LAN

- **Status:** accepted · 15/11/2026 · Claude

## Context

Passengers use their own phone (or a ship tablet) as the Eva app. On a spacecraft there is no app store and no internet: the app must install from the ship hub, open with the hub down, and not lose a journal entry typed during a hiccup.

## Decision

- `/app` is a **PWA**: `manifest.webmanifest` (standalone, black theme, start `/app`, icons from the Eva mascot at 192/512 + maskable), served at the root.
- A **service worker** (`/sw.js`, scope `/`) caches the shell (pages, CSS, JS, key images) — static files cache-first, pages network-first with cached fallback; live data (`/sse`, `/link`, `/api` GETs, `/export`) is never cached.
- **Queued posts**: journal saves, tool buttons, chat, "I'm in the shelter" and helper answers made while offline go to IndexedDB (`eva-outbox`) and are replayed when the page reports `online` or on the next successful request. The UI shows "Saved on this device · will send when the hub is back".
- **Installability needs a secure context.** `http://localhost` counts as secure, so the laptop running the demo installs over plain HTTP. **Other devices on the LAN need HTTPS.** Two ways:
  1. `eva serve --tls` (or `eva demo --tls`): the binary creates a **self-signed certificate** with rcgen in `data/certs/` for `localhost`, `127.0.0.1`, `::1`, this machine's LAN IP and any `--tls-host`. Each phone must trust it once (open `https://<ip>:8080`, accept; on iOS install the certificate profile and enable full trust; Android: install as a user CA). Service workers refuse certificates the device does not trust.
  2. **mkcert** (smoother): `mkcert -install`, then `mkcert -cert-file data/certs/eva-cert.pem -key-file data/certs/eva-key.pem localhost 127.0.0.1 <lan-ip>`, then install mkcert's root CA on each phone (`mkcert -CAROOT`). `eva --tls` reuses existing files in `data/certs/`.
- Phone first for `/app` (max width 28 rem column, bottom tab bar with Eva in the centre); desktop first for `/dash` (rail, wide roster, 12-column grid), both responsive.

## Consequences

- Demo laptop: no setup. Phones: one certificate trust step, documented in RUNNING.md.
- The SW caches by path; bump `CACHE` in `sw.js` when the shell changes.
- Background Sync API is not used (not available everywhere); replay is page-driven.
- iOS Safari supports install + SW but not every API (e.g. MediaRecorder formats differ): voice falls back to typing or browser dictation.
