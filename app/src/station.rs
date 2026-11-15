//! **The ship's medical station** as a client of the hub (ADR-011).
//!
//! The station is a wall cabinet with numbered compartments (configurable for
//! each ship; the layout lives in [`COMPARTMENTS`]), an Eva screen,
//! door and item sensors, a red emergency button, a LoRa radio and a backup
//! battery. It is a *client* node of the hub, like the wristbands and phones:
//! it says `Hello` on `/link` with role `Station`, the hub sends it
//! [`StationCmd`]s (light compartment 01, show "Take the AED", clear) and it
//! reports [`StationEvent`]s back (door opened, item taken, red button, back
//! online). The hub turns every report into a signed chain record, so it
//! reaches the ground like any other record.
//!
//! Two halves live here:
//!
//! * **Device side** ([`StationDevice`], [`run_device`]): the station's own
//!   small computer. It keeps the compartments, the message on its screen and
//!   a reliable Eva Link [`Endpoint`] whose send queue is also its buffer. If
//!   it stops hearing the hub (no frame for [`HUB_SILENT_MS`]) it goes
//!   **standalone**: a wristband's no-pulse alarm heard over LoRa lights 01
//!   and shows "Emergency · Take the AED" by itself, and the red button
//!   broadcasts an alarm to every wristband over the air. Events wait in the
//!   queue; when the hub answers again they are uploaded in order.
//! * **Hub side** ([`StationView`], [`light`], [`clear`], [`on_frame`]): what
//!   the hub knows about the station (only from its reports), the commands it
//!   queues for it, and what a report changes in the world (case timeline,
//!   dashboard tile, alarms to every app).
//!
//! In `eva demo` the station's computer is simulated inside the same binary
//! ([`spawn_local`]) and talks to the hub through an in-memory link session;
//! `/station` is its screen. Tests also drive it over a real WebSocket.

use crate::clock::MissionClock;
use crate::eva_agent;
use crate::mesh::HUB_ID;
use crate::model::{Alert, CaseStep};
use crate::protocol::frame::{decode_batch, Frame, MsgType, Priority, FLAG_REL};
use crate::protocol::payload::{alarm_code, station_ev, station_flag, station_op, Alarm, Hello, Role, StationCmd, StationEvent, StationStatus, Welcome, STATION_COMPARTMENTS};
use crate::protocol::{Delivered, Endpoint};
use crate::state::{AppState, Shared, UiEvent};
use serde::Serialize;
use serde_json::json;
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc};

/// The station goes standalone when it has heard nothing from the hub for this
/// long (the hub pings every node every 2 s, so two missed heartbeats).
pub const HUB_SILENT_MS: u64 = 5_000;
/// The station sends its status (battery, lights, buffer) at least this often.
pub const STATUS_EVERY_MS: u64 = 10_000;
/// Simulated bands repeat their LoRa no-pulse alarm this often while the hub is down.
pub const LORA_REPEAT_MS: u64 = 10_000;
/// The compartment that holds the AED.
pub const AED: u8 = 1;

// ---------------------------------------------------------------- the cabinet

/// How a compartment is drawn: one of the 3D icons (`static/img/icons/<name>.webp`)
/// or one of our line icons (`web::ui::ic`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Pic(&'static str),
    Line(&'static str),
}

#[derive(Debug, Clone, Copy)]
pub struct CompartmentDef {
    pub n: u8,
    pub label: &'static str,
    /// what is inside, one short line
    pub contents: &'static str,
    /// the item the item sensor counts ("AED" in 01)
    pub item: &'static str,
    pub icon: Icon,
    /// items when fully stocked
    pub full: u8,
}

/// The demo ship's compartments, numbered row by row as on the cabinet.
/// Configurable per ship: the UI and the API read this list, never a fixed
/// count (the wire format caps it at [`STATION_COMPARTMENTS`]).
pub const COMPARTMENTS: &[CompartmentDef] = &[
    CompartmentDef { n: 1, label: "AED", contents: "AED · spare pads · spare battery", item: "AED", icon: Icon::Pic("aed"), full: 1 },
    CompartmentDef { n: 2, label: "Breathing", contents: "Oxygen · mask · bag-valve mask", item: "oxygen kit", icon: Icon::Pic("lungs"), full: 3 },
    CompartmentDef { n: 3, label: "Diagnostics", contents: "Monitor · thermometer · glucometer", item: "diagnostic kit", icon: Icon::Pic("temperature"), full: 5 },
    CompartmentDef { n: 4, label: "Ultrasound", contents: "Probe · tablet · gel", item: "ultrasound kit", icon: Icon::Line("probe"), full: 1 },
    CompartmentDef { n: 5, label: "Wounds", contents: "Dressings · bandages · shears", item: "wound kit", icon: Icon::Line("bandage"), full: 6 },
    CompartmentDef { n: 6, label: "Restraint", contents: "Straps · splint", item: "restraint kit", icon: Icon::Line("strap"), full: 2 },
    CompartmentDef { n: 7, label: "Oral meds", contents: "Tablets · blister packs", item: "medication pack", icon: Icon::Line("pill"), full: 8 },
    CompartmentDef { n: 8, label: "Injectables", contents: "Crew medical kit · ampoules · syringes", item: "injectables kit", icon: Icon::Line("syringe"), full: 6 },
    CompartmentDef { n: 9, label: "IV access", contents: "Fluids · giving sets · cannulas", item: "IV kit", icon: Icon::Line("drip"), full: 4 },
    CompartmentDef { n: 10, label: "Eyes & dental", contents: "Eye wash · dental kit", item: "eye or dental kit", icon: Icon::Line("eye"), full: 2 },
    CompartmentDef { n: 11, label: "Airway", contents: "Airways · suction unit", item: "airway kit", icon: Icon::Line("airway"), full: 4 },
    CompartmentDef { n: 12, label: "Hygiene", contents: "Gloves · goggles · waste bags · sharps bin", item: "hygiene kit", icon: Icon::Line("hand"), full: 6 },
];
const _: () = assert!(COMPARTMENTS.len() <= STATION_COMPARTMENTS as usize, "more compartments than the wire format carries");

pub fn def(n: u8) -> Option<&'static CompartmentDef> {
    COMPARTMENTS.get((n as usize).wrapping_sub(1))
}

fn bit(n: u8) -> u16 {
    1 << (n - 1)
}

/// "00:47": minutes and seconds since the case opened.
pub fn mmss(ms: u64) -> String {
    format!("{:02}:{:02}", ms / 60_000, (ms / 1000) % 60)
}

pub fn event_name(e: u8) -> &'static str {
    match e {
        station_ev::DOOR_OPENED => "door_opened",
        station_ev::DOOR_CLOSED => "door_closed",
        station_ev::ITEM_TAKEN => "item_taken",
        station_ev::BUTTON => "red_button",
        station_ev::ONLINE => "online",
        station_ev::ALARM_HEARD => "alarm_heard",
        station_ev::SELF_LIT => "self_lit",
        _ => "unknown",
    }
}

// ---------------------------------------------------------------- device side

/// One compartment as the station's own computer sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Door {
    /// closed, light off
    Idle,
    /// closed, light strip on (the hub or the station's own rule asked for it)
    Lit,
    /// door open, item still in
    Open,
    /// the item sensor saw the item leave
    Taken,
}

impl Door {
    pub fn name(self) -> &'static str {
        match self {
            Door::Idle => "idle",
            Door::Lit => "lit",
            Door::Open => "open",
            Door::Taken => "taken",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Compartment {
    pub n: u8,
    pub state: Door,
    pub items: u8,
}

pub const READY: &str = "Ready · all compartments checked";

/// The station's small computer: compartments, screen, buffer, radio.
#[derive(Debug)]
pub struct StationDevice {
    pub comps: Vec<Compartment>,
    /// the line on the Eva screen ("Take the AED · Ana, seat 4B")
    pub message: String,
    pub emergency: bool,
    pub node_id: Option<u32>,
    /// a link session to the hub exists (the pipe is open)
    pub connected: bool,
    /// no frame from the hub for [`HUB_SILENT_MS`]: running on its own rules
    pub standalone: bool,
    pub battery: u8,
    pub on_battery: bool,
    /// real ms of the last frame from the hub
    pub last_hub_ms: u64,
    pub silent_after_ms: u64,
    /// reliable link to the hub; its send queue doubles as the event buffer
    pub ep: Endpoint,
    /// seats whose LoRa alarm the station already acted on
    pub heard: Vec<String>,
    /// frames to put on the air (LoRa broadcast)
    pub air_out: Vec<Vec<u8>>,
    /// (mission ms, text) of the last thing that happened at the station
    pub last_event: Option<(u64, String)>,
    air_seq: u32,
    last_status_ms: u64,
    status_dirty: bool,
    dirty: Option<bool>,
}

impl Default for StationDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl StationDevice {
    pub fn new() -> Self {
        StationDevice {
            comps: COMPARTMENTS.iter().map(|d| Compartment { n: d.n, state: Door::Idle, items: d.full }).collect(),
            message: READY.into(),
            emergency: false,
            node_id: None,
            connected: false,
            standalone: false,
            battery: 96,
            on_battery: false,
            last_hub_ms: 0,
            silent_after_ms: HUB_SILENT_MS,
            ep: Endpoint::new(0),
            heard: Vec::new(),
            air_out: Vec::new(),
            last_event: None,
            air_seq: 0,
            last_status_ms: 0,
            status_dirty: true,
            dirty: Some(true),
        }
    }

    fn touch(&mut self, grid: bool) {
        self.dirty = Some(grid || self.dirty.unwrap_or(false));
        self.status_dirty = true;
    }
    /// What changed on the screen since the last call (Some(true) = compartments too).
    pub fn take_dirty(&mut self) -> Option<bool> {
        self.dirty.take()
    }
    pub fn comp(&self, n: u8) -> Option<&Compartment> {
        self.comps.iter().find(|c| c.n == n)
    }
    fn comp_mut(&mut self, n: u8) -> Option<&mut Compartment> {
        self.comps.iter_mut().find(|c| c.n == n)
    }
    fn mask(&self, f: impl Fn(&Compartment) -> bool) -> u16 {
        self.comps.iter().filter(|c| f(c)).fold(0, |m, c| m | bit(c.n))
    }
    pub fn lit_mask(&self) -> u16 {
        self.mask(|c| c.state == Door::Lit)
    }

    /// Events waiting for the hub (queued or sent and not yet acknowledged).
    pub fn buffered(&self) -> usize {
        let d = self.ep.queue.depth();
        d.alarm + d.guidance + d.logs + d.inflight
    }
    pub fn mode_label(&self) -> &'static str {
        if self.standalone {
            "Standalone"
        } else if self.connected && self.node_id.is_some() {
            "Hub online"
        } else {
            "Connecting"
        }
    }

    pub fn status(&self) -> StationStatus {
        StationStatus {
            battery: self.battery,
            flags: if self.standalone { station_flag::STANDALONE } else { 0 } | if self.on_battery { station_flag::ON_BATTERY } else { 0 },
            lit: self.lit_mask(),
            open: self.mask(|c| c.state == Door::Open),
            taken: self.mask(|c| c.state == Door::Taken),
            buffered: self.buffered() as u32,
            items: self.comps.iter().map(|c| c.items).collect(),
        }
    }

    /// First frame on a new link session.
    pub fn hello(&mut self, session_ms: u64) -> Vec<u8> {
        let f = self.ep.queue.control(MsgType::Hello, Hello::new(Role::Station, "Medical station · corridor B", session_ms).encode());
        f.encode()
    }

    fn report(&mut self, now_s: u64, event: u8, compartment: u8, count: u32, subject: &str, text: String) {
        let ev = StationEvent { event, compartment, t_s: now_s, count, subject: subject.to_string() };
        let prio = if event == station_ev::BUTTON { Priority::Alarm } else { Priority::Guidance };
        self.ep.queue.push_with(MsgType::StationEvent, prio, ev.encode(), None, None);
        self.last_event = Some((now_s * 1000, text));
        self.touch(false);
    }

    /// A datagram from the hub arrived.
    pub fn on_datagram(&mut self, now_real: u64, now_s: u64, b: &[u8]) {
        self.last_hub_ms = now_real;
        let back = self.standalone;
        if back {
            // the hub answers again: say so, and send the buffer at once
            self.standalone = false;
            self.ep.queue.retransmit_now(now_real);
            let n = self.buffered() as u32;
            self.report(now_s, station_ev::ONLINE, 0, n, "", format!("Hub back · uploading {n} buffered event{}", if n == 1 { "" } else { "s" }));
            if !self.emergency {
                self.message = READY.into();
            }
            self.touch(false);
        }
        for ev in self.ep.incoming(now_real, b) {
            match ev {
                Delivered::Frame(f) => match f.kind {
                    MsgType::Welcome => {
                        if let Ok(w) = Welcome::decode(&f.payload) {
                            let first = self.node_id.is_none();
                            self.node_id = Some(w.node_id);
                            self.ep.queue.src = w.node_id;
                            if first && !back {
                                let n = self.buffered() as u32;
                                self.report(now_s, station_ev::ONLINE, 0, n, "", "Station online".into());
                            }
                            self.touch(false);
                        }
                    }
                    MsgType::StationCmd => {
                        if let Ok(c) = StationCmd::decode(&f.payload) {
                            self.on_cmd(&c);
                        }
                    }
                    _ => {}
                },
                Delivered::Acked(_) => self.touch(false),
                _ => {}
            }
        }
    }

    /// A command from the hub.
    pub fn on_cmd(&mut self, c: &StationCmd) {
        match c.op {
            station_op::LIGHT => {
                // a door already opened (the item may be gone) keeps its state and its message
                let mut fresh = true;
                if let Some(x) = self.comp_mut(c.compartment) {
                    match x.state {
                        Door::Idle => x.state = Door::Lit,
                        Door::Lit => {}
                        Door::Open | Door::Taken => fresh = false,
                    }
                }
                if fresh && !c.text.is_empty() {
                    self.message = c.text.clone();
                }
                self.emergency = true;
            }
            station_op::CLEAR => {
                if let Some(x) = self.comp_mut(c.compartment) {
                    x.state = Door::Idle;
                }
                self.settle();
            }
            station_op::CLEAR_ALL => {
                for x in self.comps.iter_mut() {
                    x.state = Door::Idle;
                }
                self.settle();
            }
            station_op::RESTOCK => {
                for (x, d) in self.comps.iter_mut().zip(COMPARTMENTS.iter()) {
                    x.state = Door::Idle;
                    x.items = d.full;
                }
                self.heard.clear();
                self.last_event = None;
                self.settle();
            }
            _ => {}
        }
        self.touch(true);
    }

    /// Nothing lit or open any more: back to the calm screen.
    fn settle(&mut self) {
        if self.comps.iter().all(|c| c.state == Door::Idle) {
            self.emergency = false;
            self.message = READY.into();
        }
    }

    /// Someone opens compartment `n` and takes an item (a tap on `/station`).
    pub fn tap(&mut self, n: u8, now_s: u64) {
        let Some(d) = def(n) else { return };
        let Some(x) = self.comp(n).copied() else { return };
        match x.state {
            Door::Taken => {
                self.message = format!("{} already taken · the light stays on until the case closes", d.item);
                self.touch(false);
            }
            _ if x.items == 0 => {
                if let Some(c) = self.comp_mut(n) {
                    c.state = Door::Open;
                }
                self.report(now_s, station_ev::DOOR_OPENED, n, 0, "", format!("Door {n:02} opened · empty"));
                self.message = format!("Compartment {n:02} is empty · tell the crew lead");
                self.touch(true);
            }
            _ => {
                let left = x.items - 1;
                if let Some(c) = self.comp_mut(n) {
                    c.state = Door::Taken;
                    c.items = left;
                }
                self.report(now_s, station_ev::DOOR_OPENED, n, x.items as u32, "", format!("Door {n:02} opened"));
                self.report(now_s, station_ev::ITEM_TAKEN, n, left as u32, "", format!("{} taken from {n:02}", d.item));
                if n == AED {
                    self.message = "AED taken · run it to the patient · the AED analyses and decides the shock".into();
                } else if !self.emergency {
                    self.message = format!("{} taken · {} left", d.item, left);
                }
                self.touch(true);
            }
        }
    }

    /// The red emergency button.
    pub fn press_button(&mut self, now_s: u64) {
        let standalone = self.standalone;
        self.report(now_s, station_ev::BUTTON, 0, 0, "", "Emergency button pressed".into());
        if standalone {
            // the hub can not relay it: broadcast to every wristband over LoRa
            let text = "Emergency button at the medical station, corridor B";
            self.air_seq = self.air_seq.wrapping_add(1).max(1);
            let mut f = Frame::new(MsgType::Alarm, self.air_seq, self.node_id.unwrap_or(0), Alarm { subject: "station".into(), code: alarm_code::STATION_BUTTON, text: text.into() }.encode()).with_crc();
            f.flags &= !FLAG_REL;
            self.air_out.push(f.encode());
            self.message = "Emergency · alarm sent to every wristband over LoRa".into();
        } else {
            self.message = "Emergency · the hub is alerting the crew".into();
        }
        self.emergency = true;
        self.touch(false);
    }

    /// A LoRa datagram heard on the air (a wristband's alarm when the hub is down).
    pub fn on_air(&mut self, b: &[u8], now_s: u64) {
        let Ok(frames) = decode_batch(b) else { return };
        for f in frames.into_iter().filter(|f| f.kind == MsgType::Alarm) {
            let Ok(a) = Alarm::decode(&f.payload) else { continue };
            if a.code != alarm_code::NO_PULSE || a.subject.is_empty() || self.heard.iter().any(|s| *s == a.subject) {
                continue;
            }
            let seat: String = a.subject.chars().take(8).collect();
            self.heard.push(seat.clone());
            self.report(now_s, station_ev::ALARM_HEARD, AED, 0, &seat, format!("LoRa alarm from seat {seat}'s wristband"));
            if self.comp(AED).is_some_and(|c| c.state == Door::Idle) {
                if let Some(c) = self.comp_mut(AED) {
                    c.state = Door::Lit;
                }
                self.report(now_s, station_ev::SELF_LIT, AED, 0, &seat, "Lit 01 by itself".into());
            }
            self.message = format!("Emergency · Take the AED · seat {seat}");
            self.emergency = true;
            self.touch(true);
        }
    }

    /// Every 100 ms: notice a silent hub, and return datagrams for the hub.
    pub fn tick(&mut self, now_real: u64) -> Vec<Vec<u8>> {
        if !self.standalone && self.node_id.is_some() && now_real.saturating_sub(self.last_hub_ms) > self.silent_after_ms {
            self.standalone = true;
            if !self.emergency {
                self.message = "Standalone · the hub is not answering · LoRa alarms still light the AED".into();
            }
            self.touch(false);
        }
        if self.standalone || !self.connected {
            // nothing reaches the hub: events wait in the queue (the buffer)
            return Vec::new();
        }
        if self.node_id.is_some() && (self.status_dirty || now_real >= self.last_status_ms + STATUS_EVERY_MS) {
            self.status_dirty = false;
            self.last_status_ms = now_real;
            self.ep.queue.purge_stale(Priority::Vitals, now_real);
            let s = self.status().encode();
            self.ep.queue.push_with(MsgType::StationStatus, Priority::Vitals, s, None, Some(now_real + STATUS_EVERY_MS));
        }
        self.ep.outgoing(now_real, usize::MAX, 16_000)
    }
}

/// The station computer plus what it needs from its surroundings: a clock,
/// the air (LoRa) and a screen to refresh. It never reads the hub's state.
pub struct StationHandle {
    pub dev: Mutex<StationDevice>,
    pub clock: Arc<MissionClock>,
    pub air: broadcast::Sender<Vec<u8>>,
    pub ui: broadcast::Sender<UiEvent>,
}

impl StationHandle {
    pub fn new(clock: Arc<MissionClock>, air: broadcast::Sender<Vec<u8>>, ui: broadcast::Sender<UiEvent>) -> Self {
        StationHandle { dev: Mutex::new(StationDevice::new()), clock, air, ui }
    }
    /// Refresh the station screen if something changed.
    pub fn notify(&self) {
        let d = self.dev.lock().expect("station").take_dirty();
        if let Some(grid) = d {
            let _ = self.ui.send(UiEvent::Station { grid });
        }
    }
    pub fn tap(&self, n: u8) {
        self.dev.lock().expect("station").tap(n, self.clock.now_s());
        self.notify();
    }
    pub fn press_button(&self) {
        self.dev.lock().expect("station").press_button(self.clock.now_s());
        self.notify();
    }
    pub fn snapshot(&self) -> StationSnapshot {
        let d = self.dev.lock().expect("station");
        StationSnapshot {
            comps: d.comps.clone(),
            message: d.message.clone(),
            emergency: d.emergency,
            node_id: d.node_id,
            mode: d.mode_label(),
            standalone: d.standalone,
            battery: d.battery,
            buffered: d.buffered(),
            last_event: d.last_event.clone(),
        }
    }
}

/// A copy of the station screen's state for rendering.
#[derive(Debug, Clone, Serialize)]
pub struct StationSnapshot {
    pub comps: Vec<Compartment>,
    pub message: String,
    pub emergency: bool,
    pub node_id: Option<u32>,
    pub mode: &'static str,
    pub standalone: bool,
    pub battery: u8,
    pub buffered: usize,
    pub last_event: Option<(u64, String)>,
}

/// The station computer's main loop over one link session (in-memory or WebSocket).
pub async fn run_device(h: Arc<StationHandle>, to_hub: mpsc::Sender<Vec<u8>>, mut from_hub: mpsc::Receiver<Vec<u8>>) {
    let mut air_rx = h.air.subscribe();
    let hello = {
        let mut d = h.dev.lock().expect("station");
        d.connected = true;
        d.last_hub_ms = h.clock.real_ms();
        d.hello(h.clock.real_ms())
    };
    if to_hub.send(hello).await.is_err() {
        return;
    }
    let mut tick = tokio::time::interval(std::time::Duration::from_millis(100));
    loop {
        tokio::select! {
            m = from_hub.recv() => match m {
                Some(b) => h.dev.lock().expect("station").on_datagram(h.clock.real_ms(), h.clock.now_s(), &b),
                None => break,
            },
            a = air_rx.recv() => match a {
                Ok(b) => h.dev.lock().expect("station").on_air(&b, h.clock.now_s()),
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(broadcast::error::RecvError::Closed) => break,
            },
            _ = tick.tick() => {
                let (out, air) = {
                    let mut d = h.dev.lock().expect("station");
                    let out = d.tick(h.clock.real_ms());
                    (out, std::mem::take(&mut d.air_out))
                };
                for a in air {
                    let _ = h.air.send(a);
                }
                let mut broken = false;
                for d in out {
                    if to_hub.send(d).await.is_err() {
                        broken = true;
                        break;
                    }
                }
                if broken {
                    break;
                }
            }
        }
        h.notify();
    }
    {
        let mut d = h.dev.lock().expect("station");
        d.connected = false;
        d.touch(false);
    }
    h.notify();
}

/// `eva demo`: one simulated station computer in this process, a client of
/// this hub through an in-memory link session. `/station` shows its screen.
pub fn spawn_local(state: &Shared) -> Arc<StationHandle> {
    let h = Arc::new(StationHandle::new(state.clock.clone(), state.air.clone(), state.tx.clone()));
    let _ = state.station_device.set(h.clone());
    let (to_hub, from_hub) = crate::link::local_session(state.clone(), None);
    tokio::spawn(run_device(h.clone(), to_hub, from_hub));
    h
}

/// A station on another machine (or a test): connect to a hub's `/link` over
/// WebSocket and return the datagram pipe for [`run_device`].
pub async fn ws_pipe(url: &str) -> anyhow::Result<(mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>)> {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;
    let (ws, _) = tokio_tungstenite::connect_async(url).await?;
    let (mut sink, mut stream) = ws.split();
    let (to_hub, mut out_rx) = mpsc::channel::<Vec<u8>>(256);
    let (in_tx, from_hub) = mpsc::channel::<Vec<u8>>(256);
    tokio::spawn(async move {
        while let Some(b) = out_rx.recv().await {
            if sink.send(Message::Binary(b.into())).await.is_err() {
                break;
            }
        }
    });
    tokio::spawn(async move {
        while let Some(Ok(m)) = stream.next().await {
            if let Message::Binary(b) = m {
                if in_tx.send(b.to_vec()).await.is_err() {
                    break;
                }
            }
        }
    });
    Ok((to_hub, from_hub))
}

// ---------------------------------------------------------------- hub side

/// What the hub knows about the station, only from what the station reported.
#[derive(Debug, Clone, Default, Serialize)]
pub struct StationView {
    /// node id of the station's link session (None = not connected)
    pub node: Option<u32>,
    pub standalone: bool,
    pub battery: Option<u8>,
    pub on_battery: bool,
    pub buffered: u32,
    pub lit: u16,
    pub open: u16,
    pub taken: u16,
    pub items: Vec<u8>,
    /// (station time ms, text) of the last report
    pub last_event: Option<(u64, String)>,
    pub events: u32,
    /// events the station buffered while it could not reach the hub, then uploaded
    pub uploaded: u32,
    /// compartments the hub asked to light: number → (text, case id)
    pub desired: BTreeMap<u8, (String, u64)>,
    /// an "AED taken" that arrived before its case was open at the hub (station time ms)
    pub pending_taken_ms: Option<u64>,
}

/// Commands queued for station sessions. Each session keeps a cursor, so
/// every connected station gets every command once, in order.
#[derive(Debug, Default)]
pub struct StationOutbox {
    pub log: VecDeque<(u64, Vec<u8>)>,
    pub next: u64,
}

fn push_cmd(state: &AppState, cmd: StationCmd) {
    let mut o = state.station_out.lock().expect("station_out");
    let i = o.next;
    o.next += 1;
    o.log.push_back((i, cmd.encode()));
    while o.log.len() > 64 {
        o.log.pop_front();
    }
}

/// Commands a session has not sent yet; moves its cursor.
pub fn cmds_since(state: &AppState, cursor: &mut u64) -> Vec<Vec<u8>> {
    let o = state.station_out.lock().expect("station_out");
    let out = o.log.iter().filter(|(i, _)| *i >= *cursor).map(|(_, b)| b.clone()).collect();
    *cursor = o.next;
    out
}

/// A station just connected: what it should show now, and the cursor to start from.
pub fn session_started(state: &AppState, id: u32) -> (Vec<Vec<u8>>, u64) {
    let next = state.station_out.lock().expect("station_out").next;
    let mut w = state.world.write().expect("world");
    w.station.node = Some(id);
    let cmds = w.station.desired.iter().map(|(n, (text, case))| StationCmd::light(*n, text.clone(), *case).encode()).collect();
    (cmds, next)
}

pub fn session_ended(state: &AppState, id: u32) {
    let mut w = state.world.write().expect("world");
    if w.station.node == Some(id) {
        w.station.node = None;
    }
}

/// Ask the station to light compartment `n` with a message.
pub fn light(state: &AppState, n: u8, text: &str, case: u64) {
    if n == 0 || n > STATION_COMPARTMENTS {
        return;
    }
    state.world.write().expect("world").station.desired.insert(n, (text.to_string(), case));
    push_cmd(state, StationCmd::light(n, text, case));
}

/// Ask the station to turn compartment `n`'s light off.
pub fn clear(state: &AppState, n: u8) {
    state.world.write().expect("world").station.desired.remove(&n);
    push_cmd(state, StationCmd::clear(n));
}

/// Demo reset: every compartment idle and full again.
pub fn restock(state: &AppState) {
    {
        let mut w = state.world.write().expect("world");
        w.station.desired.clear();
        w.station.pending_taken_ms = None;
    }
    push_cmd(state, StationCmd::all(station_op::RESTOCK));
}

/// A case opened at the hub: light 01 if an AED runner is on the way, and
/// attach an "AED taken" the station reported before the hub knew the case.
pub fn on_case_opened(state: &AppState, case: &crate::model::Case) {
    if case.helpers.iter().any(|h| h.task == "AED") {
        light(state, AED, &format!("Take the AED · {}, seat {}", case.name.split(' ').next().unwrap_or(""), case.seat), case.id);
    }
    let pending = {
        let mut w = state.world.write().expect("world");
        w.station.pending_taken_ms.take()
    };
    if let Some(t) = pending {
        if state.now().saturating_sub(t) < 10 * 60_000 {
            if let Some(title) = aed_taken(state, "station", t, None) {
                state.log("case", &case.seat, json!({"text": title, "case": case.id, "source": "station"}));
            }
        }
    }
}

/// The last no-pulse case closed: compartment 01's light goes off.
pub fn on_case_closed(state: &AppState) {
    let still = state.world.read().expect("world").cases.iter().any(|c| c.open && c.kind == "No pulse");
    if !still {
        clear(state, AED);
    }
}

/// "AED taken at the station · mm:ss" on the open no-pulse case, once.
/// `source` is "station" (item sensor) or "runner" (the runner tapped "I have it").
/// Returns the step title when a step was added.
pub fn aed_taken(state: &AppState, source: &str, at_ms: u64, who: Option<&str>) -> Option<String> {
    let title = {
        let mut w = state.world.write().expect("world");
        let open_no_pulse = w.cases.iter().any(|c| c.open && c.kind == "No pulse");
        let Some(c) = w.cases.iter_mut().find(|c| c.open && c.kind == "No pulse" && !c.steps.iter().any(|s| s.title.starts_with("AED taken"))) else {
            if source == "station" && !open_no_pulse {
                w.station.pending_taken_ms = Some(at_ms);
            }
            return None;
        };
        let at = at_ms.max(c.opened_ms);
        let title = format!("AED taken at the station · {}", mmss(at - c.opened_ms));
        let detail = match (source, who) {
            ("runner", Some(name)) => format!("{name} tapped \"I have it\" · on the way to seat {}", c.seat),
            ("runner", None) => "The runner tapped \"I have it\"".to_string(),
            _ => "Item sensor in compartment 01 · signed station report".to_string(),
        };
        c.steps.push(CaseStep { ts_ms: at, title: title.clone(), detail, tone: "info".into(), earth_ms: None });
        c.steps.sort_by_key(|s| s.ts_ms);
        c.next = "AED on its way · pads on the bare chest · follow the AED's voice.".into();
        title
    };
    state.emit(UiEvent::Tick);
    Some(title)
}

/// A station frame arrived at the hub (already deduplicated by the session's endpoint).
pub fn on_frame(state: &Shared, node: u32, f: &Frame) {
    match f.kind {
        MsgType::StationEvent => match StationEvent::decode(&f.payload) {
            Ok(ev) => on_event(state, node, &ev),
            Err(e) => tracing::warn!("station: bad event from node {node}: {e}"),
        },
        MsgType::StationStatus => {
            if let Ok(s) = StationStatus::decode(&f.payload) {
                let mut w = state.world.write().expect("world");
                let v = &mut w.station;
                v.node = Some(node);
                v.battery = Some(s.battery);
                v.on_battery = s.flags & station_flag::ON_BATTERY != 0;
                v.standalone = s.flags & station_flag::STANDALONE != 0;
                v.buffered = s.buffered;
                v.lit = s.lit;
                v.open = s.open;
                v.taken = s.taken;
                v.items = s.items;
            }
        }
        _ => {}
    }
}

fn on_event(state: &Shared, node: u32, ev: &StationEvent) {
    let now = state.now();
    let at_ms = ev.t_s * 1000;
    let d = def(ev.compartment);
    let label = d.map(|d| d.label).unwrap_or("");
    let seat_heard: String = ev.subject.chars().take(8).collect();
    let (case_id, case_seat, opened) = {
        let w = state.world.read().expect("world");
        let c = w.cases.iter().rev().find(|c| c.open && c.kind == "No pulse").or_else(|| w.open_case());
        (c.map(|c| c.id), c.map(|c| c.seat.clone()), c.map(|c| c.opened_ms))
    };
    let n = ev.compartment;
    let (kind, text) = match ev.event {
        station_ev::DOOR_OPENED => ("station", format!("Station: door {n:02} opened · {label}")),
        station_ev::DOOR_CLOSED => ("station", format!("Station: door {n:02} closed · {label}")),
        station_ev::ITEM_TAKEN => ("station", format!("Station: {} taken from compartment {n:02} · {} left", d.map(|d| d.item).unwrap_or("item"), ev.count)),
        station_ev::BUTTON => ("station_alarm", "Station: emergency button pressed".to_string()),
        station_ev::ONLINE if ev.count > 0 => ("station", format!("Station back online · {} buffered event{} uploaded", ev.count, if ev.count == 1 { "" } else { "s" })),
        station_ev::ONLINE => ("station", "Station online".to_string()),
        station_ev::ALARM_HEARD => ("station", format!("Station heard seat {seat_heard}'s wristband alarm over LoRa")),
        station_ev::SELF_LIT => ("station", format!("Station lit compartment {n:02} by itself (hub unreachable)")),
        _ => return,
    };
    // the case timeline line this report adds (also mirrored on the ground)
    let mut case_step: Option<String> = None;
    if ev.event == station_ev::DOOR_OPENED && n == AED {
        if let Some(o) = opened {
            case_step = Some(format!("Door 01 opened at the station · {}", mmss(at_ms.saturating_sub(o))));
        }
    }
    if ev.event == station_ev::ITEM_TAKEN && n == AED {
        case_step = aed_taken(state, "station", at_ms, None);
    }
    if let (Some(step), Some(seat)) = (&case_step, &case_seat) {
        if !step.starts_with("AED taken") {
            let mut w = state.world.write().expect("world");
            if let Some(c) = w.cases.iter_mut().find(|c| c.open && c.seat == *seat) {
                c.steps.push(CaseStep { ts_ms: at_ms.max(c.opened_ms), title: step.clone(), detail: "Door sensor · signed station report".into(), tone: "neutral".into(), earth_ms: None });
                c.steps.sort_by_key(|s| s.ts_ms);
            }
        }
    }
    let late_s = now.saturating_sub(at_ms) / 1000;
    let subject = if ev.event == station_ev::ALARM_HEARD && !seat_heard.is_empty() { seat_heard.clone() } else { case_seat.clone().unwrap_or_else(|| "station".into()) };
    let mut body = json!({"text": text, "event": event_name(ev.event), "compartment": n, "station_s": ev.t_s, "count": ev.count, "node": node});
    if let Some(id) = case_id {
        body["case"] = json!(id);
    }
    if let Some(s) = &case_step {
        body["case_step"] = json!(s);
    }
    if late_s >= 3 {
        body["uploaded_late_s"] = json!(late_s);
    }
    state.log(kind, &subject, body);
    {
        let mut w = state.world.write().expect("world");
        let v = &mut w.station;
        v.node = Some(node);
        v.events += 1;
        if ev.event == station_ev::ONLINE {
            v.uploaded += ev.count;
        }
        v.last_event = Some((at_ms, text.trim_start_matches("Station: ").to_string()));
        if ev.event == station_ev::ONLINE {
            v.standalone = false;
        }
    }
    match ev.event {
        station_ev::BUTTON => red_button(state, at_ms),
        station_ev::ALARM_HEARD if !seat_heard.is_empty() => alarm_heard(state, &seat_heard, at_ms),
        _ => {}
    }
    state.emit(UiEvent::Tick);
}

/// The red button: every app and dashboard gets an alarm, the crew lead an alert.
fn red_button(state: &Shared, at_ms: u64) {
    let text = "Emergency button at the medical station, corridor B";
    let mut f = Frame::new(MsgType::Alarm, state.mesh.hub_seq(), HUB_ID, Alarm { subject: "station".into(), code: alarm_code::STATION_BUTTON, text: text.into() }.encode());
    f.flags &= !FLAG_REL;
    state.mesh.broadcast(&[Role::App, Role::Dash], HUB_ID, &f.encode());
    let mut w = state.world.write().expect("world");
    let id = w.alerts.len() as u64 + 1;
    w.alerts.push(Alert { id, ts_ms: at_ms, seat: "Station".into(), who: "Medical station".into(), what: "Emergency button pressed · corridor B".into(), tone: "critical".into(), state: "Open".into() });
}

/// The station heard a wristband's LoRa alarm while the hub was down: the hub
/// opens (or dates back) the case to when the station heard it.
fn alarm_heard(state: &Shared, seat: &str, at_ms: u64) {
    if state.world.read().expect("world").passenger(seat).is_none() {
        return;
    }
    let case = eva_agent::open_emergency(state, seat, "No pulse, no movement, no answer", "no_pulse");
    let mut w = state.world.write().expect("world");
    if let Some(c) = case.and_then(|c| w.cases.iter_mut().find(|x| x.id == c.id)) {
        if at_ms < c.opened_ms {
            let shift = c.opened_ms - at_ms;
            c.opened_ms = at_ms;
            for s in c.steps.iter_mut().take(2) {
                s.ts_ms = s.ts_ms.saturating_sub(shift);
            }
        }
        c.steps.push(CaseStep { ts_ms: at_ms, title: "Station heard the wristband's LoRa alarm · lit 01 by itself".into(), detail: "Hub was down · the station ran on its own rule · report uploaded when the hub came back".into(), tone: "info".into(), earth_ms: None });
        c.steps.sort_by_key(|s| s.ts_ms);
    }
}

/// Simulated wristbands while the hub is down: a band whose own rule sees no
/// pulse + no movement + no answer broadcasts its alarm over LoRa, which the
/// station (and every other band) hears directly.
pub fn band_lora_alarms(state: &AppState) {
    let real = state.clock.real_ms();
    let mut out = Vec::new();
    {
        let mut w = state.world.write().expect("world");
        let due: Vec<(u32, String, String)> = w
            .passengers
            .iter()
            .filter(|p| eva_agent::reflex::evaluate(p).emergency)
            .filter(|p| w.band_lora.get(&p.seat).is_none_or(|t| real >= t + LORA_REPEAT_MS))
            .map(|p| (p.id, p.seat.clone(), p.name.clone()))
            .collect();
        for (id, seat, name) in due {
            w.band_lora.insert(seat.clone(), real);
            let mut f = Frame::new(MsgType::Alarm, (real / 100) as u32 | 1, id, Alarm { subject: seat, code: alarm_code::NO_PULSE, text: format!("{name} · no pulse") }.encode()).with_crc();
            f.flags &= !FLAG_REL;
            out.push(f.encode());
        }
    }
    for b in out {
        let _ = state.air.send(b);
    }
}

/// Last reported state as used by the dashboard tile.
pub fn lit_list(mask: u16) -> Vec<u8> {
    (1..=STATION_COMPARTMENTS).filter(|n| mask & bit(*n) != 0).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hub_frames(dev: &mut StationDevice, now: u64) -> Vec<Frame> {
        dev.tick(now).iter().flat_map(|d| decode_batch(d).unwrap()).collect()
    }

    #[test]
    fn light_tap_and_clear() {
        let mut d = StationDevice::new();
        d.on_cmd(&StationCmd::light(1, "Take the AED · Ana, seat 4B", 7));
        assert_eq!(d.comp(1).unwrap().state, Door::Lit);
        assert!(d.message.starts_with("Take the AED"));
        d.tap(1, 100);
        assert_eq!(d.comp(1).unwrap().state, Door::Taken);
        assert_eq!(d.comp(1).unwrap().items, 0);
        assert_eq!(d.buffered(), 2, "door opened + item taken queued for the hub");
        d.on_cmd(&StationCmd::light(1, "Take the AED · Ana, seat 4B", 7));
        assert_eq!(d.comp(1).unwrap().state, Door::Taken, "a late light command does not undo 'taken'");
        assert!(d.message.starts_with("AED taken"));
        d.on_cmd(&StationCmd::clear(1));
        assert_eq!(d.comp(1).unwrap().state, Door::Idle);
        assert_eq!(d.message, READY);
        d.on_cmd(&StationCmd::all(station_op::RESTOCK));
        assert_eq!(d.comp(1).unwrap().items, 1);
    }

    #[test]
    fn goes_standalone_and_acts_on_a_lora_alarm_by_itself() {
        let mut d = StationDevice::new();
        d.connected = true;
        d.node_id = Some(3);
        d.last_hub_ms = 1_000;
        assert!(hub_frames(&mut d, 1_100).iter().any(|f| f.kind == MsgType::StationStatus));
        assert!(!d.standalone);
        d.tick(1_000 + HUB_SILENT_MS + 1);
        assert!(d.standalone);
        let alarm = Frame::new(MsgType::Alarm, 1, 4, Alarm { subject: "4B".into(), code: alarm_code::NO_PULSE, text: String::new() }.encode()).encode();
        d.on_air(&alarm, 500);
        d.on_air(&alarm, 501);
        assert_eq!(d.comp(1).unwrap().state, Door::Lit);
        assert!(d.message.starts_with("Emergency · Take the AED"));
        assert_eq!(d.buffered(), 2, "alarm heard + self lit, once");
        d.press_button(502);
        assert_eq!(d.air_out.len(), 1, "the red button goes on the air when the hub is down");
        assert!(d.tick(9_000).is_empty(), "nothing is sent while standalone");
        assert_eq!(d.buffered(), 3);
    }
}
