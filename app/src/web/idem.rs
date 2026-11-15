//! Idempotent POSTs for the PWA outbox.
//!
//! The service worker queues posts made while the hub is unreachable and
//! replays them later; two replays can race (the `online` event and the next
//! successful request both start one), so the same post may arrive twice.
//! A client that sends `x-eva-idem: <unique id per queued item>` (or the
//! standard `Idempotency-Key`) gets the first answer back for every repeat,
//! and the action runs once. Concurrent repeats wait for the first one.
//!
//! Kept in memory: 4,096 keys, one hour. Responses over 4 MB are not replayable.

use crate::state::Shared;
use axum::body::{Body, Bytes};
use axum::extract::{Request, State};
use axum::http::{HeaderValue, Method, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Notify;

const MAX_KEYS: usize = 4_096;
const TTL: Duration = Duration::from_secs(3_600);
const MAX_CACHED_BODY: usize = 4 * 1024 * 1024;

#[derive(Clone)]
struct Stored {
    status: StatusCode,
    content_type: Option<HeaderValue>,
    body: Bytes,
}

enum Entry {
    Pending(Arc<Notify>),
    Done(Instant, Stored),
}

#[derive(Default)]
pub struct IdemCache {
    map: Mutex<HashMap<String, Entry>>,
    order: Mutex<VecDeque<String>>,
}

impl std::fmt::Debug for IdemCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IdemCache({} keys)", self.map.lock().map(|m| m.len()).unwrap_or(0))
    }
}

fn key_of(req: &Request) -> Option<String> {
    if req.method() != Method::POST {
        return None;
    }
    let h = req.headers();
    let k = h.get("x-eva-idem").or_else(|| h.get("idempotency-key"))?.to_str().ok()?.trim();
    if k.is_empty() || k.len() > 200 {
        return None;
    }
    Some(format!("{} {}", req.uri().path(), k))
}

fn replay(s: &Stored) -> Response {
    let mut r = Response::builder().status(s.status);
    if let Some(ct) = &s.content_type {
        r = r.header(axum::http::header::CONTENT_TYPE, ct.clone());
    }
    r.header("x-eva-idem-replayed", "1").body(Body::from(s.body.clone())).unwrap_or_default()
}

/// Axum middleware: `from_fn_with_state(state, idem::layer)`.
pub async fn layer(State(state): State<Shared>, req: Request, next: Next) -> Response {
    let Some(key) = key_of(&req) else { return next.run(req).await };
    let cache = &state.idem;
    loop {
        let wait = {
            let mut map = cache.map.lock().expect("idem");
            match map.get(&key) {
                Some(Entry::Done(at, s)) if at.elapsed() < TTL => return replay(s),
                Some(Entry::Pending(n)) => Some(n.clone()),
                _ => {
                    map.insert(key.clone(), Entry::Pending(Arc::new(Notify::new())));
                    None
                }
            }
        };
        match wait {
            // a concurrent repeat: wait for the first one, then answer from the cache
            Some(n) => {
                let _ = tokio::time::timeout(Duration::from_secs(30), n.notified()).await;
            }
            None => break,
        }
    }
    let resp = next.run(req).await;
    let (parts, body) = resp.into_parts();
    let bytes = axum::body::to_bytes(body, MAX_CACHED_BODY).await;
    let notify = {
        let mut map = cache.map.lock().expect("idem");
        let n = match map.remove(&key) {
            Some(Entry::Pending(n)) => Some(n),
            _ => None,
        };
        // only successful and client-error answers are final; a 5xx may be retried
        if let (Ok(b), false) = (&bytes, parts.status.is_server_error()) {
            map.insert(key.clone(), Entry::Done(Instant::now(), Stored { status: parts.status, content_type: parts.headers.get(axum::http::header::CONTENT_TYPE).cloned(), body: b.clone() }));
            let mut order = cache.order.lock().expect("idem");
            order.push_back(key.clone());
            while order.len() > MAX_KEYS {
                if let Some(old) = order.pop_front() {
                    map.remove(&old);
                }
            }
        }
        n
    };
    if let Some(n) = notify {
        n.notify_waiters();
    }
    match bytes {
        Ok(b) => Response::from_parts(parts, Body::from(b)),
        Err(_) => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from("response too large to replay")).unwrap_or_default(),
    }
}
