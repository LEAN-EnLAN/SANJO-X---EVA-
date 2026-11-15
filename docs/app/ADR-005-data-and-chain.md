# ADR-005 · Data model and the hash-chained record log

- **Status:** accepted · 15/11/2026 · Claude

## Context

Eva must keep every clinically relevant step (alarms, guidance, tool calls, ECGs, journal check-ins shared with the crew) as a record that the ground can trust: *signed on board → sent → acknowledged*. Data also includes passengers, vitals time series, alerts, cases, the mood journal, daily logs (water, meals, sleep, reminders) and Eva conversations.

## Decision

**Two layers of data:**

1. **Working set in memory** (`state::World`): passengers with their band state and a 2-hour history (one point every 15 s), alerts, cases, journal, meals, reminders, chats, messages, storm, link status. Fast to render on every SSE tick; rebuilt from seed data on start.
2. **Durable hash chain** (`chain.rs`): an append-only JSON-lines file per node (`data/chain-<node>.jsonl`). One line per record and one line per state change; replaying the file rebuilds the chain and its states.

**Record:** `seq, ts_ms (mission time), kind, subject, body (JSON), prev, hash, state, sent_ms, acked_ms, origin`.
`hash = SHA-256(prev ‖ seq ‖ ts ‖ kind ‖ 0 ‖ subject ‖ 0 ‖ origin ‖ 0 ‖ JSON(body))`. Editing or deleting any record breaks every later hash; `Chain::verify` names the first bad record, `eva verify` checks files, `/api/chain/verify` checks live.

**States:** `Signed` (appended on board) → `Sent` (in flight on the uplink) → `Acked` (the ground acknowledged). States only move forward.

**Ground mirror:** the ship sends each record with an 8-byte hash prefix; the first record of a session carries the previous full hash as an anchor. The ground recomputes SHA-256 as soon as it knows the predecessor and marks the record *Verified* (or failed). Records may arrive out of order.

**Privacy:** private journal text never enters the chain (tool calls log `"(private)"`); the chain is what travels to the ground. The person's own export (`/export/journal.csv?seat=`) includes it; the ship report includes only shared entries.

**What "signed" means:** hashed, chained and **Ed25519-signed** with the node's own key (ADR-010, from 15/11). Per-band keys are a later step.

## Why not SQLite now

rusqlite with `bundled` is fine and remains the plan for the working set once screens need queries over days of data. For the first skeleton, an append-only file is simpler to inspect, has no C build step, and is exactly the shape of an audit log. The seam is `state.rs` (world) and `chain.rs` (records).

## Consequences

- `Reset demo` truncates the chain file and re-seeds four acknowledged morning records.
- The ground's mirror is persisted (`data/mirror-<node>.jsonl`) and replayed on restart (ADR-010).
- Exports read straight from the chain: CSV (formula-safe quoting), JSON (with the verify report), printable HTML report.
