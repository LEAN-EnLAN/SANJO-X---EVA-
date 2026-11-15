//! Ship → ground uplink over Eva Link (`eva serve --role ship --peer <url>`).
//!
//! The ship connects to the ground's `/link` as a `Ship` node and pushes:
//! every new chain record (priority by kind: alarm > guidance > logs) and a
//! vitals keyframe/delta per passenger every 5 s. The ground acknowledges
//! each record; acknowledged records turn "Acknowledged" on the dashboard.
//!
//! Both directions run through a simulated [`LinkSim`] with the profile chosen
//! in the dev panel (near-space by default), so latency, bandwidth, loss and
//! contact gaps are visible in the demo. The send queue lives outside the
//! connection: if the WebSocket or the simulated contact drops, records wait
//! (store-and-forward) and flow when contact returns.

use crate::chain::SyncState;
use crate::protocol::frame::{MsgType, Priority};
use crate::protocol::payload::{Hello, RecordMsg, Role, Text};
use crate::protocol::profile::{LinkProfile, LinkSim};
use crate::protocol::{Delivered, Endpoint};
use crate::state::{Shared, UiEvent};
use futures_util::{SinkExt, StreamExt};
use std::collections::{HashMap, HashSet};
use tokio_tungstenite::tungstenite::Message;

const TICK_MS: u64 = 100;
const VITALS_EVERY_MS: u64 = 5_000;

fn link_url(peer: &str) -> String {
    let base = peer.trim_end_matches('/');
    let base = base.replacen("https://", "wss://", 1).replacen("http://", "ws://", 1);
    if base.ends_with("/link") {
        base
    } else {
        format!("{base}/link")
    }
}

pub fn priority_for(kind: &str) -> Priority {
    match kind {
        "alarm" | "station_alarm" => Priority::Alarm,
        "case" | "guidance" | "message" | "station" => Priority::Guidance,
        _ => Priority::Logs,
    }
}

pub async fn run(state: Shared, peer: String) {
    let url = link_url(&peer);
    let mut session = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(1);
    let pubkey = state.chain.lock().expect("chain").public_key().to_vec();
    // last record handed to the queue: if it changes or vanishes, the chain was reset
    let mut last_queued: Option<(u64, String)> = None;
    let profile = state.world.read().expect("world").link_profile.clone();
    let mut up = LinkSim::new(profile.clone(), session ^ 0x55);
    let mut down = LinkSim::new(profile.clone(), session ^ 0xaa);
    let mut ep = Endpoint::new(1).with_crc(profile.crc);
    let mut frame_to_chain: HashMap<u32, u64> = HashMap::new();
    let mut queued: HashSet<u64> = HashSet::new();
    let mut anchor_sent = false;
    let mut last_vitals = 0u64;
    let mut last_ping = 0u64;
    let mut tokens: usize = 0;
    let mut sink: Option<futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>> = None;
    let mut stream: Option<futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>> = None;
    let mut next_connect = 0u64;
    let mut tick = tokio::time::interval(std::time::Duration::from_millis(TICK_MS));
    tracing::info!("uplink: ship → {url}");

    loop {
        let now = state.clock.real_ms();
        if sink.is_none() && now >= next_connect {
            match tokio::time::timeout(std::time::Duration::from_secs(3), tokio_tungstenite::connect_async(&url)).await {
                Ok(Ok((ws, _))) => {
                    let (mut s, r) = ws.split();
                    let name = format!("{} · ship", state.cfg.node_name);
                    let hello = ep.queue.control(MsgType::Hello, Hello { pubkey: pubkey.clone(), ..Hello::new(Role::Ship, name, session) }.encode());
                    if s.send(Message::Binary(hello.encode().into())).await.is_ok() {
                        tracing::info!("uplink: connected");
                        sink = Some(s);
                        stream = Some(r);
                    }
                }
                _ => {
                    next_connect = now + 3_000;
                }
            }
        }

        tokio::select! {
            _ = tick.tick() => {
                let now = state.clock.real_ms();
                // 1. profile changes from the dev panel
                let wanted = state.world.read().expect("world").link_profile.clone();
                if wanted != up.profile {
                    up.set_profile(wanted.clone());
                    down.set_profile(wanted.clone());
                    ep.queue.crc = wanted.crc;
                }
                let p: LinkProfile = up.profile.clone();
                // 2. new chain records → queue. Anything not yet acknowledged counts:
                // after a restart, records left "Sent" by the old process go again
                // (the ground drops duplicates).
                {
                    let chain = state.chain.lock().expect("chain");
                    let reset = last_queued.as_ref().is_some_and(|(seq, hash)| chain.records.get(*seq as usize).map(|r| &r.hash) != Some(hash));
                    if reset {
                        // demo reset: new session so the ground's receiver starts clean too
                        tracing::info!("uplink: chain was reset; starting a new session");
                        queued.clear();
                        frame_to_chain.clear();
                        anchor_sent = false;
                        last_queued = None;
                        ep = Endpoint::new(1).with_crc(up.profile.crc);
                        session = session.wrapping_add(1);
                        sink = None;
                        stream = None;
                        next_connect = 0;
                    }
                    let fresh: Vec<&crate::chain::Record> = chain.records.iter().filter(|r| r.state != SyncState::Acked && !queued.contains(&r.seq)).collect();
                    for r in fresh {
                        let mut wrapper = serde_json::json!({"body": r.body});
                        if !anchor_sent {
                            wrapper["prev"] = serde_json::json!(r.prev);
                            anchor_sent = true;
                        }
                        let mut hash8 = [0u8; 8];
                        if let Ok(h) = hex::decode(&r.hash) {
                            hash8.copy_from_slice(&h[..8]);
                        }
                        let msg = RecordMsg { chain_seq: r.seq, ts_ms: r.ts_ms, kind: r.kind.clone(), subject: r.subject.clone(), hash8, body: serde_json::to_vec(&wrapper).unwrap_or_default(), sig: hex::decode(&r.sig).unwrap_or_default() };
                        let seq = ep.queue.push_with(MsgType::Record, priority_for(&r.kind), msg.encode(), None, None);
                        frame_to_chain.insert(seq, r.seq);
                        queued.insert(r.seq);
                        last_queued = Some((r.seq, r.hash.clone()));
                    }
                }
                // 3. vitals snapshot every 5 s (delta-encoded, latest-wins)
                if now >= last_vitals + VITALS_EVERY_MS {
                    last_vitals = now;
                    let ts = state.clock.now_s();
                    let samples: Vec<(u32, crate::protocol::delta::VitalsSample)> = state.world.read().expect("world").passengers.iter().map(|p| (p.id, p.vitals)).collect();
                    for (id, s) in samples {
                        ep.send_vitals(now, id, ts, &s);
                    }
                }
                if now >= last_ping + 2_000 {
                    last_ping = now;
                    ep.ping(now);
                }
                // 4. transmit within the profile's bandwidth while in contact
                let up_now = p.is_up(now) && sink.is_some();
                if up_now {
                    tokens = (tokens + p.budget(TICK_MS)).min(p.mtu.max(256) * 4);
                    let out = ep.outgoing(now, tokens, p.mtu.max(64));
                    for d in out {
                        tokens = tokens.saturating_sub(d.len());
                        up.send(now, d);
                    }
                } else {
                    tokens = 0;
                }
                // 5. in flight = sent
                let mission = state.now();
                {
                    let mut chain = state.chain.lock().expect("chain");
                    for s in ep.queue.inflight_seqs() {
                        if let Some(c) = frame_to_chain.get(&s) {
                            chain.set_state(*c, SyncState::Sent, mission);
                        }
                    }
                }
                // 6. deliver what the simulated link carried
                let arrived = up.recv(now);
                if let Some(s) = sink.as_mut() {
                    let mut broken = false;
                    for d in arrived {
                        if s.send(Message::Binary(d.into())).await.is_err() { broken = true; break; }
                    }
                    if broken { sink = None; stream = None; }
                }
                let mut acked_any = false;
                for d in down.recv(now) {
                    for ev in ep.incoming(now, &d) {
                        match ev {
                            Delivered::Acked(seqs) => {
                                let mut chain = state.chain.lock().expect("chain");
                                for s in seqs {
                                    if let Some(c) = frame_to_chain.remove(&s) {
                                        chain.set_state(c, SyncState::Acked, mission);
                                        acked_any = true;
                                    }
                                }
                            }
                            Delivered::Frame(f) if f.kind == MsgType::Text => {
                                if let Ok(t) = Text::decode(&f.payload) {
                                    let mut w = state.world.write().expect("world");
                                    w.messages.push(crate::model::LinkMessage { ts_ms: mission, from: t.from, to: t.to, text: t.text, channel: "ground".into() });
                                }
                            }
                            _ => {}
                        }
                    }
                }
                // 7. status for the dashboard
                {
                    let mut w = state.world.write().expect("world");
                    let l = &mut w.link;
                    l.connected = sink.is_some();
                    l.peer = Some(peer.clone());
                    l.profile = p.name.into();
                    l.profile_label = p.label.into();
                    l.up = up_now;
                    l.rtt_ms = ep.queue.rtt.srtt_ms;
                    l.rto_ms = ep.queue.rtt.rto_ms();
                    l.depth = ep.queue.depth();
                    l.bytes_sent = ep.queue.sent.total_bytes();
                    l.bytes_received = ep.received.total_bytes();
                    l.datagrams_lost = up.lost_datagrams + down.lost_datagrams;
                    l.stats = ep.queue.sent.rows();
                    if acked_any { l.last_contact_ms = Some(mission); }
                }
                if acked_any { state.emit(UiEvent::Tick); }
            }
            msg = async { match stream.as_mut() { Some(s) => s.next().await, None => std::future::pending().await } } => {
                match msg {
                    Some(Ok(Message::Binary(b))) => {
                        let now = state.clock.real_ms();
                        // the return path goes through the simulated link too
                        down.send(now, b.to_vec());
                    }
                    Some(Ok(_)) => {}
                    _ => {
                        tracing::warn!("uplink: connection lost; records stay queued");
                        sink = None;
                        stream = None;
                        next_connect = state.clock.real_ms() + 2_000;
                    }
                }
            }
        }
    }
}
