//! **Layer 1: reflex rules.** Deterministic, instant (microseconds), no
//! network, no model. They run on every vitals tick, before and independent
//! of the LLM, and are the only layer allowed to open an emergency.
//!
//! The emergency rule is deliberately conservative and needs all three signs:
//! **no pulse + no movement + no answer** to the wristband's "are you OK?"
//! prompt. Fewer signs only put the person on the watch list.

use crate::model::{Group, Passenger, Scenario, Status};
use crate::protocol::delta::vflag;

/// What the rules found for one passenger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub status: Status,
    pub group: Group,
    /// one plain line from Eva (empty when all good)
    pub line: String,
    /// true only for the emergency rule
    pub emergency: bool,
}

pub const BATTERY_LOW: u8 = 20;
pub const SPO2_LOW: u8 = 90;
pub const SIGNAL_POOR: u8 = 40;
pub const SKIN_WARM_C10: i16 = 10;

/// Evaluate the rules, most serious first.
pub fn evaluate(p: &Passenger) -> Finding {
    let v = &p.vitals;
    let ok = |status: Status| Finding { status, group: Group::Ok, line: String::new(), emergency: false };
    let watch = |status: Status, line: String| Finding { status, group: Group::Watch, line, emergency: false };

    // 1. emergency: no pulse + no movement + no answer
    if v.hr == 0 && v.motion <= 2 && v.has(vflag::NO_ANSWER) {
        return Finding { status: Status::Care, group: Group::Help, line: "No pulse, no movement, no answer · emergency flow started · the AED decides shocks".into(), emergency: true };
    }
    // 2. pulse lost but moving, or poor optical contact: check the band, not the heart
    if v.hr == 0 && v.motion > 2 {
        return watch(Status::Signal, format!("No pulse reading while {} moves · band may be loose · asked {} to tighten it", p.pronoun_subject(), p.pronoun));
    }
    if v.hr == 0 && !matches!(p.scenario, Scenario::CardiacArrest { .. }) {
        return watch(Status::Signal, "No pulse reading · checking the band and asking for an answer".into());
    }
    if v.hr == 0 {
        return watch(Status::Signal, "No pulse at the wrist · the band is asking \"are you OK?\"".into());
    }
    if v.quality < SIGNAL_POOR {
        return watch(Status::Signal, format!("Weak sensor signal ({}%) · readings less certain", v.quality));
    }
    // 3. oxygen
    if v.spo2 > 0 && v.spo2 < SPO2_LOW {
        return watch(Status::LowSpo2, format!("Oxygen {}% · below {}% · asked {} to sit and breathe slowly · recheck in 2 min", v.spo2, SPO2_LOW, p.pronoun));
    }
    // 4. wrist ECG flagged for review
    if p.ecg.as_ref().is_some_and(|e| e.result == "review") {
        return watch(Status::Ecg, "Pulse uneven twice this morning · wrist ECG sent to the ground for review".into());
    }
    // 5. battery
    if v.base_batt < BATTERY_LOW {
        let hours = (v.base_batt as f32 / 12.0).max(0.1);
        let left = if hours < 1.5 { "about 1 h left".to_string() } else { format!("about {hours:.0} h left") };
        return watch(Status::Battery, format!("Sensor base at {}%, {left} · spare base in the med bay, drawer B2", v.base_batt));
    }
    // 6. skin temperature
    let d = v.skin_c10 - p.usual_skin_c10;
    if d >= SKIN_WARM_C10 {
        return watch(Status::Warm, format!("Skin {:.1} °C above {} usual · asked {} to drink water and rest · recheck in 30 min", d as f32 / 10.0, p.pronoun_possessive(), p.pronoun));
    }
    // fine: show helper / suit roles in the OK group
    match p.scenario {
        Scenario::Helping => ok(Status::Helping),
        _ if v.has(vflag::SUIT) => ok(Status::Suit),
        _ => ok(Status::Ok),
    }
}

impl Passenger {
    pub fn pronoun_subject(&self) -> &'static str {
        if self.pronoun == "her" {
            "she"
        } else {
            "he"
        }
    }
    pub fn pronoun_possessive(&self) -> &'static str {
        if self.pronoun == "her" {
            "her"
        } else {
            "his"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed;

    fn ana() -> Passenger {
        seed::passengers().into_iter().find(|p| p.seat == "4B").unwrap()
    }

    #[test]
    fn emergency_needs_all_three_signs() {
        let mut p = ana();
        p.vitals.hr = 0;
        p.vitals.motion = 0;
        assert!(!evaluate(&p).emergency, "no pulse + no motion alone is not enough");
        p.vitals.flags |= vflag::NO_ANSWER;
        let f = evaluate(&p);
        assert!(f.emergency);
        assert_eq!(f.group, Group::Help);
        assert!(f.line.contains("AED decides"));
    }

    #[test]
    fn moving_without_pulse_is_a_band_problem() {
        let mut p = ana();
        p.vitals.hr = 0;
        p.vitals.motion = 80;
        p.vitals.flags |= vflag::NO_ANSWER;
        let f = evaluate(&p);
        assert!(!f.emergency);
        assert_eq!(f.status, Status::Signal);
    }

    #[test]
    fn watch_rules() {
        let mut p = ana();
        p.vitals.base_batt = 14;
        assert_eq!(evaluate(&p).status, Status::Battery);
        let mut p = ana();
        p.vitals.skin_c10 = p.usual_skin_c10 + 16;
        assert_eq!(evaluate(&p).status, Status::Warm);
        let mut p = ana();
        p.vitals.spo2 = 86;
        assert_eq!(evaluate(&p).status, Status::LowSpo2);
        assert_eq!(evaluate(&ana()).group, Group::Ok);
    }

    #[test]
    fn rules_are_fast() {
        let p = ana();
        let t = std::time::Instant::now();
        for _ in 0..10_000 {
            std::hint::black_box(evaluate(std::hint::black_box(&p)));
        }
        assert!(t.elapsed().as_millis() < 500, "10k evaluations took {:?}", t.elapsed());
    }
}
