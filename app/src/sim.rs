//! Simulated WearMeds wristbands.
//!
//! Two halves, so the same physics works in-process and over the network:
//! * [`baseline`] is what a healthy body + band produce: a random walk around
//!   the person's usual values, battery drain, signal quality. It runs in the
//!   simulator (in-process with `eva serve --sim`, or remote with `eva sim`).
//! * [`overlay`] applies the current [`Scenario`] (cardiac arrest, helping,
//!   warm skin…) and dev-panel overrides. It always runs on the server, so the
//!   dev panel controls scenarios even when the bands are a separate process.

use crate::model::{Passenger, Scenario};
use crate::protocol::delta::{vflag, VitalsSample};
use crate::protocol::frame::MsgType;
use crate::protocol::payload::{Hello, Role};
use crate::protocol::profile::Rng;
use crate::protocol::Endpoint;
use crate::seed;
use crate::state::{Shared, UiEvent};

/// One step of a healthy body for `dt_ms` of mission time.
pub fn baseline(prev: &VitalsSample, usual_hr: (u16, u16), usual_skin_c10: i16, dt_ms: u64, rng: &mut Rng) -> VitalsSample {
    let mut s = *prev;
    let mid = ((usual_hr.0 + usual_hr.1) / 2) as f64;
    let hr = if prev.hr == 0 { mid } else { prev.hr as f64 };
    // mean-reverting random walk
    let next = hr + (mid - hr) * 0.08 + (rng.f64() - 0.5) * 3.0;
    s.hr = next.round().clamp(40.0, 190.0) as u16;
    s.spo2 = (if prev.spo2 == 0 { 98 } else { prev.spo2 } as f64 + (98.0 - prev.spo2.max(90) as f64) * 0.2 + (rng.f64() - 0.5) * 0.9).round().clamp(93.0, 100.0) as u8;
    s.resp = (14.0 + (rng.f64() - 0.5) * 3.0).round() as u8;
    let skin = prev.skin_c10 as f64 + (usual_skin_c10 as f64 - prev.skin_c10 as f64) * 0.05 + (rng.f64() - 0.5) * 0.8;
    s.skin_c10 = skin.round() as i16;
    s.eda_c = (prev.eda_c as f64 + (rng.f64() - 0.5) * 8.0).clamp(80.0, 900.0) as u16;
    s.motion = ((prev.motion as f64) * 0.7 + rng.f64() * 12.0).round().clamp(0.0, 255.0) as u8;
    s.quality = (if s.has(vflag::SUIT) { 80.0 } else { 94.0 } + (rng.f64() - 0.5) * 6.0).round().clamp(0.0, 100.0) as u8;
    // ~1 % per 12 min of mission time for the sensor base, per 18 min for the
    // screen part; probabilistic so 1 s ticks and fast-forwards both work
    let drop = |v: u8, per_ms: f64, rng: &mut Rng| -> u8 {
        let expected = dt_ms as f64 / per_ms;
        let whole = expected.floor();
        let extra = if rng.f64() < expected - whole { 1.0 } else { 0.0 };
        (v as f64 - whole - extra).max(0.0) as u8
    };
    s.base_batt = drop(prev.base_batt, 720_000.0, rng);
    s.screen_batt = drop(prev.screen_batt, 1_080_000.0, rng);
    s.flags |= vflag::LORA;
    s.flags &= !vflag::NO_ANSWER;
    s
}

/// Apply the scenario and overrides. `now_ms` is mission time.
pub fn overlay(p: &Passenger, mut s: VitalsSample, now_ms: u64) -> VitalsSample {
    match p.scenario {
        Scenario::Normal => {}
        Scenario::Warm => s.skin_c10 = p.usual_skin_c10 + 16,
        Scenario::Exercise => {
            s.hr = s.hr.max(p.usual_hr.1 + 25);
            s.motion = 160;
        }
        Scenario::Helping => {
            s.hr = (p.usual_hr.1 as f64 * 1.42 + (now_ms / 1000 % 5) as f64) as u16;
            s.motion = 210;
            s.quality = 70;
        }
        Scenario::CardiacArrest { since_ms } => {
            let t = now_ms.saturating_sub(since_ms);
            if t < 12_000 {
                // fast and uneven at the wrist before the pulse is lost
                s.hr = 112 + ((t / 1000) * 37 % 70) as u16;
                s.motion = 40;
            } else {
                s.hr = 0;
                s.spo2 = 0;
                s.motion = 0;
                s.resp = 0;
                s.skin_c10 = p.usual_skin_c10 - ((t / 60_000) as i16 * 3).min(12) - 2;
                // the band asked "are you OK?" and got no answer (button or voice) within 8 s
                if t >= 20_000 {
                    s.flags |= vflag::NO_ANSWER;
                }
            }
        }
    }
    if let Some(b) = p.forced_base_batt {
        s.base_batt = b;
    }
    match p.forced_suit {
        Some(true) => s.flags |= vflag::SUIT,
        Some(false) => s.flags &= !vflag::SUIT,
        None => {}
    }
    if p.ecg_until_ms.is_some_and(|u| u > now_ms) {
        s.flags |= vflag::ECG;
    } else {
        s.flags &= !vflag::ECG;
    }
    s
}

/// Store a new sample on a passenger (history keeps one point per 15 s).
pub fn ingest(p: &mut Passenger, raw: VitalsSample, now_ms: u64) {
    let prev_ms = p.last_ms;
    let s = overlay(p, raw, now_ms);
    p.vitals = s;
    p.last_ms = now_ms;
    if now_ms / 15_000 != prev_ms / 15_000 || p.history.is_empty() {
        p.history.push_back(s);
        while p.history.len() > 480 {
            p.history.pop_front();
        }
    }
}

/// In-process simulator: every `interval_ms` of real time, step every band.
pub async fn run_in_process(state: Shared, interval_ms: u64) {
    let mut rng = Rng::new(0xE7A_2026);
    let mut tick = tokio::time::interval(std::time::Duration::from_millis(interval_ms));
    let mut last = state.now();
    loop {
        tick.tick().await;
        let now = state.now();
        let dt = now.saturating_sub(last).max(1);
        last = now;
        {
            let mut w = state.world.write().expect("world");
            for p in w.passengers.iter_mut() {
                let raw = baseline(&strip_overlay(&p.vitals), p.usual_hr, p.usual_skin_c10, dt, &mut rng);
                ingest(p, raw, now);
            }
        }
        crate::eva_agent::on_tick(&state);
        state.emit(UiEvent::Tick);
    }
}

/// The baseline must not inherit an overlay (e.g. hr 0 from an arrest).
fn strip_overlay(v: &VitalsSample) -> VitalsSample {
    let mut s = *v;
    s.flags &= !(vflag::NO_ANSWER | vflag::ECG);
    s
}

/// Server-side tick when the bands are remote: run Eva's rules and refresh pages.
pub async fn run_server_tick(state: Shared, interval_ms: u64) {
    let mut tick = tokio::time::interval(std::time::Duration::from_millis(interval_ms));
    loop {
        tick.tick().await;
        crate::eva_agent::on_tick(&state);
        state.emit(UiEvent::Tick);
    }
}

/// `eva sim`: N simulated wristbands streaming delta-encoded vitals to a
/// server's `/link` over one WebSocket (one "band gateway" node).
pub async fn run_remote(url: String, n: usize, interval_ms: u64) -> anyhow::Result<()> {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;
    let roster = seed::passengers();
    let n = n.clamp(1, 64);
    let mut bands: Vec<(u32, VitalsSample, (u16, u16), i16)> = (0..n)
        .map(|i| {
            let p = &roster[i % roster.len()];
            (if i < roster.len() { p.id } else { (i + 1) as u32 }, p.vitals, p.usual_hr, p.usual_skin_c10)
        })
        .collect();
    let mut rng = Rng::new(42);
    let started = std::time::Instant::now();
    loop {
        tracing::info!("sim: connecting {n} bands to {url}");
        let (ws, _) = match tokio_tungstenite::connect_async(&url).await {
            Ok(x) => x,
            Err(e) => {
                tracing::warn!("sim: connect failed ({e}); retrying in 2 s");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }
        };
        let (mut tx, mut rx) = ws.split();
        let mut ep = Endpoint::new(0);
        ep.vitals_ttl_ms = interval_ms * 3;
        let hello = Hello::new(Role::Band, format!("sim gateway · {n} bands"), started.elapsed().as_millis() as u64);
        let f = ep.queue.control(MsgType::Hello, hello.encode());
        if tx.send(Message::Binary(f.encode().into())).await.is_err() {
            continue;
        }
        let mut tick = tokio::time::interval(std::time::Duration::from_millis(interval_ms));
        let mut sent = 0u64;
        loop {
            tokio::select! {
                _ = tick.tick() => {
                    let now = started.elapsed().as_millis() as u64;
                    let ts = now / 1000;
                    for b in bands.iter_mut() {
                        b.1 = baseline(&b.1, b.2, b.3, interval_ms, &mut rng);
                        ep.send_vitals(now, b.0, ts, &b.1);
                    }
                    let mut broken = false;
                    for d in ep.outgoing(now, usize::MAX, 16_000) {
                        sent += d.len() as u64;
                        if tx.send(Message::Binary(d.into())).await.is_err() { broken = true; break; }
                    }
                    if broken { break; }
                    if ts % 10 == 0 && now % 1000 < interval_ms {
                        tracing::info!("sim: {n} bands · {sent} bytes sent so far");
                    }
                }
                msg = rx.next() => match msg {
                    Some(Ok(Message::Binary(b))) => {
                        let now = started.elapsed().as_millis() as u64;
                        ep.incoming(now, &b);
                    }
                    Some(Ok(_)) => {}
                    _ => break,
                }
            }
        }
        tracing::warn!("sim: link closed; reconnecting");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
