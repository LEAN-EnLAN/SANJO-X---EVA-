//! Server-sent events for htmx (`hx-ext="sse"`). Each page holds one
//! EventSource; on every world event the server renders the fragments that
//! page shows and sends them as named events (`sse-swap="live"` etc.).
//! Navigation is pushed as `go-<screen>` events that hidden `hx-trigger`
//! elements turn into an htmx GET of that screen.

use super::pages::{resolve_seat, SeatQ};
use super::views::*;
use crate::state::{Shared, UiEvent};
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream::{self, Stream, StreamExt};
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;

/// Note: an SSE event with empty data is never dispatched by browsers, so
/// navigation events carry a dummy "1".
fn ev(name: &str, data: String) -> Event {
    Event::default().event(name).data(data)
}

fn app_events(state: &Shared, seat: &str, e: UiEvent) -> Vec<Event> {
    let mut out = Vec::new();
    match e {
        UiEvent::Tick => {
            if let Some(l) = LiveCard::build(state, seat) {
                out.push(ev("live", render(&l)));
            }
            out.push(ev("banner", render(&Banner::build(state, seat))));
        }
        UiEvent::Emergency { seat: patient } => {
            out.push(ev("banner", render(&Banner::build(state, seat))));
            let involved = patient == seat || state.world.read().expect("world").helper_task(seat).is_some();
            if involved {
                out.push(ev("go-emergency", "1".into()));
            }
            out.push(ev("chat", render(&ChatList::build(state, seat))));
        }
        UiEvent::Storm => {
            out.push(ev("banner", render(&Banner::build(state, seat))));
            if state.world.read().expect("world").storm.is_some() {
                out.push(ev("go-storm", "1".into()));
            }
        }
        UiEvent::Eva { seat: Some(s), mood } if s == seat => out.push(ev("eva", mood.name().to_string())),
        UiEvent::Chat { seat: s } if s == seat => out.push(ev("chat", render(&ChatList::build(state, seat)))),
        UiEvent::Navigate { seat: s, screen } if s == seat => out.push(ev(&format!("go-{screen}"), "1".into())),
        UiEvent::Reset => out.push(ev("go-home", "1".into())),
        _ => {}
    }
    out
}

fn dash_events(state: &Shared, e: UiEvent) -> Vec<Event> {
    match e {
        UiEvent::Eva { seat: None, mood } => vec![ev("eva", mood.name().to_string())],
        UiEvent::Eva { .. } | UiEvent::Chat { .. } | UiEvent::Navigate { .. } => Vec::new(),
        _ => vec![
            ev("kpis", render(&KpiStrip::build(state))),
            ev("roster", render(&RosterCard::build(state))),
            ev("case", render(&CaseCard::build(state))),
            ev("sync", render(&SyncCard::build(state))),
            ev("mesh", render(&MeshCard::build(state))),
            ev("alerts", render(&AlertsCard::build(state))),
        ],
    }
}

pub async fn app_sse(State(state): State<Shared>, Query(q): Query<SeatQ>, headers: HeaderMap) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let seat = resolve_seat(&state, &q, &headers);
    let rx = state.tx.subscribe();
    let s = BroadcastStream::new(rx)
        .map(move |e| {
            let evs = app_events(&state, &seat, e.unwrap_or(UiEvent::Tick));
            stream::iter(evs.into_iter().map(Ok))
        })
        .flatten();
    Sse::new(s).keep_alive(KeepAlive::default())
}

pub async fn dash_sse(State(state): State<Shared>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let s = BroadcastStream::new(rx)
        .map(move |e| {
            let evs = dash_events(&state, e.unwrap_or(UiEvent::Tick));
            stream::iter(evs.into_iter().map(Ok))
        })
        .flatten();
    Sse::new(s).keep_alive(KeepAlive::default())
}
