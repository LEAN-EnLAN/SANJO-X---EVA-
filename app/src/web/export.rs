//! `/export/*`: quick, easy exports of the records.
//! * `records.csv` and `records.json`: the hash chain (optionally one seat)
//! * `vitals.csv`: the 15 s vitals history of every passenger (or one seat)
//! * `journal.csv?seat=`: one person's own journal (private entries included,
//!   it is their export); without a seat, shared entries only
//! * `report`: a printable HTML report (browser "Print → Save as PDF")

use super::views::{render, RecordView, ReportPage};
use crate::clock;
use crate::state::Shared;
use axum::extract::{Query, State};
use axum::http::header;
use axum::response::{Html, IntoResponse, Response};
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct ExportQ {
    pub seat: Option<String>,
}

fn csv_field(s: &str) -> String {
    // quote, double inner quotes, and neutralise spreadsheet formulas
    let s = if s.starts_with(['=', '+', '-', '@']) { format!("'{s}") } else { s.to_string() };
    format!("\"{}\"", s.replace('"', "\"\""))
}

fn attachment(name: &str, ct: &str, body: String) -> Response {
    ([(header::CONTENT_TYPE, ct.to_string()), (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{name}\""))], body).into_response()
}

fn wanted(seat: &Option<String>, subject: &str) -> bool {
    seat.as_deref().is_none_or(|s| s.eq_ignore_ascii_case(subject))
}

pub async fn records_csv(State(state): State<Shared>, Query(q): Query<ExportQ>) -> Response {
    let c = state.chain.lock().expect("chain");
    let mut out = String::from("seq,ship_time,day,kind,subject,text,state,sent_ms,acked_ms,hash,prev\n");
    for r in c.records.iter().filter(|r| wanted(&q.seat, &r.subject)) {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{}\n",
            r.seq,
            clock::hhmmss(r.ts_ms),
            r.ts_ms / clock::DAY_MS + 1,
            csv_field(&r.kind),
            csv_field(&r.subject),
            csv_field(&r.summary()),
            r.state.label(),
            r.sent_ms.map(|v| v.to_string()).unwrap_or_default(),
            r.acked_ms.map(|v| v.to_string()).unwrap_or_default(),
            r.hash,
            r.prev
        ));
    }
    attachment("eva-records.csv", "text/csv; charset=utf-8", out)
}

pub async fn records_json(State(state): State<Shared>, Query(q): Query<ExportQ>) -> Response {
    let c = state.chain.lock().expect("chain");
    let recs: Vec<_> = c.records.iter().filter(|r| wanted(&q.seat, &r.subject)).collect();
    let body = serde_json::json!({"node": state.cfg.node_name, "verify": c.verify(), "records": recs});
    attachment("eva-records.json", "application/json", serde_json::to_string_pretty(&body).unwrap_or_default())
}

pub async fn vitals_csv(State(state): State<Shared>, Query(q): Query<ExportQ>) -> Response {
    let w = state.world.read().expect("world");
    let mut out = String::from("seat,name,index,hr_bpm,spo2_pct,resp_per_min,skin_c,eda_us,motion,quality_pct,base_batt_pct,screen_batt_pct,flags\n");
    for p in w.passengers.iter().filter(|p| wanted(&q.seat, &p.seat)) {
        for (i, s) in p.history.iter().chain(std::iter::once(&p.vitals)).enumerate() {
            out.push_str(&format!("{},{},{},{},{},{},{:.1},{:.2},{},{},{},{},{}\n", p.seat, csv_field(&p.name), i, s.hr, s.spo2, s.resp, s.skin_c(), s.eda_c as f32 / 100.0, s.motion, s.quality, s.base_batt, s.screen_batt, s.flags));
        }
    }
    attachment("eva-vitals.csv", "text/csv; charset=utf-8", out)
}

pub async fn journal_csv(State(state): State<Shared>, Query(q): Query<ExportQ>) -> Response {
    let w = state.world.read().expect("world");
    let mut out = String::from("time,seat,valence,mood,labels,affects,shared,text\n");
    for e in w.journal.iter().filter(|e| match &q.seat {
        Some(s) => s.eq_ignore_ascii_case(&e.seat),
        None => e.shared,
    }) {
        out.push_str(&format!("{},{},{},{},{},{},{},{}\n", clock::stamp(e.ts_ms), e.seat, e.valence, crate::model::mood_word(e.valence), csv_field(&e.labels.join("; ")), csv_field(&e.affects.join("; ")), e.shared, csv_field(&e.text)));
    }
    attachment("eva-journal.csv", "text/csv; charset=utf-8", out)
}

pub async fn report(State(state): State<Shared>, Query(q): Query<ExportQ>) -> Response {
    let now = state.now();
    let (chain_ok, head, records) = {
        let c = state.chain.lock().expect("chain");
        let rep = c.verify();
        (rep.ok, rep.head[..16.min(rep.head.len())].to_string(), c.records.iter().filter(|r| wanted(&q.seat, &r.subject)).map(RecordView::from).collect::<Vec<_>>())
    };
    let w = state.world.read().expect("world");
    let passengers = w.passengers.iter().filter(|p| wanted(&q.seat, &p.seat)).cloned().collect();
    let cases = w.cases.iter().filter(|c| wanted(&q.seat, &c.seat)).cloned().collect();
    let journal = w.journal.iter().filter(|e| e.shared && wanted(&q.seat, &e.seat)).cloned().collect();
    let title = match &q.seat {
        Some(s) => format!("Health record · {}", w.passenger(s).map(|p| format!("{} ({})", p.name, p.seat)).unwrap_or_else(|| s.clone())),
        None => "Ship health report".to_string(),
    };
    drop(w);
    Html(render(&ReportPage { title, generated: clock::stamp(now), node: state.cfg.node_name.clone(), chain_ok, head, passengers, records, cases, journal })).into_response()
}
