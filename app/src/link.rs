//! `/link`: the Eva Link WebSocket endpoint on every node.
//!
//! One binary WebSocket message = one datagram = one or more frames. The first
//! frame from a client must be `Hello`; the hub answers `Welcome` with the
//! node id it assigned. After that, behaviour depends on the client's role:
//!
//! * `App` / `Dash` (PWA instances): mesh. Frames with a `dst` are relayed to
//!   that node; pings to the hub are answered; text is broadcast to the mesh.
//! * `Band` (a simulated band gateway): delta-encoded vitals are decoded and
//!   applied to the passengers.
//! * `Ship` (on a ground node): records of the ship's chain arrive, are
//!   verified against the hash chain **and the ship's Ed25519 signature**,
//!   mirrored to disk, and acknowledged.

use crate::chain::{compute_hash, verify_sig, Record, SyncState};
use crate::mesh::{HUB_ID, OUTBOX};
use crate::model::*;
use crate::protocol::frame::{decode_batch, Frame, MsgType};
use crate::protocol::payload::{self, Hello, RecordMsg, Role, Text, Welcome};
use crate::protocol::{Delivered, Endpoint};
use crate::state::{Shared, UiEvent};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

/// Largest WebSocket message accepted on `/link` (one datagram).
pub const MAX_DATAGRAM: usize = 256 * 1024;

#[derive(Deserialize)]
pub struct LinkQuery {
    pub seat: Option<String>,
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Shared>, Query(q): Query<LinkQuery>) -> Response {
    ws.max_message_size(MAX_DATAGRAM).max_frame_size(MAX_DATAGRAM).on_upgrade(move |socket| handle(socket, state, q.seat))
}

/// Ship name as stamped on its chain records ("ship · ship" → "ship").
pub fn origin_of(name: &str) -> String {
    name.split(" · ").next().unwrap_or("ship").to_string()
}

async fn handle(socket: WebSocket, state: Shared, seat: Option<String>) {
    let (mut sink, mut stream) = socket.split();
    // 1. Hello within 10 s
    let hello = match tokio::time::timeout(std::time::Duration::from_secs(10), stream.next()).await {
        Ok(Some(Ok(Message::Binary(b)))) => decode_batch(&b).ok().and_then(|fs| fs.into_iter().find(|f| f.kind == MsgType::Hello)).and_then(|f| Hello::decode(&f.payload).ok()),
        _ => None,
    };
    let Some(hello) = hello else {
        let _ = sink.close().await;
        return;
    };
    // bounded: a slow client loses frames instead of growing the hub's memory
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(OUTBOX);
    let name: String = hello.name.chars().take(60).collect();
    let seat: Option<String> = seat.map(|s| s.chars().take(8).collect());
    let id = state.mesh.register(hello.role, name.clone(), seat.clone(), tx, state.clock.real_ms());
    tracing::info!("link: node {id} joined as {} ({name})", hello.role.name());
    let welcome = Frame::new(MsgType::Welcome, state.mesh.hub_seq(), HUB_ID, Welcome { node_id: id, server_ms: state.clock.real_ms() }.encode()).to(id);
    let _ = sink.send(Message::Binary(welcome.encode().into())).await;
    let writer = tokio::spawn(async move {
        while let Some(b) = rx.recv().await {
            // a client that cannot take a frame within 5 s is dropped
            match tokio::time::timeout(std::time::Duration::from_secs(5), sink.send(Message::Binary(b.into()))).await {
                Ok(Ok(())) => {}
                _ => break,
            }
        }
    });

    // a ship announces its Ed25519 public key; the ground pins it on first use
    if hello.role == Role::Ship && hello.pubkey.len() == 32 {
        let mut k = [0u8; 32];
        k.copy_from_slice(&hello.pubkey);
        if let Err(e) = state.mirror.lock().expect("mirror").pin_key(&origin_of(&name), k) {
            tracing::warn!("link: {e}");
        }
    }

    // per-session reliable endpoint for ships and band gateways
    let peer_key = format!("{}#{}", name, hello.client_ms);
    let mut ep: Option<Endpoint> = match hello.role {
        Role::Ship => Some(state.peers.lock().expect("peers").remove(&peer_key).unwrap_or_else(|| Endpoint::new(HUB_ID).with_crc(true))),
        Role::Band => Some(Endpoint::new(HUB_ID)),
        _ => None,
    };
    if hello.role == Role::Ship {
        state.world.write().expect("world").link = crate::state::LinkStatus { connected: true, peer: Some(name.clone()), profile: "near-space".into(), profile_label: "Ship uplink".into(), up: true, ..Default::default() };
    }
    let mut flush = tokio::time::interval(std::time::Duration::from_millis(100));
    loop {
        tokio::select! {
            msg = stream.next() => {
                let b = match msg {
                    Some(Ok(Message::Binary(b))) => b,
                    Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) | Some(Ok(Message::Text(_))) => continue,
                    _ => break,
                };
                state.mesh.seen(id, b.len(), state.clock.real_ms());
                match ep.as_mut() {
                    Some(e) => on_endpoint_datagram(&state, id, hello.role, e, &b),
                    None => on_mesh_datagram(&state, id, seat.as_deref(), &b),
                }
            }
            _ = flush.tick() => {
                if writer.is_finished() {
                    break;
                }
                if let Some(e) = ep.as_mut() {
                    if hello.role == Role::Ship {
                        for text in state.to_ship.lock().expect("to_ship").drain(..) {
                            e.send(MsgType::Text, Text { from: "Ground".into(), to: "ship".into(), text }.encode(), None);
                        }
                    }
                    for d in e.outgoing(state.clock.real_ms(), usize::MAX, 16_000) {
                        state.mesh.send_to(id, d);
                    }
                }
            }
        }
    }
    if let (Role::Ship, Some(e)) = (hello.role, ep.take()) {
        let mut peers = state.peers.lock().expect("peers");
        // keep the endpoints of the last few sessions only (reconnects resume them)
        if peers.len() >= 16 {
            if let Some(k) = peers.keys().next().cloned() {
                peers.remove(&k);
            }
        }
        peers.insert(peer_key, e);
        drop(peers);
        state.world.write().expect("world").link.connected = false;
    }
    state.mesh.unregister(id);
    writer.abort();
    tracing::info!("link: node {id} left");
}

/// App / dashboard clients: hub pings, relay, broadcast text.
fn on_mesh_datagram(state: &Shared, id: u32, seat: Option<&str>, b: &[u8]) {
    let Ok(frames) = decode_batch(b) else { return };
    for f in frames {
        if let Some(dst) = f.dst.filter(|d| *d != HUB_ID) {
            // relay verbatim to the addressed node (end-to-end ping, direct text, acks)
            state.mesh.send_to(dst, f.encode());
            state.mesh.relayed_frames.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            continue;
        }
        match f.kind {
            MsgType::Ping => {
                state.mesh.send_frame(id, MsgType::Pong, f.payload.clone());
            }
            MsgType::Pong => {
                if let Ok(t) = payload::decode_time(&f.payload) {
                    state.mesh.rtt_sample(id, state.clock.real_ms().saturating_sub(t) as f64);
                }
            }
            MsgType::Text => {
                if let Ok(t) = Text::decode(&f.payload) {
                    state.mesh.broadcast(&[Role::App, Role::Dash], id, &f.encode());
                    let now = state.now();
                    let mut w = state.world.write().expect("world");
                    w.messages.push(LinkMessage { ts_ms: now, from: t.from.chars().take(60).collect(), to: t.to.chars().take(60).collect(), text: t.text.chars().take(500).collect(), channel: "mesh".into() });
                    let n = w.messages.len();
                    if n > 500 {
                        w.messages.drain(..n - 500);
                    }
                    drop(w);
                    state.emit(UiEvent::Tick);
                }
                ack(state, id, &f);
            }
            MsgType::Alarm => {
                if let Ok(a) = payload::Alarm::decode(&f.payload) {
                    let who = if a.subject.is_empty() { seat.unwrap_or("").to_string() } else { a.subject.chars().take(8).collect() };
                    if !who.is_empty() {
                        crate::eva_agent::tools::execute(state, &who, "request_help", &serde_json::json!({"seat": who, "reason": if a.text.is_empty() { "Help button on the app".to_string() } else { a.text.chars().take(200).collect() }}), crate::eva_agent::tools::Origin::User);
                    }
                }
                ack(state, id, &f);
            }
            _ => {}
        }
    }
}

fn ack(state: &Shared, id: u32, f: &Frame) {
    if f.is_reliable() {
        state.mesh.send_frame(id, MsgType::Ack, payload::encode_ack(&[f.seq]));
    }
}

/// Ship sessions (on the ground) and band gateways: reliable endpoint.
fn on_endpoint_datagram(state: &Shared, id: u32, role: Role, e: &mut Endpoint, b: &[u8]) {
    let now_real = state.clock.real_ms();
    let mut vitals_in = false;
    for ev in e.incoming(now_real, b) {
        match ev {
            Delivered::Vitals { sample, .. } => {
                let now = state.now();
                let mut w = state.world.write().expect("world");
                if let Some(p) = w.by_stream_mut(sample.stream) {
                    crate::sim::ingest(p, sample.sample, now);
                    vitals_in = true;
                }
            }
            Delivered::Frame(f) if role == Role::Ship => match f.kind {
                MsgType::Record | MsgType::Alarm | MsgType::Guidance => match RecordMsg::decode(&f.payload) {
                    Ok(r) => on_ship_record(state, r),
                    Err(err) => {
                        // never ack what did not reach the mirror
                        tracing::warn!("link: undecodable record frame {} ({err}); not acknowledged", f.seq);
                        e.rx.retract(f.seq);
                    }
                },
                MsgType::Text => {
                    if let Ok(t) = Text::decode(&f.payload) {
                        let now = state.now();
                        state.world.write().expect("world").messages.push(LinkMessage { ts_ms: now, from: t.from, to: t.to, text: t.text.chars().take(1000).collect(), channel: "ground".into() });
                    }
                }
                _ => {}
            },
            Delivered::Rtt(r) => state.mesh.rtt_sample(id, r),
            _ => {}
        }
    }
    // fresh band vitals: refresh open pages now (rate-limited) instead of at the next tick
    if vitals_in && role == Role::Band {
        use std::sync::atomic::Ordering;
        let last = state.mesh.last_vitals_refresh_ms.load(Ordering::Relaxed);
        if now_real >= last + crate::mesh::VITALS_REFRESH_MS && state.mesh.last_vitals_refresh_ms.compare_exchange(last, now_real, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
            state.emit(UiEvent::Tick);
        }
    }
    if role == Role::Ship {
        let mut w = state.world.write().expect("world");
        w.link.rtt_ms = e.queue.rtt.srtt_ms;
        w.link.bytes_received = e.received.total_bytes();
        w.link.stats = e.received.rows();
        w.link.last_contact_ms = Some(state.now());
    }
}

// ------------------------------------------------------------------ mirror

/// What [`Mirror::insert`] did with a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inserted {
    /// first time we see it
    New,
    /// already mirrored (retransmit, replay, reconnect): nothing changed
    Duplicate,
    /// the ship's chain was reset (same seq, different record): old mirror archived, this one kept
    NewEpoch,
}

/// One line of the ground's mirror file (`mirror-<node>.jsonl`).
#[derive(Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "lowercase")]
enum MirrorLine {
    /// a ship's pinned public key (hex)
    Key { origin: String, public: String },
    /// a record exactly as received
    Rec { origin: String, received_ms: u64, seq: u64, ts_ms: u64, kind: String, subject: String, hash8: String, wrapper: serde_json::Value, sig: String },
}

/// Ground copy of the ship's chain. Records can arrive out of order (loss,
/// retransmit, outages); each is verified as soon as its predecessor's full
/// hash is known: recompute SHA-256, compare the 8-byte prefix sent, then
/// check the ship's **Ed25519 signature** over the full hash with the key
/// pinned for that ship. Everything received is appended to a JSONL file so a
/// ground restart loses nothing.
#[derive(Debug, Default)]
pub struct Mirror {
    pub records: BTreeMap<u64, Record>,
    pub verified: BTreeMap<u64, String>,
    pub failed: Vec<u64>,
    /// why each failed record failed
    pub reasons: BTreeMap<u64, String>,
    /// full hash of the record before a session's first record (sent once per session)
    pub anchors: BTreeMap<u64, String>,
    /// pinned Ed25519 key per ship (trust on first use; `peer-<ship>.pub`)
    pub keys: BTreeMap<String, [u8; 32]>,
    /// how many times the ship's chain was reset while we mirrored it
    pub epochs: u32,
    pub duplicates: u64,
    file: Option<File>,
    path: Option<PathBuf>,
    keys_dir: Option<PathBuf>,
}

impl Mirror {
    /// Open (or create) `path` and replay it. Pinned ship keys live in `keys_dir`.
    pub fn open(path: &Path, keys_dir: &Path) -> std::io::Result<Self> {
        if let Some(d) = path.parent() {
            std::fs::create_dir_all(d)?;
        }
        let mut m = Mirror { keys_dir: Some(keys_dir.to_path_buf()), ..Default::default() };
        if let Ok(rd) = std::fs::read_dir(keys_dir) {
            for e in rd.flatten() {
                let n = e.file_name().to_string_lossy().to_string();
                if let Some(origin) = n.strip_prefix("peer-").and_then(|x| x.strip_suffix(".pub")) {
                    if let Some(k) = crate::chain::read_pub_file(&e.path()) {
                        m.keys.insert(origin.to_string(), k);
                    }
                }
            }
        }
        let lines = read_mirror_lines(path)?;
        for l in lines {
            match l {
                MirrorLine::Key { origin, public } => {
                    if let Some(k) = crate::chain::parse_public(&public) {
                        m.keys.entry(origin).or_insert(k);
                    }
                }
                MirrorLine::Rec { origin, received_ms, seq, ts_ms, kind, subject, hash8, wrapper, sig } => {
                    let mut h = [0u8; 8];
                    if let Ok(v) = hex::decode(&hash8) {
                        if v.len() == 8 {
                            h.copy_from_slice(&v);
                        }
                    }
                    let msg = RecordMsg { chain_seq: seq, ts_ms, kind, subject, hash8: h, body: serde_json::to_vec(&wrapper).unwrap_or_default(), sig: hex::decode(sig).unwrap_or_default() };
                    m.insert_inner(msg, &origin, received_ms, false);
                }
            }
        }
        m.file = Some(OpenOptions::new().create(true).append(true).open(path)?);
        m.path = Some(path.to_path_buf());
        Ok(m)
    }

    fn persist(&mut self, line: &MirrorLine) {
        if let Some(f) = self.file.as_mut() {
            if let Ok(mut s) = serde_json::to_string(line) {
                s.push('\n');
                if let Err(e) = f.write_all(s.as_bytes()) {
                    tracing::error!("mirror: write failed: {e}");
                }
            }
        }
    }

    /// Pin a ship's public key on first use. A different key for a ship we
    /// already know is refused (its records will fail verification).
    pub fn pin_key(&mut self, origin: &str, key: [u8; 32]) -> Result<(), String> {
        match self.keys.get(origin) {
            Some(k) if *k == key => Ok(()),
            Some(_) => Err(format!("ship '{origin}' presented a different public key than the one pinned on this ground; delete peer-{origin}.pub to accept it")),
            None => {
                self.keys.insert(origin.to_string(), key);
                if let Some(d) = &self.keys_dir {
                    let _ = std::fs::create_dir_all(d);
                    let _ = std::fs::write(d.join(format!("peer-{}.pub", origin.replace(['/', '\\', '.'], "_"))), format!("{}\n", hex::encode(key)));
                }
                self.persist(&MirrorLine::Key { origin: origin.to_string(), public: hex::encode(key) });
                // records that arrived before the key can be checked now
                self.failed.retain(|s| self.reasons.get(s).is_none_or(|r| !r.contains("no public key")));
                self.verify_pending();
                Ok(())
            }
        }
    }

    pub fn insert(&mut self, r: RecordMsg, origin: &str, received_ms: u64) -> Inserted {
        self.insert_inner(r, origin, received_ms, true)
    }

    fn insert_inner(&mut self, r: RecordMsg, origin: &str, received_ms: u64, persist: bool) -> Inserted {
        // body is {"body": …, "prev": hex?}
        let wrapper: serde_json::Value = serde_json::from_slice(&r.body).unwrap_or_default();
        let body = wrapper.get("body").cloned().unwrap_or(serde_json::Value::Null);
        let prefix = hex::encode(r.hash8);
        let mut outcome = Inserted::New;
        if let Some(have) = self.records.get(&r.chain_seq) {
            if have.hash.starts_with(&prefix) {
                self.duplicates += 1;
                return Inserted::Duplicate;
            }
            outcome = Inserted::NewEpoch;
        }
        let anchor = wrapper.get("prev").and_then(|p| p.as_str()).map(String::from);
        if let (Some(a), Some(s)) = (&anchor, r.chain_seq.checked_sub(1)) {
            if self.verified.get(&s).is_some_and(|v| v != a) {
                outcome = Inserted::NewEpoch;
            }
        }
        if outcome == Inserted::NewEpoch {
            self.new_epoch();
        }
        if let Some(prev) = anchor {
            self.anchors.insert(r.chain_seq.checked_sub(1).unwrap_or(u64::MAX), prev);
        }
        if persist {
            self.persist(&MirrorLine::Rec {
                origin: origin.to_string(),
                received_ms,
                seq: r.chain_seq,
                ts_ms: r.ts_ms,
                kind: r.kind.clone(),
                subject: r.subject.clone(),
                hash8: prefix.clone(),
                wrapper: wrapper.clone(),
                sig: hex::encode(&r.sig),
            });
        }
        let rec = Record {
            seq: r.chain_seq,
            ts_ms: r.ts_ms,
            kind: r.kind,
            subject: r.subject,
            body,
            prev: String::new(),
            hash: prefix,
            sig: hex::encode(&r.sig),
            state: SyncState::Acked,
            sent_ms: None,
            acked_ms: Some(received_ms),
            origin: origin.to_string(),
        };
        self.records.insert(rec.seq, rec);
        self.verify_pending();
        outcome
    }

    /// The ship reset its chain: archive what we have and start over.
    fn new_epoch(&mut self) {
        self.epochs += 1;
        tracing::warn!("mirror: the ship's chain was reset; starting mirror epoch {}", self.epochs);
        self.records.clear();
        self.verified.clear();
        self.failed.clear();
        self.reasons.clear();
        self.anchors.clear();
        if let Some(p) = self.path.clone() {
            let aside = p.with_extension(format!("epoch{}.jsonl", self.epochs));
            self.file = None;
            let _ = std::fs::rename(&p, &aside);
            self.file = OpenOptions::new().create(true).append(true).open(&p).ok();
            let keys: Vec<(String, [u8; 32])> = self.keys.iter().map(|(o, k)| (o.clone(), *k)).collect();
            for (origin, k) in keys {
                self.persist(&MirrorLine::Key { origin, public: hex::encode(k) });
            }
        }
    }

    /// Demo reset on the ground: forget the mirrored records (pinned keys stay).
    pub fn reset(&mut self) {
        self.records.clear();
        self.verified.clear();
        self.failed.clear();
        self.reasons.clear();
        self.anchors.clear();
        self.duplicates = 0;
        if let Some(p) = self.path.clone() {
            self.file = OpenOptions::new().create(true).write(true).truncate(true).open(&p).ok();
            let keys: Vec<(String, [u8; 32])> = self.keys.iter().map(|(o, k)| (o.clone(), *k)).collect();
            for (origin, k) in keys {
                self.persist(&MirrorLine::Key { origin, public: hex::encode(k) });
            }
        }
    }

    fn fail(&mut self, seq: u64, why: &str) {
        if !self.failed.contains(&seq) {
            tracing::warn!("mirror: record {seq} failed verification: {why}");
            self.failed.push(seq);
            self.reasons.insert(seq, why.to_string());
        }
    }

    fn verify_pending(&mut self) {
        loop {
            let mut progressed = false;
            let seqs: Vec<u64> = self.records.keys().copied().filter(|s| !self.verified.contains_key(s) && !self.failed.contains(s)).collect();
            for s in seqs {
                let prev = if s == 0 { Some(crate::chain::GENESIS.to_string()) } else { self.verified.get(&(s - 1)).cloned().or_else(|| self.anchors.get(&(s - 1)).cloned()) };
                let Some(prev) = prev else { continue };
                let r = self.records.get(&s).expect("record");
                let full = compute_hash(&prev, r.seq, r.ts_ms, &r.kind, &r.subject, &r.origin, &r.body);
                if !full.starts_with(&r.hash) {
                    self.fail(s, "hash does not match the chain");
                    continue;
                }
                let Some(key) = self.keys.get(&r.origin).copied() else {
                    self.fail(s, "no public key pinned for this ship");
                    continue;
                };
                let sig = hex::decode(&r.sig).unwrap_or_default();
                if sig.is_empty() {
                    self.fail(s, "record is not signed");
                    continue;
                }
                if !verify_sig(&key, &full, &sig) {
                    self.fail(s, "bad Ed25519 signature");
                    continue;
                }
                let r = self.records.get_mut(&s).expect("record");
                r.prev = prev;
                r.hash = full.clone();
                self.verified.insert(s, full);
                progressed = true;
            }
            if !progressed {
                break;
            }
        }
    }
    pub fn is_verified(&self, seq: u64) -> bool {
        self.verified.contains_key(&seq)
    }
}

fn read_mirror_lines(path: &Path) -> std::io::Result<Vec<MirrorLine>> {
    let mut out = Vec::new();
    if !path.exists() {
        return Ok(out);
    }
    for line in BufReader::new(File::open(path)?).split(b'\n') {
        let line = line?;
        if let Ok(l) = serde_json::from_slice::<MirrorLine>(&line) {
            out.push(l);
        }
    }
    Ok(out)
}

/// `eva verify` on a ground mirror file: replay it and report.
pub fn verify_mirror_file(path: &Path, keys_dir: &Path) -> std::io::Result<(usize, usize, Vec<(u64, String)>)> {
    let tmp = std::env::temp_dir().join(format!("eva-verify-{}-{}.jsonl", std::process::id(), path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()));
    std::fs::copy(path, &tmp)?;
    let m = Mirror::open(&tmp, keys_dir);
    let _ = std::fs::remove_file(&tmp);
    let m = m?;
    let failed = m.failed.iter().map(|s| (*s, m.reasons.get(s).cloned().unwrap_or_default())).collect();
    Ok((m.records.len(), m.verified.len(), failed))
}

fn on_ship_record(state: &Shared, r: RecordMsg) {
    let now = state.now();
    let origin = {
        let w = state.world.read().expect("world");
        origin_of(&w.link.peer.clone().unwrap_or_else(|| "ship".into()))
    };
    let seq = r.chain_seq;
    let outcome = state.mirror.lock().expect("mirror").insert(r, &origin, now);
    if outcome == Inserted::Duplicate {
        // a retransmit or a replayed frame: already reflected, nothing to do
        return;
    }
    let rec = state.mirror.lock().expect("mirror").records.get(&seq).cloned();
    if let Some(rec) = rec {
        reflect(state, &rec, now);
    }
    state.emit(UiEvent::Tick);
}

/// After a ground restart: rebuild the ground's view (cases, alerts, messages)
/// from the mirrored records on disk.
pub fn restore_ground(state: &Shared) {
    let recs: Vec<Record> = state.mirror.lock().expect("mirror").records.values().cloned().collect();
    for r in &recs {
        reflect(state, r, r.acked_ms.unwrap_or(r.ts_ms));
    }
    if !recs.is_empty() {
        tracing::info!("ground: restored {} mirrored records from disk", recs.len());
    }
}

/// Reflect what matters of one ship record in the ground's world view.
fn reflect(state: &Shared, r: &Record, received: u64) {
    let (kind, subject, ts) = (r.kind.as_str(), r.subject.clone(), r.ts_ms);
    let text = r.body.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string();
    let mut w = state.world.write().expect("world");
    match kind {
        "alarm" => {
            let name = w.passenger(&subject).map(|p| p.name.clone()).unwrap_or_else(|| subject.clone());
            if w.open_case_for(&subject).is_none() {
                let id = w.next_id();
                w.cases.push(Case {
                    id,
                    seat: subject.clone(),
                    name: name.clone(),
                    opened_ms: ts,
                    open: true,
                    kind: "No pulse".into(),
                    helpers: Vec::new(),
                    steps: Vec::new(),
                    now: "Ship crew is running the emergency flow.".into(),
                    why: "Alarm received from the ship's wristband rules.".into(),
                    how: "Signed record from the ship, hash and signature verified on the ground.".into(),
                    next: "Flight surgeon reviews; reply goes up as a message.".into(),
                });
                let n = w.alerts.len() as u64 + 1;
                w.alerts.push(Alert { id: n, ts_ms: ts, seat: subject.clone(), who: name, what: "No pulse at the wrist".into(), tone: "critical".into(), state: "Open".into() });
            }
            if let Some(c) = w.cases.iter_mut().find(|c| c.open && c.seat == subject) {
                c.steps.push(CaseStep { ts_ms: ts, title: text.clone(), detail: "Alarm".into(), tone: "critical".into(), earth_ms: Some(received) });
            }
        }
        "case" => {
            let closing = text.starts_with("Case closed");
            if let Some(c) = w.cases.iter_mut().find(|c| c.open && c.seat == subject) {
                c.steps.push(CaseStep { ts_ms: ts, title: text, detail: String::new(), tone: "info".into(), earth_ms: Some(received) });
                if closing {
                    c.open = false;
                }
            }
            if closing {
                for a in w.alerts.iter_mut().filter(|a| a.seat == subject && a.state == "Open") {
                    a.state = "Solved".into();
                }
            }
        }
        "message" => w.messages.push(LinkMessage { ts_ms: ts, from: r.body.get("from").and_then(|f| f.as_str()).unwrap_or("Ship").into(), to: "ground".into(), text, channel: "ground".into() }),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::Chain;
    use serde_json::json;

    fn to_msg(r: &Record, anchor: bool) -> RecordMsg {
        let mut wrapper = json!({"body": r.body});
        if anchor {
            wrapper["prev"] = json!(r.prev);
        }
        let mut hash8 = [0u8; 8];
        hash8.copy_from_slice(&hex::decode(&r.hash).unwrap()[..8]);
        RecordMsg { chain_seq: r.seq, ts_ms: r.ts_ms, kind: r.kind.clone(), subject: r.subject.clone(), hash8, body: serde_json::to_vec(&wrapper).unwrap(), sig: hex::decode(&r.sig).unwrap() }
    }

    fn ship_chain(n: u64) -> Chain {
        let mut c = Chain::in_memory("ship");
        for i in 0..n {
            c.append(i * 1000, "case", "4B", json!({"text": format!("step {i}")}));
        }
        c
    }

    #[test]
    fn verifies_hash_and_signature_out_of_order() {
        let c = ship_chain(5);
        let mut m = Mirror::default();
        m.pin_key("ship", c.public_key()).unwrap();
        for i in [3usize, 1, 4, 0, 2] {
            m.insert(to_msg(&c.records[i], false), "ship", 1);
        }
        assert_eq!(m.verified.len(), 5);
        assert!(m.failed.is_empty());
        assert_eq!(m.records[&4].hash, c.records[4].hash, "full hash restored");
    }

    #[test]
    fn forged_signature_or_other_key_fails() {
        let c = ship_chain(2);
        let mut m = Mirror::default();
        m.pin_key("ship", c.public_key()).unwrap();
        let mut bad = to_msg(&c.records[0], false);
        bad.sig[10] ^= 0xff;
        m.insert(bad, "ship", 1);
        assert_eq!(m.failed, vec![0]);
        assert!(m.reasons[&0].contains("signature"));
        // another ship key can not replace the pinned one
        let other = Chain::in_memory("ship");
        assert!(m.pin_key("ship", other.public_key()).is_err());
    }

    #[test]
    fn duplicates_and_replays_are_idempotent() {
        let c = ship_chain(3);
        let mut m = Mirror::default();
        m.pin_key("ship", c.public_key()).unwrap();
        for _ in 0..3 {
            for r in &c.records {
                m.insert(to_msg(r, false), "ship", 1);
            }
        }
        assert_eq!(m.records.len(), 3);
        assert_eq!(m.verified.len(), 3);
        assert_eq!(m.duplicates, 6);
        assert_eq!(m.records[&2].hash, c.records[2].hash, "a duplicate never clobbers the verified full hash");
    }

    #[test]
    fn persisted_mirror_survives_a_restart() {
        let dir = std::env::temp_dir().join(format!("eva-mirror-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("mirror-ground.jsonl");
        let c = ship_chain(4);
        {
            let mut m = Mirror::open(&path, &dir.join("keys")).unwrap();
            m.pin_key("ship", c.public_key()).unwrap();
            for r in &c.records {
                assert_eq!(m.insert(to_msg(r, false), "ship", 7), Inserted::New);
            }
        }
        let m = Mirror::open(&path, &dir.join("keys")).unwrap();
        assert_eq!(m.verified.len(), 4, "everything verified again after the restart");
        assert_eq!(m.keys.get("ship"), Some(&c.public_key()), "pinned key kept");
        let (n, ok, failed) = verify_mirror_file(&path, &dir.join("keys")).unwrap();
        assert_eq!((n, ok, failed.len()), (4, 4, 0));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ship_reset_starts_a_new_epoch() {
        // same signer, chain wiped and rebuilt with other content (demo reset)
        let k = NodeKey::generate();
        let old = signed_chain(&k, &["a", "b", "c"]);
        let new = signed_chain(&k, &["x"]);
        let mut m = Mirror::default();
        m.pin_key("ship", k.public()).unwrap();
        for r in &old {
            m.insert(to_msg(r, false), "ship", 1);
        }
        assert_eq!(m.verified.len(), 3);
        assert_eq!(m.insert(to_msg(&new[0], false), "ship", 2), Inserted::NewEpoch);
        assert_eq!(m.epochs, 1);
        assert_eq!(m.records.len(), 1);
        assert_eq!(m.verified.len(), 1);
    }

    use crate::chain::NodeKey;
    fn signed_chain(k: &NodeKey, texts: &[&str]) -> Vec<Record> {
        let mut prev = crate::chain::GENESIS.to_string();
        let mut out = Vec::new();
        for (i, t) in texts.iter().enumerate() {
            let body = json!({"text": t});
            let hash = compute_hash(&prev, i as u64, i as u64, "log", "ship", "ship", &body);
            let sig = k.sign_hash(&hash);
            out.push(Record { seq: i as u64, ts_ms: i as u64, kind: "log".into(), subject: "ship".into(), body, prev: prev.clone(), hash: hash.clone(), sig, state: SyncState::Signed, sent_ms: None, acked_ms: None, origin: "ship".into() });
            prev = hash;
        }
        out
    }
}
