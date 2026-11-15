//! Agent configuration, read once from the environment at start-up.
//!
//! Each layer-2 pipeline (chat, speech-to-text, text-to-speech, vision) has
//! its **own** [`PipelineConfig`]: base URL, model, timeout, and whether it is
//! enabled. They share one secret: the Groq API key, read **only** from
//! `GROQ_API_KEY`. It is kept in an [`ApiKey`] whose `Debug`/`Display` print
//! `***`; it is never logged, never written to the chain, never rendered into
//! a page and never sent to the browser. Every error text that leaves a
//! pipeline goes through [`redact`] first, in case an upstream service echoes
//! the key back. Without a key every pipeline falls back to offline behaviour.

use std::fmt;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct ApiKey(String);

impl ApiKey {
    pub fn new(k: impl Into<String>) -> Self {
        ApiKey(k.into())
    }
    pub fn expose(&self) -> &str {
        &self.0
    }
}
impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey(***)")
    }
}
impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

/// Model ids checked against https://console.groq.com/docs/models (Nov 2026).
pub mod defaults {
    pub const BASE_URL: &str = "https://api.groq.com/openai/v1";
    /// chat + tool calling (production model)
    pub const CHAT_MODEL: &str = "openai/gpt-oss-120b";
    /// image understanding (preview model, up to 3 images per request)
    pub const VISION_MODEL: &str = "qwen/qwen3.8-27b";
    /// speech to text (production)
    pub const STT_MODEL: &str = "whisper-large-v3-turbo";
    /// text to speech (preview); voices include autumn, diana, hannah, austin, daniel, troy.
    /// Needs the model terms accepted once by an org admin in the Groq console.
    pub const TTS_MODEL: &str = "canopylabs/orpheus-v1-english";
    pub const TTS_VOICE: &str = "hannah";
}

/// Remove anything that looks like a secret from a text that may be logged
/// or shown: the configured key itself and any `gsk_…` token.
pub fn redact(text: &str, key: Option<&ApiKey>) -> String {
    let mut s = text.to_string();
    if let Some(k) = key {
        if k.0.len() >= 4 {
            s = s.replace(&k.0, "***");
        }
    }
    // any Groq-style token, even one we were not configured with
    let mut out = String::with_capacity(s.len());
    let mut rest = s.as_str();
    while let Some(i) = rest.find("gsk_") {
        out.push_str(&rest[..i]);
        out.push_str("gsk_***");
        let tail = &rest[i + 4..];
        let end = tail.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-')).unwrap_or(tail.len());
        rest = &tail[end..];
    }
    out.push_str(rest);
    // bearer headers echoed back by a proxy: keep the word, hide the token
    let lower = out.to_ascii_lowercase();
    let mut res = String::with_capacity(out.len());
    let mut last = 0;
    let mut from = 0;
    while let Some(i) = lower[from..].find("bearer ") {
        let start = from + i + 7;
        let end = out[start..].find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == ')').map(|e| start + e).unwrap_or(out.len());
        res.push_str(&out[last..start]);
        res.push_str("***");
        last = end;
        from = end.max(start);
    }
    res.push_str(&out[last..]);
    let out = res;
    out
}

/// One layer-2 pipeline's settings.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// "chat" | "stt" | "tts" | "vision"
    pub name: &'static str,
    pub key: Option<ApiKey>,
    pub base_url: String,
    pub model: String,
    pub timeout_s: u64,
    /// false = this pipeline always uses its offline fallback
    pub enabled: bool,
}

impl PipelineConfig {
    /// The key, if this pipeline may go online.
    pub fn online_key(&self) -> Option<&ApiKey> {
        if self.enabled {
            self.key.as_ref()
        } else {
            None
        }
    }
    pub fn redact(&self, text: &str) -> String {
        redact(text, self.key.as_ref())
    }
    /// An `anyhow` error with the key scrubbed out.
    pub fn error(&self, text: impl fmt::Display) -> anyhow::Error {
        anyhow::anyhow!(self.redact(&text.to_string()))
    }
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }
}

/// Which voice the browser should use right now. Groq's Orpheus model answers
/// "requires terms acceptance" until an org admin accepts its terms in the
/// Groq console; then the server tells every client to use the browser's own
/// `speechSynthesis` and the UI badge says so.
#[derive(Debug, Default)]
pub struct VoiceStatus {
    /// 0 unknown/online, 1 no key or offline, 2 terms not accepted, 3 upstream error
    state: AtomicU8,
    /// real ms (process clock) until which we do not retry the online voice
    retry_after_ms: AtomicU64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceEngine {
    /// Groq Orpheus TTS
    Groq,
    /// no key / offline: browser speechSynthesis
    BrowserOffline,
    /// Groq refused: the Orpheus terms are not accepted for this org
    BrowserTerms,
    /// Groq failed for another reason (network, quota)
    BrowserError,
}

impl VoiceEngine {
    pub fn label(self) -> &'static str {
        match self {
            VoiceEngine::Groq => "Groq voice",
            VoiceEngine::BrowserOffline => "browser voice (offline)",
            VoiceEngine::BrowserTerms => "browser voice (Groq TTS terms not accepted)",
            VoiceEngine::BrowserError => "browser voice (Groq TTS unavailable)",
        }
    }
    pub fn reason(self) -> &'static str {
        match self {
            VoiceEngine::Groq => "online",
            VoiceEngine::BrowserOffline => "no_key",
            VoiceEngine::BrowserTerms => "terms_not_accepted",
            VoiceEngine::BrowserError => "upstream_error",
        }
    }
    pub fn is_browser(self) -> bool {
        self != VoiceEngine::Groq
    }
}

impl VoiceStatus {
    pub fn get(&self) -> VoiceEngine {
        match self.state.load(Ordering::Relaxed) {
            1 => VoiceEngine::BrowserOffline,
            2 => VoiceEngine::BrowserTerms,
            3 => VoiceEngine::BrowserError,
            _ => VoiceEngine::Groq,
        }
    }
    pub fn set(&self, e: VoiceEngine, now_ms: u64, retry_in_ms: u64) {
        let v = match e {
            VoiceEngine::Groq => 0,
            VoiceEngine::BrowserOffline => 1,
            VoiceEngine::BrowserTerms => 2,
            VoiceEngine::BrowserError => 3,
        };
        self.state.store(v, Ordering::Relaxed);
        self.retry_after_ms.store(now_ms + retry_in_ms, Ordering::Relaxed);
    }
    /// Whether to try the online voice again now.
    pub fn should_try(&self, now_ms: u64) -> bool {
        match self.get() {
            VoiceEngine::Groq => true,
            VoiceEngine::BrowserOffline => false,
            _ => now_ms >= self.retry_after_ms.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub chat: PipelineConfig,
    pub stt: PipelineConfig,
    pub tts: PipelineConfig,
    pub tts_voice: String,
    pub vision: PipelineConfig,
    /// force offline pipelines even with a key (demo without network)
    pub offline: bool,
    /// runtime state of the voice (shared by clones)
    pub voice: Arc<VoiceStatus>,
    /// topic guard: scope classifier, Prompt Guard, cache, rate limits (ADR-012)
    pub guard: super::guard::GuardConfig,
    /// online / on-board rules and why (`/api/eva/status`, shared by clones)
    pub health: Arc<super::status::EvaHealth>,
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty()).unwrap_or_else(|| default.to_string())
}

fn env_off(name: &str) -> bool {
    std::env::var(name).is_ok_and(|v| matches!(v.trim(), "0" | "off" | "false" | "browser" | "offline"))
}

impl AgentConfig {
    /// Read everything from the environment. The key comes only from `GROQ_API_KEY`.
    pub fn from_env() -> Self {
        let key = std::env::var("GROQ_API_KEY").ok().map(|k| k.trim().to_string()).filter(|k| !k.is_empty()).map(ApiKey);
        let base = env_or("GROQ_BASE_URL", defaults::BASE_URL);
        let p = |name: &'static str, model_env: &str, model: &str, url_env: &str, off_env: &str, timeout_s: u64| PipelineConfig {
            name,
            key: key.clone(),
            base_url: env_or(url_env, &base),
            model: env_or(model_env, model),
            timeout_s,
            enabled: !env_off(off_env),
        };
        let offline = std::env::var("EVA_OFFLINE").is_ok_and(|v| v == "1" || v == "true");
        let gd = super::guard::defaults::CLASSIFIER_MODEL;
        let safety = env_or("EVA_GUARD_SAFETY_MODEL", super::guard::defaults::SAFETY_MODEL);
        let num = |name: &str, d: u64| std::env::var(name).ok().and_then(|v| v.trim().parse::<u64>().ok()).filter(|n| *n > 0).unwrap_or(d);
        let guard = super::guard::GuardConfig::new(
            p("guard", "EVA_GUARD_MODEL", gd, "EVA_GUARD_BASE_URL", "EVA_GUARD", 2),
            if matches!(safety.trim(), "off" | "0" | "false" | "none") { None } else { Some(safety) },
            num("EVA_GUARD_TIMEOUT_MS", super::guard::defaults::TIMEOUT_MS),
            num("EVA_RATE_PER_MIN", super::guard::defaults::RATE_PER_MIN as u64) as u32,
        );
        let mut c = AgentConfig {
            chat: p("chat", "EVA_CHAT_MODEL", defaults::CHAT_MODEL, "EVA_CHAT_BASE_URL", "EVA_CHAT", 30),
            stt: p("stt", "EVA_STT_MODEL", defaults::STT_MODEL, "EVA_STT_BASE_URL", "EVA_STT", 30),
            tts: p("tts", "EVA_TTS_MODEL", defaults::TTS_MODEL, "EVA_TTS_BASE_URL", "EVA_TTS", 20),
            tts_voice: env_or("EVA_TTS_VOICE", defaults::TTS_VOICE),
            vision: p("vision", "EVA_VISION_MODEL", defaults::VISION_MODEL, "EVA_VISION_BASE_URL", "EVA_VISION", 45),
            offline: false,
            voice: Arc::new(VoiceStatus::default()),
            guard,
            health: Arc::default(),
        };
        c.set_offline(offline);
        c
    }

    /// No key, every pipeline offline (tests, `--offline`).
    pub fn offline_only() -> Self {
        let p = |name: &'static str, model: &str| PipelineConfig { name, key: None, base_url: defaults::BASE_URL.into(), model: model.into(), timeout_s: 30, enabled: false };
        let c = AgentConfig {
            chat: p("chat", defaults::CHAT_MODEL),
            stt: p("stt", defaults::STT_MODEL),
            tts: p("tts", defaults::TTS_MODEL),
            tts_voice: defaults::TTS_VOICE.into(),
            vision: p("vision", defaults::VISION_MODEL),
            offline: true,
            voice: Arc::new(VoiceStatus::default()),
            guard: super::guard::GuardConfig::new(p("guard", super::guard::defaults::CLASSIFIER_MODEL), Some(super::guard::defaults::SAFETY_MODEL.into()), super::guard::defaults::TIMEOUT_MS, super::guard::defaults::RATE_PER_MIN),
            health: Arc::default(),
        };
        c.voice.set(VoiceEngine::BrowserOffline, 0, 0);
        c
    }

    /// Point every pipeline at `base_url` with `key` (tests against a local fake server).
    pub fn with_endpoint(base_url: &str, key: &str) -> Self {
        let mut c = Self::offline_only();
        for p in [&mut c.chat, &mut c.stt, &mut c.tts, &mut c.vision, &mut c.guard.classifier] {
            p.key = Some(ApiKey(key.into()));
            p.base_url = base_url.into();
            p.enabled = true;
        }
        c.offline = false;
        c.voice = Arc::new(VoiceStatus::default());
        c.health = Arc::default();
        c
    }

    pub fn set_offline(&mut self, offline: bool) {
        self.offline = offline;
        if offline {
            for p in [&mut self.chat, &mut self.stt, &mut self.tts, &mut self.vision, &mut self.guard.classifier] {
                p.enabled = false;
            }
        }
        if self.tts.online_key().is_none() {
            self.voice.set(VoiceEngine::BrowserOffline, 0, 0);
        }
    }

    /// The key, if the chat pipeline may be used online.
    pub fn online_key(&self) -> Option<&ApiKey> {
        self.chat.online_key()
    }

    /// Short mode for badges: chat mode plus the voice when it fell back.
    pub fn mode_label(&self) -> &'static str {
        let online = self.online_key().is_some();
        match (online, self.voice.get()) {
            (true, VoiceEngine::Groq) => "Groq online",
            (true, VoiceEngine::BrowserTerms) => "Groq online · browser voice (TTS terms pending)",
            (true, VoiceEngine::BrowserError) => "Groq online · browser voice",
            (true, VoiceEngine::BrowserOffline) => "Groq online · browser voice",
            (false, _) => "offline rules · browser voice",
        }
    }

    /// Redact with the configured key.
    pub fn redact(&self, text: &str) -> String {
        redact(text, self.chat.key.as_ref())
    }

    /// For `/api/state`: models and which pipelines are online. No secrets.
    pub fn describe(&self) -> serde_json::Value {
        let p = |c: &PipelineConfig| serde_json::json!({"model": c.model, "online": c.online_key().is_some()});
        let v = self.voice.get();
        serde_json::json!({
            "mode": self.mode_label(),
            "chat": p(&self.chat), "stt": p(&self.stt), "tts": p(&self.tts), "vision": p(&self.vision),
            "guard": {"model": self.guard.classifier.model, "online": self.guard.classifier.online_key().is_some(), "safety_model": self.guard.safety_model, "timeout_ms": self.guard.timeout_ms, "rate_per_min": self.guard.rt.limiter.per_min()},
            "voice": {"engine": v, "label": v.label(), "browser": v.is_browser()},
            // flat fields kept for older clients
            "chat_model": self.chat.model, "stt_model": self.stt.model, "tts_model": self.tts.model, "vision_model": self.vision.model,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_never_prints() {
        let k = ApiKey("gsk_secret".into());
        assert_eq!(format!("{k:?} {k}"), "ApiKey(***) ***");
        let mut c = AgentConfig::with_endpoint("http://x", "gsk_secret");
        c.tts.key = Some(k);
        let dbg = format!("{c:?}");
        assert!(!dbg.contains("secret"), "{dbg}");
        assert!(!c.describe().to_string().contains("secret"));
    }

    #[test]
    fn redact_scrubs_key_tokens_and_bearer_headers() {
        let k = ApiKey("gsk_AbC123xyz".into());
        let t = "401: invalid key gsk_AbC123xyz (Authorization: Bearer gsk_AbC123xyz) other gsk_ZZZ999 and Bearer sk-live-777";
        let r = redact(t, Some(&k));
        assert!(!r.contains("AbC123xyz"), "{r}");
        assert!(!r.contains("ZZZ999"), "{r}");
        assert!(!r.contains("sk-live-777"), "{r}");
        assert!(r.contains("401: invalid key"));
        let plain = AgentConfig::with_endpoint("http://x", "plainkey12345");
        assert_eq!(plain.chat.redact("echo plainkey12345"), "echo ***");
    }

    #[test]
    fn every_pipeline_has_its_own_config() {
        let c = AgentConfig::with_endpoint("http://x", "k");
        let names: Vec<&str> = [&c.chat, &c.stt, &c.tts, &c.vision].iter().map(|p| p.name).collect();
        assert_eq!(names, ["chat", "stt", "tts", "vision"]);
        let mut d = c.clone();
        d.tts.enabled = false;
        assert!(d.tts.online_key().is_none() && d.chat.online_key().is_some(), "a pipeline can be switched off alone");
    }

    #[test]
    fn voice_status_backs_off_and_retries() {
        let v = VoiceStatus::default();
        assert!(v.should_try(0));
        v.set(VoiceEngine::BrowserTerms, 1_000, 600_000);
        assert!(!v.should_try(2_000));
        assert!(v.should_try(601_000));
        assert_eq!(v.get().reason(), "terms_not_accepted");
    }
}
