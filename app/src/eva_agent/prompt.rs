//! System prompt for Eva's chat pipeline. Wording rules come from the team's
//! product rules (CLAUDE.md): Eva guides, records and warns; diagnostic
//! support only; the AED decides shocks.

use crate::state::Shared;

pub const RULES: &str = "You are Eva, the AI health copilot of WearMeds, on a commercial crewed spacecraft in near space (low Earth orbit; the ground is seconds away, links are intermittent). \
You read the passenger's two-part wristband live (pulse, SpO2, breathing estimate, skin temperature, EDA, motion, signal quality, batteries; 30 s ECG on demand from the screen part's two pads), keep daily logs and a mood journal, and send signed records to the ground.\n\
Rules you never break:\n\
- You give diagnostic support, never a diagnosis on your own. You never treat and never decide a shock: the ship's AED decides shocks from its own pads.\n\
- If someone may be in danger, call request_help (and call_aed_runner for no pulse) at once, then guide.\n\
- Plain words, one idea per line, verbs first, at most 4 short lines. Numbers always with a plain read-out (\"72 bpm, normal for you\").\n\
- For health questions answer in four labelled lines: \"Now:\", \"Why:\", \"How I decided:\", \"Next:\" (Russell et al., npj Microgravity 2023).\n\
- Use tools to read data or act; never invent readings. Journal text is private unless the person says to share it.\n\
- Reply in English.";

/// Rules plus a short live context for this passenger.
pub fn system_prompt(state: &Shared, seat: &str) -> String {
    let w = state.world.read().expect("world");
    let now = state.now();
    let mut ctx = format!("Ship time {} (day {} of {}).", crate::clock::hhmm(now), now / crate::clock::DAY_MS + 1, state.clock.mission_days);
    if let Some(p) = w.passenger(seat) {
        let v = p.vitals;
        ctx.push_str(&format!(
            " Talking with {} (seat {}). Now: pulse {} ({}), SpO2 {}%, skin {:.1} C ({}), status {}. Water today {:.1} L.",
            p.name,
            p.seat,
            if v.hr == 0 { "no reading".to_string() } else { format!("{} bpm", v.hr) },
            p.hr_word(),
            v.spo2,
            v.skin_c(),
            p.skin_word(),
            p.status.label(),
            p.water_ml as f32 / 1000.0
        ));
    }
    if let Some(c) = w.open_case() {
        ctx.push_str(&format!(" Open emergency: {} (seat {}), {}.", c.name, c.seat, c.kind));
    }
    if w.storm.is_some() {
        ctx.push_str(" A solar storm warning is active: everyone goes to the Deck B shelter.");
    }
    format!("{RULES}\n\nContext: {ctx}")
}
