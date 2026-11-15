//! HTML pages: `/app` (astronaut PWA) and `/dash` (crew / ground dashboard).
//! Screens render as fragments for htmx navigation (`HX-Request`) and as full
//! pages otherwise, so every URL works on its own and offline from the cache.

use super::views::*;
use crate::clock;
use crate::state::{NodeRole, Shared};
use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, HeaderValue};
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct SeatQ {
    pub seat: Option<String>,
}

pub const DEFAULT_SEAT: &str = "4B";

/// Seat from the query, else the `eva_seat` cookie, else Ana (4B).
pub fn resolve_seat(state: &Shared, q: &SeatQ, headers: &HeaderMap) -> String {
    let from_cookie = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()).and_then(|c| c.split(';').find_map(|kv| kv.trim().strip_prefix("eva_seat=").map(|s| s.to_string())));
    let seat = q.seat.clone().or(from_cookie).unwrap_or_else(|| DEFAULT_SEAT.into()).to_uppercase();
    if state.world.read().expect("world").passenger(&seat).is_some() {
        seat
    } else {
        DEFAULT_SEAT.into()
    }
}

fn is_htmx(h: &HeaderMap) -> bool {
    h.get("hx-request").is_some() && h.get("hx-history-restore-request").is_none()
}

fn with_cookie(seat: &str, body: String) -> Response {
    let mut r = Html(body).into_response();
    if let Ok(v) = HeaderValue::from_str(&format!("eva_seat={seat}; Path=/; Max-Age=31536000; SameSite=Lax")) {
        r.headers_mut().insert(header::SET_COOKIE, v);
    }
    r
}

fn page(state: &Shared, seat: &str, tab: &'static str, screen: String, headers: &HeaderMap) -> Response {
    if is_htmx(headers) {
        return with_cookie(seat, screen);
    }
    let seats = state.world.read().expect("world").passengers.iter().map(|p| (p.seat.clone(), p.name.clone())).collect();
    with_cookie(seat, render(&AppPage { c: common(state, seat), tab, screen, seats }))
}

pub fn home_screen(state: &Shared, seat: &str) -> String {
    let live = LiveCard::build(state, seat).map(|l| render(&l)).unwrap_or_default();
    let banner = render(&Banner::build(state, seat));
    let now = state.now();
    let next = state.world.read().expect("world").reminders.iter().filter(|r| r.seat == seat && !r.done && r.at_ms >= now).min_by_key(|r| r.at_ms).map(|r| (clock::hhmm(r.at_ms), r.text.clone()));
    render(&HomeScreen { c: common(state, seat), live, banner, next, water: tool_vals(seat, "log_water", serde_json::json!({"ml": 250})) })
}

pub async fn app_home(State(state): State<Shared>, Query(q): Query<SeatQ>, headers: HeaderMap) -> Response {
    let seat = resolve_seat(&state, &q, &headers);
    let screen = home_screen(&state, &seat);
    page(&state, &seat, "home", screen, &headers)
}

pub fn eva_screen(state: &Shared, seat: &str) -> String {
    let live = state.world.read().expect("world").passenger(seat).cloned();
    let Some(live) = live else { return String::new() };
    let chat = render(&ChatList::build(state, seat));
    render(&EvaScreen { c: common(state, seat), live, chat })
}

pub async fn app_eva(State(state): State<Shared>, Query(q): Query<SeatQ>, headers: HeaderMap) -> Response {
    let seat = resolve_seat(&state, &q, &headers);
    let s = eva_screen(&state, &seat);
    page(&state, &seat, "eva", s, &headers)
}

pub fn emergency_screen(state: &Shared, seat: &str) -> String {
    let w = state.world.read().expect("world");
    let case = w.open_case().cloned();
    let me = w.passenger(seat).cloned();
    let mut mode = "none";
    let mut distance = String::new();
    let mut direction = String::new();
    let mut aed_helper = String::new();
    let mut step = 1;
    if let (Some(c), Some(me)) = (&case, &me) {
        aed_helper = c.helpers.iter().find(|h| h.task == "AED").map(|h| h.name.split(' ').next().unwrap_or("").to_string()).unwrap_or_default();
        if c.seat == seat {
            mode = "patient";
        } else if let Some(h) = c.helpers.iter().find(|h| h.seat == seat) {
            let answered = c.steps.iter().any(|s| s.title.starts_with(me.first_name()) && (s.title.contains("on the way") || s.title.contains("arrived")));
            mode = match (h.task.as_str(), answered) {
                ("AED", _) => "helper_aed",
                (_, true) => "cpr",
                _ => "helper_cpr",
            };
            if let Some(pat) = w.passenger(&c.seat) {
                let dx = pat.pos.0 - me.pos.0;
                distance = format!("{:.0} m", ((dx).powi(2) + (pat.pos.1 - me.pos.1).powi(2)).sqrt());
                let rows = (dx / 3.0).round() as i32;
                let side = if pat.seat.ends_with('B') { "right" } else { "left" };
                direction = match rows {
                    0 => format!("Same row. Seat {} is on your {side}.", pat.seat),
                    r if r > 0 => format!("Go forward {} row{}. Seat {} is on your {side}.", r, if r == 1 { "" } else { "s" }, pat.seat),
                    r => format!("Go back {} row{}. Seat {} is on your {side}.", -r, if r == -1 { "" } else { "s" }, pat.seat),
                };
            }
            step = 1 + c.steps.iter().filter(|s| s.title.contains("CPR started") || s.title.contains("AED attached")).count();
        } else {
            mode = "other";
        }
    }
    drop(w);
    render(&EmergencyScreen { c: common(state, seat), mode, case, distance, direction, aed_helper, step: step.min(4) })
}

pub async fn app_emergency(State(state): State<Shared>, Query(q): Query<SeatQ>, headers: HeaderMap) -> Response {
    let seat = resolve_seat(&state, &q, &headers);
    let s = emergency_screen(&state, &seat);
    page(&state, &seat, "emergency", s, &headers)
}

pub fn journal_screen(state: &Shared, seat: &str, saved: bool) -> String {
    let now = state.now();
    let w = state.world.read().expect("world");
    let mut entries: Vec<crate::model::JournalEntry> = w.journal.iter().filter(|j| j.seat == seat).cloned().collect();
    entries.sort_by_key(|e| std::cmp::Reverse(e.ts_ms));
    let today = now / clock::DAY_MS;
    let names = ["M", "T", "W", "T", "F", "S", "S"];
    let week = (0..7u64)
        .map(|i| {
            let day = today.saturating_sub(6 - i);
            let moods: Vec<u8> = w.journal.iter().filter(|j| j.seat == seat && j.ts_ms / clock::DAY_MS == day).map(|j| j.valence).collect();
            (names[((day + 1) % 7) as usize].to_string(), moods, day == today)
        })
        .collect();
    drop(w);
    render(&JournalScreen { c: common(state, seat), entries, week, labels: FEELINGS.to_vec(), affects: AFFECTS.to_vec(), saved })
}

pub async fn app_journal(State(state): State<Shared>, Query(q): Query<SeatQ>, headers: HeaderMap) -> Response {
    let seat = resolve_seat(&state, &q, &headers);
    let s = journal_screen(&state, &seat, false);
    page(&state, &seat, "journal", s, &headers)
}

pub fn storm_screen(state: &Shared, seat: &str) -> String {
    let now = state.now();
    let w = state.world.read().expect("world");
    let total = w.passengers.len();
    let (active, left, inside, me_in) = match &w.storm {
        Some(s) => {
            let ms = s.ends_ms.saturating_sub(now);
            (true, format!("{:02}:{:02}", ms / 3_600_000, (ms / 60_000) % 60), s.in_shelter.len(), s.in_shelter.iter().any(|x| x == seat))
        }
        None => (false, String::new(), 0, false),
    };
    drop(w);
    render(&StormScreen { c: common(state, seat), active, left, in_shelter: inside, total, me_in, pct: if total == 0 { 0 } else { inside * 100 / total } })
}

pub async fn app_storm(State(state): State<Shared>, Query(q): Query<SeatQ>, headers: HeaderMap) -> Response {
    let seat = resolve_seat(&state, &q, &headers);
    let s = storm_screen(&state, &seat);
    page(&state, &seat, "home", s, &headers)
}

pub fn records_screen(state: &Shared, seat: &str) -> String {
    let chain = state.chain.lock().expect("chain");
    let rep = chain.verify();
    let records: Vec<RecordView> = chain.records.iter().rev().filter(|r| r.subject == seat || r.subject == "ship").take(40).map(RecordView::from).collect();
    drop(chain);
    render(&RecordsScreen { c: common(state, seat), records, verified: rep.ok, total: rep.records })
}

pub async fn app_records(State(state): State<Shared>, Query(q): Query<SeatQ>, headers: HeaderMap) -> Response {
    let seat = resolve_seat(&state, &q, &headers);
    let s = records_screen(&state, &seat);
    page(&state, &seat, "records", s, &headers)
}

pub async fn app_plan(State(state): State<Shared>, Query(q): Query<SeatQ>, headers: HeaderMap) -> Response {
    let seat = resolve_seat(&state, &q, &headers);
    let s = render(&PlanScreen::build(&state, &seat));
    page(&state, &seat, "plan", s, &headers)
}

pub async fn app_food(State(state): State<Shared>, Query(q): Query<SeatQ>, headers: HeaderMap) -> Response {
    let seat = resolve_seat(&state, &q, &headers);
    let s = render(&FoodScreen::build(&state, &seat));
    page(&state, &seat, "food", s, &headers)
}

pub async fn app_checkin(State(state): State<Shared>, Query(q): Query<SeatQ>, headers: HeaderMap) -> Response {
    let seat = resolve_seat(&state, &q, &headers);
    let s = render(&CheckinScreen::build(&state, &seat));
    page(&state, &seat, "home", s, &headers)
}

pub async fn app_ecg(State(state): State<Shared>, Query(q): Query<SeatQ>, headers: HeaderMap) -> Response {
    let seat = resolve_seat(&state, &q, &headers);
    let s = render(&EcgScreen::build(&state, &seat));
    page(&state, &seat, "home", s, &headers)
}

pub async fn app_band(State(state): State<Shared>, Query(q): Query<SeatQ>, headers: HeaderMap) -> Response {
    let seat = resolve_seat(&state, &q, &headers);
    let s = BandScreen::build(&state, &seat).map(|b| render(&b)).unwrap_or_default();
    page(&state, &seat, "home", s, &headers)
}

/// The tools sheet (a fragment the app shows in its bottom sheet).
pub async fn app_tools(State(state): State<Shared>, Query(q): Query<SeatQ>, headers: HeaderMap) -> Response {
    let seat = resolve_seat(&state, &q, &headers);
    with_cookie(&seat, render(&ToolSheet::build(&state, &seat)))
}

pub async fn dash(State(state): State<Shared>) -> Response {
    let now = state.now();
    let ground = state.cfg.role == NodeRole::Ground;
    let (mood, art, storm) = {
        let w = state.world.read().expect("world");
        (w.dash_mood.name(), w.dash_mood.art(), w.storm.is_some())
    };
    let weekday = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"][((now / clock::DAY_MS + 1) % 7) as usize];
    let (person, person_role, initials, greeting) = if ground {
        ("Sam Patel".to_string(), "Flight surgeon, MD · ground".to_string(), "SP".to_string(), format!("{}, Sam", clock::greeting(now)))
    } else {
        ("Mara Okoye".to_string(), "Crew lead".to_string(), "MO".to_string(), format!("{}, Mara", clock::greeting(now)))
    };
    let p = DashPage {
        ground,
        eyebrow: format!("{} · day {} of {} · ship time {}", weekday, now / clock::DAY_MS + 1, state.clock.mission_days, clock::hhmm(now)),
        greeting,
        person,
        person_role,
        initials,
        kpis: render(&KpiStrip::build(&state)),
        roster: render(&RosterCard::build(&state)),
        case: render(&CaseCard::build(&state)),
        sync: render(&SyncCard::build(&state)),
        mesh: render(&MeshCard::build(&state)),
        alerts: render(&AlertsCard::build(&state)),
        mood,
        art,
        node: state.cfg.node_name.clone(),
        storm,
    };
    Html(render(&p)).into_response()
}

pub async fn root() -> Redirect {
    Redirect::to("/app")
}
