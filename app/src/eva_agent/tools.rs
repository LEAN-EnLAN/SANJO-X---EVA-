//! **Tool registry.** One list of tools that Eva (the LLM) can call and that
//! people can trigger from the app (buttons, chips, voice). Each tool has a
//! JSON-schema definition sent to the model, is validated and executed on the
//! server, and every invocation, by Eva or by a person, is written to the
//! hash chain.
//!
//! **Safety:** no tool can deliver a shock, a drug or any treatment. Shocks are
//! decided only by the ship's AED from its own pads; Eva guides, records and
//! warns. `call_aed_runner` only sends a person to fetch the AED.

use crate::clock;
use crate::model::*;
use crate::state::{Shared, UiEvent};
use serde::Serialize;
use serde_json::{json, Value};

/// Who asked for the tool call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Origin {
    /// the LLM decided to call it
    Eva,
    /// a person tapped a button, a chip or said it
    User,
    /// Layer 1 reflex rules
    Reflex,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolResult {
    pub ok: bool,
    /// plain sentence for the person (and for the model)
    pub text: String,
    pub data: Value,
    /// screen the app should move to, if any
    pub navigate: Option<String>,
    pub chain_seq: Option<u64>,
}

impl ToolResult {
    fn ok(text: impl Into<String>, data: Value) -> Self {
        ToolResult { ok: true, text: text.into(), data, navigate: None, chain_seq: None }
    }
    fn err(text: impl Into<String>) -> Self {
        ToolResult { ok: false, text: text.into(), data: Value::Null, navigate: None, chain_seq: None }
    }
    fn nav(mut self, screen: &str) -> Self {
        self.navigate = Some(screen.into());
        self
    }
}

pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    /// JSON schema of the arguments
    pub parameters: fn() -> Value,
}

/// Names that must never be registered (checked by a test).
pub const FORBIDDEN: &[&str] = &["shock", "defibrillate", "deliver_shock", "give_drug", "inject", "administer", "treat"];

pub const SCREENS: &[&str] = &["home", "eva", "journal", "emergency", "storm", "plan", "records"];

pub fn registry() -> Vec<ToolDef> {
    vec![
        ToolDef { name: "get_vitals", description: "Read the latest wristband vitals (pulse, oxygen, breathing, skin temperature, EDA, motion, signal, batteries) for a passenger, with plain read-outs against their own usual range.", parameters: || json!({"type":"object","properties":{"seat":{"type":"string","description":"Seat like 4B. Defaults to the person talking."}},"additionalProperties":false}) },
        ToolDef { name: "log_mood", description: "Record the person's State of Mind: valence 1 (very unpleasant) to 5 (very pleasant), feeling labels and what is affecting them.", parameters: || json!({"type":"object","properties":{"valence":{"type":"integer","minimum":1,"maximum":5},"labels":{"type":"array","items":{"type":"string"},"maxItems":6},"affects":{"type":"array","items":{"type":"string"},"maxItems":6}},"required":["valence"],"additionalProperties":false}) },
        ToolDef { name: "add_journal_entry", description: "Save a private journal entry in the person's words. Shared with the crew lead only if shared=true.", parameters: || json!({"type":"object","properties":{"text":{"type":"string","maxLength":4000},"valence":{"type":"integer","minimum":1,"maximum":5},"labels":{"type":"array","items":{"type":"string"}},"shared":{"type":"boolean","default":false}},"required":["text"],"additionalProperties":false}) },
        ToolDef { name: "log_meal", description: "Log a meal from a text description or from a photo already described by the vision pipeline.", parameters: || json!({"type":"object","properties":{"text":{"type":"string","maxLength":500},"kind":{"type":"string","enum":["Breakfast","Lunch","Dinner","Snack"]},"kcal":{"type":"integer","minimum":0,"maximum":3000},"from_photo":{"type":"boolean"}},"required":["text"],"additionalProperties":false}) },
        ToolDef { name: "log_water", description: "Add water the person drank, in millilitres.", parameters: || json!({"type":"object","properties":{"ml":{"type":"integer","minimum":50,"maximum":2000}},"required":["ml"],"additionalProperties":false}) },
        ToolDef { name: "start_ecg_check", description: "Start an on-demand 30 second wrist ECG: the person rests one finger of each hand on the two metal pads of the wristband's screen part.", parameters: || json!({"type":"object","properties":{},"additionalProperties":false}) },
        ToolDef { name: "request_help", description: "Open an emergency case and send the nearest trained passenger to help. Use when someone may be in danger. Eva guides; it never decides a shock.", parameters: || json!({"type":"object","properties":{"seat":{"type":"string","description":"Who needs help. Defaults to the person talking."},"reason":{"type":"string","maxLength":200}},"required":["reason"],"additionalProperties":false}) },
        ToolDef { name: "call_aed_runner", description: "Send the nearest trained passenger to fetch the AED from the emergency station in corridor B. The AED, not Eva, decides whether to shock.", parameters: || json!({"type":"object","properties":{"seat":{"type":"string","description":"Who the AED is for."}},"required":["seat"],"additionalProperties":false}) },
        ToolDef { name: "daily_summary", description: "Summarise the person's day: vitals versus their usual, water, meals, sleep, mood entries and open alerts.", parameters: || json!({"type":"object","properties":{"seat":{"type":"string"}},"additionalProperties":false}) },
        ToolDef { name: "export_report", description: "Prepare an export of records: csv, json or a printable html report (print to PDF). Returns a link.", parameters: || json!({"type":"object","properties":{"format":{"type":"string","enum":["csv","json","html"]},"seat":{"type":"string","description":"Only this passenger; omit for everyone."}},"required":["format"],"additionalProperties":false}) },
        ToolDef { name: "send_to_ground", description: "Send a message or record to the ground team over the uplink. It is signed on board, queued, and acknowledged by the ground.", parameters: || json!({"type":"object","properties":{"text":{"type":"string","maxLength":1000},"urgent":{"type":"boolean","default":false}},"required":["text"],"additionalProperties":false}) },
        ToolDef { name: "set_reminder", description: "Remind the person later, in N minutes or at a ship time HH:MM.", parameters: || json!({"type":"object","properties":{"text":{"type":"string","maxLength":200},"in_min":{"type":"integer","minimum":1,"maximum":1440},"at":{"type":"string","pattern":"^[0-2][0-9]:[0-5][0-9]$"}},"required":["text"],"additionalProperties":false}) },
        ToolDef { name: "show_screen", description: "Move the person's app to a screen.", parameters: || json!({"type":"object","properties":{"screen":{"type":"string","enum":SCREENS}},"required":["screen"],"additionalProperties":false}) },
    ]
}

/// OpenAI-compatible `tools` array for chat completions.
pub fn openai_tools() -> Value {
    Value::Array(
        registry()
            .into_iter()
            .map(|t| json!({"type":"function","function":{"name":t.name,"description":t.description,"parameters":(t.parameters)()}}))
            .collect(),
    )
}

fn arg_str<'a>(a: &'a Value, k: &str) -> Option<&'a str> {
    a.get(k).and_then(|v| v.as_str()).map(str::trim).filter(|s| !s.is_empty())
}
fn arg_int(a: &Value, k: &str) -> Option<i64> {
    a.get(k).and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
}
fn arg_list(a: &Value, k: &str) -> Vec<String> {
    a.get(k).and_then(|v| v.as_array()).map(|v| v.iter().filter_map(|x| x.as_str().map(|s| s.chars().take(40).collect())).take(6).collect()).unwrap_or_default()
}

/// Validate, execute and log one tool call for `seat`.
pub fn execute(state: &Shared, seat: &str, name: &str, args: &Value, origin: Origin) -> ToolResult {
    let args = if args.is_object() { args.clone() } else { json!({}) };
    let mut res = run(state, seat, name, &args);
    // Layer 3: every invocation goes on the chain. Private text is not copied there.
    let mut logged_args = args.clone();
    if name == "add_journal_entry" && !args.get("shared").and_then(|v| v.as_bool()).unwrap_or(false) {
        logged_args["text"] = json!("(private)");
    }
    let seq = state.log("tool", seat, json!({"text": format!("{} · {}", name.replace('_', " "), res.text), "tool": name, "origin": origin, "args": logged_args, "ok": res.ok}));
    res.chain_seq = Some(seq);
    if let Some(screen) = &res.navigate {
        state.emit(UiEvent::Navigate { seat: seat.to_string(), screen: screen.clone() });
    }
    state.emit(UiEvent::Tick);
    res
}

fn run(state: &Shared, seat: &str, name: &str, a: &Value) -> ToolResult {
    let now = state.now();
    match name {
        "get_vitals" => {
            let who = arg_str(a, "seat").unwrap_or(seat);
            let w = state.world.read().expect("world");
            let Some(p) = w.passenger(who) else { return ToolResult::err(format!("No passenger in seat {who}")) };
            let v = p.vitals;
            let text = if v.hr == 0 {
                format!("{}: no pulse reading right now.", p.first_name())
            } else {
                format!("{}: pulse {} bpm ({}), oxygen {}%, skin {:.1} °C ({}), breathing about {} a minute.", p.first_name(), v.hr, p.hr_word(), v.spo2, v.skin_c(), p.skin_word(), v.resp)
            };
            ToolResult::ok(text, json!({"seat": p.seat, "vitals": v, "usual_hr": p.usual_hr, "status": p.status, "age_s": now.saturating_sub(p.last_ms) / 1000}))
        }
        "log_mood" => {
            let Some(v) = arg_int(a, "valence").filter(|v| (1..=5).contains(v)) else { return ToolResult::err("valence must be 1 to 5") };
            let mut w = state.world.write().expect("world");
            let id = w.next_id();
            let e = JournalEntry { id, ts_ms: now, seat: seat.into(), valence: v as u8, labels: arg_list(a, "labels"), affects: arg_list(a, "affects"), text: String::new(), shared: false };
            w.journal.push(e);
            ToolResult::ok(format!("Saved: {}.", mood_word(v as u8).to_lowercase()), json!({"id": id})).nav("journal")
        }
        "add_journal_entry" => {
            let Some(text) = arg_str(a, "text") else { return ToolResult::err("text is required") };
            let valence = arg_int(a, "valence").filter(|v| (1..=5).contains(v)).unwrap_or(3) as u8;
            let shared = a.get("shared").and_then(|v| v.as_bool()).unwrap_or(false);
            let mut w = state.world.write().expect("world");
            let id = w.next_id();
            w.journal.push(JournalEntry { id, ts_ms: now, seat: seat.into(), valence, labels: arg_list(a, "labels"), affects: arg_list(a, "affects"), text: text.chars().take(4000).collect(), shared });
            ToolResult::ok(if shared { "Saved and shared with the crew lead." } else { "Saved. Only you can read it." }, json!({"id": id, "shared": shared}))
        }
        "log_meal" => {
            let Some(text) = arg_str(a, "text") else { return ToolResult::err("text is required") };
            let kcal = arg_int(a, "kcal").filter(|k| (0..=3000).contains(k)).unwrap_or(500) as u32;
            let kind = arg_str(a, "kind").filter(|k| ["Breakfast", "Lunch", "Dinner", "Snack"].contains(k)).map(String::from).unwrap_or_else(|| meal_kind(now).into());
            let from_photo = a.get("from_photo").and_then(|v| v.as_bool()).unwrap_or(false);
            let mut w = state.world.write().expect("world");
            w.meals.push(Meal { ts_ms: now, seat: seat.into(), kind: kind.clone(), text: text.chars().take(500).collect(), kcal, from_photo });
            if let Some(p) = w.passenger_mut(seat) {
                p.kcal += kcal;
            }
            ToolResult::ok(format!("{kind} logged · about {kcal} kcal."), json!({"kind": kind, "kcal": kcal}))
        }
        "log_water" => {
            let Some(ml) = arg_int(a, "ml").filter(|m| (50..=2000).contains(m)) else { return ToolResult::err("ml must be 50 to 2000") };
            let mut w = state.world.write().expect("world");
            let Some(p) = w.passenger_mut(seat) else { return ToolResult::err("unknown seat") };
            p.water_ml += ml as u32;
            let total = p.water_ml;
            ToolResult::ok(format!("Added {ml} ml · {:.1} of 2 L today.", total as f32 / 1000.0), json!({"water_ml": total}))
        }
        "start_ecg_check" => {
            let mut w = state.world.write().expect("world");
            let Some(p) = w.passenger_mut(seat) else { return ToolResult::err("unknown seat") };
            p.ecg_until_ms = Some(now + 30_000);
            p.ecg = Some(EcgResult { ts_ms: now, result: "recording".into() });
            ToolResult::ok("Rest one finger of each hand on the two metal pads. Stay still for 30 seconds.", json!({"seconds": 30}))
        }
        "request_help" => {
            let who = arg_str(a, "seat").unwrap_or(seat).to_uppercase();
            let reason = arg_str(a, "reason").unwrap_or("Asked for help").to_string();
            match super::open_emergency(state, &who, &reason, "help") {
                Some(case) => {
                    let names: Vec<String> = case.helpers.iter().map(|h| format!("{} ({})", h.name, h.task)).collect();
                    ToolResult::ok(format!("Help is on the way: {}.", if names.is_empty() { "crew lead notified".into() } else { names.join(", ") }), json!({"case": case.id, "helpers": case.helpers})).nav("emergency")
                }
                None => ToolResult::err(format!("No passenger in seat {who}")),
            }
        }
        "call_aed_runner" => {
            let Some(who) = arg_str(a, "seat").map(str::to_uppercase) else { return ToolResult::err("seat is required") };
            match super::assign_aed_runner(state, &who) {
                Some(h) => ToolResult::ok(format!("{} is bringing the AED from corridor B. The AED decides shocks.", h.name), json!({"runner": h})),
                None => ToolResult::err("No trained passenger free to fetch the AED; crew lead notified."),
            }
        }
        "daily_summary" => {
            let who = arg_str(a, "seat").unwrap_or(seat);
            let w = state.world.read().expect("world");
            let Some(p) = w.passenger(who) else { return ToolResult::err("unknown seat") };
            let today = now / clock::DAY_MS;
            let meals: Vec<&Meal> = w.meals.iter().filter(|m| m.seat == p.seat && m.ts_ms / clock::DAY_MS == today).collect();
            let moods: Vec<&JournalEntry> = w.journal.iter().filter(|j| j.seat == p.seat && j.ts_ms / clock::DAY_MS == today).collect();
            let text = format!(
                "Today for {}: pulse {} ({}), water {:.1} of 2 L, {} meals (~{} kcal), sleep {} h {:02} min, {} mood check-ins, status: {}.",
                p.first_name(),
                if p.vitals.hr == 0 { "no reading".to_string() } else { format!("{} bpm", p.vitals.hr) },
                p.hr_word(),
                p.water_ml as f32 / 1000.0,
                meals.len(),
                p.kcal,
                p.sleep_min / 60,
                p.sleep_min % 60,
                moods.len(),
                p.status.label().to_lowercase()
            );
            ToolResult::ok(text, json!({"seat": p.seat, "water_ml": p.water_ml, "kcal": p.kcal, "meals": meals.len(), "moods": moods.len()}))
        }
        "export_report" => {
            let fmt = arg_str(a, "format").unwrap_or("html");
            let q = arg_str(a, "seat").map(|s| format!("?seat={s}")).unwrap_or_default();
            let url = match fmt {
                "csv" => format!("/export/records.csv{q}"),
                "json" => format!("/export/records.json{q}"),
                "html" => format!("/export/report{q}"),
                _ => return ToolResult::err("format must be csv, json or html"),
            };
            ToolResult::ok(format!("Your {fmt} export is ready: {url}"), json!({"url": url}))
        }
        "send_to_ground" => {
            let Some(text) = arg_str(a, "text") else { return ToolResult::err("text is required") };
            let urgent = a.get("urgent").and_then(|v| v.as_bool()).unwrap_or(false);
            let from = state.world.read().expect("world").passenger(seat).map(|p| p.name.clone()).unwrap_or_else(|| seat.to_string());
            let seq = state.log(if urgent { "alarm" } else { "message" }, seat, json!({"text": text, "from": from, "to": "ground"}));
            state.world.write().expect("world").messages.push(LinkMessage { ts_ms: now, from, to: "ground".into(), text: text.into(), channel: "ground".into() });
            ToolResult::ok("Signed on board and queued for the ground. You will see when they acknowledge it.", json!({"record": seq}))
        }
        "set_reminder" => {
            let Some(text) = arg_str(a, "text") else { return ToolResult::err("text is required") };
            let at = if let Some(m) = arg_int(a, "in_min").filter(|m| (1..=1440).contains(m)) {
                now + m as u64 * 60_000
            } else if let Some(hm) = arg_str(a, "at") {
                let parts: Vec<u64> = hm.split(':').filter_map(|x| x.parse().ok()).collect();
                if parts.len() != 2 || parts[0] > 23 || parts[1] > 59 {
                    return ToolResult::err("at must be HH:MM");
                }
                let day = now / clock::DAY_MS * clock::DAY_MS;
                let t = day + parts[0] * 3_600_000 + parts[1] * 60_000;
                if t <= now {
                    t + clock::DAY_MS
                } else {
                    t
                }
            } else {
                return ToolResult::err("give in_min or at");
            };
            let mut w = state.world.write().expect("world");
            let id = w.next_id();
            w.reminders.push(Reminder { id, seat: seat.into(), at_ms: at, text: text.into(), done: false });
            ToolResult::ok(format!("I'll remind you at {}: {text}.", clock::hhmm(at)), json!({"id": id, "at": clock::hhmm(at)}))
        }
        "show_screen" => {
            let Some(screen) = arg_str(a, "screen").filter(|s| SCREENS.contains(s)) else { return ToolResult::err("unknown screen") };
            ToolResult::ok(format!("Opening {screen}."), json!({"screen": screen})).nav(screen)
        }
        _ => ToolResult::err(format!("Unknown tool {name}")),
    }
}

pub fn meal_kind(now: u64) -> &'static str {
    match (now % clock::DAY_MS) / 3_600_000 {
        4..=10 => "Breakfast",
        11..=15 => "Lunch",
        16..=21 => "Dinner",
        _ => "Snack",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_treatment_tools() {
        for t in registry() {
            for f in FORBIDDEN {
                assert!(!t.name.contains(f), "tool {} looks like a treatment", t.name);
            }
        }
    }

    #[test]
    fn definitions_are_valid_openai_tools() {
        let tools = openai_tools();
        let arr = tools.as_array().unwrap();
        assert_eq!(arr.len(), 13);
        for t in arr {
            assert_eq!(t["type"], "function");
            assert_eq!(t["function"]["parameters"]["type"], "object");
        }
    }
}
