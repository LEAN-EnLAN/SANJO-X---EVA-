//! Shared application state: the world (passengers, alerts, cases, journal…),
//! the hash chain, the mission clock, the mesh registry and the UI event bus.
//!
//! Locks are `std::sync` and are never held across an `.await`.

use crate::chain::Chain;
use crate::clock::{MissionClock, DAY_MS};
use crate::eva_agent::AgentConfig;
use crate::mesh::Mesh;
use crate::model::*;
use crate::protocol::profile::LinkProfile;
use crate::protocol::queue::QueueDepth;
use crate::protocol::stats::StatsRow;
use crate::seed;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum NodeRole {
    /// on board: owns the wristbands, signs the chain, uplinks to the ground
    Ship,
    /// on Earth (or ISS mission control): mirrors the chain, acknowledges records
    Ground,
}

impl NodeRole {
    pub fn name(self) -> &'static str {
        match self {
            NodeRole::Ship => "ship",
            NodeRole::Ground => "ground",
        }
    }
}

/// Events that make open pages refresh. SSE handlers render fragments on each.
#[derive(Debug, Clone)]
pub enum UiEvent {
    /// periodic refresh (vitals, KPIs, queue)
    Tick,
    /// an emergency was opened for `seat`; helpers' apps switch screen
    Emergency { seat: String },
    /// solar storm started or ended
    Storm,
    /// Eva's state changed (seat None = dashboard Eva)
    Eva { seat: Option<String>, mood: EvaMood },
    /// a chat message was added for this seat
    Chat { seat: String },
    /// move a seat's app to a screen (tool `show_screen`)
    Navigate { seat: String, screen: String },
    /// the medical station's own screen changed (`grid`: compartments too, not only the Eva strip)
    Station { grid: bool },
    /// the demo was reset
    Reset,
    /// Eva went online or onto her on-board rules (`/api/eva/status` changed)
    EvaStatus,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct LinkStatus {
    pub connected: bool,
    pub peer: Option<String>,
    pub profile: String,
    pub profile_label: String,
    pub up: bool,
    pub rtt_ms: f64,
    pub rto_ms: u64,
    pub depth: QueueDepth,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub datagrams_lost: u64,
    pub stats: Vec<StatsRow>,
    pub last_contact_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Storm {
    pub since_ms: u64,
    pub ends_ms: u64,
    pub in_shelter: Vec<String>,
}

#[derive(Debug)]
pub struct World {
    pub role: NodeRole,
    pub ship_name: String,
    pub passengers: Vec<Passenger>,
    pub alerts: Vec<Alert>,
    pub cases: Vec<Case>,
    pub journal: Vec<JournalEntry>,
    pub meals: Vec<Meal>,
    pub reminders: Vec<Reminder>,
    pub chats: Vec<ChatMsg>,
    pub messages: Vec<LinkMessage>,
    pub storm: Option<Storm>,
    pub moods: HashMap<String, EvaMood>,
    pub dash_mood: EvaMood,
    pub link: LinkStatus,
    /// requested uplink profile (dev panel); the uplink task applies it
    pub link_profile: LinkProfile,
    /// Eva filtered these minor events instead of alerting
    pub filtered_events: u32,
    /// the hub's view of the medical station (from its reports; see `station.rs`)
    pub station: crate::station::StationView,
    /// simulated bands: real ms of each seat's last LoRa alarm (while the hub is down)
    pub band_lora: HashMap<String, u64>,
    /// mission ms when the dev panel took the hub down (simulated outage)
    pub hub_down_since: Option<u64>,
    next_id: u64,
}

impl World {
    pub fn seeded(role: NodeRole, ship_name: &str, now_ms: u64) -> Self {
        let day_start = (now_ms / DAY_MS) * DAY_MS;
        let mut passengers = seed::passengers();
        for p in passengers.iter_mut() {
            p.last_ms = now_ms;
            if let Some(e) = p.ecg.as_mut() {
                e.ts_ms = day_start + if p.seat == "4A" { 12 * 3_600_000 + 40 * 60_000 } else { 8 * 3_600_000 + 6 * 60_000 };
            }
        }
        World {
            role,
            ship_name: ship_name.into(),
            passengers,
            alerts: seed::alerts(day_start),
            cases: Vec::new(),
            journal: seed::journal(day_start),
            meals: seed::meals(day_start),
            reminders: vec![Reminder { id: 1, seat: "4B".into(), at_ms: day_start + 14 * 3_600_000 + 30 * 60_000, text: "Drink a glass of water".into(), done: false }],
            chats: Vec::new(),
            messages: Vec::new(),
            storm: None,
            moods: HashMap::new(),
            dash_mood: EvaMood::Idle,
            link: LinkStatus { profile: "near-space".into(), profile_label: "Near-space uplink".into(), ..Default::default() },
            link_profile: LinkProfile::near_space(),
            filtered_events: 23,
            station: Default::default(),
            band_lora: HashMap::new(),
            hub_down_since: None,
            next_id: 100,
        }
    }

    pub fn next_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }
    pub fn passenger(&self, seat: &str) -> Option<&Passenger> {
        self.passengers.iter().find(|p| p.seat.eq_ignore_ascii_case(seat))
    }
    pub fn passenger_mut(&mut self, seat: &str) -> Option<&mut Passenger> {
        self.passengers.iter_mut().find(|p| p.seat.eq_ignore_ascii_case(seat))
    }
    pub fn by_stream_mut(&mut self, id: u32) -> Option<&mut Passenger> {
        self.passengers.iter_mut().find(|p| p.id == id)
    }
    pub fn open_case_for(&self, seat: &str) -> Option<&Case> {
        self.cases.iter().find(|c| c.open && c.seat == seat)
    }
    pub fn open_case(&self) -> Option<&Case> {
        self.cases.iter().rev().find(|c| c.open)
    }
    /// Eva's state for a seat. A plain idle turns into `LowBattery` when the band
    /// needs charging, or `Sleeping` at ship night, so the mascot says so on its own.
    pub fn mood(&self, seat: &str, now_ms: u64) -> EvaMood {
        let m = self.moods.get(seat).copied().unwrap_or(EvaMood::Idle);
        if m != EvaMood::Idle {
            return m;
        }
        if self.passenger(seat).is_some_and(|p| p.vitals.base_batt < 20 || p.vitals.screen_batt < 20) {
            return EvaMood::LowBattery;
        }
        if is_night(now_ms) {
            return EvaMood::Sleeping;
        }
        EvaMood::Idle
    }
    /// Seats Eva has asked to help in an open case, with their task.
    pub fn helper_task(&self, seat: &str) -> Option<(&Case, &Helper)> {
        self.cases.iter().filter(|c| c.open).find_map(|c| c.helpers.iter().find(|h| h.seat == seat).map(|h| (c, h)))
    }
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub role: NodeRole,
    pub port: u16,
    pub peer: Option<String>,
    pub data_dir: PathBuf,
    pub node_name: String,
}

pub struct AppState {
    pub cfg: ServerConfig,
    pub world: RwLock<World>,
    pub chain: Mutex<Chain>,
    pub clock: Arc<MissionClock>,
    pub tx: broadcast::Sender<UiEvent>,
    pub mesh: Mesh,
    pub agent: AgentConfig,
    pub http: reqwest::Client,
    /// ground side: one Eva Link endpoint per ship session (survives reconnects)
    pub peers: Mutex<HashMap<String, crate::protocol::Endpoint>>,
    /// ground side: mirror of the ship's chain as received
    pub mirror: Mutex<crate::link::Mirror>,
    /// ground side: messages typed on the ground for the ship
    pub to_ship: Mutex<Vec<String>>,
    /// answers to idempotent POSTs (PWA outbox replays), see `web::idem`
    pub idem: crate::web::idem::IdemCache,
    /// dev panel "Hub down": the hub stops relaying and its rules stop (simulated failure)
    pub hub_down: std::sync::atomic::AtomicBool,
    /// the simulated LoRa broadcast medium: band and station alarms that reach
    /// every radio directly, whether the hub is up or not
    pub air: broadcast::Sender<Vec<u8>>,
    /// commands for the medical station, waiting for its session(s)
    pub station_out: Mutex<crate::station::StationOutbox>,
    /// the simulated station computer whose screen `/station` shows (when one runs here)
    pub station_device: std::sync::OnceLock<Arc<crate::station::StationHandle>>,
}

pub type Shared = Arc<AppState>;

impl AppState {
    pub fn new(cfg: ServerConfig, clock: Arc<MissionClock>, agent: AgentConfig) -> anyhow::Result<Shared> {
        let now = clock.now_ms();
        let chain_path = cfg.data_dir.join(format!("chain-{}.jsonl", cfg.node_name));
        let mut chain = Chain::open(&chain_path, &cfg.node_name)?;
        if chain.is_empty() && cfg.role == NodeRole::Ship {
            seed_chain(&mut chain, now);
        }
        let report = chain.verify();
        if !report.ok {
            tracing::warn!("chain {} failed verification at record {:?}", chain_path.display(), report.first_bad);
        }
        let (tx, _) = broadcast::channel(256);
        let world = World::seeded(cfg.role, &cfg.node_name, now);
        // the ground keeps its copy of the ship's chain on disk (survives restarts)
        let mirror = if cfg.role == NodeRole::Ground { crate::link::Mirror::open(&cfg.data_dir.join(format!("mirror-{}.jsonl", cfg.node_name)), &cfg.data_dir.join("keys"))? } else { crate::link::Mirror::default() };
        let http = reqwest::Client::builder().timeout(std::time::Duration::from_secs(45)).build()?;
        Ok(Arc::new(AppState {
            cfg,
            world: RwLock::new(world),
            chain: Mutex::new(chain),
            clock,
            tx,
            mesh: Mesh::default(),
            agent,
            http,
            peers: Mutex::new(HashMap::new()),
            mirror: Mutex::new(mirror),
            to_ship: Mutex::new(Vec::new()),
            idem: Default::default(),
            hub_down: Default::default(),
            air: broadcast::channel(64).0,
            station_out: Default::default(),
            station_device: Default::default(),
        }))
    }

    pub fn now(&self) -> u64 {
        self.clock.now_ms()
    }
    pub fn emit(&self, ev: UiEvent) {
        let _ = self.tx.send(ev);
    }
    /// Dev panel "Hub down" is on: no relay, no hub rules (the station and bands fall back to LoRa).
    pub fn is_hub_down(&self) -> bool {
        self.hub_down.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Layer 3: append to the hash chain. Returns the record seq.
    pub fn log(&self, kind: &str, subject: &str, body: serde_json::Value) -> u64 {
        let now = self.now();
        self.chain.lock().expect("chain lock").append(now, kind, subject, body)
    }

    pub fn set_mood(&self, seat: Option<&str>, mood: EvaMood) {
        let now = self.now();
        let shown = {
            let mut w = self.world.write().expect("world");
            match seat {
                Some(s) => {
                    w.moods.insert(s.to_string(), mood);
                    w.mood(s, now)
                }
                None => {
                    w.dash_mood = mood;
                    mood
                }
            }
        };
        self.emit(UiEvent::Eva { seat: seat.map(String::from), mood: shown });
    }

    /// Eva's state for a seat as the app shows it (see `World::mood`).
    pub fn mood_of(&self, seat: &str) -> EvaMood {
        self.world.read().expect("world").mood(seat, self.now())
    }

    /// Demo reset: world, chain and clock back to the start.
    pub fn reset(&self) {
        self.clock.reset_to(seed::DEMO_DAY, seed::DEMO_HH, seed::DEMO_MM);
        let now = self.now();
        {
            let mut chain = self.chain.lock().expect("chain");
            chain.reset();
            if self.cfg.role == NodeRole::Ship {
                seed_chain(&mut chain, now);
            }
        }
        {
            let mut w = self.world.write().expect("world");
            let profile = w.link_profile.clone();
            *w = World::seeded(self.cfg.role, &self.cfg.node_name, now);
            w.link_profile = profile;
        }
        self.mirror.lock().expect("mirror").reset();
        self.hub_down.store(false, std::sync::atomic::Ordering::Relaxed);
        crate::station::restock(self);
        self.emit(UiEvent::Reset);
    }
}

/// The morning's records. Signed on board like any other record and sent by
/// the uplink: only the ground's ack may mark a record Acknowledged (a seeded
/// "already acknowledged" record would never reach the ground's mirror).
fn seed_chain(chain: &mut Chain, now: u64) {
    let day_start = (now / DAY_MS) * DAY_MS;
    for (ts, kind, subject, body) in seed::chain_records(day_start) {
        chain.append(ts, kind, subject, body);
    }
}

/// Ship night for the mascot: 22:30 to 06:30 mission time (the day plan's sleep window).
pub fn is_night(now_ms: u64) -> bool {
    let m = (now_ms % crate::clock::DAY_MS) / 60_000;
    !(6 * 60 + 30..22 * 60 + 30).contains(&m)
}

/// Set a short-lived state (happy, celebrate, sending…) that falls back to idle
/// after `ms` of real time, unless something else changed Eva's state meanwhile.
pub fn pulse_mood(state: &Shared, seat: &str, mood: EvaMood, ms: u64) {
    state.set_mood(Some(seat), mood);
    let Ok(rt) = tokio::runtime::Handle::try_current() else { return };
    let (st, seat) = (state.clone(), seat.to_string());
    rt.spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
        let still = st.world.read().expect("world").moods.get(&seat).copied() == Some(mood);
        if still {
            st.set_mood(Some(&seat), EvaMood::Idle);
        }
    });
}
