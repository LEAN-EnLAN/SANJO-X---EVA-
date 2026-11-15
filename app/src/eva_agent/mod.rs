//! **Eva agent.** Three layers (ADR-006):
//!
//! 1. [`reflex`]: deterministic rules on every tick (no pulse + no movement +
//!    no answer → emergency flow). Instant, offline, never waits for a model.
//! 2. Pipelines, each with its own module, trait and config:
//!    [`chat`] (Groq chat completions + tool calling), [`speech`] (Whisper STT,
//!    Orpheus TTS), [`vision`] (images; video as sampled frames).
//! 3. Ledger: every finding, case step, tool call and answer is written to the
//!    hash chain (`AppState::log`).
//!
//! [`tools`] is the single registry of actions shared by Eva and the UI.

pub mod chat;
pub mod config;
pub mod guard;
pub mod offline;
pub mod prompt;
pub mod reflex;
pub mod schema;
pub mod speech;
pub mod tools;
pub mod vision;

pub use config::AgentConfig;

use crate::model::*;
use crate::protocol::delta::vflag;
use crate::seed;
use crate::state::{Shared, UiEvent};
use serde_json::json;

fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

/// Called after every vitals tick (in-process sim or remote bands).
pub fn on_tick(state: &Shared) {
    if state.is_hub_down() {
        // the hub's rules are down (dev panel "Hub down"); the bands' own rule
        // still raises a no-pulse alarm, broadcast over LoRa to the station
        crate::station::band_lora_alarms(state);
        return;
    }
    let now = state.now();
    let mut emergencies: Vec<(String, String)> = Vec::new();
    let mut logs: Vec<(String, String, serde_json::Value)> = Vec::new();
    {
        let mut w = state.world.write().expect("world");
        let helper_roles: Vec<(String, String)> = w.cases.iter().filter(|c| c.open).flat_map(|c| c.helpers.iter().map(|h| (h.seat.clone(), h.task.clone()))).collect();
        let in_case: Vec<String> = w.cases.iter().filter(|c| c.open).map(|c| c.seat.clone()).collect();
        for p in w.passengers.iter_mut() {
            // ECG recordings finish after 30 s
            if let Some(until) = p.ecg_until_ms {
                if until <= now {
                    p.ecg_until_ms = None;
                    let result = if p.seat == "4A" { "review" } else { "regular" };
                    p.ecg = Some(EcgResult { ts_ms: now, result: result.into() });
                    logs.push(("ecg".into(), p.seat.clone(), json!({"text": format!("{} · wrist ECG 30 s · {}", p.name, result)})));
                }
            }
            let f = reflex::evaluate(p);
            let mut status = f.status;
            let mut group = f.group;
            if let Some((_, task)) = helper_roles.iter().find(|(s, _)| *s == p.seat) {
                if group == Group::Ok {
                    status = if task == "AED" { Status::Aed } else { Status::Helping };
                    p.note = format!("{} helper · pulse up, fine", task);
                }
            }
            if in_case.contains(&p.seat) {
                status = Status::Care;
                group = Group::Help;
            }
            if f.emergency && !in_case.contains(&p.seat) {
                emergencies.push((p.seat.clone(), "No pulse, no movement, no answer".into()));
            }
            p.status = status;
            p.group = group;
            if !f.line.is_empty() {
                p.eva_line = f.line;
            } else if group == Group::Ok {
                p.eva_line.clear();
            }
        }
        // reminders
        let due: Vec<usize> = w.reminders.iter().enumerate().filter(|(_, r)| !r.done && r.at_ms <= now).map(|(i, _)| i).collect();
        for i in due {
            w.reminders[i].done = true;
            let r = w.reminders[i].clone();
            w.chats.push(ChatMsg { ts_ms: now, seat: r.seat.clone(), from: "eva".into(), label: Some("Reminder".into()), text: r.text.clone() });
        }
        // storm ends
        if w.storm.as_ref().is_some_and(|s| s.ends_ms <= now) {
            w.storm = None;
            logs.push(("storm".into(), "ship".into(), json!({"text": "Solar storm warning ended · leave the shelter"})));
        }
    }
    for (k, s, b) in logs {
        state.log(&k, &s, b);
    }
    // the ground mirrors the ship's cases from signed records; it never opens its own
    if state.cfg.role == crate::state::NodeRole::Ground {
        return;
    }
    for (seat, reason) in emergencies {
        open_emergency(state, &seat, &reason, "no_pulse");
    }
    progress_cases(state);
}

/// Open (or return) the emergency case for `seat`: assign the nearest trained
/// passenger for CPR and an AED runner for a no-pulse case, log, alarm, notify.
pub fn open_emergency(state: &Shared, seat: &str, reason: &str, kind: &str) -> Option<Case> {
    let now = state.now();
    let case = {
        let mut w = state.world.write().expect("world");
        if let Some(c) = w.open_case_for(seat) {
            return Some(c.clone());
        }
        let p = w.passenger(seat)?.clone();
        let id = w.next_id();
        let cpr = nearest_trained(&w.passengers, p.pos, &[seat]);
        let mut helpers = Vec::new();
        if let Some(h) = cpr {
            helpers.push(Helper { seat: h.seat.clone(), name: h.name.clone(), task: "CPR".into(), distance_m: dist(h.pos, p.pos) });
        }
        if kind == "no_pulse" {
            let mut skip = vec![seat.to_string()];
            skip.extend(helpers.iter().map(|h| h.seat.clone()));
            let skip_ref: Vec<&str> = skip.iter().map(|s| s.as_str()).collect();
            if let Some(r) = nearest_trained(&w.passengers, seed::AED_STATION, &skip_ref) {
                helpers.push(Helper { seat: r.seat.clone(), name: r.name.clone(), task: "AED".into(), distance_m: dist(r.pos, seed::AED_STATION) });
            }
        }
        for h in &helpers {
            if let Some(hp) = w.passenger_mut(&h.seat) {
                hp.scenario = Scenario::Helping;
            }
        }
        let first = p.first_name().to_string();
        let helper_line = helpers.iter().map(|h| format!("{} ({}, {:.0} m) {}", h.name, h.seat, h.distance_m, if h.task == "AED" { "to get the AED from the emergency station, corridor B" } else { "to help now" })).collect::<Vec<_>>().join(" · ");
        let no_pulse = kind == "no_pulse";
        let case = Case {
            id,
            seat: p.seat.clone(),
            name: p.name.clone(),
            opened_ms: now,
            open: true,
            kind: if no_pulse { "No pulse".into() } else { "Help requested".into() },
            helpers: helpers.clone(),
            steps: vec![
                CaseStep { ts_ms: now, title: if no_pulse { format!("No pulse at {first}'s wrist") } else { format!("{first} needs help") }, detail: if no_pulse { "Wristband: no pulse, no movement, no answer · Eva sounded the alarm".into() } else { reason.into() }, tone: "critical".into(), earth_ms: None },
                CaseStep { ts_ms: now, title: format!("Eva sent {} helper{}", helpers.len(), if helpers.len() == 1 { "" } else { "s" }), detail: if helper_line.is_empty() { "No trained passenger free · crew lead notified".into() } else { helper_line }, tone: "info".into(), earth_ms: None },
            ],
            now: if no_pulse { "Start CPR. Stand clear when the AED says so.".into() } else { "Stay with them. Keep them still and talk to them.".into() },
            why: if no_pulse { format!("{first}'s heart may not be pumping. CPR and the AED give the best chance.") } else { reason.to_string() },
            how: if no_pulse { "Her band: no pulse, no movement, no answer. The AED decides shocks.".into() } else { "Asked for help in the app.".into() },
            next: if no_pulse { "CPR on the helper's band · AED coming · then the AED's voice.".into() } else { "The helper arrives; Eva rechecks the band.".into() },
        };
        w.cases.push(case.clone());
        let n = w.alerts.len() as u64 + 1;
        w.alerts.push(Alert { id: n, ts_ms: now, seat: p.seat.clone(), who: p.name.clone(), what: if no_pulse { "No pulse at the wrist".into() } else { reason.into() }, tone: "critical".into(), state: "Open".into() });
        if let Some(pm) = w.passenger_mut(seat) {
            pm.status = Status::Care;
            pm.group = Group::Help;
        }
        case
    };
    // Layer 3: alarm first (highest priority on the uplink), then the helper assignment
    state.log("alarm", seat, json!({"text": format!("{} · {}", case.name, case.steps[0].detail), "code": kind, "case": case.id}));
    state.log("case", seat, json!({"text": case.steps[1].detail, "case": case.id, "helpers": case.helpers}));
    state.set_mood(None, EvaMood::Alert);
    state.set_mood(Some(seat), EvaMood::Alert);
    for h in &case.helpers {
        state.set_mood(Some(&h.seat), EvaMood::Alert);
        state.world.write().expect("world").chats.push(ChatMsg { ts_ms: now, seat: h.seat.clone(), from: "eva".into(), label: Some("What to do now".into()), text: if h.task == "AED" { format!("Get the AED from the emergency station, corridor B, and bring it to {} in seat {}.", case.name, case.seat) } else { format!("{} needs help in seat {}. Go now. I'll guide CPR on your band.", case.name, case.seat) } });
    }
    // the medical station lights compartment 01 for the AED runner
    crate::station::on_case_opened(state, &case);
    state.emit(UiEvent::Emergency { seat: seat.to_string() });
    Some(case)
}

/// Nearest trained passenger with a pulse, not in a suit, not the crew lead
/// (who coordinates), not in `skip`.
fn nearest_trained<'a>(ps: &'a [Passenger], to: (f32, f32), skip: &[&str]) -> Option<&'a Passenger> {
    ps.iter()
        .filter(|p| p.trained && p.has_pulse() && !p.suit() && p.role.as_deref() != Some("Crew lead") && !skip.contains(&p.seat.as_str()))
        .min_by(|a, b| dist(a.pos, to).total_cmp(&dist(b.pos, to)))
}

pub fn assign_aed_runner(state: &Shared, seat: &str) -> Option<Helper> {
    let mut w = state.world.write().expect("world");
    let skip: Vec<String> = std::iter::once(seat.to_string()).chain(w.cases.iter().filter(|c| c.open).flat_map(|c| c.helpers.iter().map(|h| h.seat.clone()))).collect();
    let skip_ref: Vec<&str> = skip.iter().map(|s| s.as_str()).collect();
    let r = nearest_trained(&w.passengers, seed::AED_STATION, &skip_ref)?;
    let h = Helper { seat: r.seat.clone(), name: r.name.clone(), task: "AED".into(), distance_m: dist(r.pos, seed::AED_STATION) };
    if let Some(c) = w.cases.iter_mut().find(|c| c.open && c.seat == seat) {
        c.helpers.push(h.clone());
    }
    if let Some(p) = w.passenger_mut(&h.seat) {
        p.scenario = Scenario::Helping;
    }
    let case = w.cases.iter().find(|c| c.open && c.seat == seat).map(|c| (c.id, c.name.split(' ').next().unwrap_or("").to_string()));
    drop(w);
    if let Some((id, first)) = case {
        crate::station::light(state, crate::station::AED, &format!("Take the AED · {first}, seat {seat}"), id);
    }
    Some(h)
}

/// A helper answered "I'm going" / "I can't" on their app.
pub fn helper_answer(state: &Shared, helper_seat: &str, going: bool) {
    let now = state.now();
    let log;
    {
        let mut w = state.world.write().expect("world");
        let name = w.passenger(helper_seat).map(|p| p.name.clone()).unwrap_or_default();
        let Some(c) = w.cases.iter_mut().find(|c| c.open && c.helpers.iter().any(|h| h.seat == helper_seat)) else { return };
        let title = if going { format!("{} is on the way", name.split(' ').next().unwrap_or("")) } else { format!("{} can't help", name.split(' ').next().unwrap_or("")) };
        c.steps.push(CaseStep { ts_ms: now, title: title.clone(), detail: if going { "Answered on the app".into() } else { "Eva is asking the next nearest person".into() }, tone: "info".into(), earth_ms: None });
        if !going {
            c.helpers.retain(|h| h.seat != helper_seat);
        }
        log = Some((c.seat.clone(), title, c.id));
    }
    if let Some((seat, title, id)) = log {
        state.log("case", &seat, json!({"text": title, "case": id}));
        if !going {
            let _ = assign_aed_runner(state, &seat);
        }
    }
    // the AED runner tapped "I have it": the AED left the station
    let runner = going && state.world.read().expect("world").helper_task(helper_seat).is_some_and(|(_, h)| h.task == "AED");
    if runner {
        let name = state.world.read().expect("world").passenger(helper_seat).map(|p| p.first_name().to_string());
        if let Some(title) = crate::station::aed_taken(state, "runner", now, name.as_deref()) {
            let case = state.world.read().expect("world").helper_task(helper_seat).map(|(c, _)| (c.seat.clone(), c.id));
            if let Some((seat, id)) = case {
                state.log("case", &seat, json!({"text": title, "case": id, "source": "runner"}));
            }
        }
    }
    // the CPR helper gets a calm Eva while she counts the pushes; the AED runner stays on alert
    let cpr = going && state.world.read().expect("world").helper_task(helper_seat).is_some_and(|(_, h)| h.task == "CPR");
    state.set_mood(Some(helper_seat), if cpr { EvaMood::Calm } else if going { EvaMood::Alert } else { EvaMood::Idle });
    state.emit(UiEvent::Tick);
}

/// Demo timeline of a no-pulse case after it opens (mission time).
fn progress_cases(state: &Shared) {
    let now = state.now();
    let mut logs = Vec::new();
    {
        let mut w = state.world.write().expect("world");
        let crew_lead = w.passengers.iter().find(|p| p.role.as_deref() == Some("Crew lead")).map(|p| p.first_name().to_string()).unwrap_or_else(|| "Crew lead".into());
        for c in w.cases.iter_mut().filter(|c| c.open && c.kind == "No pulse") {
            let t = now.saturating_sub(c.opened_ms);
            let has = |c: &Case, s: &str| c.steps.iter().any(|x| x.title.contains(s));
            let cpr = c.helpers.iter().find(|h| h.task == "CPR").map(|h| h.name.split(' ').next().unwrap_or("").to_string());
            let aed = c.helpers.iter().find(|h| h.task == "AED").map(|h| h.name.split(' ').next().unwrap_or("").to_string());
            if t >= 30_000 && !has(c, "notified") {
                c.steps.push(CaseStep { ts_ms: c.opened_ms + 30_000, title: format!("Crew lead {crew_lead} notified"), detail: "Hub opened the case and started recording".into(), tone: "neutral".into(), earth_ms: None });
                logs.push((c.seat.clone(), format!("Crew lead {crew_lead} notified"), c.id));
            }
            if let Some(name) = &cpr {
                if t >= 55_000 && !has(c, "CPR started") {
                    c.steps.push(CaseStep { ts_ms: c.opened_ms + 55_000, title: format!("{name} arrived, CPR started"), detail: "Eva guiding CPR on the helper's wristband · 110 a minute".into(), tone: "info".into(), earth_ms: None });
                    c.now = "Keep CPR going. Push hard and fast in the middle of the chest.".into();
                    c.next = "AED arrives · pads on · follow the AED's voice.".into();
                    logs.push((c.seat.clone(), format!("{name} arrived, CPR started"), c.id));
                }
            }
            if aed.is_some() && t >= 120_000 && !has(c, "AED attached") {
                c.steps.push(CaseStep { ts_ms: c.opened_ms + 120_000, title: "AED attached, shock advised by the AED".into(), detail: "The AED read the heart through its own pads · Eva only guides".into(), tone: "critical".into(), earth_ms: None });
                c.now = "Stand clear when the AED says so. Then keep CPR going.".into();
                c.next = "Follow the AED's voice. Then check breathing and move her to the med bay.".into();
                logs.push((c.seat.clone(), "AED attached, shock advised by the AED".into(), c.id));
            }
        }
    }
    for (seat, text, id) in logs {
        state.log("case", &seat, json!({"text": text, "case": id}));
    }
}

/// Close the open case for `seat` (crew lead decision) and stand helpers down.
pub fn close_case(state: &Shared, seat: &str) {
    let now = state.now();
    let stood_down: Vec<String>;
    {
        let mut w = state.world.write().expect("world");
        let mut helpers = Vec::new();
        for c in w.cases.iter_mut().filter(|c| c.open && c.seat == seat) {
            c.open = false;
            c.steps.push(CaseStep { ts_ms: now, title: "Case closed by the crew lead".into(), detail: "Moved to the med bay · records kept on the chain".into(), tone: "neutral".into(), earth_ms: None });
            helpers.extend(c.helpers.iter().map(|h| h.seat.clone()));
        }
        for h in &helpers {
            if let Some(p) = w.passenger_mut(h) {
                p.scenario = Scenario::Normal;
            }
        }
        stood_down = helpers;
        if let Some(p) = w.passenger_mut(seat) {
            p.scenario = Scenario::Normal;
            p.vitals.flags &= !vflag::NO_ANSWER;
        }
        for a in w.alerts.iter_mut().filter(|a| a.seat == seat && a.state == "Open") {
            a.state = "Solved".into();
        }
    }
    state.log("case", seat, json!({"text": "Case closed by the crew lead"}));
    crate::station::on_case_closed(state);
    state.set_mood(None, EvaMood::Idle);
    for s in std::iter::once(seat.to_string()).chain(stood_down) {
        state.set_mood(Some(&s), EvaMood::Idle);
    }
}

/// A person talks to Eva (text, or text from STT). Emergencies and
/// treatment requests are answered first (never filtered), then the topic
/// guard (ADR-012) decides whether the request is in scope, then the chat
/// pipeline (online or offline) answers, and the reply goes through the
/// output check. Everything is logged; filtered requests as `guard` records.
pub async fn converse(state: &Shared, seat: &str, text: &str) -> chat::ChatReply {
    let now = state.now();
    let shown: String = text.chars().take(guard::MAX_CHAT_CHARS).collect();
    state.world.write().expect("world").chats.push(ChatMsg { ts_ms: now, seat: seat.into(), from: "me".into(), label: None, text: shown });
    state.emit(UiEvent::Chat { seat: seat.into() });
    state.set_mood(Some(seat), EvaMood::Thinking);
    let pipe = chat::pipeline(&state.agent);
    let lower = text.to_lowercase();
    // treatment / shock requests never reach a model: the answer is fixed
    let treatment = offline::asks_for_treatment(&lower);
    // a life-threatening message is never held up by the topic guard
    let urgent = guard::is_strong_emergency(&lower);
    let decision = if treatment || urgent { None } else { Some(guard::check_input(state, seat, text, "chat").await) };
    let reply = match &decision {
        _ if treatment => chat::offline_reply(state, seat, text),
        Some(d) if !d.allowed() => chat::ChatReply { text: d.verdict.reply().to_string(), model: format!("topic-guard ({})", d.verdict.category.as_str()), ..Default::default() },
        _ => {
            let asked = decision.as_ref().map(|d| d.text.clone()).unwrap_or_else(|| text.to_string());
            let history = model_history(state, seat, &asked);
            match pipe.reply(state, seat, history, asked.clone()).await {
                Ok(r) if !r.text.is_empty() => r,
                Ok(_) => chat::offline_reply(state, seat, &asked),
                Err(e) => {
                    // never leave the person without an answer: fall back to offline rules
                    tracing::warn!("chat pipeline failed, answering offline: {}", state.agent.redact(&e.to_string()));
                    let mut r = chat::offline_reply(state, seat, &asked);
                    r.model = format!("offline-rules (fallback from {})", pipe.name());
                    r
                }
            }
        }
    };
    let mut reply = reply;
    reply.text = guard::guard_output(state, seat, "chat", &reply.text);
    {
        let mut w = state.world.write().expect("world");
        w.chats.push(ChatMsg { ts_ms: state.now(), seat: seat.into(), from: "eva".into(), label: None, text: reply.text.clone() });
    }
    state.log("agent", seat, json!({"text": format!("Eva answered ({})", reply.model), "tools": reply.tools_used, "model": reply.model}));
    let concerned = { state.world.read().expect("world").open_case_for(seat).is_some() };
    state.set_mood(Some(seat), if concerned { EvaMood::Concerned } else { EvaMood::Speaking });
    state.emit(UiEvent::Chat { seat: seat.into() });
    reply
}

/// The turns the model sees: this seat's chat without the exchanges the
/// guard refused, every earlier user turn neutralised, and the current turn
/// as the guard passed it.
fn model_history(state: &Shared, seat: &str, asked: &str) -> Vec<chat::ChatTurn> {
    let w = state.world.read().expect("world");
    let msgs: Vec<&ChatMsg> = w.chats.iter().filter(|m| m.seat == seat).collect();
    let mut out: Vec<chat::ChatTurn> = Vec::new();
    let n = msgs.len();
    for (i, m) in msgs.iter().enumerate() {
        if i + 1 == n && m.from == "me" {
            break; // the current turn, added below as the guard passed it
        }
        if m.from == "me" {
            let refused = msgs.get(i + 1).is_some_and(|r| r.from != "me" && guard::GUARD_REPLIES.contains(&r.text.as_str()));
            if !refused {
                let t = guard::neutralise(&m.text).text;
                if !t.is_empty() {
                    out.push(chat::ChatTurn { role: "user".into(), content: t });
                }
            }
        } else if !guard::GUARD_REPLIES.contains(&m.text.as_str()) {
            out.push(chat::ChatTurn { role: "assistant".into(), content: m.text.clone() });
        }
    }
    out.push(chat::ChatTurn { role: "user".into(), content: asked.to_string() });
    out
}
