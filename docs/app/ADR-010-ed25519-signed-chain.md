# ADR-010 · Ed25519 signatures on every chain record

- **Status:** accepted · 15/11/2026 · Claude (for Sanjo-X)
- **Extends:** ADR-005 (data and the hash chain), ADR-003 (Eva Link)

## Context

The hash chain (ADR-005) makes edits *visible*, but anyone with write access to the JSONL file can edit a record and recompute every hash after it; the chain then verifies again. The ground also had no way to tell a record really came from the ship. Its mirror lived in memory, so a ground restart lost it.

## Decision

1. **One Ed25519 key pair per node**, created on first run: `data/keys/<node>.ed25519` (PKCS#8, file mode 0600) and `data/keys/<node>.pub` (hex). Library: `ring` (already in the build through rustls; no new C or cmake dependency).
2. **Every record is signed**: `sig = Ed25519(sk, "eva-record-v1\0" ‖ SHA-256 hash)`. The hash already covers prev, seq, time, kind, subject, origin and body, so signing the hash signs everything. The domain prefix keeps a record signature from ever being valid for anything else.
3. **The chain file says which key signs**: its first line is `{"t":"key","public":…,"from_seq":0}`. If the node's key changes, a new key line applies from that seq on. A file written before signatures existed is moved aside (`*.legacy-unsigned.jsonl`) and a fresh signed chain starts.
4. **Verification** (`Chain::verify`, `GET /api/chain/verify`, `eva verify`): sequence, links, hashes, then signatures. `eva verify` also checks the file's signing key against the node's pinned `<node>.pub`, so replacing the key line and re-signing the whole chain is caught. Exit code 1 on any failure.
5. **On the wire**: optional trailing fields, backward compatible. `Hello` from a ship carries its 32-byte public key; `Record` carries the 64-byte signature (126 bytes per signed record vs 61 unsigned; BENCHMARKS.md).
6. **The ground pins the ship's key on first use** (`data/keys/peer-<ship>.pub`) and verifies each record's hash *and* signature as soon as its predecessor is known (records may arrive out of order). A different key for a known ship is refused and logged; deleting the pin file accepts a new ship key.
7. **The ground's mirror is persisted** (`data/mirror-<node>.jsonl`, one line per received record, plus key lines), replayed on start, and the ground's cases are rebuilt from it. Inserting a record already held (retransmit, replay, reconnect) is a no-op. If the ship resets its chain (demo reset: same seq, different record), the old mirror is archived as `mirror-<node>.epochN.jsonl` and a new epoch starts.
8. **Ship restart**: records left *Sent* by a killed process are queued again (the ground drops duplicates); a chain reset opens a new uplink session.

## Consequences

- Tampering with a byte anywhere in a record, or rewriting and re-hashing the chain, is reported (tests: `chain::tests::flipping_one_byte_in_the_file_is_detected`, `tests/signatures.rs`, `tests/break_restart.rs`).
- Signing costs ~22 µs per record; verifying ~36 µs (BENCHMARKS.md). Irrelevant at one record per second.
- Trust on first use: whoever connects first as a ship name gets pinned. Acceptable for the demo; a real deployment provisions ship public keys on the ground before flight (copy `ship.pub` to `peer-ship.pub`).
- The private key is a plain file. Losing it means the node signs with a new key from then on (a new key line); stealing it allows forging. Hardware keys (TPM / secure element on the hub and bands) are future work.
- Per-band keys (each WearMeds band signing its own vitals) would use the same record layout; not done.
