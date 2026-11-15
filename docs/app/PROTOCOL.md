# Eva Link v1 · protocol specification

The wire format between wristbands, PWA clients, the ship hub and the ground. Designed for **near space** (seconds of latency, intermittent and narrow links; ADR-002). Rust: `app/src/protocol/`. Browser: `app/static/js/link.js`. Byte sizes below are asserted by tests.

## 1. Transport

- On the ship LAN and to the ground: **one binary WebSocket message = one datagram** at `GET /link` (`ws://` or `wss://` with `--tls`).
- On the band radio (LoRa-like): one radio packet = one datagram, ≤ 222 bytes, CRC on.
- A datagram holds **one or more frames** back to back (batching amortises radio wake-ups and WebSocket overhead).

## 2. Primitive encodings

| name | encoding |
|---|---|
| `uvar` | unsigned LEB128: 7 bits per byte, little-endian groups, high bit = more follows (`127 → 7f`, `300 → ac 02`) |
| `ivar` | zigzag then `uvar` (0, −1, 1, −2 → 0, 1, 2, 3) |
| `bytes` | `uvar` length + raw bytes |
| `str` | `bytes` holding UTF-8 |
| `crc16` | CRC-16/CCITT-FALSE (poly 0x1021, init 0xFFFF), big-endian; `"123456789" → 0x29B1` |

## 3. Frame

```text
byte 0   VVV FFFFF   version = 1 (3 bits) | flags (5 bits)
byte 1   PPP TTTTT   priority class (3 bits) | message type (5 bits)
uvar     seq         per-sender sequence number (wraps, never 0)
uvar     src         sender node id (0 = hub)
[uvar]   dst         present if FLAG_DST; absent = to the hub
[uvar]   ack         present if FLAG_ACK (piggybacked ack, reserved)
uvar     len         payload length
bytes    payload
[u16]    crc16       present if FLAG_CRC; covers every byte of this frame before it
```

Flags: `DST=0x01`, `ACK=0x02`, `RETX=0x04` (this is a retransmission), `CRC=0x08`, `REL=0x10` (reliable: the receiver must ack `seq`).

Smallest frame: 6 bytes (ping with a 1-byte payload). Unknown versions and types are rejected.

## 4. Priority classes

| class | value | types | behaviour |
|---|---|---|---|
| **alarm** | 0 | Alarm, alarm records, the station's red-button event | always leaves first, reliable |
| **guidance** | 1 | Guidance, Text, case/message/station records, StationCmd, StationEvent | reliable |
| **vitals** | 2 | Vitals, StationStatus | latest-wins, TTL (vitals 30 s, station status 10 s), never retransmitted |
| **logs** | 3 | Record (other kinds) | reliable, waits behind everything else |

Control frames (Hello, Welcome, Ping, Pong, Ack, Nack, Presence, Bye) bypass the queue.

## 5. Message types and payloads

| type | id | payload |
|---|---|---|
| Hello | 0 | `u8 role` · `str name` · `uvar client_ms` (session id) · [`bytes pubkey`] (optional, 32-byte Ed25519 public key; ships send it, the ground pins it on first use) |
| Welcome | 1 | `uvar node_id` · `uvar server_ms` |
| Ping | 2 | `uvar t_ms` (sender's clock) |
| Pong | 3 | the ping payload echoed unchanged → RTT = now − t_ms, no clock sync needed |
| Ack | 4 | `uvar n` · first seq absolute · then gaps as `uvar` (10 consecutive seqs = 12 bytes) |
| Nack | 5 | `uvar seq` · `u8 reason` (0 malformed, 1 need keyframe) · `uvar stream` |
| Vitals | 6 | see §6 |
| Alarm | 7 | `str subject` (seat) · `u8 code` (1 no pulse, 2 low SpO₂, 3 solar storm, 4 help request, 5 battery, 6 the medical station's red button) · `str text` |
| Guidance | 8 | same as Text |
| Record | 9 | `uvar chain_seq` · `uvar ts_ms` · `str kind` · `str subject` · `8 bytes hash prefix` · `bytes body` (JSON `{"body":…, "prev"?:hex}`) · [`bytes sig`] (optional, 64-byte Ed25519 signature over `"eva-record-v1\0" ‖ full SHA-256`; ADR-010) |
| Text | 10 | `str from` · `str to` · `str text` |
| Presence | 11 | `uvar n` · n × (`uvar id` · `u8 role` · `str name` · `uvar rtt_ms`) |
| Bye | 12 | empty |
| StationCmd | 13 | hub → station: `u8 op` · `u8 compartment` · `uvar case` · `str text` (§11) |
| StationEvent | 14 | station → hub: `u8 event` · `u8 compartment` · `uvar t_s` · `uvar count` · `str subject` (§11) |
| StationStatus | 15 | station → hub: `u8 battery` · `u8 flags` · `uvar lit` · `uvar open` · `uvar taken` · `uvar buffered` · `bytes items` (§11) |

Roles: 0 hub, 1 ship, 2 ground, 3 app, 4 dash, 5 band, 6 station.

Optional trailing fields (`[…]` above) are read only if bytes remain after the mandatory ones, so a v1 decoder that does not know them still works, and they cost nothing when absent. A present `pubkey` must be 32 bytes and a present `sig` 64 bytes, or the payload is rejected.

## 6. Delta-encoded vitals

A stream = one wristband (stream id = passenger id). Fixed-point fields, in order: `hr` bpm (0 = no pulse), `spo2` %, `resp` /min, `skin_c10` (0.1 °C), `eda_c` (0.01 µS), `motion` 0–255, `quality` %, `base_batt` %, `screen_batt` %, `flags` (1 suit mode, 2 ECG recording, 4 no answer, 8 LoRa ok).

```text
keyframe:  u8 (0x80 | kf_id) · uvar stream · uvar ts_s · 10 × ivar field
delta:     u8 kf_id          · uvar stream · uvar (ts_s − kf_ts) · uvar mask · ivar (field − keyframe field) for each set bit
```

Deltas are **against the last keyframe, not the previous sample**: a lost delta never corrupts the next one. A keyframe goes out every 10 samples (or after 120 s). If a delta names a keyframe the receiver does not have, it answers `Nack(NeedKeyframe, stream)` and the sender's next sample is a keyframe. Typical sizes: keyframe 24 B payload, delta 4 to 6 B, unchanged sample 4 B.

## 7. Reliability and store-and-forward

- Reliable frames enter an **in-flight table** when sent; the receiver acks each `seq` (duplicates are acked again but delivered once).
- **RTO** follows RFC 6298: `srtt`, `rttvar`, RTO = srtt + 4·rttvar, clamped 200 ms … 60 s, 1 s before the first sample. Samples come from pongs and from acks of frames never retransmitted (Karn). On a timeout the **RTO itself backs off** (×2, §5.5) and stays backed off until a valid sample arrives; each further retry of the same frame doubles again. Without the global backoff, a link slower than 1 s retransmits every frame and never gets a clean sample.
- **Never give up** on reliable frames: during a contact gap they stay queued; `poll(now, budget)` with budget 0 sends nothing.
- `poll` takes a **byte budget** (token bucket from the profile's bandwidth) and always drains alarm → guidance → vitals → logs, retransmissions first within a class.
- Unreliable vitals carry a TTL and are dropped when stale (`dropped_stale`), both when polled and when a new sample is queued, so a long contact gap holds at most one TTL of vitals.
- **Dedup and replays:** the receiver remembers the last 65,536 reliable seqs per peer (O(1) lookup); duplicates are acked again but delivered once. A replay older than that window reaches the application, where the ground's mirror drops a record it already holds (same chain seq and hash prefix). Records are idempotent end to end.
- **Acknowledged means mirrored.** A chain record turns *Acked* on the ship only when the ground's Ack for its frame arrives, and the ground acks a record frame only after decoding it into its mirror (and its file). Nothing is marked acknowledged locally. Invariant, checked by the e2e test and `tests/break_restart.rs`: once the ship has no record *Signed* or *Sent*, the ground's mirror for the current epoch holds exactly the ship's chain (same count, every record verified). A demo reset starts a new chain on the ship and a new mirror epoch on the ground; the invariant applies to the current epoch.
- **Clock skew:** nothing needs synchronised clocks. RTT comes from echoed ping times on the sender's own clock; vitals timestamps are relative to the stream's keyframe, and a timestamp that goes backwards forces a new keyframe.

## 8. Link profiles (simulator)

| profile | one way | jitter | bandwidth | loss | MTU | contact gaps | CRC |
|---|---|---|---|---|---|---|---|
| Ship LAN | 2 ms | 3 ms | 50 Mbit/s | 0 | 65 000 | none | no |
| LoRa-like radio | 60 ms | 60 ms | 5.47 kbit/s | 5% | 222 | none | yes |
| **Near-space uplink** | 1.2 s | 0.8 s | 64 kbit/s | 2% | 1024 | 20 s every 2 min | yes |
| Uplink during solar storm | 2.5 s | 1.5 s | 9.6 kbit/s | 15% | 512 | 25 s every 1 min | yes |
| Link outage | — | — | 0 | 100% | 0 | always | yes |

`LinkSim` is deterministic (seeded xorshift, explicit clock). The ship's uplink runs *both directions* through it; switch profiles live in the dev panel.

## 9. Mesh behaviour at the hub

Frames with a `dst` other than 0 are relayed verbatim to that node. Each client has a bounded outbox (512 datagrams); a client that falls behind loses frames (vitals and presence are latest-wins, reliable frames are retransmitted by their sender) and is closed if it cannot take a frame within 5 s. A WebSocket message above 256 KB closes that connection. Band vitals refresh open pages at once, at most 5 times a second. The hub pings every node every 2 s and keeps an RTT estimate per client; it broadcasts Presence to apps and dashboards every 2 s. Two browsers measure their end-to-end RTT through the hub by pinging each other's node id (dev panel → "Ping every client end to end"). See ADR-008.

## 10. Byte accounting

`ByteStats` counts frames, bytes, payload bytes and retransmissions per type on both sides. Exposed in `/api/link`, the dashboard's sync card, the dev panel and `eva bench`; the table of typical sizes is `GET /api/protocol/sizes`. Measured sizes, throughput and per-profile delivery: [BENCHMARKS.md](BENCHMARKS.md).

## 11. The medical station (types 13–15)

The ship's medical station (12 compartments, an Eva screen, door and item sensors, a red button, a LoRa radio, a backup battery) is a **client of the hub** (ADR-011). It says `Hello` with role 6, gets a node id, answers the hub's pings like any node, and keeps one reliable session: the hub queues `StationCmd`s on it, the station queues `StationEvent`s. Code: `app/src/station.rs`.

Compartments are numbered 1–12 as on the cabinet: 01 AED, 02 Breathing, 03 Diagnostics, 04 Ultrasound, 05 Wounds, 06 Restraint, 07 Oral meds, 08 Injectables (the crew's medical kit), 09 IV access, 10 Eyes & dental, 11 Airway, 12 Hygiene. Masks use one bit per compartment, bit 0 = compartment 1; anything above bit 11 is rejected.

**StationCmd** (13, hub → station, guidance, reliable)

| field | encoding | values |
|---|---|---|
| op | `u8` | 0 clear all lights, 1 light one, 2 clear one, 3 restock (demo reset: all idle and full) |
| compartment | `u8` | 1–12; 0 only with ops 0 and 3 |
| case | `uvar` | the case that asked for it, 0 = none |
| text | `str` | what the Eva screen says ("Take the AED · Ana, seat 4B"); may be empty |

Light 01 with "Take the AED", case 101: `01 01 65 0c 54 61 6b 65 20 74 68 65 20 41 45 44`, 16 bytes of payload, 23 on the wire (with the station's node id as `dst` and a two-byte seq).

**StationEvent** (14, station → hub, reliable; guidance, the red button in the alarm class)

| field | encoding | values |
|---|---|---|
| event | `u8` | 1 door opened, 2 door closed, 3 item taken, 4 red button, 5 online (back), 6 LoRa alarm heard, 7 lit by itself |
| compartment | `u8` | 1–12, or 0 when not about one compartment |
| t_s | `uvar` | the station's own clock, mission seconds: a buffered event keeps the time it happened |
| count | `uvar` | items left (item events), events it buffered (online), else 0 |
| subject | `str` | the seat for "LoRa alarm heard", else empty |

"AED taken" at 13:58:47 on day 97 (t_s 8 344 727): `03 01 97 a9 fd 03 00 00`, 8 bytes of payload, 14 on the wire.

**StationStatus** (15, station → hub, vitals class: latest-wins, never retransmitted). `u8 battery` (0–100) · `u8 flags` (1 standalone, 2 on backup battery) · `uvar lit` · `uvar open` · `uvar taken` (masks) · `uvar buffered` · `bytes items` (one count per compartment, at most 12). Sent on every change and at least every 10 s; 19 bytes of payload.

**What the hub does with a report.** Every `StationEvent` becomes one hash-chained, Ed25519-signed record (kind `station`, the red button `station_alarm`), so it reaches the ground like any other record; the body carries `event`, `compartment`, `station_s`, `count` and, when the report adds a line to the case timeline, `case_step` (for example "AED taken at the station · 00:47"), which the ground mirrors. "AED taken" (event 3 in compartment 1) marks the case once, whether it comes from the item sensor or from the runner tapping "I have it". The red button is broadcast to every app and dashboard as an `Alarm` with code 6.

**Hub down (fallback).** If the station hears nothing from the hub for 5 s (two missed heartbeats) it runs **standalone**: events wait in its send queue (store-and-forward, §7), a wristband's no-pulse `Alarm` heard on the air (LoRa broadcast, unreliable, CRC on) lights 01 and shows "Emergency · Take the AED" without the hub, and its red button goes on the air as an `Alarm` with code 6 to every wristband. When the hub's first frame arrives again it queues `online` with the number of buffered events, retransmits everything in flight at once (`SendQueue::retransmit_now`, the RTO backoff is reset), and the hub signs each event into the chain in the order it was queued, with the station time it happened. In the demo, the dev panel's **Hub down** stops the hub's relay, heartbeat and rules, and `AppState::air` stands in for the radio (apps and dashboards hear it through their sessions, as their wristbands would). Tests: `tests/station.rs`.

| message | bytes on the wire |
|---|---|
| station: light 01 "Take the AED" | 23 |
| station: AED taken | 14 |
| station: status | 25 |
