//! Hash-chained, Ed25519-signed record log.
//!
//! Every clinically relevant step (alarm, guidance, journal entry shared with
//! the crew lead, tool call, daily summary…) becomes a [`Record`]. Each record
//! carries the SHA-256 of the previous one, so any later edit or deletion
//! breaks the chain, and an **Ed25519 signature** over its hash made with the
//! node's own key, so nobody without that key can rewrite the chain and
//! recompute the hashes. [`Chain::verify`] checks both and reports where it
//! breaks.
//!
//! Records move through three states shown in the UI:
//! **signed on board** → **sent** (on the uplink) → **acknowledged** (the
//! ground confirmed it, verified against its own copy of the chain).
//!
//! Storage is an append-only JSON-lines file:
//! * `{"t":"key", …}`: the public key that signs the records after it (first
//!   line of the file; a new one only if the node's key changed),
//! * `{"t":"rec", …}`: one record,
//! * `{"t":"state", …}`: one state change.
//!
//! Replaying the file rebuilds the chain. The private key lives next to it in
//! `<data>/keys/<node>.ed25519` (PKCS#8, created on first run, mode 0600) and
//! the public key in `<data>/keys/<node>.pub` (hex). See ADR-010.

use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncState {
    /// hashed, chained and signed on board
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
    /// Ed25519 signature (hex, 64 bytes) over `SIG_DOMAIN ‖ hash`, by `origin`'s key
    #[serde(default)]
    pub sig: String,
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

/// Domain separation for record signatures (a signature over a record hash
/// can never be confused with a signature over anything else).
pub const SIG_DOMAIN: &[u8] = b"eva-record-v1\0";

// ------------------------------------------------------------------ keys

/// A node's Ed25519 key pair. `Debug` shows only the public key.
pub struct NodeKey {
    pair: Ed25519KeyPair,
}

impl std::fmt::Debug for NodeKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeKey(pub {})", &self.public_hex()[..16])
    }
}

fn safe_name(node: &str) -> String {
    node.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' }).collect()
}

impl NodeKey {
    /// A fresh key in memory (tests, in-memory chains).
    pub fn generate() -> Self {
        let der = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).expect("system randomness");
        Self::from_pkcs8(der.as_ref()).expect("fresh pkcs8")
    }
    pub fn from_pkcs8(der: &[u8]) -> Result<Self, String> {
        Ed25519KeyPair::from_pkcs8(der).or_else(|_| Ed25519KeyPair::from_pkcs8_maybe_unchecked(der)).map(|pair| NodeKey { pair }).map_err(|e| format!("bad key file: {e}"))
    }
    /// Path of the private key for `node` under `dir`.
    pub fn key_path(dir: &Path, node: &str) -> PathBuf {
        dir.join(format!("{}.ed25519", safe_name(node)))
    }
    pub fn pub_path(dir: &Path, node: &str) -> PathBuf {
        dir.join(format!("{}.pub", safe_name(node)))
    }
    /// Load `<dir>/<node>.ed25519`, or create it (and `<node>.pub`) on first run.
    pub fn load_or_create(dir: &Path, node: &str) -> std::io::Result<Self> {
        let kp = Self::key_path(dir, node);
        if kp.exists() {
            let der = std::fs::read(&kp)?;
            return Self::from_pkcs8(&der).map_err(std::io::Error::other);
        }
        std::fs::create_dir_all(dir)?;
        let der = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).map_err(|_| std::io::Error::other("no system randomness"))?;
        let mut opts = OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        opts.open(&kp)?.write_all(der.as_ref())?;
        let key = Self::from_pkcs8(der.as_ref()).map_err(std::io::Error::other)?;
        std::fs::write(Self::pub_path(dir, node), format!("{}\n", key.public_hex()))?;
        tracing::info!("chain: new Ed25519 key for '{node}' in {}", kp.display());
        Ok(key)
    }
    pub fn public(&self) -> [u8; 32] {
        let mut out = [0u8; 32];
        out.copy_from_slice(self.pair.public_key().as_ref());
        out
    }
    pub fn public_hex(&self) -> String {
        hex::encode(self.pair.public_key().as_ref())
    }
    /// Sign a record hash (hex). Returns the signature as hex.
    pub fn sign_hash(&self, hash_hex: &str) -> String {
        let mut msg = SIG_DOMAIN.to_vec();
        msg.extend(hex::decode(hash_hex).unwrap_or_default());
        hex::encode(self.pair.sign(&msg).as_ref())
    }
}

/// Read a pinned public key file (`<node>.pub`, hex).
pub fn read_pub_file(path: &Path) -> Option<[u8; 32]> {
    let s = std::fs::read_to_string(path).ok()?;
    parse_public(s.trim())
}

pub fn parse_public(hex_str: &str) -> Option<[u8; 32]> {
    let v = hex::decode(hex_str).ok()?;
    (v.len() == 32).then(|| {
        let mut k = [0u8; 32];
        k.copy_from_slice(&v);
        k
    })
}

/// Check a record signature: Ed25519(public, SIG_DOMAIN ‖ hash).
pub fn verify_sig(public: &[u8], hash_hex: &str, sig: &[u8]) -> bool {
    let Ok(h) = hex::decode(hash_hex) else { return false };
    if h.len() != 32 || sig.len() != 64 || public.len() != 32 {
        return false;
    }
    let mut msg = SIG_DOMAIN.to_vec();
    msg.extend(h);
    UnparsedPublicKey::new(&ED25519, public).verify(&msg, sig).is_ok()
}

pub fn verify_sig_hex(public: &[u8], hash_hex: &str, sig_hex: &str) -> bool {
    hex::decode(sig_hex).is_ok_and(|s| verify_sig(public, hash_hex, &s))
}

// ------------------------------------------------------------------ file

#[derive(Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "lowercase")]
enum Line {
    Key { origin: String, public: String, from_seq: u64 },
    Rec(Record),
    State { seq: u64, state: SyncState, at: u64 },
}

/// Everything read back from a chain file.
#[derive(Debug, Default)]
pub struct FileContents {
    pub records: Vec<Record>,
    /// (first seq signed by this key, public key)
    pub keys: Vec<(u64, [u8; 32])>,
    /// lines that were not valid JSON (damage or tampering)
    pub unreadable: usize,
}

/// Parse a chain file without opening it for writing.
pub fn read_file(path: &Path) -> std::io::Result<FileContents> {
    let mut out = FileContents::default();
    if !path.exists() {
        return Ok(out);
    }
    let mut rd = BufReader::new(File::open(path)?);
    let mut raw = Vec::new();
    loop {
        raw.clear();
        if rd.read_until(b'\n', &mut raw)? == 0 {
            break;
        }
        let Ok(line) = std::str::from_utf8(&raw) else {
            out.unreadable += 1;
            continue;
        };
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Line>(line) {
            Ok(Line::Rec(r)) => out.records.push(r),
            Ok(Line::State { seq, state, at }) => {
                if let Some(r) = out.records.get_mut(seq as usize) {
                    if r.state.step() < state.step() {
                        apply_state(r, state, at);
                    }
                }
            }
            Ok(Line::Key { public, from_seq, .. }) => match parse_public(&public) {
                Some(k) => out.keys.push((from_seq, k)),
                None => out.unreadable += 1,
            },
            Err(_) => out.unreadable += 1,
        }
    }
    Ok(out)
}

#[derive(Debug)]
pub struct Chain {
    pub origin: String,
    pub records: Vec<Record>,
    file: Option<File>,
    pub path: Option<PathBuf>,
    key: NodeKey,
    /// (first seq, public key) for every key that signed part of this chain
    keys: Vec<(u64, [u8; 32])>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerifyReport {
    pub ok: bool,
    pub records: usize,
    pub first_bad: Option<u64>,
    pub head: String,
    /// records whose Ed25519 signature checked out
    pub signatures: usize,
    /// why the chain is broken (None when ok)
    pub reason: Option<String>,
}

/// Verify a list of records: sequence, hash links, hashes and signatures.
pub fn verify_records(records: &[Record], keys: &[(u64, [u8; 32])]) -> VerifyReport {
    let head = records.last().map(|r| r.hash.clone()).unwrap_or_else(|| GENESIS.into());
    let bad = |i: usize, sigs: usize, why: String| VerifyReport { ok: false, records: records.len(), first_bad: Some(i as u64), head: head.clone(), signatures: sigs, reason: Some(why) };
    let mut prev = GENESIS.to_string();
    let mut sigs = 0;
    for (i, r) in records.iter().enumerate() {
        if r.seq as usize != i {
            return bad(i, sigs, format!("record {i} missing (found seq {})", r.seq));
        }
        if r.prev != prev {
            return bad(i, sigs, format!("record {i} does not link to record {}", i as i64 - 1));
        }
        let want = compute_hash(&prev, r.seq, r.ts_ms, &r.kind, &r.subject, &r.origin, &r.body);
        if r.hash != want {
            return bad(i, sigs, format!("record {i} was changed after it was signed (hash mismatch)"));
        }
        let Some((_, key)) = keys.iter().rev().find(|(from, _)| *from <= r.seq) else {
            return bad(i, sigs, format!("record {i}: no public key to check its signature"));
        };
        if r.sig.is_empty() {
            return bad(i, sigs, format!("record {i} is not signed"));
        }
        if !verify_sig_hex(key, &r.hash, &r.sig) {
            return bad(i, sigs, format!("record {i}: bad Ed25519 signature"));
        }
        sigs += 1;
        prev = r.hash.clone();
    }
    VerifyReport { ok: true, records: records.len(), first_bad: None, head, signatures: sigs, reason: None }
}

/// `eva verify`: check a chain file on disk. If `pinned` is given (the node's
/// `<node>.pub`), the file's signing key must match it.
pub fn verify_file(path: &Path, pinned: Option<[u8; 32]>) -> std::io::Result<VerifyReport> {
    let c = read_file(path)?;
    let mut rep = verify_records(&c.records, &c.keys);
    if rep.ok && c.unreadable > 0 {
        rep.ok = false;
        rep.first_bad = Some(c.records.len() as u64);
        rep.reason = Some(format!("{} damaged line(s) in the file", c.unreadable));
    }
    if let (true, Some(p), Some((_, k))) = (rep.ok, pinned, c.keys.last()) {
        if *k != p {
            rep.ok = false;
            rep.reason = Some("signed with a key other than the node's pinned public key".into());
        }
    }
    Ok(rep)
}

impl Chain {
    /// A chain in memory with a fresh key (tests).
    pub fn in_memory(origin: &str) -> Self {
        let key = NodeKey::generate();
        let keys = vec![(0, key.public())];
        Self { origin: origin.into(), records: Vec::new(), file: None, path: None, key, keys }
    }

    /// Open (or create) the JSONL file and replay it. The node key lives in
    /// `<file dir>/keys/` and is created on first run.
    pub fn open(path: &Path, origin: &str) -> std::io::Result<Self> {
        let dir = path.parent().unwrap_or(Path::new("."));
        let key = NodeKey::load_or_create(&dir.join("keys"), origin)?;
        Self::open_with_key(path, origin, key)
    }

    pub fn open_with_key(path: &Path, origin: &str, key: NodeKey) -> std::io::Result<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut contents = read_file(path)?;
        // a file written before signatures existed: keep it aside, start a signed chain
        if !contents.records.is_empty() && contents.keys.is_empty() && contents.records.iter().all(|r| r.sig.is_empty()) {
            let aside = path.with_extension("legacy-unsigned.jsonl");
            tracing::warn!("chain: {} has no signatures (older version); moved to {}", path.display(), aside.display());
            std::fs::rename(path, &aside)?;
            contents = FileContents::default();
        }
        let mut file = OpenOptions::new().create(true).read(true).append(true).open(path)?;
        // a crash in the middle of a line must not glue the next line onto it
        let len = file.metadata()?.len();
        if len > 0 {
            let mut last = [0u8; 1];
            file.seek(SeekFrom::Start(len - 1))?;
            file.read_exact(&mut last)?;
            if last[0] != b'\n' {
                file.write_all(b"\n")?;
            }
        }
        let mut c = Self { origin: origin.into(), records: contents.records, file: Some(file), path: Some(path.to_path_buf()), key, keys: contents.keys };
        if c.keys.last().map(|k| k.1) != Some(c.key.public()) {
            if !c.keys.is_empty() {
                tracing::warn!("chain: {} was signed with another key; new records use this node's key", path.display());
            }
            c.write_key_line();
        }
        Ok(c)
    }

    fn write_key_line(&mut self) {
        let from_seq = self.records.len() as u64;
        self.keys.push((from_seq, self.key.public()));
        self.write(&Line::Key { origin: self.origin.clone(), public: self.key.public_hex(), from_seq });
    }

    pub fn public_key(&self) -> [u8; 32] {
        self.key.public()
    }
    pub fn public_hex(&self) -> String {
        self.key.public_hex()
    }
    pub fn keys(&self) -> &[(u64, [u8; 32])] {
        &self.keys
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
        let sig = self.key.sign_hash(&hash);
        let r = Record { seq, ts_ms, kind: kind.into(), subject: subject.into(), body, prev, hash, sig, state: SyncState::Signed, sent_ms: None, acked_ms: None, origin: self.origin.clone() };
        self.write(&Line::Rec(r.clone()));
        self.records.push(r);
        seq
    }

    /// Insert a record received from elsewhere, keeping its hash and signature.
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
        verify_records(&self.records, &self.keys)
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
                // one write per line: a killed process leaves whole lines behind
                if let Err(e) = f.write_all(s.as_bytes()) {
                    tracing::error!("chain: write failed: {e}");
                }
            }
        }
    }

    /// Wipe the chain (demo reset): truncates the file, keeps the node key.
    pub fn reset(&mut self) {
        self.records.clear();
        self.keys.clear();
        if let Some(p) = &self.path {
            if let Ok(f) = OpenOptions::new().create(true).write(true).truncate(true).open(p) {
                drop(f);
            }
            self.file = OpenOptions::new().append(true).open(p).ok();
        }
        self.write_key_line();
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

    fn tmp(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("eva-chain-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn append_verify_and_tamper() {
        let mut c = Chain::in_memory("ship-1");
        c.append(1, "alarm", "4B", json!({"text": "No pulse"}));
        c.append(2, "guidance", "3B", json!({"text": "Start CPR"}));
        c.append(3, "log", "ship", json!({"n": 3}));
        let rep = c.verify();
        assert!(rep.ok);
        assert_eq!(rep.signatures, 3);
        c.records[1].body = json!({"text": "edited later"});
        let rep = c.verify();
        assert!(!rep.ok);
        assert_eq!(rep.first_bad, Some(1));
    }

    #[test]
    fn rehashing_after_an_edit_is_caught_by_the_signature() {
        let mut c = Chain::in_memory("ship-1");
        for i in 0..4 {
            c.append(i, "log", "ship", json!({"i": i}));
        }
        // an attacker edits record 1 and recomputes every hash after it
        c.records[1].body = json!({"i": 99});
        let mut prev = c.records[0].hash.clone();
        for r in c.records.iter_mut().skip(1) {
            r.prev = prev.clone();
            r.hash = compute_hash(&r.prev, r.seq, r.ts_ms, &r.kind, &r.subject, &r.origin, &r.body);
            prev = r.hash.clone();
        }
        let rep = c.verify();
        assert!(!rep.ok, "hashes line up, signatures do not");
        assert_eq!(rep.first_bad, Some(1));
        assert!(rep.reason.unwrap().contains("signature"));
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
    fn file_replay_rebuilds_chain_states_and_key() {
        let dir = tmp("replay");
        let path = dir.join("chain.jsonl");
        let public;
        {
            let mut c = Chain::open(&path, "ship-1").unwrap();
            public = c.public_key();
            c.append(1, "alarm", "4B", json!({"text": "No pulse"}));
            let s = c.append(2, "log", "ship", json!({}));
            c.set_state(s, SyncState::Sent, 5);
        }
        assert!(NodeKey::key_path(&dir.join("keys"), "ship-1").exists(), "key created on first run");
        let c = Chain::open(&path, "ship-1").unwrap();
        assert_eq!(c.public_key(), public, "same key after a restart");
        assert_eq!(c.len(), 2);
        assert!(c.verify().ok);
        assert_eq!(c.records[1].state, SyncState::Sent);
        assert_eq!(c.keys().len(), 1, "no duplicate key line on reopen");
        let pinned = read_pub_file(&NodeKey::pub_path(&dir.join("keys"), "ship-1"));
        assert!(verify_file(&path, pinned).unwrap().ok);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn flipping_one_byte_in_the_file_is_detected() {
        let dir = tmp("flip");
        let path = dir.join("chain.jsonl");
        {
            let mut c = Chain::open(&path, "ship-1").unwrap();
            c.append(1, "alarm", "4B", json!({"text": "No pulse at the wrist"}));
            c.append(2, "case", "4B", json!({"text": "Leo started CPR"}));
            c.append(3, "case", "4B", json!({"text": "AED attached"}));
        }
        let clean = std::fs::read(&path).unwrap();
        assert!(verify_file(&path, None).unwrap().ok);
        // flip one bit in every byte position of the second record's line, one at a time
        let start = clean.windows(5).position(|w| w == b"Leo s").unwrap();
        let mut caught = 0;
        let mut tried = 0;
        for pos in start..start + 15 {
            let mut bad = clean.clone();
            bad[pos] ^= 0x01;
            std::fs::write(&path, &bad).unwrap();
            tried += 1;
            if !verify_file(&path, None).unwrap().ok {
                caught += 1;
            }
        }
        assert_eq!(caught, tried, "every flipped byte is reported");
        // a flipped signature byte
        let s = String::from_utf8(clean.clone()).unwrap();
        let sig_at = s.find("\"sig\":\"").unwrap() + 7;
        let mut bad = clean.clone();
        bad[sig_at] = if bad[sig_at] == b'a' { b'b' } else { b'a' };
        std::fs::write(&path, &bad).unwrap();
        let rep = verify_file(&path, None).unwrap();
        assert!(!rep.ok);
        assert!(rep.reason.unwrap().contains("signature"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reset_keeps_the_key_and_starts_a_clean_signed_chain() {
        let dir = tmp("reset");
        let path = dir.join("chain.jsonl");
        let mut c = Chain::open(&path, "ship-1").unwrap();
        c.append(1, "log", "ship", json!({}));
        let k = c.public_key();
        c.reset();
        c.append(2, "log", "ship", json!({"after": true}));
        assert_eq!(c.public_key(), k);
        assert!(verify_file(&path, None).unwrap().ok);
        assert_eq!(read_file(&path).unwrap().records.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn signature_of_another_key_fails() {
        let a = NodeKey::generate();
        let b = NodeKey::generate();
        let h = compute_hash(GENESIS, 0, 0, "log", "ship", "ship", &json!({}));
        let sig = a.sign_hash(&h);
        assert!(verify_sig_hex(&a.public(), &h, &sig));
        assert!(!verify_sig_hex(&b.public(), &h, &sig));
        assert!(!format!("{a:?}").contains(&hex::encode([0u8; 4])) || format!("{a:?}").starts_with("NodeKey(pub"));
    }
}
