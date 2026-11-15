//! **Speech pipelines.**
//!
//! * Speech-to-text: the browser records with `MediaRecorder` and uploads to
//!   `POST /api/eva/stt`; the server forwards to Groq Whisper
//!   (`whisper-large-v3-turbo`) and returns `{ "text": … }`.
//! * Text-to-speech: `POST /api/eva/tts` streams audio (WAV) from Groq's
//!   Orpheus TTS back to the browser. Without a key the endpoint answers
//!   `503` with `{"fallback":"speechSynthesis"}` and the browser speaks with
//!   its own `speechSynthesis`.

use super::AgentConfig;
use futures_util::future::BoxFuture;

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
    fn speak<'a>(&'a self, http: &'a reqwest::Client, text: String) -> BoxFuture<'a, anyhow::Result<reqwest::Response>>;
}

pub fn stt(cfg: &AgentConfig) -> Option<Box<dyn SttPipeline>> {
    cfg.online_key().map(|_| Box::new(GroqWhisper { cfg: cfg.clone() }) as Box<dyn SttPipeline>)
}
pub fn tts(cfg: &AgentConfig) -> Option<Box<dyn TtsPipeline>> {
    cfg.online_key().map(|_| Box::new(GroqTts { cfg: cfg.clone() }) as Box<dyn TtsPipeline>)
}

pub struct GroqWhisper {
    pub cfg: AgentConfig,
}

impl SttPipeline for GroqWhisper {
    fn name(&self) -> String {
        format!("groq:{}", self.cfg.stt_model)
    }
    fn transcribe<'a>(&'a self, http: &'a reqwest::Client, audio: AudioInput) -> BoxFuture<'a, anyhow::Result<String>> {
        Box::pin(async move {
            let key = self.cfg.online_key().ok_or_else(|| anyhow::anyhow!("no key"))?;
            if audio.bytes.len() > 24 * 1024 * 1024 {
                anyhow::bail!("audio too large");
            }
            let part = reqwest::multipart::Part::bytes(audio.bytes).file_name(audio.filename).mime_str(&audio.mime)?;
            let form = reqwest::multipart::Form::new().part("file", part).text("model", self.cfg.stt_model.clone()).text("response_format", "json").text("language", "en").text("temperature", "0");
            let resp = http.post(format!("{}/audio/transcriptions", self.cfg.base_url)).bearer_auth(key.expose()).multipart(form).send().await?;
            let status = resp.status();
            let v: serde_json::Value = resp.json().await?;
            if !status.is_success() {
                anyhow::bail!("groq stt {status}: {}", v.pointer("/error/message").and_then(|m| m.as_str()).unwrap_or("failed"));
            }
            Ok(v.get("text").and_then(|t| t.as_str()).unwrap_or("").trim().to_string())
        })
    }
}

pub struct GroqTts {
    pub cfg: AgentConfig,
}

impl TtsPipeline for GroqTts {
    fn name(&self) -> String {
        format!("groq:{}:{}", self.cfg.tts_model, self.cfg.tts_voice)
    }
    fn speak<'a>(&'a self, http: &'a reqwest::Client, text: String) -> BoxFuture<'a, anyhow::Result<reqwest::Response>> {
        Box::pin(async move {
            let key = self.cfg.online_key().ok_or_else(|| anyhow::anyhow!("no key"))?;
            let input: String = text.chars().take(1_000).collect();
            let body = serde_json::json!({"model": self.cfg.tts_model, "input": input, "voice": self.cfg.tts_voice, "response_format": "wav"});
            let resp = http.post(format!("{}/audio/speech", self.cfg.base_url)).bearer_auth(key.expose()).json(&body).send().await?;
            if !resp.status().is_success() {
                let status = resp.status();
                let v: serde_json::Value = resp.json().await.unwrap_or_default();
                anyhow::bail!("groq tts {status}: {}", v.pointer("/error/message").and_then(|m| m.as_str()).unwrap_or("failed"));
            }
            Ok(resp)
        })
    }
}
