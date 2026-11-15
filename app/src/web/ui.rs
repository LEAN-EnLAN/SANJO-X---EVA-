//! Template helpers: static class lookups (DESIGN.md: "no dynamic class
//! strings, map tones with static lookup objects") and our own line icons.
//!
//! Icons are drawn here from scratch (simple 24×24 strokes) so the app has no
//! copied icon set. Call from templates: `{{ crate::web::ui::ic("mic", 18)|safe }}`.

/// Askama passes call arguments by reference (sometimes twice); `Val` lets
/// numeric helpers accept `T`, `&T` or `&&T` alike.
pub trait Val<T> {
    fn val(&self) -> T;
}
impl<T: Copy> Val<T> for T {
    fn val(&self) -> T {
        *self
    }
}
impl<T: Copy> Val<T> for &T {
    fn val(&self) -> T {
        **self
    }
}
impl<T: Copy> Val<T> for &&T {
    fn val(&self) -> T {
        ***self
    }
}

pub fn stamp(ms: impl Val<u64>) -> String {
    crate::clock::stamp(ms.val())
}
pub fn hhmm(ms: impl Val<u64>) -> String {
    crate::clock::hhmm(ms.val())
}
pub fn hhmmss(ms: impl Val<u64>) -> String {
    crate::clock::hhmmss(ms.val())
}
pub fn mood_word(v: impl Val<u8>) -> &'static str {
    crate::model::mood_word(v.val())
}

/// Badge fill and text for a tone: the tone at 16 % behind the tone's own text, no border.
pub fn badge(tone: &str) -> &'static str {
    match tone {
        "ok" => "bg-ok/15 text-ok",
        "warn" => "bg-warn/15 text-warn",
        "critical" => "bg-critical/15 text-critical",
        "info" => "bg-info/15 text-info",
        "brand" => "bg-brand/15 text-brand",
        _ => "bg-white/8 text-muted",
    }
}
pub fn dot(tone: &str) -> &'static str {
    match tone {
        "ok" => "bg-ok",
        "warn" => "bg-warn",
        "critical" => "bg-critical",
        "info" => "bg-info",
        "brand" => "bg-brand",
        "ink" => "bg-ink",
        _ => "bg-faint",
    }
}
pub fn text(tone: &str) -> &'static str {
    match tone {
        "ok" => "text-ok",
        "warn" => "text-warn",
        "critical" => "text-critical",
        "info" => "text-info",
        "ink" => "text-ink",
        _ => "text-muted",
    }
}
pub fn avatar(tone: &str) -> &'static str {
    match tone {
        "ok" => "bg-ok/15 text-ok",
        "warn" => "bg-warn/15 text-warn",
        "critical" => "bg-critical/20 text-critical",
        "info" => "bg-info/15 text-info",
        _ => "bg-elevated text-ink",
    }
}
pub fn stroke(tone: &str) -> &'static str {
    match tone {
        "ok" => "stroke-ok",
        "warn" => "stroke-warn",
        "critical" => "stroke-critical",
        "info" => "stroke-info",
        "ink" => "stroke-ink",
        _ => "stroke-line",
    }
}
/// meter fill: warn under 20 %, ok otherwise
pub fn batt_tone(v: impl Val<u8>) -> &'static str {
    if v.val() < 20 {
        "warn"
    } else {
        "ok"
    }
}
pub fn sync_fill(step: impl Val<u8>) -> &'static str {
    match step.val() {
        0 => "bg-ink",
        1 => "bg-info",
        _ => "bg-ok",
    }
}
pub fn orb(v: impl Val<u8>) -> &'static str {
    match v.val() {
        1 => "orb-1",
        2 => "orb-2",
        3 => "orb-3",
        4 => "orb-4",
        _ => "orb-5",
    }
}
pub fn mood_row(v: impl Val<u8>) -> &'static str {
    match v.val() {
        1 => "row-start-5",
        2 => "row-start-4",
        3 => "row-start-3",
        4 => "row-start-2",
        _ => "row-start-1",
    }
}

/// Polyline points for a small trend chart with the usual band as a rect.
/// Returns (points, band_y, band_h, last_x, last_y).
pub fn trend(data: &[u16], usual: (u16, u16), width: f32, height: f32) -> (String, f32, f32, f32, f32) {
    let data: Vec<f32> = if data.is_empty() { vec![0.0, 0.0] } else { data.iter().map(|v| *v as f32).collect() };
    let lo = data.iter().cloned().fold(usual.0 as f32, f32::min);
    let hi = data.iter().cloned().fold(usual.1 as f32, f32::max);
    let pad = 5.0;
    let span = (hi - lo).max(1.0);
    let y = |v: f32| height - pad - (v - lo) / span * (height - pad * 2.0);
    let n = data.len().max(2) as f32 - 1.0;
    let pts: Vec<String> = data.iter().enumerate().map(|(i, v)| format!("{:.1},{:.1}", i as f32 / n * (width - 5.0), y(*v))).collect();
    let by = y(usual.1 as f32);
    let bh = y(usual.0 as f32) - by;
    (pts.join(" "), by, bh, width - 5.0, y(*data.last().unwrap_or(&0.0)))
}

/// One 3D icon from the Codex kit (`static/img/icons/<name>.webp`, 256 px) on its soft
/// disc (`.pic` in styles/app.css). `class` sizes the disc (`size-12`) and may add more.
/// Content icons use these; tab bar, toolbars, chips and buttons with words keep the
/// line icons below (`ic`). The art is AI-generated; the notice is in README.md.
pub fn pic(name: &str, class: &str) -> String {
    format!(r#"<span class="pic {class}" aria-hidden="true"><img src="/static/img/icons/{name}.webp" alt="" width="256" height="256" decoding="async" draggable="false"></span>"#)
}

/// Filled variants for the active tab (own drawings).
fn ic_fill(name: &str) -> Option<&'static str> {
    Some(match name {
        "home-fill" => r#"<path d="M3.2 10.6 12 3l8.8 7.6V21h-6.3v-6.3h-5V21H3.2z" fill="currentColor" stroke="none"/>"#,
        "list-fill" => r##"<rect x="3" y="3.5" width="18" height="17" rx="4" fill="currentColor" stroke="none"/><path d="M10 8.5h7M10 12h7M10 15.5h7M6.5 8.5h.01M6.5 12h.01M6.5 15.5h.01" stroke="#0A0E1A"/>"##,
        "utensils-fill" => r#"<path d="M5 3v5.2a2.2 2.2 0 0 0 1.4 2V21h1.8V10.2A2.2 2.2 0 0 0 9.6 8.2V3H8.4v5H7.7V3H6.9v5h-.7V3z" fill="currentColor" stroke="none"/><path d="M18.4 3c-2.6.9-4.3 3.7-4.3 7.9h2.6V21h1.7z" fill="currentColor" stroke="none"/>"#,
        "records-fill" => r##"<rect x="4.5" y="2.5" width="15" height="19" rx="3" fill="currentColor" stroke="none"/><path d="M8.5 8h7M8.5 12h7M8.5 16h4" stroke="#0A0E1A"/>"##,
        _ => return None,
    })
}

/// A tab icon: line variant plus filled variant (CSS shows the filled one when active).
pub fn tab_ic(name: &str) -> String {
    format!(r#"<span class="ic-line">{}</span><span class="ic-fill">{}</span>"#, ic(name, 24), ic(&format!("{name}-fill"), 24))
}

/// A vitals ring (§3.1) as SVG: track + gradient arc. `pct` 0–100, `grad` a domain name.
pub fn ring_svg(grad: &str, pct: impl Val<u32>, stroke: f32) -> String {
    let r = 50.0 - stroke / 2.0;
    format!(
        r#"<svg viewBox="0 0 100 100" aria-hidden="true"><defs><linearGradient id="g-{grad}" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="var(--g1-{grad})"/><stop offset="1" stop-color="var(--g2-{grad})"/></linearGradient></defs><circle class="ring-track" cx="50" cy="50" r="{r}" fill="none" stroke-width="{stroke}"/><circle class="ring-arc" cx="50" cy="50" r="{r}" fill="none" stroke="url(#g-{grad})" stroke-width="{stroke}" stroke-linecap="round" pathLength="100" style="--pct:{p}"/></svg>"#,
        p = pct.val().min(100)
    )
}

/// Our own minimal stroke icons.
pub fn ic(name: &str, size: u32) -> String {
    if let Some(body) = ic_fill(name) {
        return format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" class="shrink-0">{body}</svg>"#);
    }
    let body = match name {
        "mic" => r#"<rect x="9" y="3" width="6" height="11" rx="3"/><path d="M5 11a7 7 0 0 0 14 0M12 18v3"/>"#,
        "bell" => r#"<path d="M6 16V11a6 6 0 0 1 12 0v5l1.5 2h-15z"/><path d="M10 20.5a2 2 0 0 0 4 0"/>"#,
        "grid" => r#"<rect x="4" y="4" width="6" height="6" rx="1.5"/><rect x="14" y="4" width="6" height="6" rx="1.5"/><rect x="4" y="14" width="6" height="6" rx="1.5"/><rect x="14" y="14" width="6" height="6" rx="1.5"/>"#,
        "back" => r#"<path d="M15 5l-7 7 7 7"/>"#,
        "arrow" => r#"<path d="M5 12h14M13 6l6 6-6 6"/>"#,
        "upleft" => r#"<path d="M18 18L7 7M7 15V7h8"/>"#,
        "check" => r#"<path d="M5 12.5l4.5 4.5L19 7.5"/>"#,
        "x" => r#"<path d="M6 6l12 12M18 6L6 18"/>"#,
        "send" => r#"<path d="M4 12l16-8-6 16-2.5-6.5z"/><path d="M11.5 13.5L20 4"/>"#,
        "home" => r#"<path d="M4 11l8-7 8 7v9h-5v-6H9v6H4z"/>"#,
        "list" => r#"<path d="M9 6h11M9 12h11M9 18h11"/><path d="M4 6l1 1 2-2M4 12l1 1 2-2M4 18l1 1 2-2"/>"#,
        "book" => r#"<path d="M5 4h9a4 4 0 0 1 4 4v12H9a4 4 0 0 1-4-4z"/><path d="M5 16a4 4 0 0 1 4-4h9"/>"#,
        "records" => r#"<rect x="5" y="3" width="14" height="18" rx="2"/><path d="M9 8h6M9 12h6M9 16h3"/>"#,
        "camera" => r#"<path d="M4 8h3l2-3h6l2 3h3v11H4z"/><circle cx="12" cy="13" r="3.5"/>"#,
        "video" => r#"<rect x="3" y="6" width="13" height="12" rx="2"/><path d="M16 10l5-3v10l-5-3"/>"#,
        "zap" => r#"<path d="M13 3L5 14h6l-1 7 8-11h-6z"/>"#,
        "lock" => r#"<rect x="5" y="11" width="14" height="10" rx="2"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/>"#,
        "users" => r#"<circle cx="9" cy="8" r="3.5"/><path d="M2.5 20a6.5 6.5 0 0 1 13 0"/><path d="M16 4.5a3.5 3.5 0 0 1 0 7M18 14a6 6 0 0 1 3.5 6"/>"#,
        "siren" => r#"<path d="M6 18v-6a6 6 0 0 1 12 0v6"/><path d="M4 18h16v3H4zM12 2v2M4.5 5l1.5 1.5M19.5 5L18 6.5"/>"#,
        "eye" => r#"<path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="3"/>"#,
        "satellite" => r#"<path d="M9 15l-4 4M7 9l4-4 8 8-4 4z"/><path d="M14 4l2-2M20 10l2-2"/><path d="M3 13a8 8 0 0 0 8 8"/>"#,
        "radiation" => r#"<circle cx="12" cy="12" r="2"/><path d="M12 10V3a9 9 0 0 1 7.8 4.5L13.7 11M10.3 13l-6.1 3.5A9 9 0 0 0 12 21v-7"/><path d="M13.7 13l6.1 3.5"/>"#,
        "magnet" => r#"<path d="M6 4v8a6 6 0 0 0 12 0V4h-4v8a2 2 0 0 1-4 0V4z"/><path d="M6 8h4M14 8h4"/>"#,
        "shirt" => r#"<path d="M8 3l-5 3 2 5 2-1v11h10V10l2 1 2-5-5-3a4 4 0 0 1-8 0z"/>"#,
        "sparkles" => r#"<path d="M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8z"/><path d="M19 16l.8 2.2L22 19l-2.2.8L19 22l-.8-2.2L16 19l2.2-.8z"/>"#,
        "map" => r#"<path d="M3 6l6-2 6 2 6-2v14l-6 2-6-2-6 2z"/><path d="M9 4v14M15 6v14"/>"#,
        "sun" => r#"<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M2 12h2M20 12h2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/>"#,
        "file" => r#"<path d="M6 3h8l4 4v14H6z"/><path d="M14 3v4h4M9 12h6M9 16h6"/>"#,
        "heart" => r#"<path d="M12 20s-8-4.7-8-10.5A4.5 4.5 0 0 1 12 7a4.5 4.5 0 0 1 8 2.5C20 15.3 12 20 12 20z"/><path d="M7 12h2.5l1.5-2.5 2 5 1.5-2.5H17"/>"#,
        "droplet" => r#"<path d="M12 3s6 6.5 6 11a6 6 0 0 1-12 0c0-4.5 6-11 6-11z"/>"#,
        "download" => r#"<path d="M12 4v11M7 10l5 5 5-5M5 20h14"/>"#,
        "radio" => r#"<circle cx="12" cy="12" r="2"/><path d="M8.5 8.5a5 5 0 0 0 0 7M15.5 8.5a5 5 0 0 1 0 7M5.6 5.6a9 9 0 0 0 0 12.8M18.4 5.6a9 9 0 0 1 0 12.8"/>"#,
        "plus" => r#"<path d="M12 5v14M5 12h14"/>"#,
        "settings" => r#"<circle cx="12" cy="12" r="3"/><path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9L7 7M17 17l2.1 2.1M4.9 19.1L7 17M17 7l2.1-2.1"/>"#,
        "clock" => r#"<circle cx="12" cy="12" r="8.5"/><path d="M12 7.5V12l3 2"/>"#,
        "pin" => r#"<path d="M12 21s-6-5.4-6-11a6 6 0 0 1 12 0c0 5.6-6 11-6 11z"/><circle cx="12" cy="10" r="2"/>"#,
        "keyboard" => r#"<rect x="3" y="6" width="18" height="12" rx="2"/><path d="M7 10h1M11 10h1M15 10h1M7 14h10"/>"#,
        "history" => r#"<path d="M4 12a8 8 0 1 0 2.4-5.7L4 8.5"/><path d="M4 4v4.5h4.5M12 8v4l3 2"/>"#,
        "utensils" => r#"<path d="M7 3v8M5 3v5a2 2 0 0 0 4 0V3M7 11v10"/><path d="M18 3c-2.2 1-3.5 3.6-3.5 7.5H18V21"/>"#,
        "calendar" => r#"<rect x="4" y="5" width="16" height="15" rx="2"/><path d="M4 10h16M9 3v4M15 3v4"/>"#,
        "hand" => r#"<path d="M8 13V6.5a1.5 1.5 0 0 1 3 0V11M11 11V4.5a1.5 1.5 0 0 1 3 0V11M14 11V6.5a1.5 1.5 0 0 1 3 0V14a7 7 0 0 1-7 7h-.5A5.5 5.5 0 0 1 5 16.5L3.8 14a1.5 1.5 0 0 1 2.6-1.5L8 14.5"/>"#,
        "smile" => r#"<circle cx="12" cy="12" r="8.5"/><path d="M8.5 14a4 4 0 0 0 7 0M9 9.5h.01M15 9.5h.01"/>"#,
        "meh" => r#"<circle cx="12" cy="12" r="8.5"/><path d="M9 15h6M9 9.5h.01M15 9.5h.01"/>"#,
        "frown" => r#"<circle cx="12" cy="12" r="8.5"/><path d="M8.5 16a4 4 0 0 1 7 0M9 9.5h.01M15 9.5h.01"/>"#,
        "battery" => r#"<rect x="3" y="8" width="16" height="9" rx="2"/><path d="M21.5 11v3M6 11v3"/>"#,
        "signal" => r#"<path d="M5 19v-3M10 19v-6M15 19v-9M20 19V6"/>"#,
        "info" => r#"<circle cx="12" cy="12" r="8.5"/><path d="M12 11v5M12 8h.01"/>"#,
        "pulse" => r#"<path d="M3 12h4l2-5 3 10 2-5h7"/>"#,
        "moon" => r#"<path d="M19.5 14.5A8 8 0 0 1 9.5 4.5a8 8 0 1 0 10 10z"/>"#,
        "pill" => r#"<path d="M10.5 20.5a5 5 0 0 1-7-7l6-6a5 5 0 0 1 7 7z"/><path d="M8.5 8.5l7 7"/>"#,
        "run" => r#"<circle cx="14.5" cy="4.5" r="1.8"/><path d="M8 21l3.5-6-3-3 3.5-4 3 3h3.5M5 11l3.5-3"/>"#,
        "chat" => r#"<path d="M4 5h16v11H9.5L5 20v-4H4z"/>"#,
        "phone" => r#"<path d="M5 4h4l2 5-2.5 1.5a11 11 0 0 0 5 5L15 13l5 2v4a2 2 0 0 1-2 2A16 16 0 0 1 3 6a2 2 0 0 1 2-2z"/>"#,
        "glass" => r#"<path d="M6 3h12l-1.5 17a1 1 0 0 1-1 1h-7a1 1 0 0 1-1-1z"/><path d="M6.6 9h10.8"/>"#,
        "chevron" => r#"<path d="M9 5l7 7-7 7"/>"#,
        "share" => r#"<path d="M12 4v11M7.5 8.5L12 4l4.5 4.5M5 13v6h14v-6"/>"#,
        "speaker" => r#"<path d="M4 9h4l5-4v14l-5-4H4z"/><path d="M16.5 8.5a5 5 0 0 1 0 7M19 6a8.5 8.5 0 0 1 0 12"/>"#,
        "stop" => r#"<rect x="7" y="7" width="10" height="10" rx="2"/>"#,
        "shield" => r#"<path d="M12 3l7 3v5.5c0 4.6-3 8.1-7 9.5-4-1.4-7-4.9-7-9.5V6z"/><path d="M9 12l2 2 4-4"/>"#,
        "arrow-up" => r#"<path d="M12 19V5M6 11l6-6 6 6"/>"#,
        // medical station compartments (own drawings)
        "probe" => r#"<path d="M9 3h6l1 7a4 4 0 0 1-8 0z"/><path d="M12 14v3a4 4 0 0 0 4 4h3"/><path d="M10 7h4"/>"#,
        "bandage" => r#"<rect x="2.5" y="8" width="19" height="8" rx="4" transform="rotate(-35 12 12)"/><path d="M10.5 10.5h.01M13.5 13.5h.01M13.5 10.5h.01M10.5 13.5h.01"/>"#,
        "strap" => r#"<rect x="3" y="8" width="18" height="8" rx="2"/><rect x="9" y="6" width="6" height="12" rx="1.5"/><path d="M11 12h2"/>"#,
        "syringe" => r#"<path d="M18 2l4 4M20 4l-3 3M17 7L8 16l-3-3 9-9z"/><path d="M5 13l-3 3 1 1M8 16l-3 3-1-1M11 7l1.5 1.5M8.5 9.5L10 11"/>"#,
        "airway" => r#"<path d="M6 4h12"/><path d="M9 4v5a6 6 0 0 0 6 6h0a3 3 0 0 1 3 3v2"/><path d="M15 4v5"/><path d="M15 20h6"/>"#,
        "drip" => r#"<path d="M8 3h8v7a4 4 0 0 1-8 0z"/><path d="M8 7h8M12 14v3M12 17a3 3 0 0 0 3 3h4"/>"#,
        _ => r#"<circle cx="12" cy="12" r="8"/>"#,
    };
    format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" class="shrink-0">{body}</svg>"#)
}

/// One-lead ECG trace as SVG polyline points: P wave, QRS spike, T wave.
/// `uneven` varies the gap between beats, the way an irregular rhythm looks.
pub fn ecg_points(beats: usize, uneven: bool, width: f32, height: f32) -> String {
    const BEAT: [f32; 20] = [0.0, 0.0, 0.12, 0.18, 0.1, 0.0, 0.0, -0.12, 1.0, -0.3, 0.0, 0.0, 0.08, 0.22, 0.28, 0.18, 0.05, 0.0, 0.0, 0.0];
    const GAPS: [usize; 6] = [0, 6, 1, 9, 3, 7];
    let mut ys: Vec<f32> = Vec::new();
    for b in 0..beats {
        ys.extend_from_slice(&BEAT);
        if uneven {
            ys.extend(std::iter::repeat_n(0.0, GAPS[b % GAPS.len()]));
        }
    }
    let n = ys.len().max(2) as f32 - 1.0;
    let mid = height * 0.68;
    ys.iter().enumerate().map(|(i, y)| format!("{:.1},{:.1}", i as f32 / n * width, mid - y * height * 0.6)).collect::<Vec<_>>().join(" ")
}
