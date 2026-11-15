//! Server-side mesh: presence, routing and relay between every client that
//! talks Eva Link over `/link` (PWA apps, dashboards, band gateways, peers).
//!
//! The binary runs as the local hub ("triangulation" through the server):
//! clients never need to find each other; each holds one WebSocket to the hub,
//! the hub knows who is online, measures each client's RTT with its own pings,
//! and relays frames addressed to another node id. End-to-end RTT between two
//! clients is then measured by the clients themselves (ping with `dst`).

use crate::protocol::frame::{Frame, MsgType};
use crate::protocol::payload::{self, PresenceEntry, Role};
use crate::protocol::queue::RttEstimator;
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use tokio::sync::mpsc;

pub const HUB_ID: u32 = 0;

/// Frames waiting for one client's WebSocket. A client that falls this far
/// behind (frozen tab, dying Wi-Fi) loses frames instead of growing the hub's
/// memory; vitals and presence are latest-wins and reliable frames are
/// retransmitted by their sender.
pub const OUTBOX: usize = 512;

#[derive(Debug)]
pub struct Node {
    pub id: u32,
    pub role: Role,
    pub name: String,
    pub seat: Option<String>,
    pub tx: mpsc::Sender<Vec<u8>>,
    pub rtt: RttEstimator,
    pub connected_ms: u64,
    pub last_seen_ms: u64,
    pub frames_in: u64,
    pub frames_out: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeView {
    pub id: u32,
    pub role: Role,
    pub name: String,
    pub seat: Option<String>,
    pub rtt_ms: f64,
    pub rtt_samples: u64,
    pub connected_s: u64,
    pub frames_in: u64,
    pub frames_out: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
}

#[derive(Debug, Default)]
pub struct Mesh {
    nodes: Mutex<BTreeMap<u32, Node>>,
    next: AtomicU32,
    hub_seq: AtomicU32,
    pub relayed_frames: AtomicU32,
    /// frames dropped because a client's outbox was full (slow client)
    pub dropped_frames: AtomicU32,
    /// real ms of the last page refresh triggered by band vitals (rate limit)
    pub last_vitals_refresh_ms: std::sync::atomic::AtomicU64,
}

/// Band vitals refresh open pages at most this often (the 1 Hz server tick
/// still runs Eva's rules); keeps vitals-in → dashboard latency in
/// milliseconds instead of up to one tick.
pub const VITALS_REFRESH_MS: u64 = 200;

impl Mesh {
    pub fn register(&self, role: Role, name: String, seat: Option<String>, tx: mpsc::Sender<Vec<u8>>, now_ms: u64) -> u32 {
        let id = self.next.fetch_add(1, Ordering::Relaxed) + 1;
        let node = Node { id, role, name, seat, tx, rtt: RttEstimator::default(), connected_ms: now_ms, last_seen_ms: now_ms, frames_in: 0, frames_out: 0, bytes_in: 0, bytes_out: 0 };
        self.nodes.lock().expect("mesh").insert(id, node);
        id
    }
    pub fn unregister(&self, id: u32) {
        self.nodes.lock().expect("mesh").remove(&id);
    }
    pub fn hub_seq(&self) -> u32 {
        self.hub_seq.fetch_add(1, Ordering::Relaxed) + 1
    }
    pub fn seen(&self, id: u32, bytes: usize, now_ms: u64) {
        if let Some(n) = self.nodes.lock().expect("mesh").get_mut(&id) {
            n.frames_in += 1;
            n.bytes_in += bytes as u64;
            n.last_seen_ms = now_ms;
        }
    }
    pub fn rtt_sample(&self, id: u32, rtt_ms: f64) {
        if let Some(n) = self.nodes.lock().expect("mesh").get_mut(&id) {
            n.rtt.sample(rtt_ms);
        }
    }
    /// Send raw bytes to one node. Returns false if it is gone.
    pub fn send_to(&self, id: u32, bytes: Vec<u8>) -> bool {
        let mut nodes = self.nodes.lock().expect("mesh");
        match nodes.get_mut(&id) {
            Some(n) => {
                n.frames_out += 1;
                n.bytes_out += bytes.len() as u64;
                match n.tx.try_send(bytes) {
                    Ok(()) => true,
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        self.dropped_frames.fetch_add(1, Ordering::Relaxed);
                        true
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => false,
                }
            }
            None => false,
        }
    }
    /// Send a hub-originated frame to one node.
    pub fn send_frame(&self, id: u32, kind: MsgType, payload: Vec<u8>) -> bool {
        let f = Frame::new(kind, self.hub_seq(), HUB_ID, payload).to(id);
        self.send_to(id, f.encode())
    }
    /// Relay raw bytes to every node of the given roles except `except`.
    pub fn broadcast(&self, roles: &[Role], except: u32, bytes: &[u8]) -> usize {
        let mut nodes = self.nodes.lock().expect("mesh");
        let mut n = 0;
        for node in nodes.values_mut().filter(|x| x.id != except && roles.contains(&x.role)) {
            node.frames_out += 1;
            node.bytes_out += bytes.len() as u64;
            match node.tx.try_send(bytes.to_vec()) {
                Ok(()) => n += 1,
                Err(mpsc::error::TrySendError::Full(_)) => {
                    self.dropped_frames.fetch_add(1, Ordering::Relaxed);
                }
                Err(_) => {}
            }
        }
        n
    }
    pub fn presence(&self) -> Vec<PresenceEntry> {
        self.nodes.lock().expect("mesh").values().map(|n| PresenceEntry { id: n.id, role: n.role, name: n.name.clone(), rtt_ms: n.rtt.srtt_ms.round() as u32 }).collect()
    }
    pub fn views(&self, now_ms: u64) -> Vec<NodeView> {
        self.nodes
            .lock()
            .expect("mesh")
            .values()
            .map(|n| NodeView {
                id: n.id,
                role: n.role,
                name: n.name.clone(),
                seat: n.seat.clone(),
                rtt_ms: (n.rtt.srtt_ms * 10.0).round() / 10.0,
                rtt_samples: n.rtt.samples,
                connected_s: now_ms.saturating_sub(n.connected_ms) / 1000,
                frames_in: n.frames_in,
                frames_out: n.frames_out,
                bytes_in: n.bytes_in,
                bytes_out: n.bytes_out,
            })
            .collect()
    }
    pub fn count(&self) -> usize {
        self.nodes.lock().expect("mesh").len()
    }
    pub fn ids(&self) -> Vec<u32> {
        self.nodes.lock().expect("mesh").keys().copied().collect()
    }
    pub fn role_of(&self, id: u32) -> Option<Role> {
        self.nodes.lock().expect("mesh").get(&id).map(|n| n.role)
    }
}

/// Hub heartbeat: ping every node every 2 s (RTT per client), presence every 2 s.
pub async fn run_heartbeat(state: crate::state::Shared) {
    let mut tick = tokio::time::interval(std::time::Duration::from_secs(2));
    loop {
        tick.tick().await;
        if state.is_hub_down() {
            // simulated hub failure: no heartbeat, so clients notice the silence
            continue;
        }
        let now = state.clock.real_ms();
        for id in state.mesh.ids() {
            state.mesh.send_frame(id, MsgType::Ping, payload::encode_time(now));
        }
        let pres = payload::encode_presence(&state.mesh.presence());
        let f = Frame::new(MsgType::Presence, state.mesh.hub_seq(), HUB_ID, pres).encode();
        state.mesh.broadcast(&[Role::App, Role::Dash], HUB_ID, &f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slow_client_drops_frames_instead_of_growing_memory() {
        let m = Mesh::default();
        let (tx, mut rx) = mpsc::channel(OUTBOX);
        let id = m.register(Role::App, "slow".into(), None, tx, 0);
        for _ in 0..OUTBOX * 3 {
            assert!(m.send_to(id, vec![0; 100]));
        }
        assert_eq!(m.dropped_frames.load(Ordering::Relaxed) as usize, OUTBOX * 2);
        let mut got = 0;
        while rx.try_recv().is_ok() {
            got += 1;
        }
        assert_eq!(got, OUTBOX, "the outbox is bounded");
        drop(rx);
        assert!(!m.send_to(id, vec![1]), "a gone client is reported");
        m.unregister(id);
        assert_eq!(m.count(), 0);
    }
}
