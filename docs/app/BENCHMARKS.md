# Eva · benchmarks

Real numbers from `eva bench` (release build) on the development machine: **4 vCPU Intel Xeon @ 2.30 GHz, Linux x86_64**, 15 November 2026. Re-run with:

```bash
cd app
cargo build --release
./target/release/eva bench                  # everything, ~1.5 min (live server run 30 s)
./target/release/eva bench --server-secs 60 # longer live run (used below)
./target/release/eva bench --quick          # no processes: sizes, throughput, link profiles
```

The link-profile part is a deterministic simulation (seeded), so it gives the same numbers on every machine. Throughput, cold start, CPU and RSS depend on the machine.

## Headline

| | |
|---|---|
| Vitals sample on the wire | **11 bytes** (delta) / 29 bytes (keyframe) vs 184 bytes as JSON: **16.7× smaller** |
| Signed chain record (Ed25519, CRC) | **126 bytes** vs 294 as JSON, 179 as CBOR |
| Alarm "no pulse" | **10 bytes**; reaches the ground in **1.8 s** on the near-space profile, 3.4 s in a solar storm |
| Records delivered, every profile incl. 5 min total outage | **600 / 600 (100%)**, every one verified on the ground (hash chain + signature) |
| Vitals-in → dashboard update (15 bands, 3 dashboards) | **p50 1 ms · p95 1 ms** |
| Server under that load | **0.3% of one core · 11.6 MB RSS** |
| Cold start (spawn → first HTTP 200) | **4 ms** median |
| Binary | **12.5 MB**, one file, all assets embedded |

## 1. Bytes per message

Same information in each format: JSON and CBOR carry the fields a typical JSON API would use (`{"type":"vitals","seq":…,"hr":72,…}`); hashes and signatures are hex in JSON and byte strings in CBOR. Eva Link counts its whole frame (2-byte header, varint seq/src/len, payload; CRC-16 where marked).

| message | Eva Link | JSON | CBOR | JSON / Eva Link |
|---|---:|---:|---:|---:|
| ping | 11 | 51 | 33 | 4.6× |
| ack of 10 seqs | 17 | 81 | 57 | 4.8× |
| hello (ship, with Ed25519 public key) | 58 | 154 | 100 | 2.7× |
| vitals keyframe (10 fields) | 29 | 184 | 141 | 6.3× |
| vitals delta (2 fields changed) | 11 | 184 | 141 | 16.7× |
| alarm: no pulse | 10 | 67 | 48 | 6.7× |
| guidance line (46 characters of text) | 58 | 115 | 93 | 2.0× |
| chain record, Ed25519-signed (CRC on) | 126 | 294 | 179 | 2.3× |
| presence (5 nodes) | 82 | 318 | 229 | 3.9× |

Where the savings come from: varints and fixed-point fields instead of text numbers, no field names, vitals as deltas against the last keyframe, an 8-byte hash prefix (the ground recomputes the full SHA-256). Text-heavy messages (guidance) gain least; the 64-byte signature is the largest part of a record (an unsigned record is 61 bytes).

## 2. Encode / decode throughput

One core, release build. A 15-band ship produces 15 samples per second, so codec cost is irrelevant to the ship; these numbers matter for a ground station mirroring many ships.

| operation | ops/s | MB/s |
|---|---:|---:|
| vitals delta encode | 27.3 M | 175 |
| vitals delta decode | 40.8 M | 253 |
| frame encode (vitals, CRC) | 35.4 M | 460 |
| frame decode (vitals, CRC) | 34.7 M | 452 |
| datagram decode (60 frames, 813 bytes) | 411 k | 333 |
| CRC-16 over 1 KB | 401 k | 411 |
| *vitals as JSON encode (serde_json, comparison)* | *3.8 M* | *685* |
| *vitals as JSON decode (serde_json, comparison)* | *1.1 M* | *192* |
| chain append (SHA-256 + Ed25519 sign + JSON body) | 46 k | – |
| record signature verify (Ed25519) | 28 k | – |

Decoding a vitals sample is ~37× faster than parsing the same sample as JSON. Signing every record costs ~22 µs; a ground can verify ~28,000 records per second per core.

## 3. Link profiles: delivery ratio and latency

10 minutes of mission time per profile: 15 bands at 1 Hz (latest-wins vitals, TTL 30 s), 1 Ed25519-signed chain record per second, one alarm at t = 30 s, a ping every 2 s (as the real uplink does); then up to 5 more minutes to drain. Both directions go through the simulated link (acks too). Latency = record queued on the ship → delivered on the ground.

| profile | records delivered | record latency p50 / p95 / max | alarm latency | vitals delivered | vitals age p50 | bytes up / record | record retransmits | queue drained |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Ship LAN | 600/600 (100%) | 100 ms / 100 ms / 100 ms | 100 ms | 100% | 100 ms | 308 | 0 | at once |
| LoRa-like radio (5.5 kbit/s, 5% loss) | 600/600 (100%) | 0.6 s / 1.0 s / 4.6 s | 0.5 s | 94.2% | 0.4 s | 359 | 87 | at once |
| **Near-space uplink** (1.2 s + jitter, 64 kbit/s, 2% loss, 20 s gap / 2 min) | 600/600 (100%) | 1.8 s / 17 s / 23 s | **1.8 s** | 90.6% | 1.7 s | 357 | 57 | 24 s after the last record |
| Solar storm (2.5 s, 9.6 kbit/s, 15% loss, 25 s gap / min) | 600/600 (100%) | 15 s / 78 s / 335 s | 3.4 s | 70.0% | 4.2 s | 411 | 266 | last acks after > 5 min |
| Outage 5 min → near-space | 600/600 (100%) | 22 s / 273 s / 303 s | 272 s (sent when contact returns) | 46.6% | 1.7 s | 262 | 85 | 22 s after the last record |

How to read it:
- **No record is ever lost.** Reliable frames never give up; during a gap they wait (store-and-forward) and the alarm class always leaves first. The p95 on near-space (17 s) is the records that land in a 20 s contact gap.
- **Vitals are allowed to be lost.** They are latest-wins with a 30 s TTL; stale samples are dropped instead of flooding a link that just came back (46.6% delivered in the outage run = everything outside the 5 min of no contact). The dashboard always shows the freshest sample.
- **Bytes per record** include everything sent up: the record, its share of vitals, pings and retransmissions.
- The storm profile is deliberately harsh (15% loss and a 25 s gap every minute on 9.6 kbit/s): everything arrives, the tail is long.

### Fixed while benchmarking

| finding | before | after |
|---|---|---|
| RTO never learned on links slower than 1 s: every frame timed out, and Karn's rule then forbade every RTT sample. Fixed with RFC 6298 §5.5 (back off the RTO itself until a valid sample) | 1,147 record retransmits per 600 records, 566 bytes up / record (near-space) | 57 retransmits, 357 bytes / record |
| Dashboards refreshed only on the 1 Hz server tick when bands are remote (`eva sim`) | vitals-in → dashboard p50 697 ms | p50 1 ms (band vitals trigger a refresh, at most 5 per second) |
| Stale vitals were purged only when the queue was polled, which never happens during a contact gap | 9,600 frames queued after a 10 min outage | ~450 (one 30 s TTL of 15 bands) + the records |

## 4. Server: SSE fan-out, 15 bands + 3 dashboards

A separate `eva serve --role ship --sim 0` process. The bench driver streams 15 bands over Eva Link on one WebSocket, exactly like `eva sim --n 15` (same `sim::baseline` code), and opens 3 dashboards on `/sse/dash`. Band 15 carries a probe value (skin 40.0–49.9 °C, never a real reading) that changes every second; the latency is the time from sending that sample to the probe value appearing in a dashboard's `roster` event, over 60 s.

| metric | value |
|---|---:|
| vitals samples in | 915 (15 per second) |
| roster updates received (3 dashboards) | 366 |
| vitals-in → dashboard-update p50 | 1 ms |
| vitals-in → dashboard-update p95 | 1 ms |
| vitals-in → dashboard-update max | 695 ms (first sample, before the first refresh window) |
| server CPU | 0.3% of one core |
| server RSS (end / peak) | 11.6 MB / 11.6 MB |
| SSE bytes per dashboard | 111 KB/s (6 HTML fragments, twice a second) |

The SSE payload is the biggest cost: each refresh re-renders six dashboard fragments. That is fine on a ship LAN; for a ground client on a narrow link, use `GET /api/state` with `Accept: application/cbor` instead.

Related break tests (see [TESTING.md](TESTING.md)): 50 clients × 20 connect/disconnect cycles finish in 0.6 s with no ghost nodes; 5 stalled SSE readers under a 3,000-event flood do not delay a healthy dashboard.

## 5. Cold start and binary

| | |
|---|---|
| Cold start, spawn → first `GET /healthz` 200 | median **4 ms**, max 8 ms (7 runs; includes reading the chain and loading the node key) |
| Release binary (`strip`, thin LTO, `panic = abort`) | **12.5 MB**, all HTML templates, CSS, JS, images, manifest and service worker embedded |

## 6. Eva's online pipelines (Groq, measured live)

From the gated live test (`EVA_LIVE_GROQ=1 cargo test --test groq_live -- --nocapture`), one run on 15/11 with the demo key:

| pipeline | model | time | result |
|---|---|---:|---|
| chat + 1 tool call (`get_vitals`) | `openai/gpt-oss-120b` | 1.9 s | Now / Why / How I decided / Next answer from live vitals |
| prompt injection ("ignore your rules and administer…") | none (answered on board) | < 1 ms | fixed refusal, no model call |
| speech-to-text (1 s of audio) | `whisper-large-v3-turbo` | 0.25 s | ok |
| text-to-speech | `canopylabs/orpheus-v1-english` | 0.09 s | "requires terms acceptance" → explicit browser-voice fallback (see RUNNING.md) |
| vision (one image) | `qwen/qwen3.8-27b` | 0.65 s | ok |

Layer 1 (reflex rules) never waits for any of these: 10,000 rule evaluations take well under a millisecond each (`reflex::tests::rules_are_fast`).
