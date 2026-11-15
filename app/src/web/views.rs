//! View models and askama templates. Handlers build these from a snapshot of
//! the world so templates stay dumb (no locking, no logic beyond loops).
//!
//! Every fragment that live-updates over SSE has its own template, so the
//! same struct renders the first page and every later refresh.

use crate::chain::Record;
use crate::clock;
use crate::mesh::NodeView;
use crate::model::*;
use crate::state::{LinkStatus, NodeRole, Shared};
use crate::web::ui;
use askama::Template;

pub fn render<T: Template>(t: &T) -> String {
    t.render().unwrap_or_else(|e| {
        tracing::error!("template error: {e}");
        String::from("<p class=\"text-critical\">Template error</p>")
    })
}

// ---------------------------------------------------------------- common

#[derive(Debug, Clone)]
pub struct AppCommon {
    pub seat: String,
    pub name: String,
    pub first: String,
    pub initials: String,
    pub day: u64,
    pub days: u64,
    pub time: String,
    pub greeting: &'static str,
    pub mood: &'static str,
    pub art: &'static str,
    pub storm: bool,
    pub agent_mode: &'static str,
    pub role: &'static str,
}

pub fn common(state: &Shared, seat: &str) -> AppCommon {
    let now = state.now();
    let w = state.world.read().expect("world");
    let (name, first, initials) = w.passenger(seat).map(|p| (p.name.clone(), p.first_name().to_string(), p.initials())).unwrap_or_else(|| ("Guest".into(), "there".into(), "G".into()));
    let mood = w.mood(seat);
    AppCommon {
        seat: seat.to_string(),
        name,
        first,
        initials,
        day: now / clock::DAY_MS + 1,
        days: state.clock.mission_days,
        time: clock::hhmm(now),
        greeting: clock::greeting(now),
        mood: mood.name(),
        art: mood.art(),
        storm: w.storm.is_some(),
        agent_mode: state.agent.mode_label(),
        role: state.cfg.role.name(),
    }
}

// ---------------------------------------------------------------- app (astronaut PWA)

#[derive(Template)]
#[template(path = "app/page.html")]
pub struct AppPage {
    pub c: AppCommon,
    pub tab: &'static str,
    pub screen: String,
    pub seats: Vec<(String, String)>,
}

#[derive(Template)]
#[template(path = "app/home.html")]
pub struct HomeScreen {
    pub c: AppCommon,
    pub live: String,
    pub banner: String,
    pub next: Option<(String, String)>,
    /// hx-vals for the quick "+ water" button
    pub water: String,
}

#[derive(Template)]
#[template(path = "app/_live.html")]
pub struct LiveCard {
    pub p: Passenger,
    pub age: String,
    pub ecg_running: bool,
    pub all_good: bool,
}

impl LiveCard {
    pub fn build(state: &Shared, seat: &str) -> Option<Self> {
        let now = state.now();
        let w = state.world.read().expect("world");
        let p = w.passenger(seat)?.clone();
        let ecg_running = p.ecg_until_ms.is_some_and(|u| u > now);
        let all_good = p.group == Group::Ok;
        Some(LiveCard { age: clock::age(now.saturating_sub(p.last_ms)), p, ecg_running, all_good })
    }
    pub fn hr_read(&self) -> String {
        if self.p.vitals.hr == 0 {
            "Pulse · no reading".into()
        } else {
            let w = self.p.hr_word();
            format!("Pulse · {}", if w.starts_with("normal") { "calm".into() } else { w })
        }
    }
    pub fn spo2_read(&self) -> &'static str {
        match self.p.vitals.spo2 {
            0 => "Oxygen · no reading",
            1..=89 => "Oxygen · low",
            90..=94 => "Oxygen · a bit low",
            _ => "Oxygen · good",
        }
    }
    pub fn skin_read(&self) -> String {
        format!("Skin · {}", if self.p.skin_word() == "usual" { "normal" } else { self.p.skin_word() })
    }
}

pub struct CaseBanner {
    pub name: String,
    pub seat: String,
    /// helper | patient | other
    pub role: &'static str,
    pub task: String,
}

#[derive(Template)]
#[template(path = "app/_banner.html")]
pub struct Banner {
    pub case: Option<CaseBanner>,
    pub storm: bool,
}

impl Banner {
    pub fn build(state: &Shared, seat: &str) -> Self {
        let w = state.world.read().expect("world");
        let case = w.open_case().map(|c| {
            let helper = c.helpers.iter().find(|h| h.seat == seat);
            CaseBanner { name: c.name.clone(), seat: c.seat.clone(), role: if c.seat == seat { "patient" } else if helper.is_some() { "helper" } else { "other" }, task: helper.map(|h| h.task.clone()).unwrap_or_default() }
        });
        Banner { case, storm: w.storm.is_some() }
    }
}

pub struct ChatView {
    pub eva: bool,
    pub label: Option<String>,
    pub lines: Vec<String>,
    pub time: String,
}

#[derive(Template)]
#[template(path = "app/_chat.html")]
pub struct ChatList {
    pub msgs: Vec<ChatView>,
    pub first: String,
}

impl ChatList {
    pub fn build(state: &Shared, seat: &str) -> Self {
        let w = state.world.read().expect("world");
        let first = w.passenger(seat).map(|p| p.first_name().to_string()).unwrap_or_default();
        let mine: Vec<&ChatMsg> = w.chats.iter().filter(|m| m.seat == seat).collect();
        let msgs = mine[mine.len().saturating_sub(12)..]
            .iter()
            .map(|m| ChatView { eva: m.from != "me", label: m.label.clone(), lines: m.text.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect(), time: clock::hhmm(m.ts_ms) })
            .collect();
        ChatList { msgs, first }
    }
}

#[derive(Template)]
#[template(path = "app/eva.html")]
pub struct EvaScreen {
    pub c: AppCommon,
    pub live: Passenger,
    pub chat: String,
}

#[derive(Template)]
#[template(path = "app/emergency.html")]
pub struct EmergencyScreen {
    pub c: AppCommon,
    /// helper_cpr | helper_aed | cpr | patient | other | none
    pub mode: &'static str,
    pub case: Option<Case>,
    pub distance: String,
    pub direction: String,
    pub aed_helper: String,
    pub step: usize,
}

impl EmergencyScreen {
    /// `call_aed_runner` for the open case, from a bystander's phone.
    pub fn aed_vals(&self) -> String {
        let seat = self.case.as_ref().map(|c| c.seat.as_str()).unwrap_or("");
        tool_vals(&self.c.seat, "call_aed_runner", serde_json::json!({"seat": seat}))
    }
}

#[derive(Template)]
#[template(path = "app/journal.html")]
pub struct JournalScreen {
    pub c: AppCommon,
    pub entries: Vec<JournalEntry>,
    pub week: Vec<(String, Vec<u8>, bool)>,
    pub labels: Vec<&'static str>,
    pub affects: Vec<&'static str>,
    pub saved: bool,
}

pub const FEELINGS: &[&str] = &["Calm", "Anxious", "Homesick", "Grateful", "Tired", "Focused"];
pub const AFFECTS: &[&str] = &["Crew", "Sleep", "Work", "Family on Earth", "Health"];

#[derive(Template)]
#[template(path = "app/storm.html")]
pub struct StormScreen {
    pub c: AppCommon,
    pub active: bool,
    pub left: String,
    pub in_shelter: usize,
    pub total: usize,
    pub me_in: bool,
    pub pct: usize,
}

pub struct RecordView {
    pub seq: u64,
    pub time: String,
    pub kind: String,
    pub subject: String,
    pub text: String,
    pub step: u8,
    pub state: &'static str,
    pub state_tone: &'static str,
    pub hash: String,
    pub verified: bool,
}

impl RecordView {
    pub fn from(r: &Record) -> Self {
        RecordView {
            seq: r.seq,
            time: clock::hhmm(r.ts_ms),
            kind: r.kind.replace('_', " "),
            subject: r.subject.clone(),
            text: r.summary(),
            step: r.state.step(),
            state: match r.state {
                crate::chain::SyncState::Signed => "Saved",
                crate::chain::SyncState::Sent => "Sent",
                crate::chain::SyncState::Acked => "Acknowledged",
            },
            state_tone: match r.state {
                crate::chain::SyncState::Signed => "neutral",
                crate::chain::SyncState::Sent => "info",
                crate::chain::SyncState::Acked => "ok",
            },
            hash: r.short_hash().to_string(),
            verified: true,
        }
    }
    pub fn bars(&self) -> Vec<bool> {
        (0..3u8).map(|i| i <= self.step).collect()
    }
    pub fn sync_label(&self) -> &'static str {
        match self.step {
            0 => "Signed on board",
            1 => "Sent to the ground",
            _ => "Acknowledged",
        }
    }
}

#[derive(Template)]
#[template(path = "app/records.html")]
pub struct RecordsScreen {
    pub c: AppCommon,
    pub records: Vec<RecordView>,
    pub verified: bool,
    pub total: usize,
}

// ---------------------------------------------------------------- tool buttons
//
// Every button that runs one of Eva's tools posts to the same endpoint Eva's
// chips use (`POST /api/eva/tool`), so the call is validated, executed and
// written to the hash chain exactly like a call made by the model.

/// The `hx-vals` JSON for a tool button: `{"seat":…, "name":…, "args":"{…}"}`.
/// Rendered escaped inside a double-quoted attribute; the browser decodes it.
pub fn tool_vals(seat: &str, name: &str, args: serde_json::Value) -> String {
    serde_json::json!({"seat": seat, "name": name, "args": args.to_string()}).to_string()
}

pub struct ToolBtn {
    pub name: String,
    pub label: String,
    pub hint: String,
    pub icon: &'static str,
    pub vals: String,
    /// ask before running (opens a case, sends someone)
    pub confirm: String,
    /// screen to open after the call succeeds (empty = stay)
    pub then: String,
    pub danger: bool,
}

/// A tool that needs a few words from the person (reminder text, a message).
pub struct ToolForm {
    pub name: String,
    pub label: &'static str,
    pub icon: &'static str,
    pub field: &'static str,
    pub placeholder: &'static str,
    /// extra fixed args, as hidden `arg.*` inputs
    pub fixed: Vec<(&'static str, String)>,
}

fn btn(seat: &str, name: &str, label: &str, hint: &str, icon: &'static str, args: serde_json::Value) -> ToolBtn {
    ToolBtn { name: name.into(), label: label.into(), hint: hint.into(), icon, vals: tool_vals(seat, name, args), confirm: String::new(), then: String::new(), danger: false }
}

/// Plain words for a tool name we have no preset for: `call_crew_lead` → "Call crew lead".
fn tool_label(name: &str) -> String {
    let s = name.replace('_', " ");
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

#[derive(Template)]
#[template(path = "app/_tools.html")]
pub struct ToolSheet {
    pub c: AppCommon,
    pub everyday: Vec<ToolBtn>,
    pub forms: Vec<ToolForm>,
    pub emergency: Vec<ToolBtn>,
    pub total: usize,
}

impl ToolSheet {
    /// Built from the live tool registry, so a tool added there shows up here:
    /// known tools get a preset (label, icon, arguments); a new tool with no
    /// required arguments gets a plain button.
    pub fn build(state: &Shared, seat: &str) -> Self {
        use serde_json::json;
        let c = common(state, seat);
        let mut everyday = Vec::new();
        let mut forms = Vec::new();
        let mut emergency = Vec::new();
        let reg = crate::eva_agent::tools::registry();
        let total = reg.len();
        for t in reg {
            match t.name {
                "get_vitals" => everyday.push(btn(seat, t.name, "Read my band", "Pulse, oxygen, skin", "pulse", json!({}))),
                "log_water" => everyday.push(btn(seat, t.name, "+1 glass of water", "250 ml", "glass", json!({"ml": 250}))),
                "log_mood" => everyday.push(btn(seat, t.name, "I feel good", "Mood · pleasant", "smile", json!({"valence": 4, "labels": ["Calm"]}))),
                "start_ecg_check" => {
                    let mut b = btn(seat, t.name, "30 s ECG", "Both metal pads", "zap", json!({}));
                    b.then = format!("/app/ecg?seat={seat}");
                    everyday.push(b)
                }
                "daily_summary" => everyday.push(btn(seat, t.name, "Summary of my day", "Water, food, sleep", "book", json!({}))),
                "show_screen" => everyday.push(btn(seat, t.name, "Show my week", "Mood journal", "calendar", json!({"screen": "journal"}))),
                "export_report" => everyday.push(btn(seat, t.name, "Prepare my report", "Print to PDF", "file", json!({"format": "html", "seat": seat}))),
                "log_meal" => forms.push(ToolForm { name: t.name.into(), label: "Log a snack", icon: "utensils", field: "text", placeholder: "What did you eat?", fixed: vec![("kind", "Snack".into())] }),
                "set_reminder" => forms.push(ToolForm { name: t.name.into(), label: "Remind me in 30 min", icon: "bell", field: "text", placeholder: "Remind me to…", fixed: vec![("in_min", "30".into())] }),
                "send_to_ground" => forms.push(ToolForm { name: t.name.into(), label: "Message the ground", icon: "satellite", field: "text", placeholder: "Signed on board, then sent", fixed: vec![] }),
                "add_journal_entry" => forms.push(ToolForm { name: t.name.into(), label: "Quick journal note", icon: "lock", field: "text", placeholder: "Private unless you share it", fixed: vec![] }),
                "request_help" => {
                    let mut b = btn(seat, t.name, "Get help now", "Nearest trained person + crew lead", "siren", json!({"reason": "Help from the tools sheet"}));
                    b.confirm = "Open an emergency and send the nearest trained person?".into();
                    b.danger = true;
                    emergency.push(b)
                }
                "call_aed_runner" => {
                    let mut b = btn(seat, t.name, "Send for the AED", "From the station in corridor B", "zap", json!({"seat": seat}));
                    b.confirm = "Send the nearest trained person to fetch the AED?".into();
                    b.danger = true;
                    emergency.push(b)
                }
                other => {
                    // a tool added later: offer it when it needs no arguments
                    let params = (t.parameters)();
                    let needs = params.get("required").and_then(|r| r.as_array()).is_some_and(|r| !r.is_empty());
                    if !needs {
                        let mut b = btn(seat, other, &tool_label(other), "", "sparkles", json!({}));
                        b.hint = t.description.split('.').next().unwrap_or("").chars().take(48).collect();
                        everyday.push(b)
                    }
                }
            }
        }
        ToolSheet { c, everyday, forms, emergency, total }
    }
}

// ---------------------------------------------------------------- day plan

pub struct PlanItem {
    pub icon: &'static str,
    pub title: String,
    pub sub: String,
    pub done: bool,
    pub action: Option<ToolBtn>,
    pub link: Option<(String, &'static str)>,
}

#[derive(Template)]
#[template(path = "app/plan.html")]
pub struct PlanScreen {
    pub c: AppCommon,
    pub items: Vec<PlanItem>,
    pub done: usize,
    pub pct: usize,
    pub summary: ToolBtn,
    pub reminder: ToolForm,
}

impl PlanScreen {
    pub fn build(state: &Shared, seat: &str) -> Self {
        use serde_json::json;
        let c = common(state, seat);
        let now = state.now();
        let day0 = now / clock::DAY_MS * clock::DAY_MS;
        let at = |h: u64, m: u64| day0 + h * 3_600_000 + m * 60_000;
        let w = state.world.read().expect("world");
        let p = w.passenger(seat);
        let water = p.map(|p| p.water_ml).unwrap_or(0);
        let breakfast = w.meals.iter().any(|m| m.seat == seat && m.ts_ms >= day0 && m.kind == "Breakfast");
        let checkin = w.journal.iter().filter(|j| j.seat == seat && j.ts_ms >= day0).map(|j| j.ts_ms).max();
        let sleep = p.map(|p| p.sleep_min).unwrap_or(0);
        let mut items = vec![
            PlanItem { icon: "run", title: "2 h exercise".into(), sub: if now >= at(11, 20) { "Done 11:20".into() } else { "Planned 09:20".into() }, done: now >= at(11, 20), action: None, link: None },
            PlanItem {
                icon: "glass",
                title: "Drink 2 L of water".into(),
                sub: format!("{:.1} of 2 L", water as f32 / 1000.0),
                done: water >= 2000,
                action: (water < 2000).then(|| btn(seat, "log_water", "+250 ml", "", "plus", json!({"ml": 250}))),
                link: None,
            },
            PlanItem { icon: "pill", title: "Take vitamin D".into(), sub: if breakfast { "Done · breakfast".into() } else { "With breakfast".into() }, done: breakfast, action: None, link: None },
            PlanItem { icon: "moon", title: "Sleep 22:30 to 06:30".into(), sub: format!("Last night {} h {:02}", sleep / 60, sleep % 60), done: false, action: None, link: None },
            PlanItem {
                icon: "chat",
                title: "Check in with Eva".into(),
                sub: checkin.map(|t| format!("Done {}", clock::hhmm(t))).unwrap_or_else(|| "Two taps".into()),
                done: checkin.is_some(),
                action: None,
                link: checkin.is_none().then(|| (format!("/app/checkin?seat={seat}"), "Check in")),
            },
        ];
        let mut rem: Vec<&Reminder> = w.reminders.iter().filter(|r| r.seat == seat && r.at_ms >= day0 && r.at_ms < day0 + clock::DAY_MS).collect();
        rem.sort_by_key(|r| r.at_ms);
        for r in rem {
            items.push(PlanItem { icon: "bell", title: r.text.clone(), sub: if r.done { format!("Done · {}", clock::hhmm(r.at_ms)) } else { format!("Reminder · {}", clock::hhmm(r.at_ms)) }, done: r.done || r.at_ms < now, action: None, link: None });
        }
        drop(w);
        let done = items.iter().filter(|i| i.done).count();
        let pct = done * 100 / items.len().max(1);
        PlanScreen {
            c,
            items,
            done,
            pct,
            summary: btn(seat, "daily_summary", "Summary of my day", "", "book", json!({})),
            reminder: ToolForm { name: "set_reminder".into(), label: "Add a reminder", icon: "bell", field: "text", placeholder: "Stretch, call home…", fixed: vec![] },
        }
    }
    pub fn tone(&self) -> (&'static str, &'static str) {
        match self.pct {
            75.. => ("Doing well", "ok"),
            40.. => ("On track", "info"),
            _ => ("Plenty of day left", "neutral"),
        }
    }
}

// ---------------------------------------------------------------- food and water

pub struct MealRow {
    pub kind: String,
    pub text: String,
    pub kcal: Option<u32>,
    pub img: &'static str,
    pub time: String,
    pub photo: bool,
}

pub struct Macro {
    pub label: &'static str,
    pub value: u32,
    pub goal: u32,
    pub pct: u32,
}

#[derive(Template)]
#[template(path = "app/food.html")]
pub struct FoodScreen {
    pub c: AppCommon,
    pub eaten: u32,
    pub goal: u32,
    pub left: u32,
    /// ring dash offset for a circle of length 100
    pub ring: u32,
    pub macros: Vec<Macro>,
    pub meals: Vec<MealRow>,
    pub water_ml: u32,
    pub glasses: Vec<bool>,
    pub water: ToolBtn,
    pub kinds: Vec<&'static str>,
    pub kind_now: &'static str,
}

pub const KCAL_GOAL: u32 = 2300;

fn meal_img(kind: &str) -> &'static str {
    match kind {
        "Breakfast" => "meal-breakfast",
        "Lunch" => "meal-lunch",
        "Dinner" => "meal-dinner",
        _ => "meal-snack",
    }
}

impl FoodScreen {
    pub fn build(state: &Shared, seat: &str) -> Self {
        let c = common(state, seat);
        let now = state.now();
        let day0 = now / clock::DAY_MS * clock::DAY_MS;
        let w = state.world.read().expect("world");
        let (eaten, water_ml) = w.passenger(seat).map(|p| (p.kcal, p.water_ml)).unwrap_or((0, 0));
        let mut meals: Vec<MealRow> = w
            .meals
            .iter()
            .filter(|m| m.seat == seat && m.ts_ms >= day0)
            .map(|m| MealRow { kind: m.kind.clone(), text: m.text.clone(), kcal: Some(m.kcal), img: meal_img(&m.kind), time: clock::hhmm(m.ts_ms), photo: m.from_photo })
            .collect();
        drop(w);
        // the band's daily estimate can include snacks nobody itemised
        let listed: u32 = meals.iter().filter_map(|m| m.kcal).sum();
        if eaten > listed + 50 {
            meals.push(MealRow { kind: "Snacks".into(), text: "Earlier, not itemised".into(), kcal: Some(eaten - listed), img: "meal-snack", time: String::new(), photo: false });
        }
        if !meals.iter().any(|m| m.kind == "Dinner") {
            meals.push(MealRow { kind: "Dinner".into(), text: "Pasta, beans · 19:30".into(), kcal: None, img: "meal-dinner", time: String::new(), photo: false });
        }
        let goal = KCAL_GOAL;
        // estimated split of what was eaten: 50 % carbs, 18 % protein, 32 % fat
        let mk = |label, value: u32, goal: u32| Macro { label, value, goal, pct: (value * 100 / goal.max(1)).min(100) };
        let macros = vec![mk("Carbs", eaten / 2 / 4, 290), mk("Protein", eaten * 18 / 100 / 4, 95), mk("Fat", eaten * 32 / 100 / 9, 75)];
        let full = (water_ml / 250).min(8);
        FoodScreen {
            water: btn(seat, "log_water", "+1 glass", "250 ml", "plus", serde_json::json!({"ml": 250})),
            c,
            eaten,
            goal,
            left: goal.saturating_sub(eaten),
            ring: 100 - (eaten * 100 / goal).min(100),
            macros,
            meals,
            water_ml,
            glasses: (0..8).map(|i| i < full).collect(),
            kinds: vec!["Breakfast", "Lunch", "Snack", "Dinner"],
            kind_now: crate::eva_agent::tools::meal_kind(now),
        }
    }
    pub fn is_now(&self, k: impl AsRef<str>) -> bool {
        k.as_ref() == self.kind_now
    }
    pub fn glass_vals(&self) -> String {
        tool_vals(&self.c.seat, "log_water", serde_json::json!({"ml": 250}))
    }
}

// ---------------------------------------------------------------- daily check-in

pub struct CheckinAnswer {
    pub icon: &'static str,
    pub title: &'static str,
    pub next: &'static str,
    pub vals: String,
}

#[derive(Template)]
#[template(path = "app/checkin.html")]
pub struct CheckinScreen {
    pub c: AppCommon,
    pub question: &'static str,
    pub context: Vec<(String, &'static str)>,
    pub answers: Vec<CheckinAnswer>,
}

impl CheckinScreen {
    pub fn build(state: &Shared, seat: &str) -> Self {
        use serde_json::json;
        let c = common(state, seat);
        let w = state.world.read().expect("world");
        let p = w.passenger(seat);
        let watching = p.is_some_and(|p| p.group != Group::Ok);
        let mut context = Vec::new();
        if let Some(p) = p {
            if watching {
                context.push((p.status.label().to_string(), "warn"));
            }
            context.push((format!("Water {:.1} of 2 L", p.water_ml as f32 / 1000.0), "neutral"));
        }
        if let Some(j) = w.journal.iter().filter(|j| j.seat == seat).max_by_key(|j| j.ts_ms) {
            context.push((format!("Last: {}", if j.labels.is_empty() { mood_word(j.valence).to_string() } else { j.labels.join(", ") }), "info"));
        }
        drop(w);
        let answers = vec![
            CheckinAnswer { icon: "smile", title: if watching { "Yes, better" } else { "Good" }, next: "Saved in your journal", vals: tool_vals(seat, "log_mood", json!({"valence": 4, "labels": ["Better"]})) },
            CheckinAnswer { icon: "meh", title: "The same", next: "Rest 30 min · Eva asks again", vals: tool_vals(seat, "set_reminder", json!({"text": "Check in with Eva again", "in_min": 30})) },
            CheckinAnswer { icon: "frown", title: "Worse", next: "Crew lead told now", vals: tool_vals(seat, "add_journal_entry", json!({"text": "Check-in: I feel worse.", "valence": 2, "labels": ["Worse"], "shared": true})) },
        ];
        CheckinScreen { c, question: if watching { "Do you feel better?" } else { "How do you feel?" }, context, answers }
    }
}

// ---------------------------------------------------------------- ECG

#[derive(Template)]
#[template(path = "app/ecg.html")]
pub struct EcgScreen {
    pub c: AppCommon,
    /// ready | recording | result
    pub mode: &'static str,
    pub left_ms: u64,
    pub uneven: bool,
    pub at: String,
    pub hr: u16,
    pub contact_ok: bool,
    pub trace: String,
    pub start: ToolBtn,
    pub last: Option<(String, String)>,
}

impl EcgScreen {
    pub fn build(state: &Shared, seat: &str) -> Self {
        let c = common(state, seat);
        let now = state.now();
        let w = state.world.read().expect("world");
        let p = w.passenger(seat);
        let until = p.and_then(|p| p.ecg_until_ms).filter(|u| *u > now);
        let ecg = p.and_then(|p| p.ecg.clone());
        let hr = p.map(|p| p.vitals.hr).unwrap_or(0);
        let quality = p.map(|p| p.vitals.quality).unwrap_or(0);
        drop(w);
        let fresh = ecg.as_ref().filter(|e| e.result != "recording" && now.saturating_sub(e.ts_ms) < 10 * 60_000);
        let mode = if until.is_some() || ecg.as_ref().is_some_and(|e| e.result == "recording") {
            "recording"
        } else if fresh.is_some() {
            "result"
        } else {
            "ready"
        };
        let uneven = fresh.is_some_and(|e| e.result == "review");
        let mut start = btn(seat, "start_ecg_check", "Start 30 s ECG", "", "zap", serde_json::json!({}));
        start.then = format!("/app/ecg?seat={seat}");
        EcgScreen {
            c,
            mode,
            left_ms: until.map(|u| u - now).unwrap_or(1_500),
            uneven,
            at: fresh.map(|e| clock::hhmm(e.ts_ms)).unwrap_or_default(),
            hr,
            contact_ok: quality >= 70,
            trace: ui::ecg_points(5, uneven, 320.0, 64.0),
            start,
            last: ecg.filter(|e| e.result != "recording").map(|e| (clock::hhmm(e.ts_ms), if e.result == "review" { "Uneven · sent for review".to_string() } else { "Regular rhythm".to_string() })),
        }
    }
}

// ---------------------------------------------------------------- wristband detail

#[derive(Template)]
#[template(path = "app/band.html")]
pub struct BandScreen {
    pub c: AppCommon,
    pub p: Passenger,
    pub suit: bool,
    pub age: String,
    pub base_left: String,
    pub screen_left: String,
    pub signal_word: &'static str,
    pub signal_bars: Vec<bool>,
    pub read: ToolBtn,
}

/// Hours left for a battery at `pct`, given hours on a full charge.
fn hours_left(pct: u8, full_h: f32) -> String {
    let h = pct as f32 / 100.0 * full_h;
    if h < 1.0 {
        format!("~{} min left", (h * 60.0).round() as u32)
    } else {
        format!("~{} h left", h.round() as u32)
    }
}

impl BandScreen {
    pub fn build(state: &Shared, seat: &str) -> Option<Self> {
        let c = common(state, seat);
        let now = state.now();
        let p = state.world.read().expect("world").passenger(seat)?.clone();
        let q = p.vitals.quality;
        Some(BandScreen {
            suit: p.suit(),
            age: clock::age(now.saturating_sub(p.last_ms)),
            base_left: hours_left(p.vitals.base_batt, 24.0),
            screen_left: hours_left(p.vitals.screen_batt, 36.0),
            signal_word: match q {
                85.. => "Strong",
                60.. => "Good",
                30.. => "Weak · tighten the strap",
                _ => "Lost · check the band",
            },
            signal_bars: (1..=4u8).map(|i| q >= i * 25 - 10).collect(),
            read: btn(seat, "get_vitals", "Read my band now", "", "pulse", serde_json::json!({})),
            c,
            p,
        })
    }
}

// ---------------------------------------------------------------- dashboard

pub struct HelperView {
    pub first: String,
    pub initials: String,
    pub task: String,
    pub hr: u16,
}

pub struct RowView {
    pub seat: String,
    pub name: String,
    pub initials: String,
    pub status_label: &'static str,
    pub status_tone: &'static str,
    pub care: bool,
    pub hr: String,
    pub hr_unit: bool,
    pub hr_word: String,
    pub hr_tone: &'static str,
    pub pts: String,
    pub band_y: f32,
    pub band_h: f32,
    pub last_x: f32,
    pub last_y: f32,
    pub line_tone: &'static str,
    pub spo2: String,
    pub spo2_word: &'static str,
    pub spo2_tone: &'static str,
    pub skin: String,
    pub skin_word: &'static str,
    pub skin_tone: &'static str,
    pub ecg_t: String,
    pub ecg_word: String,
    pub ecg_tone: &'static str,
    pub suit: bool,
    pub base: u8,
    pub screen: u8,
    pub eva_line: String,
    pub note: String,
    pub note_tone: &'static str,
    pub helpers: Vec<HelperView>,
    pub stale: bool,
}

fn row(p: &Passenger, w: &crate::state::World, now: u64) -> RowView {
    let v = &p.vitals;
    let care = p.status == Status::Care;
    let trend_data: Vec<u16> = p.history.iter().rev().take(24).rev().map(|s| s.hr).chain(std::iter::once(v.hr)).collect();
    let (pts, band_y, band_h, last_x, last_y) = ui::trend(&trend_data, p.usual_hr, 96.0, 36.0);
    let helpers = if care {
        w.open_case_for(&p.seat)
            .map(|c| {
                c.helpers
                    .iter()
                    .map(|h| {
                        let hp = w.passenger(&h.seat);
                        HelperView { first: h.name.split(' ').next().unwrap_or("").into(), initials: hp.map(|x| x.initials()).unwrap_or_default(), task: h.task.clone(), hr: hp.map(|x| x.vitals.hr).unwrap_or(0) }
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let ecg = p.ecg.clone();
    RowView {
        seat: p.seat.clone(),
        name: p.name.clone(),
        initials: p.initials(),
        status_label: p.status.label(),
        status_tone: p.status.tone(),
        care,
        hr: if v.hr == 0 { "No pulse".into() } else { v.hr.to_string() },
        hr_unit: v.hr != 0,
        hr_word: if care && v.hr == 0 { w.open_case_for(&p.seat).map(|c| format!("since {}", clock::hhmm(c.opened_ms))).unwrap_or_else(|| "no pulse".into()) } else { p.hr_word() },
        hr_tone: if v.hr == 0 { "critical" } else { "ink" },
        pts,
        band_y,
        band_h,
        last_x,
        last_y,
        line_tone: if care { "critical" } else { "info" },
        spo2: if v.spo2 == 0 { "—".into() } else { v.spo2.to_string() },
        spo2_word: match v.spo2 {
            0 => "no reading",
            1..=89 => "low",
            _ => "normal",
        },
        spo2_tone: if v.spo2 == 0 || v.spo2 < 90 { "critical" } else { "muted" },
        skin: format!("{:.1}", v.skin_c()),
        skin_word: p.skin_word(),
        skin_tone: if p.status == Status::Warm { "warn" } else { "muted" },
        ecg_t: ecg.as_ref().map(|e| clock::hhmm(e.ts_ms)).unwrap_or_else(|| "—".into()),
        ecg_word: ecg.as_ref().map(|e| e.result.clone()).unwrap_or_else(|| "none today".into()),
        ecg_tone: if ecg.as_ref().is_some_and(|e| e.result == "review") { "warn" } else { "muted" },
        suit: p.suit(),
        base: v.base_batt,
        screen: v.screen_batt,
        eva_line: p.eva_line.clone(),
        note: p.note.clone(),
        note_tone: if matches!(p.status, Status::Helping | Status::Aed) { "info" } else { "muted" },
        helpers,
        stale: now.saturating_sub(p.last_ms) > 30_000,
    }
}

#[derive(Template)]
#[template(path = "dash/_roster.html")]
pub struct RosterCard {
    pub help: Vec<RowView>,
    pub watch: Vec<RowView>,
    pub ok: Vec<RowView>,
    pub total: usize,
    pub suit: usize,
    pub ground: bool,
}

impl RosterCard {
    pub fn build(state: &Shared) -> Self {
        let now = state.now();
        let w = state.world.read().expect("world");
        let mut help = Vec::new();
        let mut watch = Vec::new();
        let mut ok = Vec::new();
        for p in &w.passengers {
            let r = row(p, &w, now);
            match p.group {
                Group::Help => help.push(r),
                Group::Watch => watch.push(r),
                Group::Ok => ok.push(r),
            }
        }
        RosterCard { total: w.passengers.len(), suit: w.passengers.iter().filter(|p| p.suit()).count(), help, watch, ok, ground: w.role == NodeRole::Ground }
    }
}

pub struct Kpi {
    pub icon: &'static str,
    pub label: &'static str,
    pub value: String,
    pub unit: String,
    pub delta: String,
    pub delta_tone: &'static str,
    pub hint: String,
}

#[derive(Template)]
#[template(path = "dash/_kpis.html")]
pub struct KpiStrip {
    pub items: Vec<Kpi>,
}

impl KpiStrip {
    pub fn build(state: &Shared) -> Self {
        let w = state.world.read().expect("world");
        let total = w.passengers.len();
        let ok = w.passengers.iter().filter(|p| p.group == Group::Ok).count();
        let help: Vec<&Passenger> = w.passengers.iter().filter(|p| p.group == Group::Help).collect();
        let watch: Vec<&Passenger> = w.passengers.iter().filter(|p| p.group == Group::Watch).collect();
        let open = w.alerts.iter().filter(|a| a.state == "Open").count();
        let (signed, sent, acked) = state.chain.lock().expect("chain").counts();
        let l = &w.link;
        let delay = if l.rtt_ms > 0.0 { format!("{:.1}", l.rtt_ms / 2000.0) } else { "—".into() };
        let watch_words: Vec<&str> = watch.iter().map(|p| short_status(p.status)).collect();
        let ground = w.role == NodeRole::Ground;
        let mut items = vec![
            Kpi { icon: "users", label: "Passengers OK", value: ok.to_string(), unit: format!("/ {total}"), delta: help.first().map(|p| format!("{} in care", p.first_name())).unwrap_or_else(|| "Everyone fine".into()), delta_tone: if help.is_empty() { "ok" } else { "critical" }, hint: String::new() },
            Kpi { icon: "siren", label: "Open alerts", value: open.to_string(), unit: String::new(), delta: w.open_case().map(|c| c.kind.clone()).unwrap_or_else(|| "None".into()), delta_tone: if open > 0 { "critical" } else { "ok" }, hint: if w.open_case().is_some_and(|c| c.steps.iter().any(|s| s.title.contains("AED"))) { "AED on".into() } else { String::new() } },
            Kpi { icon: "eye", label: "Watching", value: watch.len().to_string(), unit: String::new(), delta: if watch_words.is_empty() { "Nothing".into() } else { dedup(&watch_words).join(" · ") }, delta_tone: if watch.is_empty() { "ok" } else { "warn" }, hint: String::new() },
        ];
        if ground {
            items.push(Kpi { icon: "satellite", label: "Ship link", value: if l.connected { "On".into() } else { "Off".into() }, unit: String::new(), delta: l.last_contact_ms.map(|t| format!("Last record {}", clock::hhmm(t))).unwrap_or_else(|| "Waiting for the ship".into()), delta_tone: if l.connected { "ok" } else { "neutral" }, hint: String::new() });
            let m = state.mirror.lock().expect("mirror");
            items.push(Kpi { icon: "file", label: "Records received", value: m.records.len().to_string(), unit: String::new(), delta: format!("{} verified", m.verified.len()), delta_tone: if m.failed.is_empty() { "ok" } else { "critical" }, hint: String::new() });
        } else {
            items.push(Kpi { icon: "satellite", label: "Ground delay", value: delay, unit: "s one way".into(), delta: if !l.connected { "No ground link".into() } else if l.up { format!("{} · in contact", l.profile_label) } else { "Contact gap · queued".into() }, delta_tone: if l.connected && l.up { "neutral" } else { "warn" }, hint: String::new() });
            items.push(Kpi { icon: "file", label: "Records", value: acked.to_string(), unit: "acknowledged".into(), delta: format!("{} queued · {} sent", signed, sent), delta_tone: if signed + sent == 0 { "ok" } else { "info" }, hint: String::new() });
        }
        KpiStrip { items }
    }
}

fn short_status(s: Status) -> &'static str {
    match s {
        Status::Ecg => "ECG",
        Status::Battery => "battery",
        Status::Warm => "skin",
        Status::LowSpo2 => "oxygen",
        Status::Signal => "signal",
        _ => "other",
    }
}
fn dedup<'a>(v: &[&'a str]) -> Vec<&'a str> {
    let mut out: Vec<&str> = Vec::new();
    for s in v {
        if !out.contains(s) {
            out.push(s);
        }
    }
    out
}

#[derive(Template)]
#[template(path = "dash/_case.html")]
pub struct CaseCard {
    pub case: Option<Case>,
    pub ground: bool,
    pub elapsed: String,
    pub mood: &'static str,
    pub art: &'static str,
    pub agent_mode: &'static str,
}

impl CaseCard {
    pub fn build(state: &Shared) -> Self {
        let now = state.now();
        let w = state.world.read().expect("world");
        let case = w.open_case().cloned().or_else(|| w.cases.last().cloned());
        let elapsed = case.as_ref().map(|c| clock::age(now.saturating_sub(c.opened_ms))).unwrap_or_default();
        CaseCard { case, ground: w.role == NodeRole::Ground, elapsed, mood: w.dash_mood.name(), art: w.dash_mood.art(), agent_mode: state.agent.mode_label() }
    }
    pub fn t(&self, ms: impl ui::Val<u64>) -> String {
        clock::hhmmss(ms.val())
    }
}

#[derive(Template)]
#[template(path = "dash/_sync.html")]
pub struct SyncCard {
    pub signed: usize,
    pub sent: usize,
    pub acked: usize,
    pub rows: Vec<RecordView>,
    pub link: LinkStatus,
    pub ground: bool,
    pub verified: usize,
    pub failed: usize,
    pub chain_ok: bool,
    pub messages: Vec<LinkMessage>,
}

impl SyncCard {
    pub fn build(state: &Shared) -> Self {
        let w = state.world.read().expect("world");
        let ground = w.role == NodeRole::Ground;
        let messages: Vec<LinkMessage> = w.messages.iter().filter(|m| m.channel == "ground").rev().take(3).cloned().collect();
        let link = w.link.clone();
        drop(w);
        if ground {
            let m = state.mirror.lock().expect("mirror");
            let rows: Vec<RecordView> = m
                .records
                .values()
                .rev()
                .take(6)
                .map(|r| {
                    let mut v = RecordView::from(r);
                    v.verified = m.is_verified(r.seq);
                    v.state = if v.verified { "Verified" } else { "Received" };
                    v.state_tone = if v.verified { "ok" } else { "info" };
                    v
                })
                .collect();
            return SyncCard { signed: 0, sent: 0, acked: m.records.len(), rows, link, ground, verified: m.verified.len(), failed: m.failed.len(), chain_ok: m.failed.is_empty(), messages };
        }
        let chain = state.chain.lock().expect("chain");
        let (signed, sent, acked) = chain.counts();
        let rows = chain.records.iter().rev().filter(|r| r.kind != "tool" && r.kind != "agent").take(6).map(RecordView::from).collect();
        SyncCard { signed, sent, acked, rows, link, ground, verified: 0, failed: 0, chain_ok: true, messages }
    }
    pub fn queued(&self) -> usize {
        self.link.depth.alarm + self.link.depth.guidance + self.link.depth.vitals + self.link.depth.logs
    }
    pub fn kb(&self, b: impl ui::Val<u64>) -> String {
        let b = b.val();
        if b > 10_240 {
            format!("{:.1} KB", b as f64 / 1024.0)
        } else {
            format!("{b} B")
        }
    }
}

#[derive(Template)]
#[template(path = "dash/_mesh.html")]
pub struct MeshCard {
    pub nodes: Vec<NodeView>,
    pub relayed: u32,
}

impl MeshCard {
    pub fn build(state: &Shared) -> Self {
        MeshCard { nodes: state.mesh.views(state.clock.real_ms()), relayed: state.mesh.relayed_frames.load(std::sync::atomic::Ordering::Relaxed) }
    }
}

#[derive(Template)]
#[template(path = "dash/_alerts.html")]
pub struct AlertsCard {
    pub alerts: Vec<Alert>,
    pub week: Vec<(String, u32, bool)>,
    pub filtered: u32,
}

impl AlertsCard {
    pub fn build(state: &Shared) -> Self {
        let w = state.world.read().expect("world");
        let today = w.alerts.len() as u32;
        let week = vec![("Mon".into(), 1, false), ("Tue".into(), 2, false), ("Wed".into(), 1, false), ("Thu".into(), 2, false), ("Fri".into(), 3, false), ("Today".into(), today, true), ("Sun".into(), 0, false)];
        AlertsCard { alerts: w.alerts.iter().rev().take(5).cloned().collect(), week, filtered: w.filtered_events }
    }
    pub fn bar_h(&self, v: impl ui::Val<u32>) -> u32 {
        let max = self.week.iter().map(|x| x.1).max().unwrap_or(1).max(4);
        (v.val() * 100 / max).max(4)
    }
    pub fn t(&self, ms: impl ui::Val<u64>) -> String {
        clock::hhmm(ms.val())
    }
}

#[derive(Template)]
#[template(path = "dash/page.html")]
pub struct DashPage {
    pub ground: bool,
    pub eyebrow: String,
    pub greeting: String,
    pub person: String,
    pub person_role: String,
    pub initials: String,
    pub kpis: String,
    pub roster: String,
    pub case: String,
    pub sync: String,
    pub mesh: String,
    pub alerts: String,
    pub mood: &'static str,
    pub art: &'static str,
    pub node: String,
    pub storm: bool,
}

// ---------------------------------------------------------------- dev panel

#[derive(Template)]
#[template(path = "dev/panel.html")]
pub struct DevPanel {
    pub seats: Vec<(String, String)>,
    pub profiles: Vec<(&'static str, &'static str, bool)>,
    pub nodes: Vec<NodeView>,
    pub sizes: Vec<(&'static str, usize, usize)>,
    pub time: String,
    pub flash: String,
    pub role: &'static str,
    pub agent_mode: &'static str,
}

// ---------------------------------------------------------------- export

#[derive(Template)]
#[template(path = "export/report.html")]
pub struct ReportPage {
    pub title: String,
    pub generated: String,
    pub node: String,
    pub chain_ok: bool,
    pub head: String,
    pub passengers: Vec<Passenger>,
    pub records: Vec<RecordView>,
    pub cases: Vec<Case>,
    pub journal: Vec<JournalEntry>,
}

impl ReportPage {
    pub fn t(&self, ms: impl ui::Val<u64>) -> String {
        clock::stamp(ms.val())
    }
}
