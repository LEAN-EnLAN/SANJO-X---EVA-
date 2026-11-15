//! Break tests for the HTTP layer: malformed and huge bodies on every POST,
//! path traversal on static routes, idempotent replays of queued posts.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use eva::clock::MissionClock;
use eva::eva_agent::AgentConfig;
use eva::state::{AppState, NodeRole, ServerConfig, Shared};
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::ServiceExt;

fn node(tag: &str) -> Shared {
    node_with(tag, AgentConfig::offline_only())
}

fn node_with(tag: &str, agent: AgentConfig) -> Shared {
    let dir = std::env::temp_dir().join(format!("eva-http-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let cfg = ServerConfig { role: NodeRole::Ship, port: 0, peer: None, data_dir: dir, node_name: format!("http-{tag}") };
    AppState::new(cfg, Arc::new(MissionClock::new(96, 13, 58, 212)), agent).unwrap()
}

async fn send(s: &Shared, req: Request<Body>) -> (StatusCode, String) {
    let r = eva::web::router(s.clone()).oneshot(req).await.unwrap();
    let st = r.status();
    let b = r.into_body().collect().await.map(|b| b.to_bytes()).unwrap_or_default();
    (st, String::from_utf8_lossy(&b).chars().take(300).collect())
}

const POSTS: &[&str] = &["/api/eva/chat", "/api/eva/tool", "/api/eva/stt", "/api/eva/tts", "/api/eva/vision", "/api/journal", "/api/help", "/api/shelter", "/api/ground/message", "/api/ingest/vitals", "/api/dev/cardiac", "/api/dev/reset"];

#[tokio::test]
async fn path_traversal_on_static_routes_is_refused() {
    let s = node("trav");
    for uri in [
        "/static/../Cargo.toml",
        "/static/..%2fCargo.toml",
        "/static/%2e%2e/Cargo.toml",
        "/static/%2e%2e%2fsrc%2fmain.rs",
        "/static/css/../../Cargo.toml",
        "/static/css/..%5c..%5cCargo.toml",
        "/static//etc/passwd",
        "/static/%2Fetc%2Fpasswd",
        "/static/..%252fCargo.toml",
        "/static/js/%00htmx.min.js",
        "/static/....//Cargo.toml",
    ] {
        let (st, body) = send(&s, Request::get(uri).body(Body::empty()).unwrap()).await;
        assert!(st == StatusCode::NOT_FOUND || st == StatusCode::BAD_REQUEST, "{uri} → {st}");
        assert!(!body.contains("[package]") && !body.contains("root:") && !body.contains("fn main"), "{uri} leaked a file: {body}");
    }
    let (st, _) = send(&s, Request::get("/static/js/htmx.min.js").body(Body::empty()).unwrap()).await;
    assert_eq!(st, StatusCode::OK);
}

#[tokio::test]
async fn malformed_bodies_on_every_post_never_500() {
    let s = node("malformed");
    let bodies: Vec<(&str, Vec<u8>)> = vec![
        ("application/json", b"{\"seat\": \"4B\", \"hr\": ".to_vec()),
        ("application/json", b"[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[".to_vec()),
        ("application/json", b"{\"seat\": 4, \"hr\": -1}".to_vec()),
        ("application/x-www-form-urlencoded", b"%%%%&&&==\xff\xfe".to_vec()),
        ("application/x-www-form-urlencoded", b"text=".to_vec()),
        ("multipart/form-data; boundary=x", b"--x\r\nContent-Disposition: form-data; name=\"a\"\r\n\r\nbroken".to_vec()),
        ("text/plain", vec![0u8; 100]),
        ("application/cbor", vec![0xff; 64]),
    ];
    for uri in POSTS {
        for (ct, b) in &bodies {
            let (st, body) = send(&s, Request::post(*uri).header("content-type", *ct).body(Body::from(b.clone())).unwrap()).await;
            // 503 from stt/tts without a key is the documented explicit fallback, not a crash
            let fallback = (uri.ends_with("/tts") || uri.ends_with("/stt")) && st == StatusCode::SERVICE_UNAVAILABLE && body.contains("fallback");
            assert!(!st.is_server_error() || fallback, "{uri} with {ct} → {st}: {body}");
        }
    }
}

#[tokio::test]
async fn huge_payloads_hit_the_body_limit() {
    // a key pointing nowhere, so the media routes really read their bodies
    let s = node_with("huge", AgentConfig::with_endpoint("http://127.0.0.1:9", "test-key"));
    let small_limit = 64 * 1024;
    for uri in POSTS {
        let big = vec![b'a'; small_limit * 2];
        let mut form = b"text=".to_vec();
        form.extend(&big);
        let ct = if uri.contains("ingest") { "application/json" } else { "application/x-www-form-urlencoded" };
        let is_media = uri.ends_with("/stt") || uri.ends_with("/vision");
        let (st, _) = send(&s, Request::post(*uri).header("content-type", ct).body(Body::from(form)).unwrap()).await;
        if !is_media {
            assert_eq!(st, StatusCode::PAYLOAD_TOO_LARGE, "{uri}: a 128 KB form must be refused");
        }
        // 30 MB to the media routes: refused too (limit 26 MB)
        if is_media {
            let huge = vec![b'x'; 30 * 1024 * 1024];
            let (st, _) = send(&s, Request::post(*uri).header("content-type", "multipart/form-data; boundary=x").body(Body::from(huge)).unwrap()).await;
            assert!(st == StatusCode::PAYLOAD_TOO_LARGE || st.is_client_error(), "{uri} 30 MB → {st}");
        }
    }
    // nothing above was logged as a journal entry or tool call
    assert!(s.chain.lock().unwrap().records.iter().all(|r| r.body.to_string().len() < 10_000));
}

async fn post_form(s: &Shared, uri: &str, body: &str, idem: Option<&str>) -> (StatusCode, String) {
    let mut req = Request::post(uri).header("content-type", "application/x-www-form-urlencoded").header("x-eva-replayed", "1");
    if let Some(k) = idem {
        req = req.header("x-eva-idem", k);
    }
    send(s, req.body(Body::from(body.to_string())).unwrap()).await
}

#[tokio::test]
async fn pwa_outbox_replayed_twice_is_idempotent() {
    let s = node("idem");
    let water = |s: &Shared| s.world.read().unwrap().passenger("4B").unwrap().water_ml;
    let tools = |s: &Shared| s.chain.lock().unwrap().records.iter().filter(|r| r.kind == "tool").count();
    let w0 = water(&s);
    let t0 = tools(&s);
    let body = "seat=4B&name=log_water&args=%7B%22ml%22%3A250%7D";
    // the same queued post replayed twice (two replay() runs racing), then a third time later
    let (a, ra) = post_form(&s, "/api/eva/tool", body, Some("outbox-4B-1731650000000-1")).await;
    let (b, rb) = post_form(&s, "/api/eva/tool", body, Some("outbox-4B-1731650000000-1")).await;
    let (c, _) = post_form(&s, "/api/eva/tool", body, Some("outbox-4B-1731650000000-1")).await;
    assert_eq!((a, b, c), (StatusCode::OK, StatusCode::OK, StatusCode::OK));
    assert_eq!(ra, rb, "the replay gets the first answer back");
    assert_eq!(water(&s), w0 + 250, "logged once");
    assert_eq!(tools(&s), t0 + 1, "one tool record on the chain");
    // a different queued item with the same content is a different action
    post_form(&s, "/api/eva/tool", body, Some("outbox-4B-1731650000000-2")).await;
    assert_eq!(water(&s), w0 + 500);
    // journal: same
    let j0 = s.world.read().unwrap().journal.len();
    for _ in 0..3 {
        post_form(&s, "/api/journal", "seat=4B&valence=4&text=Saw+the+aurora", Some("outbox-4B-j-9")).await;
    }
    assert_eq!(s.world.read().unwrap().journal.len(), j0 + 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_replays_of_the_same_item_run_once() {
    let s = node("race");
    let w0 = s.world.read().unwrap().passenger("4B").unwrap().water_ml;
    let body = "seat=4B&name=log_water&args=%7B%22ml%22%3A300%7D";
    let mut handles = Vec::new();
    for _ in 0..8 {
        let s = s.clone();
        handles.push(tokio::spawn(async move { post_form(&s, "/api/eva/tool", body, Some("outbox-race-1")).await }));
    }
    let mut answers = Vec::new();
    for h in handles {
        answers.push(h.await.unwrap());
    }
    assert!(answers.iter().all(|a| a == &answers[0]), "every racer gets the same answer");
    assert_eq!(s.world.read().unwrap().passenger("4B").unwrap().water_ml, w0 + 300, "8 racing replays, logged once");
}
