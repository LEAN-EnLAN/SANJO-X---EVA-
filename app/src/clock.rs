//! Mission clock: milliseconds since mission start (day 1, 00:00 ship time).
//!
//! Real time drives it, plus an offset the dev panel can push forward
//! ("fast-forward 10 min"). Every timestamp in the world state, the chain and
//! the UI comes from here, so a fast-forward moves everything consistently.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub const DAY_MS: u64 = 86_400_000;

#[derive(Debug)]
pub struct MissionClock {
    started: Instant,
    start_ms: AtomicU64,
    offset_ms: AtomicU64,
    pub mission_days: u64,
}

impl MissionClock {
    /// Start at `day` (1-based) and `hh:mm` ship time.
    pub fn new(day: u64, hh: u64, mm: u64, mission_days: u64) -> Self {
        Self { started: Instant::now(), start_ms: AtomicU64::new((day - 1) * DAY_MS + hh * 3_600_000 + mm * 60_000), offset_ms: AtomicU64::new(0), mission_days }
    }
    pub fn now_ms(&self) -> u64 {
        self.start_ms.load(Ordering::Relaxed) + self.started.elapsed().as_millis() as u64 + self.offset_ms.load(Ordering::Relaxed)
    }
    /// Seconds since mission start (vitals timestamps on the wire).
    pub fn now_s(&self) -> u64 {
        self.now_ms() / 1000
    }
    pub fn fast_forward(&self, ms: u64) {
        self.offset_ms.fetch_add(ms, Ordering::Relaxed);
    }
    /// Jump back to a given day and time (demo reset).
    pub fn reset_to(&self, day: u64, hh: u64, mm: u64) {
        let target = (day - 1) * DAY_MS + hh * 3_600_000 + mm * 60_000;
        let base = self.started.elapsed().as_millis() as u64;
        self.offset_ms.store(0, Ordering::Relaxed);
        self.start_ms.store(target.saturating_sub(base), Ordering::Relaxed);
    }
    /// Real milliseconds since the process started (for link timing, not the story clock).
    pub fn real_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }
    pub fn day(&self) -> u64 {
        self.now_ms() / DAY_MS + 1
    }
}

/// "14:06" for a mission timestamp.
pub fn hhmm(ms: u64) -> String {
    let m = (ms % DAY_MS) / 60_000;
    format!("{:02}:{:02}", m / 60, m % 60)
}
/// "14:06:08" for a mission timestamp.
pub fn hhmmss(ms: u64) -> String {
    let s = (ms % DAY_MS) / 1000;
    format!("{:02}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
}
/// "Day 96 · 14:06"
pub fn stamp(ms: u64) -> String {
    format!("Day {} · {}", ms / DAY_MS + 1, hhmm(ms))
}
/// Short human age: "now", "12 s", "4 min", "2 h".
pub fn age(ms: u64) -> String {
    let s = ms / 1000;
    if s < 3 {
        "now".into()
    } else if s < 60 {
        format!("{s} s")
    } else if s < 3600 {
        format!("{} min", s / 60)
    } else {
        format!("{} h", s / 3600)
    }
}
/// "Good morning" / "Good afternoon" / "Good evening" for the ship time.
pub fn greeting(ms: u64) -> &'static str {
    match (ms % DAY_MS) / 3_600_000 {
        5..=11 => "Good morning",
        12..=17 => "Good afternoon",
        _ => "Good evening",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn formats() {
        let t = 95 * DAY_MS + 14 * 3_600_000 + 2 * 60_000 + 8_000;
        assert_eq!(hhmm(t), "14:02");
        assert_eq!(hhmmss(t), "14:02:08");
        assert_eq!(stamp(t), "Day 96 · 14:02");
        assert_eq!(greeting(t), "Good afternoon");
        assert_eq!(age(125_000), "2 min");
    }
    #[test]
    fn fast_forward_moves_clock() {
        let c = MissionClock::new(96, 13, 58, 212);
        let a = c.now_ms();
        c.fast_forward(600_000);
        assert!(c.now_ms() >= a + 600_000);
        assert_eq!(c.day(), 96);
        c.reset_to(96, 13, 58);
        assert!(c.now_ms() < a + 1_000);
    }
}
