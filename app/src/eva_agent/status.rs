//! Is Eva online, or on her on-board rules, and why? (`GET /api/eva/status`)
//!
//! The answer is real, not just "is the variable set": at start-up one probe to
//! the provider (`GET {base}/models`, 4 s) tells a refused key (`bad_key`) from a
//! hub with no internet (`unreachable`); afterwards every chat / STT / vision call
//! updates the same state, and while it is unhealthy the monitor re-probes at most
//! every 60 s, so a ship that gets its link back flips to `online` by itself.
//! A change is broadcast as the `eva-status` SSE event. Never contains the key.

use super::config::{AgentConfig, VoiceEngine};
use crate::state::{Shared, UiEvent};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    Ok,
    NoKey,
    OfflineFlag,
    BadKey,
    Unreachable,
}

impl Reason {
    pub fn as_str(self) -> &'static str {
        match self {
            Reason::Ok => "ok",
            Reason::NoKey => "no_key",
            Reason::OfflineFlag => "offline_flag",
            Reason::BadKey => "bad_key",
            Reason::Unreachable => "unreachable",
        }
    }
    pub fn hint(self) -> &'static str {
        match self {
            Reason::Ok => "",
            Reason::NoKey => "Set GROQ_API_KEY and restart",
            Reason::BadKey => "The Groq key was refused. Check GROQ_API_KEY and restart",
            Reason::Unreachable => "No internet from the hub. Eva will reconnect by herself",
            Reason::OfflineFlag => "Started with --offline",
        }
    }
    fn code(self) -> u8 {
        match self {
            Reason::Ok => 0,
            Reason::NoKey => 1,
            Reason::OfflineFlag => 2,
            Reason::BadKey => 3,
            Reason::Unreachable => 4,
        }
    }
    fn from_code(c: u8) -> Self {
        match c {
            3 => Reason::BadKey,
            4 => Reason::Unreachable,
            _ => Reason::Ok,
        }
    }
}

/// Runtime health of the online pipelines (shared by every clone of the config).
#[derive(Debug, Default)]
pub struct EvaHealth {
    /// last observed upstream state: 0 ok/unknown, 3 bad key, 4 unreachable
    state: AtomicU8,
    /// unix ms of the last observation
    checked_ms: AtomicU64,
    /// unix ms of the last probe
    probed_ms: AtomicU64,
    /// started with `--offline` / `EVA_OFFLINE`
    offline_flag: AtomicBool,
}

fn unix_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

impl EvaHealth {
    pub fn mark_offline_flag(&self) {
        self.offline_flag.store(true, Ordering::Relaxed);
    }
    pub fn offline_flag(&self) -> bool {
        self.offline_flag.load(Ordering::Relaxed)
    }
    /// Store an upstream observation; returns true when it changed.
    pub fn observe(&self, r: Reason) -> bool {
        let code = match r {
            Reason::BadKey | Reason::Unreachable => r.code(),
            _ => 0,
        };
        self.checked_ms.store(unix_ms(), Ordering::Relaxed);
        self.state.swap(code, Ordering::Relaxed) != code
    }
    pub fn upstream(&self) -> Reason {
        Reason::from_code(self.state.load(Ordering::Relaxed))
    }
}

/// What a failed call's error text says about the upstream (None: not about the link or key).
pub fn classify_error(text: &str) -> Option<Reason> {
    let t = text.to_ascii_lowercase();
    if t.contains("401") || t.contains("403") || t.contains("invalid api key") || t.contains("unauthorized") {
        Some(Reason::BadKey)
    } else if t.contains("error sending request") || t.contains("timed out") || t.contains("timeout") || t.contains("connect") || t.contains("dns") || t.contains("network") {
        Some(Reason::Unreachable)
    } else {
        None
    }
}

/// The current reason, from the flags, the key and the last observation.
pub fn reason(cfg: &AgentConfig) -> Reason {
    if cfg.health.offline_flag() || (cfg.offline && cfg.chat.key.is_some()) {
        Reason::OfflineFlag
    } else if cfg.chat.key.is_none() {
        Reason::NoKey
    } else if cfg.online_key().is_none() {
        Reason::OfflineFlag
    } else {
        cfg.health.upstream()
    }
}

fn rfc3339(ms: u64) -> String {
    let secs = ms / 1000;
    let (days, rem) = (secs / 86_400, secs % 86_400);
    // civil-from-days (Howard Hinnant's algorithm, public domain arithmetic)
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, (rem / 60) % 60, rem % 60)
}

/// The `/api/eva/status` body (and the `eva-status` SSE payload). No secrets.
pub fn status_json(cfg: &AgentConfig) -> serde_json::Value {
    let r = reason(cfg);
    let online = r == Reason::Ok;
    let key_source = if cfg.chat.key.is_none() { "none" } else { crate::envfile::key_source().filter(|k| *k != crate::envfile::KeySource::None).map(|k| k.as_str()).unwrap_or("environment") };
    let checked = cfg.health.checked_ms.load(Ordering::Relaxed);
    serde_json::json!({
        "mode": if online { "online" } else { "rules" },
        "reason": r.as_str(),
        "label": if online { "Eva online" } else { "On-board rules" },
        "hint": r.hint(),
        "chat": online && cfg.chat.online_key().is_some(),
        "voice_in": online && cfg.stt.online_key().is_some(),
        "voice_out": if online && cfg.voice.get() == VoiceEngine::Groq { "server" } else { "browser" },
        "vision": online && cfg.vision.online_key().is_some(),
        "key_source": key_source,
        "checked_at": rfc3339(if checked == 0 { unix_ms() } else { checked }),
        "version": env!("CARGO_PKG_VERSION"),
    })
}

/// One probe: `GET {base}/models` with the key, 4 s timeout.
pub async fn probe(cfg: &AgentConfig, http: &reqwest::Client) -> Reason {
    let Some(key) = cfg.chat.online_key() else { return reason(cfg) };
    cfg.health.probed_ms.store(unix_ms(), Ordering::Relaxed);
    let res = http.get(cfg.chat.url("/models")).bearer_auth(key.expose()).timeout(Duration::from_secs(4)).send().await;
    match res {
        Ok(r) if r.status() == reqwest::StatusCode::UNAUTHORIZED || r.status() == reqwest::StatusCode::FORBIDDEN => Reason::BadKey,
        Ok(_) => Reason::Ok,
        Err(_) => Reason::Unreachable,
    }
}

/// Record the outcome of a real pipeline call (chat, STT, vision): Ok → healthy,
/// an error that is about the key or the link → that reason. Emits `eva-status` on change.
pub fn record(state: &Shared, outcome: Result<(), &str>) {
    if state.agent.chat.online_key().is_none() {
        return;
    }
    let r = match outcome {
        Ok(()) => Reason::Ok,
        Err(e) => match classify_error(e) {
            Some(r) => r,
            None => return,
        },
    };
    if state.agent.health.observe(r) {
        state.emit(UiEvent::EvaStatus);
    }
}

/// Start-up probe, then a re-probe at most every 60 s while unhealthy.
pub async fn run_monitor(state: Shared) {
    if state.agent.chat.online_key().is_none() {
        return;
    }
    let first = probe(&state.agent, &state.http).await;
    match first {
        Reason::BadKey => tracing::warn!("Eva: on-board rules · {}", Reason::BadKey.hint()),
        Reason::Unreachable => tracing::warn!("Eva: on-board rules for now · {}", Reason::Unreachable.hint()),
        _ => {}
    }
    if state.agent.health.observe(first) {
        state.emit(UiEvent::EvaStatus);
    }
    loop {
        tokio::time::sleep(Duration::from_secs(15)).await;
        if state.agent.health.upstream() == Reason::Ok {
            continue;
        }
        if unix_ms().saturating_sub(state.agent.health.probed_ms.load(Ordering::Relaxed)) < 60_000 {
            continue;
        }
        let r = probe(&state.agent, &state.http).await;
        if state.agent.health.observe(r) {
            if r == Reason::Ok {
                tracing::info!("Eva: online again");
            }
            state.emit(UiEvent::EvaStatus);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reasons_follow_flags_and_key() {
        let c = AgentConfig::offline_only();
        assert_eq!(reason(&c), Reason::NoKey);
        let mut k = AgentConfig::with_endpoint("http://x", "gsk_abc");
        assert_eq!(reason(&k), Reason::Ok);
        k.health.observe(Reason::BadKey);
        assert_eq!(reason(&k), Reason::BadKey);
        k.set_offline(true);
        assert_eq!(reason(&k), Reason::OfflineFlag);
        let j = status_json(&k).to_string();
        assert!(!j.contains("gsk_abc"), "{j}");
    }

    #[test]
    fn errors_are_classified() {
        assert_eq!(classify_error("upstream 401: Invalid API Key"), Some(Reason::BadKey));
        assert_eq!(classify_error("error sending request for url"), Some(Reason::Unreachable));
        assert_eq!(classify_error("model said nonsense"), None);
    }

    #[test]
    fn timestamps_are_rfc3339() {
        assert_eq!(rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(rfc3339(1_794_753_130_000), "2026-11-15T14:32:10Z");
    }
}
