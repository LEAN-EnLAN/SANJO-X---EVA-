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
//!   verified against the hash chain, mirrored, and acknowledged.

use crate::chain::{compute_hash, Record, SyncState};
use crate::mesh::HUB_ID;
use crate::model::*;
use crate::protocol::frame::{decode_batch, Frame, MsgType};
use crate::protocol::payload::{self, Hello, RecordMsg, Role, Text, Welcome};
use crate::protocol::{Delivered, Endpoint};
use crate::state::{Shared, UiEvent};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::collections::BTreeMap;
use tokio::sync::mpsc;

#[derive(Deserialize)]
pub struct LinkQuery {
    pub seat: Option<String>,
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Shared>, Query(q): Query<LinkQuery>) -> Response {
    ws.max_message_size(256 * 1024).on_upgrade(move |socket| handle(socket, state, q.seat))
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
    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let name: String = hello.name.chars().take(60).collect();
    let id = state.mesh.register(hello.role, name.clone(), seat.clone(), tx, state.clock.real_ms());
    tracing::info!("link: node {id} joined as {} ({name})", hello.role.name());
    let welcome = Frame::new(MsgType::Welcome, state.mesh.hub_seq(), HUB_ID, Welcome { node_id: id, server_ms: state.clock.real_ms() }.encode()).to(id);
    let _ = sink.send(Message::Binary(welcome.encode().into())).await;
    let writer = tokio::spawn(async move {
        while let Some(b) = rx.recv().await {
            if sink.send(Message::Binary(b.into())).await.is_err() {
                break;
            }
        }
    });

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
        state.peers.lock().expect("peers").insert(peer_key, e);
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
                    state.world.write().expect("world").messages.push(LinkMessage { ts_ms: now, from: t.from, to: t.to, text: t.text.chars().take(500).collect(), channel: "mesh".into() });
                    state.emit(UiEvent::Tick);
                }
                ack(state, id, &f);
            }
            MsgType::Alarm => {
                if let Ok(a) = payload::Alarm::decode(&f.payload) {
                    let who = if a.subject.is_empty() { seat.unwrap_or("").to_string() } else { a.subject };
                    if !who.is_empty() {
                        crate::eva_agent::tools::execute(state, &who, "request_help", &serde_json::json!({"seat": who, "reason": if a.text.is_empty() { "Help button on the app".to_string() } else { a.text }}), crate::eva_agent::tools::Origin::User);
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
    for ev in e.incoming(now_real, b) {
        match ev {
            Delivered::Vitals { sample, .. } => {
                let now = state.now();
                let mut w = state.world.write().expect("world");
                if let Some(p) = w.by_stream_mut(sample.stream) {
                    crate::sim::ingest(p, sample.sample, now);
                }
            }
            Delivered::Frame(f) if role == Role::Ship => match f.kind {
                MsgType::Record | MsgType::Alarm | MsgType::Guidance => {
                    if let Ok(r) = RecordMsg::decode(&f.payload) {
                        on_ship_record(state, r);
                    }
                }
                MsgType::Text => {
                    if let Ok(t) = Text::decode(&f.payload) {
                        let now = state.now();
                        state.world.write().expect("world").messages.push(LinkMessage { ts_ms: now, from: t.from, to: t.to, text: t.text, channel: "ground".into() });
                    }
                }
                _ => {}
            },
            Delivered::Rtt(r) => state.mesh.rtt_sample(id, r),
            _ => {}
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

/// Ground copy of the ship's chain. Records can arrive out of order (loss,
/// retransmit, outages); each is verified as soon as its predecessor's full
/// hash is known: recompute SHA-256 and compare the 8-byte prefix sent.
#[derive(Debug, Default)]
pub struct Mirror {
    pub records: BTreeMap<u64, Record>,
    pub verified: BTreeMap<u64, String>,
    pub failed: Vec<u64>,
    /// full hash of the record before a session's first record (sent once per session)
    pub anchors: BTreeMap<u64, String>,
}

impl Mirror {
    pub fn insert(&mut self, r: RecordMsg, origin: &str, received_ms: u64) {
        // body is {"body": …, "prev": hex?}
        let wrapper: serde_json::Value = serde_json::from_slice(&r.body).unwrap_or_default();
        let body = wrapper.get("body").cloned().unwrap_or(serde_json::Value::Null);
        if let Some(prev) = wrapper.get("prev").and_then(|p| p.as_str()) {
            if r.chain_seq > 0 {
                self.anchors.insert(r.chain_seq - 1, prev.to_string());
            } else {
                self.anchors.insert(u64::MAX, prev.to_string());
            }
        }
        let rec = Record {
            seq: r.chain_seq,
            ts_ms: r.ts_ms,
            kind: r.kind,
            subject: r.subject,
            body,
            prev: String::new(),
            hash: hex::encode(r.hash8),
            state: SyncState::Acked,
            sent_ms: None,
            acked_ms: Some(received_ms),
            origin: origin.to_string(),
        };
        self.records.insert(rec.seq, rec);
        self.verify_pending();
    }

    fn verify_pending(&mut self) {
        loop {
            let mut progressed = false;
            let seqs: Vec<u64> = self.records.keys().copied().filter(|s| !self.verified.contains_key(s) && !self.failed.contains(s)).collect();
            for s in seqs {
                let prev = if s == 0 { Some(crate::chain::GENESIS.to_string()) } else { self.verified.get(&(s - 1)).cloned().or_else(|| self.anchors.get(&(s - 1)).cloned()) };
                let Some(prev) = prev else { continue };
                let r = self.records.get_mut(&s).expect("record");
                let full = compute_hash(&prev, r.seq, r.ts_ms, &r.kind, &r.subject, &r.origin, &r.body);
                if full.starts_with(&r.hash) {
                    r.prev = prev;
                    r.hash = full.clone();
                    self.verified.insert(s, full);
                    progressed = true;
                } else {
                    self.failed.push(s);
                }
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

fn on_ship_record(state: &Shared, r: RecordMsg) {
    let now = state.now();
    let origin = {
        let w = state.world.read().expect("world");
        w.link.peer.clone().unwrap_or_else(|| "ship".into())
    };
    let origin = origin.split(" · ").next().unwrap_or("ship").to_string();
    let (kind, subject, ts) = (r.kind.clone(), r.subject.clone(), r.ts_ms);
    state.mirror.lock().expect("mirror").insert(r, &origin, now);
    let body = state.mirror.lock().expect("mirror").records.values().rev().find(|x| x.ts_ms == ts && x.kind == kind).map(|x| x.body.clone()).unwrap_or_default();
    let text = body.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string();
    // reflect what matters in the ground's world view
    let mut w = state.world.write().expect("world");
    match kind.as_str() {
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
                    how: "Signed record from the ship, hash verified on the ground.".into(),
                    next: "Flight surgeon reviews; reply goes up as a message.".into(),
                });
                let n = w.alerts.len() as u64 + 1;
                w.alerts.push(Alert { id: n, ts_ms: ts, seat: subject.clone(), who: name, what: "No pulse at the wrist".into(), tone: "critical".into(), state: "Open".into() });
            }
            if let Some(c) = w.cases.iter_mut().find(|c| c.open && c.seat == subject) {
                c.steps.push(CaseStep { ts_ms: ts, title: text.clone(), detail: "Alarm".into(), tone: "critical".into(), earth_ms: Some(now) });
            }
        }
        "case" => {
            let closing = text.starts_with("Case closed");
            if let Some(c) = w.cases.iter_mut().find(|c| c.open && c.seat == subject) {
                c.steps.push(CaseStep { ts_ms: ts, title: text, detail: String::new(), tone: "info".into(), earth_ms: Some(now) });
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
        "message" => w.messages.push(LinkMessage { ts_ms: ts, from: body.get("from").and_then(|f| f.as_str()).unwrap_or("Ship").into(), to: "ground".into(), text, channel: "ground".into() }),
        _ => {}
    }
    drop(w);
    state.emit(UiEvent::Tick);
}
