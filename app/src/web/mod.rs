//! HTTP layer: router, embedded static assets, plain or TLS serving.

pub mod api;
pub mod export;
pub mod idem;
pub mod pages;
pub mod sse;
pub mod ui;
pub mod views;

use crate::state::Shared;
use axum::extract::Path;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use rust_embed::RustEmbed;
use tower_http::compression::CompressionLayer;

/// Everything under `app/static/` is compiled into the binary.
#[derive(RustEmbed)]
#[folder = "static/"]
pub struct Assets;

fn asset(path: &str) -> Response {
    match Assets::get(path) {
        Some(f) => {
            let mime = f.metadata.mimetype().to_string();
            let cache = if path.ends_with(".css") || path.ends_with(".js") || path.ends_with(".webmanifest") { "no-cache" } else { "public, max-age=86400" };
            ([(header::CONTENT_TYPE, mime), (header::CACHE_CONTROL, cache.to_string())], f.data.into_owned()).into_response()
        }
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

async fn static_file(Path(path): Path<String>) -> Response {
    asset(&path)
}

async fn manifest() -> Response {
    let mut r = asset("manifest.webmanifest");
    r.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("application/manifest+json"));
    r
}

/// The service worker must be served from the root so its scope is `/`.
async fn service_worker() -> Response {
    let mut r = asset("sw.js");
    r.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/javascript; charset=utf-8"));
    r.headers_mut().insert("service-worker-allowed", HeaderValue::from_static("/"));
    r.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    r
}

async fn healthz() -> &'static str {
    "ok"
}

/// Largest body for forms and JSON posts.
pub const BODY_LIMIT: usize = 64 * 1024;
/// Largest body for `/api/eva/stt` and `/api/eva/vision` (audio, up to 3 photos).
pub const MEDIA_BODY_LIMIT: usize = 26 * 1024 * 1024;

pub fn router(state: Shared) -> Router {
    Router::new()
        .route("/", get(pages::root))
        .route("/app", get(pages::app_home))
        .route("/app/home", get(pages::app_home))
        .route("/app/eva", get(pages::app_eva))
        .route("/app/emergency", get(pages::app_emergency))
        .route("/app/journal", get(pages::app_journal))
        .route("/app/storm", get(pages::app_storm))
        .route("/app/records", get(pages::app_records))
        .route("/app/plan", get(pages::app_plan))
        .route("/app/food", get(pages::app_food))
        .route("/app/checkin", get(pages::app_checkin))
        .route("/app/ecg", get(pages::app_ecg))
        .route("/app/band", get(pages::app_band))
        .route("/app/vitals", get(pages::app_vitals))
        .route("/app/about", get(pages::app_about))
        .route("/app/tools", get(pages::app_tools))
        .route("/dash", get(pages::dash))
        .route("/sse/app", get(sse::app_sse))
        .route("/sse/dash", get(sse::dash_sse))
        .route("/link", get(crate::link::ws_handler))
        .route("/api/state", get(api::state_json))
        .route("/api/passengers/{seat}", get(api::passenger_json))
        .route("/api/chain", get(api::chain_json))
        .route("/api/chain/verify", get(api::chain_verify))
        .route("/api/mesh", get(api::mesh_json))
        .route("/api/link", get(api::link_json))
        .route("/api/protocol/sizes", get(api::protocol_sizes))
        .route("/api/tools", get(api::tools_json))
        .route("/api/eva/chat", post(api::eva_chat))
        .route("/api/eva/tool", post(api::eva_tool))
        .route("/api/eva/stt", post(api::eva_stt).layer(axum::extract::DefaultBodyLimit::max(MEDIA_BODY_LIMIT)))
        .route("/api/eva/tts", post(api::eva_tts))
        .route("/api/eva/vision", post(api::eva_vision).layer(axum::extract::DefaultBodyLimit::max(MEDIA_BODY_LIMIT)))
        .route("/api/journal", post(api::journal_post))
        .route("/api/help", post(api::help_post))
        .route("/api/shelter", post(api::shelter_post))
        .route("/api/ground/message", post(api::ground_message))
        .route("/api/ingest/vitals", post(api::ingest_vitals))
        .route("/dev/panel", get(api::dev_panel))
        .route("/api/dev/{action}", post(api::dev_action))
        .route("/export/records.csv", get(export::records_csv))
        .route("/export/records.json", get(export::records_json))
        .route("/export/vitals.csv", get(export::vitals_csv))
        .route("/export/journal.csv", get(export::journal_csv))
        .route("/export/report", get(export::report))
        .route("/static/{*path}", get(static_file))
        .route("/manifest.webmanifest", get(manifest))
        .route("/sw.js", get(service_worker))
        .route("/healthz", get(healthz))
        // forms and JSON are small; only voice and photo uploads may be large
        .layer(axum::extract::DefaultBodyLimit::max(BODY_LIMIT))
        .layer(axum::middleware::from_fn_with_state(state.clone(), idem::layer))
        .layer(CompressionLayer::new())
        .with_state(state)
}
