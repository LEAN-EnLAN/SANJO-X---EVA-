//! Agent configuration, read once from the environment at start-up.
//!
//! The Groq API key is read **only** from `GROQ_API_KEY`. It is kept in an
//! [`ApiKey`] whose `Debug`/`Display` print `***`, it is never logged, never
//! written to the chain, never rendered into a page and never sent to the
//! browser. Without a key every pipeline falls back to offline behaviour.

use std::fmt;

#[derive(Clone)]
pub struct ApiKey(String);

impl ApiKey {
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
    /// image understanding with tool calling (preview model, up to 3 images per request)
    pub const VISION_MODEL: &str = "qwen/qwen3.8-27b";
    /// speech to text (production)
    pub const STT_MODEL: &str = "whisper-large-v3-turbo";
    /// text to speech (preview); voices include autumn, diana, hannah, austin, daniel, troy
    pub const TTS_MODEL: &str = "canopylabs/orpheus-v1-english";
    pub const TTS_VOICE: &str = "hannah";
}

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub key: Option<ApiKey>,
    pub base_url: String,
    pub chat_model: String,
    pub vision_model: String,
    pub stt_model: String,
    pub tts_model: String,
    pub tts_voice: String,
    /// force offline pipelines even with a key (demo without network)
    pub offline: bool,
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty()).unwrap_or_else(|| default.to_string())
}

impl AgentConfig {
    pub fn from_env() -> Self {
        let key = std::env::var("GROQ_API_KEY").ok().map(|k| k.trim().to_string()).filter(|k| !k.is_empty()).map(ApiKey);
        AgentConfig {
            key,
            base_url: env_or("GROQ_BASE_URL", defaults::BASE_URL),
            chat_model: env_or("EVA_CHAT_MODEL", defaults::CHAT_MODEL),
            vision_model: env_or("EVA_VISION_MODEL", defaults::VISION_MODEL),
            stt_model: env_or("EVA_STT_MODEL", defaults::STT_MODEL),
            tts_model: env_or("EVA_TTS_MODEL", defaults::TTS_MODEL),
            tts_voice: env_or("EVA_TTS_VOICE", defaults::TTS_VOICE),
            offline: std::env::var("EVA_OFFLINE").is_ok_and(|v| v == "1" || v == "true"),
        }
    }
    pub fn offline_only() -> Self {
        AgentConfig { key: None, offline: true, ..Self::from_env_defaults() }
    }
    fn from_env_defaults() -> Self {
        AgentConfig {
            key: None,
            base_url: defaults::BASE_URL.into(),
            chat_model: defaults::CHAT_MODEL.into(),
            vision_model: defaults::VISION_MODEL.into(),
            stt_model: defaults::STT_MODEL.into(),
            tts_model: defaults::TTS_MODEL.into(),
            tts_voice: defaults::TTS_VOICE.into(),
            offline: true,
        }
    }
    /// The key, if online pipelines may be used.
    pub fn online_key(&self) -> Option<&ApiKey> {
        if self.offline {
            None
        } else {
            self.key.as_ref()
        }
    }
    pub fn mode_label(&self) -> &'static str {
        if self.online_key().is_some() {
            "Groq online"
        } else {
            "offline rules"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn key_never_prints() {
        let k = ApiKey("gsk_secret".into());
        assert_eq!(format!("{k:?} {k}"), "ApiKey(***) ***");
        let c = AgentConfig { key: Some(k), ..AgentConfig::offline_only() };
        assert!(!format!("{c:?}").contains("secret"));
    }
}
