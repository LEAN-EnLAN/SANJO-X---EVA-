//! End-to-end flows without a network: reflex → emergency, tools → chain,
//! privacy of journal text, and the HTTP router.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use eva::clock::MissionClock;
use eva::eva_agent::{self, tools, AgentConfig};
use eva::model::Scenario;
use eva::state::{AppState, NodeRole, ServerConfig, Shared};
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::ServiceExt;

fn node(tag: &str) -> Shared {
    let dir = std::env::temp_dir().join(format!("eva-test-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let cfg = ServerConfig { role: NodeRole::Ship, port: 0, peer: None, data_dir: dir, node_name: format!("test-{tag}") };
    AppState::new(cfg, Arc::new(MissionClock::new(96, 13, 58, 212)), AgentConfig::offline_only()).unwrap()
}

#[test]
fn cardiac_arrest_opens_case_with_nearest_helpers() {
    let s = node("cardiac");
    let now = s.now();
    {
        let mut w = s.world.write().unwrap();
        let p = w.passenger_mut("4B").unwrap();
        p.scenario = Scenario::CardiacArrest { since_ms: now - 30_000 };
        let raw = p.vitals;
        eva::sim::ingest(p, raw, now);
        assert_eq!(p.vitals.hr, 0);
    }
    eva_agent::on_tick(&s);
    let w = s.world.read().unwrap();
    let case = w.open_case_for("4B").expect("case opened by the reflex rule");
    let cpr = case.helpers.iter().find(|h| h.task == "CPR").unwrap();
    let aed = case.helpers.iter().find(|h| h.task == "AED").unwrap();
    assert_eq!(cpr.seat, "3B", "Leo is the nearest trained passenger");
    assert_eq!(aed.seat, "1B", "Tomás is nearest to the AED station");
    drop(w);
    let chain = s.chain.lock().unwrap();
    assert!(chain.records.iter().any(|r| r.kind == "alarm" && r.subject == "4B"));
    assert!(chain.verify().ok);
}

#[test]
fn tools_are_logged_and_private_text_stays_private() {
    let s = node("tools");
    let before = s.chain.lock().unwrap().len();
    let r = tools::execute(&s, "4B", "log_water", &serde_json::json!({"ml": 250}), tools::Origin::User);
    assert!(r.ok);
    let bad = tools::execute(&s, "4B", "log_water", &serde_json::json!({"ml": 99999}), tools::Origin::Eva);
    assert!(!bad.ok, "validation rejects out-of-range values");
    tools::execute(&s, "4B", "add_journal_entry", &serde_json::json!({"text": "I miss the sea"}), tools::Origin::User);
    let chain = s.chain.lock().unwrap();
    assert_eq!(chain.len(), before + 3, "every invocation is on the chain, even rejected ones");
    let all = serde_json::to_string(&chain.records).unwrap();
    assert!(!all.contains("I miss the sea"), "private journal text never goes on the chain");
    assert!(chain.verify().ok);
}

#[tokio::test]
async fn offline_chat_answers_with_tools() {
    let s = node("chat");
    let reply = eva_agent::converse(&s, "4B", "How am I?").await;
    assert!(reply.tools_used.contains(&"get_vitals".to_string()));
    assert!(reply.text.contains("pulse"));
    let help = eva_agent::converse(&s, "4A", "help me, chest pain").await;
    assert!(help.tools_used.contains(&"request_help".to_string()));
    assert!(s.world.read().unwrap().open_case_for("4A").is_some());
}

async fn get(s: &Shared, uri: &str) -> (StatusCode, String) {
    let r = eva::web::router(s.clone()).oneshot(Request::get(uri).body(Body::empty()).unwrap()).await.unwrap();
    let st = r.status();
    let b = r.into_body().collect().await.unwrap().to_bytes();
    (st, String::from_utf8_lossy(&b).to_string())
}

#[tokio::test]
async fn pages_api_and_exports_respond() {
    let s = node("http");
    let (st, body) = get(&s, "/app?seat=4B").await;
    assert_eq!(st, StatusCode::OK);
    assert!(body.contains("Good afternoon, Ana"));
    let (st, body) = get(&s, "/dash").await;
    assert_eq!(st, StatusCode::OK);
    assert!(body.contains("Passengers now"));
    for uri in ["/app/eva", "/app/emergency", "/app/journal", "/app/storm", "/app/records", "/dev/panel", "/export/report"] {
        assert_eq!(get(&s, uri).await.0, StatusCode::OK, "{uri}");
    }
    let (_, body) = get(&s, "/api/tools").await;
    assert_eq!(serde_json::from_str::<serde_json::Value>(&body).unwrap().as_array().unwrap().len(), 13);
    let (_, body) = get(&s, "/export/records.csv").await;
    assert!(body.starts_with("seq,ship_time"));
    let (st, body) = get(&s, "/static/js/htmx.min.js").await;
    assert_eq!(st, StatusCode::OK);
    assert!(body.len() > 10_000);
    assert_eq!(get(&s, "/sw.js").await.0, StatusCode::OK);
    assert_eq!(get(&s, "/manifest.webmanifest").await.0, StatusCode::OK);
}
