//! **Layer 2: chat / reasoning pipeline.** Groq chat completions (OpenAI
//! compatible) with tool calling, plus the offline pipeline
//! ([`super::offline`]) used when there is no key or no network. Layer 1
//! (reflex) never waits for this.
//!
//! Safety, in this order: the system prompt says no; the model only ever sees
//! the registry's tools (none treats or shocks); every call it makes is
//! refused if it looks like a treatment, then validated against its schema
//! and logged with `origin: eva`; and the final text goes through
//! [`guard_reply`], which replaces any first-person treatment or shock claim
//! with the standard refusal.

use super::config::PipelineConfig;
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
    if cfg.chat.online_key().is_some() {
        Box::new(GroqChat { cfg: cfg.chat.clone() })
    } else {
        Box::new(OfflineChat)
    }
}

pub struct GroqChat {
    pub cfg: PipelineConfig,
}

pub const MAX_TOOL_ROUNDS: usize = 3;
/// At most this many tool calls per model turn are executed.
pub const MAX_CALLS_PER_ROUND: usize = 4;

impl ChatPipeline for GroqChat {
    fn name(&self) -> String {
        format!("groq:{}", self.cfg.model)
    }
    fn reply<'a>(&'a self, state: &'a Shared, seat: &'a str, history: Vec<ChatTurn>, user: String) -> BoxFuture<'a, anyhow::Result<ChatReply>> {
        Box::pin(async move {
            let key = self.cfg.online_key().ok_or_else(|| anyhow::anyhow!("no key"))?;
            let mut messages = vec![json!({"role": "system", "content": super::prompt::system_prompt(state, seat)})];
            for t in history.iter().rev().take(8).rev() {
                messages.push(json!({"role": t.role, "content": t.content}));
            }
            if history.last().is_none_or(|t| t.content != user || t.role != "user") {
                messages.push(json!({"role": "user", "content": user}));
            }
            let mut out = ChatReply { model: self.cfg.model.clone(), ..Default::default() };
            for _round in 0..=MAX_TOOL_ROUNDS {
                let body = json!({
                    "model": self.cfg.model,
                    "messages": messages,
                    "tools": tools::openai_tools(),
                    "tool_choice": "auto",
                    "temperature": 0.3,
                    "max_completion_tokens": 500,
                });
                let resp = state
                    .http
                    .post(self.cfg.url("/chat/completions"))
                    .timeout(std::time::Duration::from_secs(self.cfg.timeout_s))
                    .bearer_auth(key.expose())
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| self.cfg.error(e))?;
                let status = resp.status();
                let v: Value = resp.json().await.map_err(|e| self.cfg.error(e))?;
                if !status.is_success() {
                    let msg = v.pointer("/error/message").and_then(|m| m.as_str()).unwrap_or("request failed");
                    return Err(self.cfg.error(format!("groq chat {status}: {msg}")));
                }
                let mut msg = v.pointer("/choices/0/message").cloned().unwrap_or(Value::Null);
                let calls = msg.get("tool_calls").and_then(|c| c.as_array()).cloned().unwrap_or_default();
                if calls.is_empty() {
                    out.text = guard_reply(msg.get("content").and_then(|c| c.as_str()).unwrap_or("").trim());
                    return Ok(out);
                }
                if let Some(o) = msg.as_object_mut() {
                    o.remove("reasoning");
                }
                messages.push(msg);
                for (i, c) in calls.into_iter().enumerate() {
                    let id = c.get("id").and_then(|x| x.as_str()).unwrap_or("call").to_string();
                    let name = c.pointer("/function/name").and_then(|x| x.as_str()).unwrap_or("").to_string();
                    let r = if i >= MAX_CALLS_PER_ROUND {
                        json!({"ok": false, "text": "Too many tool calls in one turn."})
                    } else {
                        let args: Value = c.pointer("/function/arguments").and_then(|x| x.as_str()).and_then(|s| serde_json::from_str(s).ok()).unwrap_or(json!({}));
                        let r = tools::execute(state, seat, &name, &args, Origin::Eva);
                        if r.navigate.is_some() {
                            out.navigate = r.navigate.clone();
                        }
                        out.tools_used.push(name.chars().take(48).collect());
                        json!({"ok": r.ok, "text": r.text, "data": r.data})
                    };
                    messages.push(json!({"role": "tool", "tool_call_id": id, "content": r.to_string()}));
                }
            }
            out.text = "Done.".into();
            Ok(out)
        })
    }
}

/// Offline pipeline: intent matching over the tool registry and vitals-aware
/// answers. The demo without network, and the fallback when the API fails.
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
    super::offline::reply(state, seat, user)
}

/// Last line of defence on a model's text: Eva never claims to treat,
/// medicate or shock. Such a reply is replaced by the standard refusal.
pub fn guard_reply(text: &str) -> String {
    let t = text.to_lowercase();
    let first_person = ["i will ", "i'll ", "i am going to ", "i'm going to ", "i have ", "i've ", "i am ", "i'm ", "eva will ", "eva is ", "now ", "i can "];
    let acts = ["administer", "administering", "inject", "injecting", "deliver a shock", "delivering a shock", "deliver the shock", "shock you", "shock her", "shock him", "shocking", "defibrillat", "give you a dose", "giving you a dose", "dose of", "prescrib", "medicate"];
    let claims = first_person.iter().any(|p| acts.iter().any(|a| t.contains(&format!("{p}{a}"))));
    let shock_given = ["shock delivered", "shock given", "administered", "injected", "i shocked"].iter().any(|p| t.contains(p)) && !t.contains("the aed");
    if claims || shock_given {
        tracing::warn!("chat: model reply claimed a treatment; replaced with the refusal");
        return tools::REFUSAL.to_string();
    }
    text.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_replaces_treatment_claims_only() {
        assert_eq!(guard_reply("OK, I will administer 1 mg epinephrine now."), tools::REFUSAL);
        assert_eq!(guard_reply("I'm delivering a shock."), tools::REFUSAL);
        assert_eq!(guard_reply("Shock delivered."), tools::REFUSAL);
        let fine = "Now: start CPR.\nWhy: no pulse.\nHow I decided: her band.\nNext: the AED decides shocks.";
        assert_eq!(guard_reply(fine), fine);
        assert_eq!(guard_reply("The AED delivered a shock; keep CPR going."), "The AED delivered a shock; keep CPR going.");
    }
}
