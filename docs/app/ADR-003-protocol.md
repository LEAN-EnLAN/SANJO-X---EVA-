# ADR-003 · Eva Link: a super-light protocol of our own

- **Status:** accepted · 15/11/2026 · Claude
- **Spec:** [PROTOCOL.md](PROTOCOL.md) · **Code:** `app/src/protocol/` (Rust), `app/static/js/link.js` (browser)

## Context

The same messages travel over very different links: the band's LoRa-like radio (~5 kbit/s, 222-byte packets), the ship LAN between PWA clients and the hub, and the near-space uplink (seconds, gaps, loss; ADR-002). JSON over HTTP costs ~120 bytes for one vitals sample; we need an order of magnitude less, and we need priorities, acknowledgements and offline queuing.

## Options

1. **JSON over WebSocket**: easy to read, 5 to 10× larger, no priorities or acks by itself.
2. **MQTT / CoAP**: good fits in general, but add a broker or a stack the team does not know, and still need our own payloads and priority rules.
3. **CBOR/postcard payloads + own framing**: compact, but postcard's layout is an implementation detail and hard to mirror in the browser.
4. **Own tiny binary format, specified byte by byte** ✔

## Decision

Option 4. A 2-byte header (version, flags, priority, type) + LEB128 varints for seq/src/dst/ack/len, typed payloads, optional CRC-16 on links without integrity. Reliability (selective ACK, NACK, retransmit with RTT-based RTO, dedup) and four **priority classes (alarm > guidance > vitals > logs)** live in a `SendQueue` that is also the **store-and-forward** buffer. Vitals are **delta-encoded against the last keyframe** and latest-wins with a TTL. A deterministic **link simulator** (LAN, LoRa-like, near-space, storm, outage) and **byte accounting** per message type make the trade-offs measurable.

## Measured (from `eva bench`, release numbers identical)

| message | bytes on the wire |
|---|---|
| ping | 11 |
| vitals delta (2 fields changed) | 11 |
| vitals keyframe (10 fields) | 29 |
| alarm: no pulse | 10 |
| ack of 10 records | 17 |
| chain record (CRC on) | 52 |
| *same vitals as JSON* | *123* |

10 minutes, 15 bands at 1 Hz + 1 record/s + 1 alarm: the alarm arrives in 0.1 s (LAN), 0.4 s (LoRa), 1.7 s (near space), 4 s (storm); during an outage everything queues and nothing is lost.

## Consequences

- Browsers speak it too (`link.js`, ~150 lines), so every PWA instance is a mesh node (ADR-008).
- Adding a message type = enum value + payload struct + test; the version byte allows breaking changes later.
- No encryption in v1: the ship LAN and the uplink are assumed to be protected below us (WPA3 / link encryption). TLS covers browser ↔ hub when `--tls` is on. Payload signing beyond the hash chain is future work.
