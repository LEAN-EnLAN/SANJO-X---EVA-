//! **Vision pipeline**: image and short-video understanding.
//!
//! Groq's vision model (`qwen/qwen3.8-27b`, preview) takes up to 3 images per
//! request as base64 data URIs. Video is never uploaded whole: the browser
//! samples frames from the clip (3 evenly spaced stills, see
//! `static/js/eva.js` `sampleVideoFrames`) and sends them as images.
//!
//! Without a key, [`OfflineVision`] answers with a neutral placeholder so the
//! flow (photo → meal log) still works in a demo.

use super::config::PipelineConfig;
use super::AgentConfig;
use base64::Engine;
use futures_util::future::BoxFuture;
use serde_json::json;

pub const MAX_IMAGES: usize = 3;

#[derive(Debug, Clone)]
pub struct ImageInput {
    pub bytes: Vec<u8>,
    pub mime: String,
}

impl ImageInput {
    pub fn data_uri(&self) -> String {
        format!("data:{};base64,{}", self.mime, base64::engine::general_purpose::STANDARD.encode(&self.bytes))
    }
}

pub trait VisionPipeline: Send + Sync {
    fn name(&self) -> String;
    /// Describe what is in the images for `task` ("meal", "symptom", "general").
    fn describe<'a>(&'a self, http: &'a reqwest::Client, images: Vec<ImageInput>, task: String, question: String) -> BoxFuture<'a, anyhow::Result<String>>;
}

pub fn pipeline(cfg: &AgentConfig) -> Box<dyn VisionPipeline> {
    if cfg.vision.online_key().is_some() {
        Box::new(GroqVision { cfg: cfg.vision.clone() })
    } else {
        Box::new(OfflineVision)
    }
}

/// The instruction sent with the images. Every task carries the topic
/// guard's scope (ADR-012): an off-topic image is answered `OFF_TOPIC` and
/// the app shows Eva's redirect instead.
pub fn task_prompt(task: &str, question: &str) -> String {
    match task {
        "meal" => "Describe this meal in one short line and estimate total kcal. Answer only JSON: {\"text\": \"...\", \"kcal\": 0, \"kind\": \"Breakfast|Lunch|Dinner|Snack\"}. If the image shows no food or drink, answer only {\"off_topic\": true}.".to_string(),
        "symptom" => format!("You give diagnostic support to a passenger in space, never a diagnosis. In at most 3 short plain lines, say what is visible and whether it needs the crew lead. {} Question: {question}", super::guard::VISION_SCOPE),
        _ => format!("You are Eva, a health copilot on a spacecraft. Describe what you see in at most 3 short plain lines. {} {question}", super::guard::VISION_SCOPE),
    }
}

pub struct GroqVision {
    pub cfg: PipelineConfig,
}

impl VisionPipeline for GroqVision {
    fn name(&self) -> String {
        format!("groq:{}", self.cfg.model)
    }
    fn describe<'a>(&'a self, http: &'a reqwest::Client, images: Vec<ImageInput>, task: String, question: String) -> BoxFuture<'a, anyhow::Result<String>> {
        Box::pin(async move {
            let key = self.cfg.online_key().ok_or_else(|| anyhow::anyhow!("no key"))?;
            let mut content = vec![json!({"type": "text", "text": task_prompt(&task, &question)})];
            for img in images.iter().take(MAX_IMAGES) {
                content.push(json!({"type": "image_url", "image_url": {"url": img.data_uri()}}));
            }
            let body = json!({"model": self.cfg.model, "messages": [{"role": "user", "content": content}], "temperature": 0.2, "max_completion_tokens": 300});
            let resp = http
                .post(self.cfg.url("/chat/completions"))
                .timeout(std::time::Duration::from_secs(self.cfg.timeout_s))
                .bearer_auth(key.expose())
                .json(&body)
                .send()
                .await
                .map_err(|e| self.cfg.error(e))?;
            let status = resp.status();
            let v: serde_json::Value = resp.json().await.unwrap_or_default();
            if !status.is_success() {
                return Err(self.cfg.error(format!("groq vision {status}: {}", v.pointer("/error/message").and_then(|m| m.as_str()).unwrap_or("failed"))));
            }
            Ok(v.pointer("/choices/0/message/content").and_then(|c| c.as_str()).unwrap_or("").trim().to_string())
        })
    }
}

pub struct OfflineVision;

impl VisionPipeline for OfflineVision {
    fn name(&self) -> String {
        "offline".into()
    }
    fn describe<'a>(&'a self, _http: &'a reqwest::Client, images: Vec<ImageInput>, task: String, _question: String) -> BoxFuture<'a, anyhow::Result<String>> {
        Box::pin(async move {
            Ok(match task.as_str() {
                "meal" => r#"{"text": "Meal from a photo (estimate, check it)", "kcal": 550}"#.to_string(),
                _ => format!("Got {} image(s). Image understanding needs the online model; I saved them to your log.", images.len()),
            })
        })
    }
}
