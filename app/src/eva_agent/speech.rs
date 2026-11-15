//! **Speech pipelines**, each with its own trait and [`PipelineConfig`].
//!
//! * Speech-to-text ([`SttPipeline`]): the browser records with
//!   `MediaRecorder` and uploads to `POST /api/eva/stt`; the server forwards
//!   to Groq Whisper (`whisper-large-v3-turbo`) and returns `{ "text": … }`.
//! * Text-to-speech ([`TtsPipeline`]): `POST /api/eva/tts` streams audio
//!   (WAV) from Groq's Orpheus model. **Explicit fallback:** when there is no
//!   key, or Groq refuses (the Orpheus model needs its terms accepted once by
//!   an org admin in the Groq console, otherwise it answers *"requires terms
//!   acceptance"*), the endpoint answers `503` with
//!   `{"fallback":"speechSynthesis","engine":…,"reason":…,"message":…}` and the
//!   header `x-eva-voice: browser`; the browser then speaks with its own
//!   `speechSynthesis`, and the Eva badge shows "browser voice". The server
//!   remembers the refusal for 10 minutes so each reply does not pay a failed
//!   round trip.

use super::config::{PipelineConfig, VoiceEngine};
use super::AgentConfig;
use futures_util::future::BoxFuture;

/// How long to keep using the browser voice after Groq refused.
pub const TTS_RETRY_MS: u64 = 10 * 60_000;
pub const MAX_AUDIO_BYTES: usize = 24 * 1024 * 1024;
pub const MAX_TTS_CHARS: usize = 1_000;

#[derive(Debug, Clone)]
pub struct AudioInput {
    pub bytes: Vec<u8>,
    pub filename: String,
    pub mime: String,
}

pub trait SttPipeline: Send + Sync {
    fn name(&self) -> String;
    fn transcribe<'a>(&'a self, http: &'a reqwest::Client, audio: AudioInput) -> BoxFuture<'a, anyhow::Result<String>>;
}

pub trait TtsPipeline: Send + Sync {
    fn name(&self) -> String;
    /// Returns the upstream response so the caller can stream its body.
    fn speak<'a>(&'a self, http: &'a reqwest::Client, text: String) -> BoxFuture<'a, Result<reqwest::Response, TtsError>>;
}

/// Why the online voice is not available.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TtsError {
    /// the model's terms are not accepted for this Groq org (admin action in the console)
    TermsNotAccepted(String),
    /// any other failure (network, quota, 5xx), already redacted
    Upstream(String),
}

impl std::fmt::Display for TtsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TtsError::TermsNotAccepted(m) => write!(f, "groq tts: terms not accepted: {m}"),
            TtsError::Upstream(m) => write!(f, "groq tts: {m}"),
        }
    }
}
impl std::error::Error for TtsError {}

impl TtsError {
    pub fn engine(&self) -> VoiceEngine {
        match self {
            TtsError::TermsNotAccepted(_) => VoiceEngine::BrowserTerms,
            TtsError::Upstream(_) => VoiceEngine::BrowserError,
        }
    }
}

/// Groq's wording for a model whose terms nobody accepted yet.
pub fn is_terms_error(msg: &str) -> bool {
    let m = msg.to_ascii_lowercase();
    m.contains("terms") && (m.contains("accept") || m.contains("agree"))
}

pub fn stt(cfg: &AgentConfig) -> Option<Box<dyn SttPipeline>> {
    cfg.stt.online_key().map(|_| Box::new(GroqWhisper { cfg: cfg.stt.clone() }) as Box<dyn SttPipeline>)
}
pub fn tts(cfg: &AgentConfig) -> Option<Box<dyn TtsPipeline>> {
    cfg.tts.online_key().map(|_| Box::new(GroqTts { cfg: cfg.tts.clone(), voice: cfg.tts_voice.clone() }) as Box<dyn TtsPipeline>)
}

/// Result of asking for Eva's voice.
pub enum Spoken {
    /// stream this audio to the browser
    Audio(reqwest::Response),
    /// the browser must use `speechSynthesis`; why, and a sentence for logs / UI
    Browser(VoiceEngine, String),
}

/// One place that decides between Groq TTS and the browser voice, and keeps
/// [`super::config::VoiceStatus`] up to date for the UI badge.
pub async fn speak_or_fallback(cfg: &AgentConfig, http: &reqwest::Client, text: String, now_ms: u64) -> Spoken {
    let Some(pipe) = tts(cfg) else {
        cfg.voice.set(VoiceEngine::BrowserOffline, now_ms, 0);
        return Spoken::Browser(VoiceEngine::BrowserOffline, "No GROQ_API_KEY on the server (or offline mode): using the browser voice.".into());
    };
    if !cfg.voice.should_try(now_ms) {
        let e = cfg.voice.get();
        return Spoken::Browser(e, message_for(e));
    }
    match pipe.speak(http, text).await {
        Ok(r) => {
            cfg.voice.set(VoiceEngine::Groq, now_ms, 0);
            Spoken::Audio(r)
        }
        Err(e) => {
            let engine = e.engine();
            tracing::warn!("{}", cfg.tts.redact(&e.to_string()));
            cfg.voice.set(engine, now_ms, TTS_RETRY_MS);
            Spoken::Browser(engine, message_for(engine))
        }
    }
}

pub fn message_for(e: VoiceEngine) -> String {
    match e {
        VoiceEngine::Groq => "Groq voice".into(),
        VoiceEngine::BrowserOffline => "No GROQ_API_KEY on the server (or offline mode): using the browser voice.".into(),
        VoiceEngine::BrowserTerms => "Groq TTS needs its model terms accepted by an org admin in the Groq console (Orpheus). Using the browser voice until then.".into(),
        VoiceEngine::BrowserError => "Groq TTS is not reachable right now. Using the browser voice.".into(),
    }
}

pub struct GroqWhisper {
    pub cfg: PipelineConfig,
}

impl SttPipeline for GroqWhisper {
    fn name(&self) -> String {
        format!("groq:{}", self.cfg.model)
    }
    fn transcribe<'a>(&'a self, http: &'a reqwest::Client, audio: AudioInput) -> BoxFuture<'a, anyhow::Result<String>> {
        Box::pin(async move {
            let key = self.cfg.online_key().ok_or_else(|| anyhow::anyhow!("no key"))?;
            if audio.bytes.is_empty() || audio.bytes.len() > MAX_AUDIO_BYTES {
                anyhow::bail!("audio must be 1 byte to 24 MB");
            }
            let part = reqwest::multipart::Part::bytes(audio.bytes).file_name(audio.filename).mime_str(&audio.mime).map_err(|e| self.cfg.error(e))?;
            let form = reqwest::multipart::Form::new().part("file", part).text("model", self.cfg.model.clone()).text("response_format", "json").text("language", "en").text("temperature", "0");
            let resp = http
                .post(self.cfg.url("/audio/transcriptions"))
                .timeout(std::time::Duration::from_secs(self.cfg.timeout_s))
                .bearer_auth(key.expose())
                .multipart(form)
                .send()
                .await
                .map_err(|e| self.cfg.error(e))?;
            let status = resp.status();
            let v: serde_json::Value = resp.json().await.unwrap_or_default();
            if !status.is_success() {
                return Err(self.cfg.error(format!("groq stt {status}: {}", v.pointer("/error/message").and_then(|m| m.as_str()).unwrap_or("failed"))));
            }
            Ok(v.get("text").and_then(|t| t.as_str()).unwrap_or("").trim().to_string())
        })
    }
}

pub struct GroqTts {
    pub cfg: PipelineConfig,
    pub voice: String,
}

impl TtsPipeline for GroqTts {
    fn name(&self) -> String {
        format!("groq:{}:{}", self.cfg.model, self.voice)
    }
    fn speak<'a>(&'a self, http: &'a reqwest::Client, text: String) -> BoxFuture<'a, Result<reqwest::Response, TtsError>> {
        Box::pin(async move {
            let key = self.cfg.online_key().ok_or_else(|| TtsError::Upstream("no key".into()))?;
            let input: String = text.chars().take(MAX_TTS_CHARS).collect();
            let body = serde_json::json!({"model": self.cfg.model, "input": input, "voice": self.voice, "response_format": "wav"});
            let resp = http
                .post(self.cfg.url("/audio/speech"))
                .timeout(std::time::Duration::from_secs(self.cfg.timeout_s))
                .bearer_auth(key.expose())
                .json(&body)
                .send()
                .await
                .map_err(|e| TtsError::Upstream(self.cfg.redact(&e.to_string())))?;
            if !resp.status().is_success() {
                let status = resp.status();
                let v: serde_json::Value = resp.json().await.unwrap_or_default();
                let msg = self.cfg.redact(v.pointer("/error/message").and_then(|m| m.as_str()).unwrap_or("failed"));
                let code = v.pointer("/error/code").and_then(|m| m.as_str()).unwrap_or("");
                return Err(if is_terms_error(&msg) || is_terms_error(code) { TtsError::TermsNotAccepted(format!("{status}: {msg}")) } else { TtsError::Upstream(format!("{status}: {msg}")) });
            }
            Ok(resp)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_groq_terms_wording() {
        assert!(is_terms_error("The model `canopylabs/orpheus-v1-english` requires terms acceptance. Please have the org admin accept the terms at https://console.groq.com/playground?model=canopylabs/orpheus-v1-english"));
        assert!(is_terms_error("model_terms_required: accept the terms"));
        assert!(!is_terms_error("rate limit reached"));
    }

    #[tokio::test]
    async fn no_key_means_browser_voice_explicitly() {
        let cfg = AgentConfig::offline_only();
        match speak_or_fallback(&cfg, &reqwest::Client::new(), "hello".into(), 0).await {
            Spoken::Browser(e, msg) => {
                assert_eq!(e, VoiceEngine::BrowserOffline);
                assert!(msg.contains("browser voice"));
            }
            Spoken::Audio(_) => panic!("no key, no audio"),
        }
        assert!(cfg.mode_label().contains("browser voice"));
    }
}
