//! Demo data matching the wireframes: day 96 of 212, fifteen people on board,
//! Ana Ruiz in seat 4B (the cardiac case), Leo and Tomás as helpers, Priya's
//! wrist ECG under review, Diego's low sensor battery, Kwame running warm,
//! three people in a suit drill.

use crate::model::*;
use crate::protocol::delta::{vflag, VitalsSample};
use std::collections::VecDeque;

/// Where the emergency station with the AED is (corridor B), deck metres.
pub const AED_STATION: (f32, f32) = (4.0, 3.0);
pub const SHELTER: &str = "Deck B storm shelter";

pub const DEMO_DAY: u64 = 96;
pub const DEMO_HH: u64 = 13;
pub const DEMO_MM: u64 = 58;
pub const MISSION_DAYS: u64 = 212;

struct P {
    seat: &'static str,
    name: &'static str,
    role: Option<&'static str>,
    trained: bool,
    pronoun: &'static str,
    hr: u16,
    usual: (u16, u16),
    spo2: u8,
    skin: i16,
    base: u8,
    screen: u8,
    suit: bool,
    note: &'static str,
}

const ROSTER: &[P] = &[
    P { seat: "4B", name: "Ana Ruiz", role: None, trained: false, pronoun: "her", hr: 70, usual: (58, 78), spo2: 98, skin: 341, base: 66, screen: 71, suit: false, note: "Usual for her" },
    P { seat: "4A", name: "Priya Nair", role: None, trained: false, pronoun: "her", hr: 70, usual: (60, 80), spo2: 98, skin: 340, base: 58, screen: 62, suit: false, note: "Usual for her" },
    P { seat: "5A", name: "Diego Luna", role: None, trained: false, pronoun: "him", hr: 76, usual: (62, 84), spo2: 97, skin: 342, base: 14, screen: 52, suit: false, note: "Usual for him" },
    P { seat: "6A", name: "Kwame Mensah", role: None, trained: false, pronoun: "him", hr: 79, usual: (60, 82), spo2: 97, skin: 340, base: 61, screen: 70, suit: false, note: "Usual for him" },
    P { seat: "1A", name: "Mara Okoye", role: Some("Crew lead"), trained: true, pronoun: "her", hr: 68, usual: (56, 80), spo2: 98, skin: 338, base: 80, screen: 77, suit: false, note: "Crew lead · usual for her" },
    P { seat: "1B", name: "Tomás Rey", role: Some("Flight engineer"), trained: true, pronoun: "him", hr: 72, usual: (60, 82), spo2: 97, skin: 339, base: 74, screen: 69, suit: false, note: "Usual for him" },
    P { seat: "3B", name: "Leo Brandt", role: None, trained: true, pronoun: "him", hr: 66, usual: (56, 76), spo2: 98, skin: 337, base: 70, screen: 66, suit: false, note: "Usual for him" },
    P { seat: "2A", name: "Yuki Sato", role: None, trained: true, pronoun: "her", hr: 88, usual: (60, 82), spo2: 98, skin: 343, base: 62, screen: 58, suit: true, note: "Suit drill until 14:30" },
    P { seat: "2B", name: "Omar Haddad", role: None, trained: true, pronoun: "him", hr: 91, usual: (60, 84), spo2: 97, skin: 344, base: 57, screen: 60, suit: true, note: "Suit drill until 14:30" },
    P { seat: "6B", name: "Sofia Rossi", role: None, trained: false, pronoun: "her", hr: 84, usual: (60, 82), spo2: 98, skin: 342, base: 65, screen: 63, suit: true, note: "Suit drill until 14:30" },
    P { seat: "3A", name: "Lena Vogel", role: None, trained: true, pronoun: "her", hr: 62, usual: (54, 74), spo2: 99, skin: 336, base: 77, screen: 74, suit: false, note: "Usual for her" },
    P { seat: "5B", name: "Chloé Martin", role: None, trained: false, pronoun: "her", hr: 66, usual: (56, 78), spo2: 98, skin: 339, base: 72, screen: 70, suit: false, note: "Usual for her" },
    P { seat: "7A", name: "Ivan Petrov", role: None, trained: false, pronoun: "him", hr: 64, usual: (54, 76), spo2: 98, skin: 338, base: 68, screen: 71, suit: false, note: "Usual for him" },
    P { seat: "7B", name: "Hana Kim", role: None, trained: false, pronoun: "her", hr: 69, usual: (58, 80), spo2: 99, skin: 340, base: 75, screen: 73, suit: false, note: "Usual for her" },
    P { seat: "8A", name: "Noah Clark", role: None, trained: false, pronoun: "him", hr: 71, usual: (60, 82), spo2: 97, skin: 341, base: 69, screen: 67, suit: false, note: "Usual for him" },
];

/// Deck position of a seat: row number every 3 m along the cabin, A/B 1 m apart.
pub fn seat_pos(seat: &str) -> (f32, f32) {
    let n: f32 = seat.trim_end_matches(|c: char| c.is_ascii_alphabetic()).parse().unwrap_or(0.0);
    let side = if seat.ends_with('B') { 1.0 } else { 0.0 };
    (n * 3.0, side)
}

pub fn passengers() -> Vec<Passenger> {
    ROSTER
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let mut flags = vflag::LORA;
            if p.suit {
                flags |= vflag::SUIT;
            }
            let v = VitalsSample { hr: p.hr, spo2: p.spo2, resp: 14, skin_c10: p.skin, eda_c: 220, motion: 10, quality: if p.suit { 82 } else { 95 }, base_batt: p.base, screen_batt: p.screen, flags };
            let history: VecDeque<VitalsSample> = (0..32)
                .map(|k| {
                    let mut s = v;
                    s.hr = (p.hr as i32 + ((k * 7 + i * 3) % 9) as i32 - 4).max(40) as u16;
                    s
                })
                .collect();
            Passenger {
                id: i as u32 + 1,
                seat: p.seat.into(),
                name: p.name.into(),
                role: p.role.map(Into::into),
                trained: p.trained,
                pronoun: p.pronoun.into(),
                pos: seat_pos(p.seat),
                usual_hr: p.usual,
                usual_skin_c10: p.skin,
                vitals: v,
                last_ms: 0,
                history,
                scenario: if p.seat == "6A" { Scenario::Warm } else { Scenario::Normal },
                status: Status::Ok,
                group: Group::Ok,
                eva_line: String::new(),
                note: p.note.into(),
                ecg: Some(EcgResult { ts_ms: 0, result: if p.seat == "4A" { "review".into() } else { "regular".into() } }),
                water_ml: if p.seat == "4B" { 1200 } else { 900 + (i as u32 * 70) % 700 },
                kcal: 1640,
                sleep_min: 430,
                forced_base_batt: None,
                forced_suit: None,
                ecg_until_ms: None,
            }
        })
        .collect()
}

pub fn journal(day_ms: u64) -> Vec<JournalEntry> {
    let h = |hh: u64, mm: u64| day_ms + hh * 3_600_000 + mm * 60_000;
    vec![
        JournalEntry { id: 1, ts_ms: h(19, 5) - 3 * crate::clock::DAY_MS, seat: "4B".into(), valence: 5, labels: vec!["Grateful".into()], affects: vec!["Crew".into()], text: "Movie night with the crew.".into(), shared: false },
        JournalEntry { id: 2, ts_ms: h(22, 40) - 2 * crate::clock::DAY_MS, seat: "4B".into(), valence: 2, labels: vec!["Tired".into(), "Anxious".into()], affects: vec!["Sleep".into(), "Work".into()], text: "5 h sleep. Long shift.".into(), shared: true },
        JournalEntry { id: 3, ts_ms: h(21, 10) - crate::clock::DAY_MS, seat: "4B".into(), valence: 4, labels: vec!["Calm".into(), "Homesick".into()], affects: vec!["Family on Earth".into()], text: "Mum's message came through.".into(), shared: false },
    ]
}

pub fn alerts(day_ms: u64) -> Vec<Alert> {
    let h = |hh: u64, mm: u64| day_ms + hh * 3_600_000 + mm * 60_000;
    vec![
        Alert { id: 1, ts_ms: h(11, 15), seat: "5A".into(), who: "Diego Luna".into(), what: "Sensor base battery low".into(), tone: "warn".into(), state: "Watching".into() },
        Alert { id: 2, ts_ms: h(12, 40), seat: "4A".into(), who: "Priya Nair".into(), what: "Wrist ECG uneven".into(), tone: "warn".into(), state: "Review".into() },
    ]
}

pub fn meals(day_ms: u64) -> Vec<Meal> {
    let h = |hh: u64, mm: u64| day_ms + hh * 3_600_000 + mm * 60_000;
    vec![
        Meal { ts_ms: h(7, 40), seat: "4B".into(), kind: "Breakfast".into(), text: "Oats, dried fruit, coffee".into(), kcal: 520, from_photo: false },
        Meal { ts_ms: h(12, 30), seat: "4B".into(), kind: "Lunch".into(), text: "Rice, lentils, greens".into(), kcal: 760, from_photo: true },
    ]
}

/// Records already on the chain at the start of the demo (acknowledged by Earth).
pub fn chain_records(day_ms: u64) -> Vec<(u64, &'static str, &'static str, serde_json::Value)> {
    let h = |hh: u64, mm: u64| day_ms + hh * 3_600_000 + mm * 60_000;
    vec![
        (h(8, 0), "vitals_summary", "ship", serde_json::json!({"text": "Daily vitals · 15 passengers"})),
        (h(9, 30), "log", "ship", serde_json::json!({"text": "Cabin dose log · morning"})),
        (h(11, 15), "alert", "5A", serde_json::json!({"text": "Diego Luna · sensor base battery low"})),
        (h(12, 40), "ecg", "4A", serde_json::json!({"text": "Priya Nair · wrist ECG, irregular · review asked"})),
    ]
}
