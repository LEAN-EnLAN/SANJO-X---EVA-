//! Hash-chained record log.
//!
//! Every clinically relevant step (alarm, guidance, journal entry shared with
//! the crew lead, tool call, daily summary…) becomes a [`Record`]. Each record
//! carries the SHA-256 of the previous one, so any later edit or deletion
//! breaks the chain and [`Chain::verify`] reports where.
//!
//! Records move through three states shown in the UI:
//! **signed on board** → **sent** (on the uplink) → **acknowledged** (the
//! ground confirmed it, verified against its own copy of the chain).
//!
//! Storage is an append-only JSON-lines file: one line per record, plus one
//! line per state change. Replaying the file rebuilds the chain. "Signed" means
//! hashed and chained with the ship id; per-device Ed25519 signatures are a
//! documented next step (ADR-005).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncState {
    /// hashed and chained on board
    Signed,
    /// handed to the uplink, waiting for the ground
    Sent,
    /// the ground acknowledged it
    Acked,
}

impl SyncState {
    pub fn label(self) -> &'static str {
        match self {
            SyncState::Signed => "Signed on board",
            SyncState::Sent => "Sent",
            SyncState::Acked => "Acknowledged",
        }
    }
    pub fn step(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub seq: u64,
    pub ts_ms: u64,
    /// alarm | guidance | case | vitals_summary | journal | log | tool | agent | link | export | note
    pub kind: String,
    /// seat ("4B"), "ship", "crew"…
    pub subject: String,
    pub body: serde_json::Value,
    pub prev: String,
    pub hash: String,
    pub state: SyncState,
    pub sent_ms: Option<u64>,
    pub acked_ms: Option<u64>,
    /// node that signed it (ship id)
    pub origin: String,
}

impl Record {
    /// One plain line for lists: body.text if present, else the kind.
    pub fn summary(&self) -> String {
        self.body.get("text").and_then(|t| t.as_str()).map(|s| s.to_string()).unwrap_or_else(|| self.kind.replace('_', " "))
    }
    pub fn short_hash(&self) -> &str {
        &self.hash[..12.min(self.hash.len())]
    }
}

/// SHA-256 over prev ‖ seq ‖ ts ‖ kind ‖ 0 ‖ subject ‖ 0 ‖ origin ‖ 0 ‖ canonical JSON body.
pub fn compute_hash(prev_hex: &str, seq: u64, ts_ms: u64, kind: &str, subject: &str, origin: &str, body: &serde_json::Value) -> String {
    let mut h = Sha256::new();
    h.update(hex::decode(prev_hex).unwrap_or_default());
    h.update(seq.to_be_bytes());
    h.update(ts_ms.to_be_bytes());
    for part in [kind, subject, origin] {
        h.update(part.as_bytes());
        h.update([0u8]);
    }
    // serde_json::Value objects are BTreeMap-ordered unless preserve_order is on: canonical enough
    h.update(serde_json::to_vec(body).unwrap_or_default());
    hex::encode(h.finalize())
}

pub const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "lowercase")]
enum Line {
    Rec(Record),
    State { seq: u64, state: SyncState, at: u64 },
}

#[derive(Debug)]
pub struct Chain {
    pub origin: String,
    pub records: Vec<Record>,
    file: Option<File>,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerifyReport {
    pub ok: bool,
    pub records: usize,
    pub first_bad: Option<u64>,
    pub head: String,
}

impl Chain {
    pub fn in_memory(origin: &str) -> Self {
        Self { origin: origin.into(), records: Vec::new(), file: None, path: None }
    }

    /// Open (or create) the JSONL file and replay it.
    pub fn open(path: &Path, origin: &str) -> std::io::Result<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut records: Vec<Record> = Vec::new();
        if path.exists() {
            for line in BufReader::new(File::open(path)?).lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                match serde_json::from_str::<Line>(&line) {
                    Ok(Line::Rec(r)) => records.push(r),
                    Ok(Line::State { seq, state, at }) => {
                        if let Some(r) = records.get_mut(seq as usize) {
                            apply_state(r, state, at);
                        }
                    }
                    Err(e) => tracing::warn!("chain: skipping unreadable line: {e}"),
                }
            }
        }
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self { origin: origin.into(), records, file: Some(file), path: Some(path.to_path_buf()) })
    }

    pub fn head(&self) -> &str {
        self.records.last().map(|r| r.hash.as_str()).unwrap_or(GENESIS)
    }
    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Append a record signed on board. Returns its seq.
    pub fn append(&mut self, ts_ms: u64, kind: &str, subject: &str, body: serde_json::Value) -> u64 {
        let seq = self.records.len() as u64;
        let prev = self.head().to_string();
        let hash = compute_hash(&prev, seq, ts_ms, kind, subject, &self.origin, &body);
        let r = Record { seq, ts_ms, kind: kind.into(), subject: subject.into(), body, prev, hash, state: SyncState::Signed, sent_ms: None, acked_ms: None, origin: self.origin.clone() };
        self.write(&Line::Rec(r.clone()));
        self.records.push(r);
        seq
    }

    /// Insert a record received from elsewhere (the ground's mirror), keeping its hash.
    pub fn insert_foreign(&mut self, r: Record) {
        if (r.seq as usize) < self.records.len() {
            return;
        }
        self.write(&Line::Rec(r.clone()));
        self.records.push(r);
    }

    pub fn set_state(&mut self, seq: u64, state: SyncState, at: u64) {
        let changed = match self.records.get_mut(seq as usize) {
            Some(r) if r.state.step() < state.step() => {
                apply_state(r, state, at);
                true
            }
            _ => false,
        };
        if changed {
            self.write(&Line::State { seq, state, at });
        }
    }

    pub fn verify(&self) -> VerifyReport {
        let mut prev = GENESIS.to_string();
        for (i, r) in self.records.iter().enumerate() {
            let want = compute_hash(&prev, r.seq, r.ts_ms, &r.kind, &r.subject, &r.origin, &r.body);
            if r.seq as usize != i || r.prev != prev || r.hash != want {
                return VerifyReport { ok: false, records: self.records.len(), first_bad: Some(i as u64), head: self.head().into() };
            }
            prev = r.hash.clone();
        }
        VerifyReport { ok: true, records: self.records.len(), first_bad: None, head: self.head().into() }
    }

    pub fn counts(&self) -> (usize, usize, usize) {
        let mut c = (0, 0, 0);
        for r in &self.records {
            match r.state {
                SyncState::Signed => c.0 += 1,
                SyncState::Sent => c.1 += 1,
                SyncState::Acked => c.2 += 1,
            }
        }
        c
    }

    fn write(&mut self, line: &Line) {
        if let Some(f) = self.file.as_mut() {
            if let Ok(mut s) = serde_json::to_string(line) {
                s.push('\n');
                if let Err(e) = f.write_all(s.as_bytes()) {
                    tracing::error!("chain: write failed: {e}");
                }
            }
        }
    }

    /// Wipe the chain (demo reset): truncates the file.
    pub fn reset(&mut self) {
        self.records.clear();
        if let Some(p) = &self.path {
            if let Ok(f) = OpenOptions::new().create(true).write(true).truncate(true).open(p) {
                drop(f);
            }
            self.file = OpenOptions::new().append(true).open(p).ok();
        }
    }
}

fn apply_state(r: &mut Record, state: SyncState, at: u64) {
    r.state = state;
    match state {
        SyncState::Sent => r.sent_ms = Some(at),
        SyncState::Acked => {
            r.acked_ms = Some(at);
            r.sent_ms.get_or_insert(at);
        }
        SyncState::Signed => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn append_verify_and_tamper() {
        let mut c = Chain::in_memory("ship-1");
        c.append(1, "alarm", "4B", json!({"text": "No pulse"}));
        c.append(2, "guidance", "3B", json!({"text": "Start CPR"}));
        c.append(3, "log", "ship", json!({"n": 3}));
        assert!(c.verify().ok);
        c.records[1].body = json!({"text": "edited later"});
        let rep = c.verify();
        assert!(!rep.ok);
        assert_eq!(rep.first_bad, Some(1));
    }

    #[test]
    fn states_only_move_forward() {
        let mut c = Chain::in_memory("ship-1");
        let s = c.append(1, "alarm", "4B", json!({}));
        c.set_state(s, SyncState::Acked, 10);
        c.set_state(s, SyncState::Sent, 11);
        assert_eq!(c.records[0].state, SyncState::Acked);
        assert_eq!(c.counts(), (0, 0, 1));
    }

    #[test]
    fn file_replay_rebuilds_chain_and_states() {
        let dir = std::env::temp_dir().join(format!("eva-chain-test-{}", std::process::id()));
        let path = dir.join("chain.jsonl");
        let _ = std::fs::remove_file(&path);
        {
            let mut c = Chain::open(&path, "ship-1").unwrap();
            c.append(1, "alarm", "4B", json!({"text": "No pulse"}));
            let s = c.append(2, "log", "ship", json!({}));
            c.set_state(s, SyncState::Sent, 5);
        }
        let c = Chain::open(&path, "ship-1").unwrap();
        assert_eq!(c.len(), 2);
        assert!(c.verify().ok);
        assert_eq!(c.records[1].state, SyncState::Sent);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
