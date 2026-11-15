//! Domain model: passengers and their wristbands, alerts, cases, journal,
//! daily logs, Eva conversations. Plain data; behaviour lives in `sim`,
//! `eva_agent` and `state`.

use crate::protocol::delta::{vflag, VitalsSample};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// What the simulator is doing to a passenger's body right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scenario {
    Normal,
    /// cardiac arrest started at this mission ms
    CardiacArrest { since_ms: u64 },
    /// doing CPR or carrying the AED: pulse up, lots of motion
    Helping,
    /// skin warmer than usual
    Warm,
    Exercise,
}

/// Triage group shown on the dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Group {
    Help,
    Watch,
    Ok,
}

/// Status badge on the roster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ok,
    Care,
    Ecg,
    Battery,
    Warm,
    LowSpo2,
    Signal,
    Helping,
    Aed,
    Suit,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Status::Ok => "All good",
            Status::Care => "In care",
            Status::Ecg => "ECG review",
            Status::Battery => "Battery low",
            Status::Warm => "Skin warm",
            Status::LowSpo2 => "Oxygen low",
            Status::Signal => "Weak signal",
            Status::Helping => "CPR helper",
            Status::Aed => "AED helper",
            Status::Suit => "Suit mode",
        }
    }
    pub fn tone(self) -> &'static str {
        match self {
            Status::Care => "critical",
            Status::Ecg | Status::Battery | Status::Warm | Status::LowSpo2 | Status::Signal => "warn",
            Status::Helping | Status::Aed => "info",
            _ => "ok",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcgResult {
    pub ts_ms: u64,
    /// "regular" | "review" | "recording"
    pub result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Passenger {
    /// stream id on Eva Link (stable per seat)
    pub id: u32,
    pub seat: String,
    pub name: String,
    pub role: Option<String>,
    /// trained in CPR / AED use: Eva can ask them to help
    pub trained: bool,
    pub pronoun: String,
    /// position on the deck plan, metres
    pub pos: (f32, f32),
    pub usual_hr: (u16, u16),
    pub usual_skin_c10: i16,
    pub vitals: VitalsSample,
    pub last_ms: u64,
    /// one point every 15 s of mission time, newest last (2 h window)
    #[serde(skip)]
    pub history: VecDeque<VitalsSample>,
    pub scenario: Scenario,
    pub status: Status,
    pub group: Group,
    /// one plain line from Eva for help / watch rows
    pub eva_line: String,
    /// one plain line for everyone else
    pub note: String,
    pub ecg: Option<EcgResult>,
    pub water_ml: u32,
    pub kcal: u32,
    pub sleep_min: u32,
    /// dev panel overrides applied on top of whatever the band reports
    #[serde(default)]
    pub forced_base_batt: Option<u8>,
    #[serde(default)]
    pub forced_suit: Option<bool>,
    /// an on-demand 30 s ECG is recording until this mission ms
    #[serde(default)]
    pub ecg_until_ms: Option<u64>,
}

impl Passenger {
    pub fn first_name(&self) -> &str {
        self.name.split(' ').next().unwrap_or(&self.name)
    }
    pub fn initials(&self) -> String {
        self.name.split(' ').filter_map(|w| w.chars().next()).take(2).collect()
    }
    pub fn suit(&self) -> bool {
        self.vitals.has(vflag::SUIT)
    }
    pub fn has_pulse(&self) -> bool {
        self.vitals.hr > 0
    }
    pub fn hr_word(&self) -> String {
        let v = &self.vitals;
        if v.hr == 0 {
            return "no pulse".into();
        }
        let (lo, hi) = self.usual_hr;
        if v.hr < lo {
            "below usual".into()
        } else if v.hr > hi + 15 {
            "high".into()
        } else if v.hr > hi {
            "a bit up".into()
        } else {
            format!("normal for {}", self.pronoun)
        }
    }
    /// one word for tight spaces: "Calm", "A bit up", "High", "Low", "No pulse"
    pub fn hr_short(&self) -> &'static str {
        let (lo, hi) = self.usual_hr;
        match self.vitals.hr {
            0 => "No pulse",
            h if h < lo => "Low",
            h if h > hi + 15 => "High",
            h if h > hi => "A bit up",
            _ => "Calm",
        }
    }
    pub fn skin_word(&self) -> &'static str {
        let d = self.vitals.skin_c10 - self.usual_skin_c10;
        if d >= 10 {
            "warmer than usual"
        } else if d <= -10 {
            "falling"
        } else {
            "usual"
        }
    }
    pub fn pulse_trend(&self) -> Vec<u16> {
        self.history.iter().map(|s| s.hr).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: u64,
    pub ts_ms: u64,
    pub seat: String,
    pub who: String,
    pub what: String,
    /// critical | warn | info
    pub tone: String,
    /// Open | Review | Watching | Solved
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseStep {
    pub ts_ms: u64,
    pub title: String,
    pub detail: String,
    /// critical | info | neutral
    pub tone: String,
    /// when the ground received it (ground role only)
    pub earth_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Helper {
    pub seat: String,
    pub name: String,
    /// "CPR" | "AED"
    pub task: String,
    pub distance_m: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    pub id: u64,
    pub seat: String,
    pub name: String,
    pub opened_ms: u64,
    pub open: bool,
    pub kind: String,
    pub helpers: Vec<Helper>,
    pub steps: Vec<CaseStep>,
    /// the four Russell et al. (2023) questions, kept current
    pub now: String,
    pub why: String,
    pub how: String,
    pub next: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: u64,
    pub ts_ms: u64,
    pub seat: String,
    /// 1 very unpleasant … 5 very pleasant
    pub valence: u8,
    pub labels: Vec<String>,
    pub affects: Vec<String>,
    pub text: String,
    /// shared with the crew lead (otherwise private)
    pub shared: bool,
}

pub fn mood_word(v: u8) -> &'static str {
    match v {
        1 => "Very unpleasant",
        2 => "Unpleasant",
        3 => "Neutral",
        4 => "Pleasant",
        _ => "Very pleasant",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meal {
    pub ts_ms: u64,
    pub seat: String,
    pub kind: String,
    pub text: String,
    pub kcal: u32,
    pub from_photo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub id: u64,
    pub seat: String,
    pub at_ms: u64,
    pub text: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMsg {
    pub ts_ms: u64,
    pub seat: String,
    /// "eva" | "me" | a person's name
    pub from: String,
    /// What to do now | Why | How Eva decided | What comes next
    pub label: Option<String>,
    pub text: String,
}

/// A text message on the mesh or to/from the ground.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkMessage {
    pub ts_ms: u64,
    pub from: String,
    pub to: String,
    pub text: String,
    /// "ground" | "mesh"
    pub channel: String,
}

/// Eva's visible state; drives the mascot animation in both views. Each state has
/// its own mascot image (`static/img/eva/eva-<name>.webp`, 18 in the Codex kit;
/// `eva-face` is only the tab-bar avatar, not a state).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum EvaMood {
    #[default]
    Idle,
    /// greeting when a person opens Eva
    Hello,
    /// recording the person's voice
    Listening,
    /// waiting for the model (chat, vision)
    Thinking,
    /// answering (TTS or a fresh reply)
    Speaking,
    /// a tool ran fine
    Happy,
    /// a tap on Eva
    Wink,
    /// someone near needs attention
    Concerned,
    /// emergency: guiding a case or a storm
    Alert,
    /// CPR guidance: steady voice, steady rhythm
    Calm,
    /// ship night (22:30 to 06:30) while nothing else is going on
    Sleeping,
    /// a daily goal was met
    Celebrate,
    /// check-in, the person said they feel worse, or a patient waiting for help
    Care,
    /// showing records or a summary
    Reading,
    /// journaling
    Writing,
    /// uplink: a record or report on its way to the ground
    Sending,
    /// the band's sensor base or screen part is below 20 %
    LowBattery,
}

impl EvaMood {
    pub const ALL: [EvaMood; 17] = [
        EvaMood::Idle,
        EvaMood::Hello,
        EvaMood::Listening,
        EvaMood::Thinking,
        EvaMood::Speaking,
        EvaMood::Happy,
        EvaMood::Wink,
        EvaMood::Concerned,
        EvaMood::Alert,
        EvaMood::Calm,
        EvaMood::Sleeping,
        EvaMood::Celebrate,
        EvaMood::Care,
        EvaMood::Reading,
        EvaMood::Writing,
        EvaMood::Sending,
        EvaMood::LowBattery,
    ];
    pub fn name(self) -> &'static str {
        match self {
            EvaMood::Idle => "idle",
            EvaMood::Hello => "hello",
            EvaMood::Listening => "listening",
            EvaMood::Thinking => "thinking",
            EvaMood::Speaking => "speaking",
            EvaMood::Happy => "happy",
            EvaMood::Wink => "wink",
            EvaMood::Concerned => "concerned",
            EvaMood::Alert => "alert",
            EvaMood::Calm => "calm",
            EvaMood::Sleeping => "sleeping",
            EvaMood::Celebrate => "celebrate",
            EvaMood::Care => "care",
            EvaMood::Reading => "reading",
            EvaMood::Writing => "writing",
            EvaMood::Sending => "sending",
            EvaMood::LowBattery => "low-battery",
        }
    }
    /// mascot image for the state (`eva-<art>.webp`): every state has its own
    pub fn art(self) -> &'static str {
        self.name()
    }
    /// short-lived states fall back to idle after a moment (see `state::pulse_mood`)
    pub fn transient(self) -> bool {
        matches!(self, EvaMood::Hello | EvaMood::Happy | EvaMood::Wink | EvaMood::Celebrate | EvaMood::Sending | EvaMood::Reading)
    }
}
