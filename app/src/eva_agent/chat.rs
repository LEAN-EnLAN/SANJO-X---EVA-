//! **Layer 2: chat / reasoning pipeline.** Groq chat completions (OpenAI
//! compatible) with tool calling, plus an offline rule-based pipeline used
//! when there is no key or no network. Layer 1 (reflex) never waits for this.

use super::tools::{self, Origin};
use super::AgentConfig;
use crate::state::Shared;
use futures_util::future::BoxFuture;
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct ChatTurn {
    /// "user" | "assistant"
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Default)]
pub struct ChatReply {
    pub text: String,
    pub tools_used: Vec<String>,
    pub navigate: Option<String>,
    pub model: String,
}

pub trait ChatPipeline: Send + Sync {
    fn name(&self) -> String;
    fn reply<'a>(&'a self, state: &'a Shared, seat: &'a str, history: Vec<ChatTurn>, user: String) -> BoxFuture<'a, anyhow::Result<ChatReply>>;
}

pub fn pipeline(cfg: &AgentConfig) -> Box<dyn ChatPipeline> {
    if cfg.online_key().is_some() {
        Box::new(GroqChat { cfg: cfg.clone() })
    } else {
        Box::new(OfflineChat)
    }
}

pub struct GroqChat {
    pub cfg: AgentConfig,
}

const MAX_TOOL_ROUNDS: usize = 3;

impl ChatPipeline for GroqChat {
    fn name(&self) -> String {
        format!("groq:{}", self.cfg.chat_model)
    }
    fn reply<'a>(&'a self, state: &'a Shared, seat: &'a str, history: Vec<ChatTurn>, user: String) -> BoxFuture<'a, anyhow::Result<ChatReply>> {
        Box::pin(async move {
            let key = self.cfg.online_key().ok_or_else(|| anyhow::anyhow!("no key"))?;
            let mut messages = vec![json!({"role": "system", "content": super::prompt::system_prompt(state, seat)})];
            for t in history.iter().rev().take(8).rev() {
                messages.push(json!({"role": t.role, "content": t.content}));
            }
            messages.push(json!({"role": "user", "content": user}));
            let mut out = ChatReply { model: self.cfg.chat_model.clone(), ..Default::default() };
            for _round in 0..=MAX_TOOL_ROUNDS {
                let body = json!({
                    "model": self.cfg.chat_model,
                    "messages": messages,
                    "tools": tools::openai_tools(),
                    "tool_choice": "auto",
                    "temperature": 0.3,
                    "max_completion_tokens": 500,
                });
                let resp = state.http.post(format!("{}/chat/completions", self.cfg.base_url)).bearer_auth(key.expose()).json(&body).send().await?;
                let status = resp.status();
                let v: Value = resp.json().await?;
                if !status.is_success() {
                    let msg = v.pointer("/error/message").and_then(|m| m.as_str()).unwrap_or("request failed");
                    anyhow::bail!("groq chat {status}: {msg}");
                }
                let mut msg = v.pointer("/choices/0/message").cloned().unwrap_or(Value::Null);
                let calls = msg.get("tool_calls").and_then(|c| c.as_array()).cloned().unwrap_or_default();
                if calls.is_empty() {
                    out.text = msg.get("content").and_then(|c| c.as_str()).unwrap_or("").trim().to_string();
                    return Ok(out);
                }
                if let Some(o) = msg.as_object_mut() {
                    o.remove("reasoning");
                }
                messages.push(msg);
                for c in calls {
                    let id = c.get("id").and_then(|x| x.as_str()).unwrap_or("call").to_string();
                    let name = c.pointer("/function/name").and_then(|x| x.as_str()).unwrap_or("").to_string();
                    let args: Value = c.pointer("/function/arguments").and_then(|x| x.as_str()).and_then(|s| serde_json::from_str(s).ok()).unwrap_or(json!({}));
                    let r = tools::execute(state, seat, &name, &args, Origin::Eva);
                    if r.navigate.is_some() {
                        out.navigate = r.navigate.clone();
                    }
                    out.tools_used.push(name);
                    messages.push(json!({"role": "tool", "tool_call_id": id, "content": serde_json::to_string(&json!({"ok": r.ok, "text": r.text, "data": r.data}))?}));
                }
            }
            out.text = "Done.".into();
            Ok(out)
        })
    }
}

/// Offline pipeline: keyword intents mapped to the same tools. Good enough for
/// a demo without network, and the fallback when the API fails.
pub struct OfflineChat;

impl ChatPipeline for OfflineChat {
    fn name(&self) -> String {
        "offline-rules".into()
    }
    fn reply<'a>(&'a self, state: &'a Shared, seat: &'a str, _history: Vec<ChatTurn>, user: String) -> BoxFuture<'a, anyhow::Result<ChatReply>> {
        Box::pin(async move { Ok(offline_reply(state, seat, &user)) })
    }
}

pub fn offline_reply(state: &Shared, seat: &str, user: &str) -> ChatReply {
    let u = user.to_lowercase();
    let has = |ws: &[&str]| ws.iter().any(|w| u.contains(w));
    let mut out = ChatReply { model: "offline-rules".into(), ..Default::default() };
    let call = |name: &str, args: Value, out: &mut ChatReply| {
        let r = tools::execute(state, seat, name, &args, Origin::Eva);
        out.tools_used.push(name.into());
        if r.navigate.is_some() {
            out.navigate = r.navigate.clone();
        }
        r
    };
    if has(&["emergency", "help me", "can't breathe", "cannot breathe", "chest pain", "collapsed", "not breathing"]) {
        let r = call("request_help", json!({"reason": user.chars().take(120).collect::<String>()}), &mut out);
        out.text = format!("Now: stay where you are, I'm getting help.\nWhy: you said \"{}\".\nHow I decided: your words, plus your band.\nNext: {}", user.chars().take(60).collect::<String>(), r.text);
    } else if has(&["unwell", "sick", "dizzy", "headache", "nausea", "cough"]) {
        let r = call("get_vitals", json!({}), &mut out);
        out.text = format!("Now: sit down and tell me what you feel.\nWhy: {}\nHow I decided: your band, live.\nNext: I can start a 30 s ECG, or call the crew lead.", r.text);
    } else if has(&["ecg"]) {
        out.text = call("start_ecg_check", json!({}), &mut out).text;
    } else if has(&["water", "drank", "glass"]) {
        out.text = call("log_water", json!({"ml": 250}), &mut out).text;
    } else if has(&["remind"]) {
        out.text = call("set_reminder", json!({"text": user, "in_min": 30}), &mut out).text;
    } else if has(&["earth", "ground", "houston"]) {
        out.text = call("send_to_ground", json!({"text": user}), &mut out).text;
    } else if has(&["export", "report", "pdf"]) {
        out.text = call("export_report", json!({"format": "html"}), &mut out).text;
    } else if has(&["summary", "my day", "log my day"]) {
        out.text = call("daily_summary", json!({}), &mut out).text;
    } else if has(&["week", "journal", "mood", "feel"]) {
        call("show_screen", json!({"screen": "journal"}), &mut out);
        out.text = "Let's write it down. How do you feel right now, from very unpleasant to very pleasant?".into();
    } else if has(&["how am i", "vitals", "pulse", "heart", "oxygen", "how am"]) {
        let r = call("get_vitals", json!({}), &mut out);
        out.text = format!("{}\nNothing to do. Keep your day.", r.text);
    } else {
        out.text = "I'm here. I can read your band, log water, meals and mood, start an ECG, or get help.".into();
    }
    out
}
