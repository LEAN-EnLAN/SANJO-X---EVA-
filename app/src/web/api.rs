//! `/api/*`: JSON (or CBOR with `Accept: application/cbor`) for machines,
//! small HTML fragments for htmx, and Eva's pipelines (chat, tools, STT, TTS,
//! vision). The Groq key never leaves the server: the browser only ever
//! talks to these endpoints.

use super::pages::{self, resolve_seat, SeatQ};
use super::views::*;
use crate::eva_agent::{self, speech, tools, vision};
use crate::model::{ChatMsg, EvaMood, LinkMessage, Scenario};
use crate::state::Storm;
use crate::protocol::profile::LinkProfile;
use crate::state::{Shared, UiEvent};
use axum::body::{Body, Bytes};
use axum::extract::{Multipart, Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Json, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// JSON by default, CBOR when the client asks for it (smaller on slow links).
pub fn negotiate<T: Serialize>(headers: &HeaderMap, v: &T) -> Response {
    let wants_cbor = headers.get(header::ACCEPT).and_then(|a| a.to_str().ok()).is_some_and(|a| a.contains("application/cbor"));
    if wants_cbor {
        let mut buf = Vec::new();
        if ciborium::into_writer(v, &mut buf).is_ok() {
            return ([(header::CONTENT_TYPE, "application/cbor")], buf).into_response();
        }
    }
    Json(serde_json::to_value(v).unwrap_or(Value::Null)).into_response()
}

#[derive(Serialize)]
struct PassengerOut<'a> {
    seat: &'a str,
    name: &'a str,
    status: crate::model::Status,
    group: crate::model::Group,
    vitals: crate::protocol::delta::VitalsSample,
    eva_line: &'a str,
    last_ms: u64,
}

pub async fn state_json(State(state): State<Shared>, headers: HeaderMap) -> Response {
    let w = state.world.read().expect("world");
    let passengers: Vec<PassengerOut> = w.passengers.iter().map(|p| PassengerOut { seat: &p.seat, name: &p.name, status: p.status, group: p.group, vitals: p.vitals, eva_line: &p.eva_line, last_ms: p.last_ms }).collect();
    let (signed, sent, acked) = state.chain.lock().expect("chain").counts();
    let v = json!({
        "role": state.cfg.role,
        "node": state.cfg.node_name,
        "ship_time_ms": state.now(),
        "passengers": passengers,
        "alerts": w.alerts,
        "cases": w.cases,
        "storm": w.storm,
        "link": w.link,
        "chain": {"signed": signed, "sent": sent, "acked": acked},
        "mesh": state.mesh.views(state.clock.real_ms()),
        "agent": {"mode": state.agent.mode_label(), "chat_model": state.agent.chat_model, "stt_model": state.agent.stt_model, "tts_model": state.agent.tts_model, "vision_model": state.agent.vision_model},
    });
    drop(w);
    negotiate(&headers, &v)
}

pub async fn passenger_json(State(state): State<Shared>, Path(seat): Path<String>, headers: HeaderMap) -> Response {
    let w = state.world.read().expect("world");
    match w.passenger(&seat) {
        Some(p) => {
            let history: Vec<_> = p.history.iter().collect();
            let v = json!({"passenger": p, "history_15s": history});
            drop(w);
            negotiate(&headers, &v)
        }
        None => (StatusCode::NOT_FOUND, "unknown seat").into_response(),
    }
}

#[derive(Deserialize)]
pub struct ChainQ {
    pub from: Option<u64>,
    pub limit: Option<usize>,
}

pub async fn chain_json(State(state): State<Shared>, Query(q): Query<ChainQ>, headers: HeaderMap) -> Response {
    let c = state.chain.lock().expect("chain");
    let from = q.from.unwrap_or(0) as usize;
    let recs: Vec<_> = c.records.iter().skip(from).take(q.limit.unwrap_or(200).min(2000)).cloned().collect();
    drop(c);
    negotiate(&headers, &recs)
}

pub async fn chain_verify(State(state): State<Shared>) -> Json<Value> {
    let rep = state.chain.lock().expect("chain").verify();
    let m = state.mirror.lock().expect("mirror");
    Json(json!({"chain": rep, "mirror": {"records": m.records.len(), "verified": m.verified.len(), "failed": m.failed}}))
}

pub async fn mesh_json(State(state): State<Shared>) -> Json<Value> {
    Json(json!({"nodes": state.mesh.views(state.clock.real_ms()), "relayed_frames": state.mesh.relayed_frames.load(std::sync::atomic::Ordering::Relaxed)}))
}

pub async fn link_json(State(state): State<Shared>) -> Json<Value> {
    let w = state.world.read().expect("world");
    Json(json!({"status": w.link, "profiles": LinkProfile::all()}))
}

pub async fn protocol_sizes() -> Json<Value> {
    let rows: Vec<Value> = crate::protocol::message_size_table().into_iter().map(|(n, b, p)| json!({"message": n, "bytes": b, "payload_bytes": p})).collect();
    Json(json!({"version": crate::protocol::frame::VERSION, "messages": rows}))
}

pub async fn tools_json() -> Json<Value> {
    Json(tools::openai_tools())
}

// ---------------------------------------------------------------- Eva

#[derive(Deserialize)]
pub struct ChatForm {
    pub seat: Option<String>,
    pub text: String,
}

pub async fn eva_chat(State(state): State<Shared>, headers: HeaderMap, axum::Form(f): axum::Form<ChatForm>) -> Response {
    let seat = resolve_seat(&state, &SeatQ { seat: f.seat.clone() }, &headers);
    let text: String = f.text.trim().chars().take(1000).collect();
    if text.is_empty() {
        return Html(render(&ChatList::build(&state, &seat))).into_response();
    }
    eva_agent::converse(&state, &seat, &text).await;
    Html(render(&ChatList::build(&state, &seat))).into_response()
}

#[derive(Deserialize)]
pub struct ToolForm {
    pub seat: Option<String>,
    pub name: String,
    pub args: Option<String>,
}

/// A person triggers a tool from a button or chip.
pub async fn eva_tool(State(state): State<Shared>, headers: HeaderMap, axum::Form(f): axum::Form<ToolForm>) -> Response {
    let seat = resolve_seat(&state, &SeatQ { seat: f.seat.clone() }, &headers);
    let args: Value = f.args.as_deref().and_then(|a| serde_json::from_str(a).ok()).unwrap_or(json!({}));
    let r = tools::execute(&state, &seat, &f.name, &args, tools::Origin::User);
    let now = state.now();
    state.world.write().expect("world").chats.push(ChatMsg { ts_ms: now, seat: seat.clone(), from: "eva".into(), label: None, text: r.text.clone() });
    state.set_mood(Some(&seat), if r.ok { EvaMood::Happy } else { EvaMood::Concerned });
    state.emit(UiEvent::Chat { seat: seat.clone() });
    let target = headers.get("hx-target").and_then(|v| v.to_str().ok()).unwrap_or("");
    if target == "toast" {
        return Html(toast(&r.text, r.ok)).into_response();
    }
    Html(render(&ChatList::build(&state, &seat))).into_response()
}

pub fn toast(text: &str, ok: bool) -> String {
    let tone = if ok { "border-ok/40 bg-ok/10 text-ok" } else { "border-critical/50 bg-critical/15 text-critical" };
    let esc = text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    format!(r#"<p class="toast pointer-events-auto rounded-full border px-4 py-2 text-sm {tone}" data-toast>{esc}</p>"#)
}

async fn read_multipart(mut mp: Multipart) -> (Vec<(String, String, Vec<u8>)>, std::collections::HashMap<String, String>) {
    let mut files = Vec::new();
    let mut fields = std::collections::HashMap::new();
    let mut total = 0usize;
    while let Ok(Some(field)) = mp.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        let mime = field.content_type().unwrap_or("application/octet-stream").to_string();
        let is_file = field.file_name().is_some();
        let fname = field.file_name().unwrap_or("upload").to_string();
        let Ok(bytes) = field.bytes().await else { break };
        total += bytes.len();
        if total > 25 * 1024 * 1024 {
            break;
        }
        if is_file {
            files.push((fname, mime, bytes.to_vec()));
        } else {
            fields.insert(name, String::from_utf8_lossy(&bytes).to_string());
        }
    }
    (files, fields)
}

/// Browser `MediaRecorder` upload → Groq Whisper → `{ "text": … }`.
pub async fn eva_stt(State(state): State<Shared>, mp: Multipart) -> Response {
    let Some(pipe) = speech::stt(&state.agent) else {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error": "speech-to-text needs GROQ_API_KEY on the server", "fallback": "type, or use the browser's own dictation"}))).into_response();
    };
    let (files, _fields) = read_multipart(mp).await;
    let Some((filename, mime, bytes)) = files.into_iter().next() else {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "no audio"}))).into_response();
    };
    state.set_mood(None, EvaMood::Listening);
    match pipe.transcribe(&state.http, speech::AudioInput { bytes, filename, mime }).await {
        Ok(text) => Json(json!({"text": text, "model": pipe.name()})).into_response(),
        Err(e) => {
            tracing::warn!("stt failed: {e}");
            (StatusCode::BAD_GATEWAY, Json(json!({"error": "transcription failed"}))).into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct TtsForm {
    pub text: String,
    pub seat: Option<String>,
}

/// Groq TTS streamed back as WAV. Without a key: 503 + speechSynthesis fallback.
pub async fn eva_tts(State(state): State<Shared>, axum::Form(f): axum::Form<TtsForm>) -> Response {
    let Some(pipe) = speech::tts(&state.agent) else {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"fallback": "speechSynthesis"}))).into_response();
    };
    if let Some(s) = &f.seat {
        state.set_mood(Some(s), EvaMood::Speaking);
    }
    match pipe.speak(&state.http, f.text).await {
        Ok(resp) => {
            let ct = resp.headers().get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("audio/wav").to_string();
            let body = Body::from_stream(resp.bytes_stream());
            ([(header::CONTENT_TYPE, ct), (header::CACHE_CONTROL, "no-store".to_string())], body).into_response()
        }
        Err(e) => {
            tracing::warn!("tts failed: {e}");
            (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"fallback": "speechSynthesis"}))).into_response()
        }
    }
}

/// Photo(s) or sampled video frames → vision pipeline. `task=meal` logs a meal.
pub async fn eva_vision(State(state): State<Shared>, headers: HeaderMap, mp: Multipart) -> Response {
    let (files, fields) = read_multipart(mp).await;
    let seat = resolve_seat(&state, &SeatQ { seat: fields.get("seat").cloned() }, &headers);
    let task = fields.get("task").cloned().unwrap_or_else(|| "general".into());
    let question = fields.get("question").cloned().unwrap_or_default();
    let images: Vec<vision::ImageInput> = files.into_iter().filter(|(_, m, _)| m.starts_with("image/")).take(vision::MAX_IMAGES).map(|(_, mime, bytes)| vision::ImageInput { bytes, mime }).collect();
    if images.is_empty() {
        return (StatusCode::BAD_REQUEST, "no image").into_response();
    }
    let now = state.now();
    let n = images.len();
    state.world.write().expect("world").chats.push(ChatMsg { ts_ms: now, seat: seat.clone(), from: "me".into(), label: None, text: format!("Sent {} {}", n, if fields.contains_key("video") { "video frames" } else if n == 1 { "photo" } else { "photos" }) });
    state.set_mood(Some(&seat), EvaMood::Thinking);
    let pipe = vision::pipeline(&state.agent);
    let text = match pipe.describe(&state.http, images, task.clone(), question).await {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("vision failed: {e}");
            "I could not read the image right now. It is saved in your log.".into()
        }
    };
    let reply = if task == "meal" {
        let start = text.find('{').unwrap_or(0);
        let end = text.rfind('}').map(|e| e + 1).unwrap_or(text.len());
        let v: Value = serde_json::from_str(&text[start..end]).unwrap_or(json!({"text": "Meal from a photo", "kcal": 500}));
        let r = tools::execute(&state, &seat, "log_meal", &json!({"text": v.get("text").and_then(|x| x.as_str()).unwrap_or("Meal from a photo"), "kcal": v.get("kcal").and_then(|x| x.as_i64()).unwrap_or(500), "from_photo": true}), tools::Origin::Eva);
        format!("{} {}", v.get("text").and_then(|x| x.as_str()).unwrap_or(""), r.text).trim().to_string()
    } else {
        state.log("agent", &seat, json!({"text": "Eva looked at an image", "model": pipe.name()}));
        text
    };
    state.world.write().expect("world").chats.push(ChatMsg { ts_ms: state.now(), seat: seat.clone(), from: "eva".into(), label: None, text: reply });
    state.set_mood(Some(&seat), EvaMood::Speaking);
    state.emit(UiEvent::Chat { seat: seat.clone() });
    Html(render(&ChatList::build(&state, &seat))).into_response()
}

// ---------------------------------------------------------------- forms

/// Journal form: repeated `labels` / `affects` keys, so parse pairs by hand.
pub async fn journal_post(State(state): State<Shared>, headers: HeaderMap, body: Bytes) -> Response {
    let pairs: Vec<(String, String)> = serde_urlencoded::from_bytes(&body).unwrap_or_default();
    let get = |k: &str| pairs.iter().find(|(a, _)| a == k).map(|(_, v)| v.clone());
    let all = |k: &str| pairs.iter().filter(|(a, _)| a == k).map(|(_, v)| v.clone()).collect::<Vec<_>>();
    let seat = resolve_seat(&state, &SeatQ { seat: get("seat") }, &headers);
    let valence: i64 = get("valence").and_then(|v| v.parse().ok()).unwrap_or(3).clamp(1, 5);
    let text = get("text").unwrap_or_default();
    let shared = get("shared").is_some();
    if text.trim().is_empty() {
        tools::execute(&state, &seat, "log_mood", &json!({"valence": valence, "labels": all("labels"), "affects": all("affects")}), tools::Origin::User);
    } else {
        tools::execute(&state, &seat, "add_journal_entry", &json!({"text": text, "valence": valence, "labels": all("labels"), "affects": all("affects"), "shared": shared}), tools::Origin::User);
    }
    state.set_mood(Some(&seat), EvaMood::Happy);
    Html(pages::journal_screen(&state, &seat, true)).into_response()
}

#[derive(Deserialize)]
pub struct HelpForm {
    pub seat: String,
    pub going: String,
}

pub async fn help_post(State(state): State<Shared>, axum::Form(f): axum::Form<HelpForm>) -> Response {
    let seat = f.seat.to_uppercase();
    eva_agent::helper_answer(&state, &seat, f.going == "yes");
    Html(pages::emergency_screen(&state, &seat)).into_response()
}

#[derive(Deserialize)]
pub struct SeatForm {
    pub seat: Option<String>,
}

pub async fn shelter_post(State(state): State<Shared>, headers: HeaderMap, axum::Form(f): axum::Form<SeatForm>) -> Response {
    let seat = resolve_seat(&state, &SeatQ { seat: f.seat }, &headers);
    {
        let mut w = state.world.write().expect("world");
        if let Some(s) = w.storm.as_mut() {
            if !s.in_shelter.contains(&seat) {
                s.in_shelter.push(seat.clone());
            }
        }
    }
    state.log("storm", &seat, json!({"text": format!("{seat} is in the shelter")}));
    state.emit(UiEvent::Tick);
    Html(pages::storm_screen(&state, &seat)).into_response()
}

#[derive(Deserialize)]
pub struct TextForm {
    pub text: String,
}

/// Ground → ship message (reliable Text frame on the ship's session).
pub async fn ground_message(State(state): State<Shared>, axum::Form(f): axum::Form<TextForm>) -> StatusCode {
    let text: String = f.text.trim().chars().take(500).collect();
    if text.is_empty() {
        return StatusCode::BAD_REQUEST;
    }
    let now = state.now();
    state.to_ship.lock().expect("to_ship").push(text.clone());
    state.world.write().expect("world").messages.push(LinkMessage { ts_ms: now, from: "Ground".into(), to: "ship".into(), text, channel: "ground".into() });
    state.emit(UiEvent::Tick);
    StatusCode::NO_CONTENT
}

/// Seam for external telemetry (a real band gateway, or a NASA Hermes /
/// XTCE dictionary adapter later): JSON vitals by seat.
#[derive(Deserialize)]
pub struct IngestVitals {
    pub seat: String,
    pub hr: Option<u16>,
    pub spo2: Option<u8>,
    pub resp: Option<u8>,
    pub skin_c: Option<f32>,
    pub motion: Option<u8>,
    pub quality: Option<u8>,
    pub base_batt: Option<u8>,
    pub screen_batt: Option<u8>,
}

pub async fn ingest_vitals(State(state): State<Shared>, Json(v): Json<IngestVitals>) -> StatusCode {
    let now = state.now();
    let mut w = state.world.write().expect("world");
    let Some(p) = w.passenger_mut(&v.seat) else { return StatusCode::NOT_FOUND };
    let mut s = p.vitals;
    if let Some(x) = v.hr {
        s.hr = x;
    }
    if let Some(x) = v.spo2 {
        s.spo2 = x;
    }
    if let Some(x) = v.resp {
        s.resp = x;
    }
    if let Some(x) = v.skin_c {
        s.skin_c10 = (x * 10.0).round() as i16;
    }
    if let Some(x) = v.motion {
        s.motion = x;
    }
    if let Some(x) = v.quality {
        s.quality = x;
    }
    if let Some(x) = v.base_batt {
        s.base_batt = x;
    }
    if let Some(x) = v.screen_batt {
        s.screen_batt = x;
    }
    crate::sim::ingest(p, s, now);
    StatusCode::NO_CONTENT
}

// ---------------------------------------------------------------- dev panel

pub fn dev_panel_html(state: &Shared, flash: &str) -> String {
    let (seats, current) = {
        let w = state.world.read().expect("world");
        (w.passengers.iter().map(|p| (p.seat.clone(), p.name.clone())).collect(), w.link_profile.name)
    };
    let profiles = LinkProfile::all().into_iter().map(|p| (p.name, p.label, p.name == current)).collect();
    render(&DevPanel {
        seats,
        profiles,
        nodes: state.mesh.views(state.clock.real_ms()),
        sizes: crate::protocol::message_size_table(),
        time: crate::clock::hhmm(state.now()),
        flash: flash.to_string(),
        role: state.cfg.role.name(),
        agent_mode: state.agent.mode_label(),
    })
}

pub async fn dev_panel(State(state): State<Shared>) -> Html<String> {
    Html(dev_panel_html(&state, ""))
}

#[derive(Deserialize, Default)]
pub struct DevForm {
    pub seat: Option<String>,
    pub profile: Option<String>,
    pub minutes: Option<u64>,
}

pub async fn dev_action(State(state): State<Shared>, Path(action): Path<String>, axum::Form(f): axum::Form<DevForm>) -> Html<String> {
    let seat = f.seat.clone().unwrap_or_else(|| "4B".into()).to_uppercase();
    let now = state.now();
    let flash = match action.as_str() {
        "cardiac" => {
            let mut w = state.world.write().expect("world");
            match w.passenger_mut(&seat) {
                Some(p) => {
                    p.scenario = Scenario::CardiacArrest { since_ms: now };
                    format!("Cardiac arrest started for {}. Eva's reflex rule opens the case in about 20 s (no pulse + no movement + no answer).", p.name)
                }
                None => "Unknown seat".into(),
            }
        }
        "close_case" => {
            eva_agent::close_case(&state, &seat);
            format!("Case for {seat} closed.")
        }
        "suit" => {
            let mut w = state.world.write().expect("world");
            match w.passenger_mut(&seat) {
                Some(p) => {
                    let on = !p.suit();
                    p.forced_suit = Some(on);
                    format!("Suit mode {} for {}. The sensor base runs on its own battery.", if on { "on" } else { "off" }, p.name)
                }
                None => "Unknown seat".into(),
            }
        }
        "battery" => {
            let mut w = state.world.write().expect("world");
            match w.passenger_mut(&seat) {
                Some(p) => {
                    p.forced_base_batt = Some(9);
                    format!("{}'s sensor base battery set to 9%.", p.name)
                }
                None => "Unknown seat".into(),
            }
        }
        "warm" => {
            let mut w = state.world.write().expect("world");
            match w.passenger_mut(&seat) {
                Some(p) => {
                    p.scenario = Scenario::Warm;
                    format!("{}'s skin is now warmer than usual.", p.name)
                }
                None => "Unknown seat".into(),
            }
        }
        "spo2" => {
            let mut w = state.world.write().expect("world");
            match w.passenger_mut(&seat) {
                Some(p) => {
                    p.vitals.spo2 = 86;
                    format!("Oxygen dip for {} (one reading).", p.name)
                }
                None => "Unknown seat".into(),
            }
        }
        "storm" => {
            {
                let mut w = state.world.write().expect("world");
                w.storm = Some(Storm { since_ms: now, ends_ms: now + 6 * 3_600_000, in_shelter: Vec::new() });
            }
            state.log("alarm", "ship", json!({"text": "Solar storm warning · everyone to the Deck B shelter", "code": "solar_storm"}));
            state.set_mood(None, EvaMood::Alert);
            state.emit(UiEvent::Storm);
            "Solar storm warning sent to every app.".into()
        }
        "storm_end" => {
            state.world.write().expect("world").storm = None;
            state.log("storm", "ship", json!({"text": "Solar storm warning ended"}));
            state.set_mood(None, EvaMood::Idle);
            state.emit(UiEvent::Storm);
            "Storm over.".into()
        }
        "profile" => {
            let name = f.profile.clone().unwrap_or_default();
            match LinkProfile::by_name(&name) {
                Some(p) => {
                    let label = p.label;
                    state.world.write().expect("world").link_profile = p;
                    state.log("link", "ship", json!({"text": format!("Uplink profile: {label}")}));
                    format!("Uplink profile: {label}.")
                }
                None => "Unknown profile".into(),
            }
        }
        "ff" => {
            let m = f.minutes.unwrap_or(10).min(24 * 60);
            state.clock.fast_forward(m * 60_000);
            format!("Clock moved forward {m} min.")
        }
        "reset" => {
            state.reset();
            "Demo reset.".into()
        }
        _ => "Unknown action".into(),
    };
    state.emit(UiEvent::Tick);
    Html(dev_panel_html(&state, &flash))
}
