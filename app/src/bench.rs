//! `eva bench`: the numbers in `docs/app/BENCHMARKS.md`.
//!
//! * [`sizes`]: bytes per message type, Eva Link vs JSON vs CBOR (same
//!   information in each).
//! * [`throughput`]: encode / decode / sign / verify rates on this machine.
//! * [`profiles`]: delivery ratio and latency per link profile (deterministic
//!   simulation, 10 min of mission time each, plus outage-then-recovery).
//! * [`server`]: a real `eva serve` process, 15 bands streaming over Eva
//!   Link (same code as `eva sim --n 15`) and 3 dashboards on SSE: latency
//!   from vitals-in to dashboard-update, CPU and RSS of the server.
//! * [`cold_start`]: process start to first HTTP 200.

use crate::chain::Chain;
use crate::protocol::delta::{DeltaDecoder, DeltaEncoder, VitalsSample};
use crate::protocol::frame::{decode_batch, encode_batch, Frame, MsgType};
use crate::protocol::payload::{self, Alarm, Hello, PresenceEntry, RecordMsg, Role, Text};
use crate::protocol::profile::{LinkProfile, LinkSim, Rng};
use crate::protocol::wire::crc16;
use crate::protocol::{Delivered, Endpoint};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::{Duration, Instant};

fn sample() -> VitalsSample {
    VitalsSample { hr: 72, spo2: 98, resp: 14, skin_c10: 344, eda_c: 210, motion: 12, quality: 95, base_batt: 66, screen_batt: 71, flags: 8 }
}

fn vitals_json(v: &VitalsSample, seq: u64, stream: u32, ts: u64) -> Value {
    json!({"type": "vitals", "seq": seq, "src": 3, "stream": stream, "ts": ts, "hr": v.hr, "spo2": v.spo2, "resp": v.resp, "skin_c": v.skin_c10 as f64 / 10.0, "eda_us": v.eda_c as f64 / 100.0, "motion": v.motion, "quality": v.quality, "base_batt": v.base_batt, "screen_batt": v.screen_batt, "flags": v.flags})
}

/// JSON → CBOR, turning the hex fields into byte strings (what a CBOR API would send).
fn to_cbor_value(v: &Value) -> ciborium::Value {
    match v {
        Value::Null => ciborium::Value::Null,
        Value::Bool(b) => ciborium::Value::Bool(*b),
        Value::Number(n) => n.as_i64().map(|i| ciborium::Value::Integer(i.into())).or_else(|| n.as_u64().map(|u| ciborium::Value::Integer(u.into()))).unwrap_or_else(|| ciborium::Value::Float(n.as_f64().unwrap_or(0.0))),
        Value::String(s) => ciborium::Value::Text(s.clone()),
        Value::Array(a) => ciborium::Value::Array(a.iter().map(to_cbor_value).collect()),
        Value::Object(o) => ciborium::Value::Map(
            o.iter()
                .map(|(k, v)| {
                    let val = match (k.as_str(), v.as_str()) {
                        ("hash8" | "sig" | "pubkey", Some(h)) => ciborium::Value::Bytes(hex::decode(h).unwrap_or_default()),
                        _ => to_cbor_value(v),
                    };
                    (ciborium::Value::Text(k.clone()), val)
                })
                .collect(),
        ),
    }
}

fn cbor_len(v: &Value) -> usize {
    let mut b = Vec::new();
    ciborium::into_writer(&to_cbor_value(v), &mut b).ok();
    b.len()
}

pub struct SizeRow {
    pub name: &'static str,
    pub eva: usize,
    pub json: usize,
    pub cbor: usize,
}

/// Bytes per message type. JSON and CBOR carry the same fields (names a
/// typical JSON API would use); Eva Link includes its frame header (and CRC
/// where marked).
pub fn sizes() -> Vec<SizeRow> {
    let v = sample();
    let mut enc = DeltaEncoder::new(10);
    let kf = enc.encode(4, 1_731_650_000, &v);
    let v2 = VitalsSample { hr: 74, motion: 15, ..v };
    let d = enc.encode(4, 1_731_650_001, &v2);
    let sig = vec![0xcd; 64];
    let rec = RecordMsg { chain_seq: 812, ts_ms: 1_731_650_000_000, kind: "alarm".into(), subject: "4B".into(), hash8: [0xab; 8], body: br#"{"body":{"code":"no_pulse"}}"#.to_vec(), sig: sig.clone() };
    let presence: Vec<PresenceEntry> = (1..=5).map(|i| PresenceEntry { id: i, role: if i == 1 { Role::Dash } else { Role::App }, name: format!("Passenger {i}"), rtt_ms: 3 }).collect();
    let rows: Vec<(&'static str, Frame, Value)> = vec![
        ("ping", Frame::new(MsgType::Ping, 120, 3, payload::encode_time(1_731_650_000_123)), json!({"type": "ping", "seq": 120, "src": 3, "t": 1_731_650_000_123u64})),
        ("ack of 10 seqs", Frame::new(MsgType::Ack, 121, 0, payload::encode_ack(&(500..510).collect::<Vec<_>>())), json!({"type": "ack", "seq": 121, "src": 0, "acks": (500..510).collect::<Vec<_>>()})),
        ("hello (ship, with key)", Frame::new(MsgType::Hello, 1, 0, Hello { pubkey: vec![0x11; 32], ..Hello::new(Role::Ship, "ship · ship", 1_731_650_000_000) }.encode()), json!({"type": "hello", "role": "ship", "name": "ship · ship", "client_ms": 1_731_650_000_000u64, "pubkey": hex::encode([0x11u8; 32])})),
        ("vitals keyframe", Frame::new(MsgType::Vitals, 122, 3, kf), vitals_json(&v, 122, 4, 1_731_650_000)),
        ("vitals delta (2 fields changed)", Frame::new(MsgType::Vitals, 123, 3, d), vitals_json(&v2, 123, 4, 1_731_650_001)),
        ("alarm: no pulse", Frame::new(MsgType::Alarm, 124, 3, Alarm { subject: "4B".into(), code: payload::alarm_code::NO_PULSE, text: String::new() }.encode()), json!({"type": "alarm", "seq": 124, "src": 3, "subject": "4B", "code": "no_pulse"})),
        ("guidance line", Frame::new(MsgType::Guidance, 125, 0, Text { from: "Eva".into(), to: "3B".into(), text: "Push hard and fast in the middle of the chest".into() }.encode()), json!({"type": "guidance", "seq": 125, "src": 0, "from": "Eva", "to": "3B", "text": "Push hard and fast in the middle of the chest"})),
        (
            "chain record, signed (CRC on)",
            Frame::new(MsgType::Record, 127, 1, rec.encode()).with_crc(),
            json!({"type": "record", "seq": 127, "src": 1, "chain_seq": 812, "ts_ms": 1_731_650_000_000u64, "kind": "alarm", "subject": "4B", "hash8": hex::encode([0xabu8; 8]), "body": {"code": "no_pulse"}, "sig": hex::encode(&sig)}),
        ),
        ("presence (5 nodes)", Frame::new(MsgType::Presence, 128, 0, payload::encode_presence(&presence)), json!({"type": "presence", "seq": 128, "src": 0, "nodes": presence.iter().map(|p| json!({"id": p.id, "role": p.role, "name": p.name, "rtt_ms": p.rtt_ms})).collect::<Vec<_>>()})),
    ];
    rows.into_iter().map(|(name, f, j)| SizeRow { name, eva: f.wire_len(), json: serde_json::to_vec(&j).map(|b| b.len()).unwrap_or(0), cbor: cbor_len(&j) }).collect()
}

/// Run `f` repeatedly for about `ms` milliseconds; ops per second.
fn rate(ms: u64, mut f: impl FnMut() -> usize) -> (f64, f64) {
    let t = Instant::now();
    let mut ops = 0u64;
    let mut bytes = 0u64;
    while t.elapsed() < Duration::from_millis(ms) {
        for _ in 0..64 {
            bytes += f() as u64;
            ops += 1;
        }
    }
    let s = t.elapsed().as_secs_f64();
    (ops as f64 / s, bytes as f64 / s / 1e6)
}

/// (name, (ops/s, MB/s))
pub fn throughput() -> Vec<(&'static str, (f64, f64))> {
    let mut out = Vec::new();
    let v = sample();
    let mut enc = DeltaEncoder::new(10);
    let mut i = 0u64;
    out.push(("vitals delta encode", rate(400, || {
        i += 1;
        std::hint::black_box(enc.encode((i % 15) as u32 + 1, i / 15, &VitalsSample { hr: 70 + (i % 9) as u16, ..v })).len()
    })));
    let mut enc2 = DeltaEncoder::new(10);
    let stream: Vec<Vec<u8>> = (0..1500u64).map(|i| enc2.encode((i % 15) as u32 + 1, i / 15, &VitalsSample { hr: 70 + (i % 9) as u16, ..v })).collect();
    let mut dec = DeltaDecoder::new();
    let mut k = 0;
    out.push(("vitals delta decode", rate(400, || {
        let b = &stream[k % stream.len()];
        k += 1;
        let _ = std::hint::black_box(dec.decode(b));
        b.len()
    })));
    let f = Frame::new(MsgType::Vitals, 1234, 3, stream[20].clone()).with_crc();
    out.push(("frame encode (vitals, CRC)", rate(400, || std::hint::black_box(f.encode()).len())));
    let fb = f.encode();
    out.push(("frame decode (vitals, CRC)", rate(400, || {
        let _ = std::hint::black_box(Frame::decode(&fb));
        fb.len()
    })));
    let frames: Vec<Frame> = (0..60).map(|i| Frame::new(MsgType::Vitals, i, 3, stream[i as usize].clone())).collect();
    let batch = encode_batch(&frames);
    out.push(("datagram decode (60 frames)", rate(400, || {
        let _ = std::hint::black_box(decode_batch(&batch));
        batch.len()
    })));
    let kb = vec![0x5au8; 1024];
    out.push(("CRC-16 over 1 KB", rate(400, || {
        std::hint::black_box(crc16(&kb));
        kb.len()
    })));
    let j = vitals_json(&v, 1, 4, 1_731_650_000);
    out.push(("vitals as JSON encode (comparison)", rate(400, || std::hint::black_box(serde_json::to_vec(&j).map(|b| b.len()).unwrap_or(0)))));
    let jb = serde_json::to_vec(&j).unwrap_or_default();
    out.push(("vitals as JSON decode (comparison)", rate(400, || {
        let _ = std::hint::black_box(serde_json::from_slice::<Value>(&jb));
        jb.len()
    })));
    let mut c = Chain::in_memory("bench");
    let mut n = 0u64;
    out.push(("chain append (SHA-256 + Ed25519 sign)", rate(600, || {
        n += 1;
        c.append(n, "case", "4B", json!({"text": "CPR started"}));
        0
    })));
    let r = c.records[c.records.len() / 2].clone();
    let key = c.public_key();
    out.push(("record signature verify (Ed25519)", rate(600, || {
        std::hint::black_box(crate::chain::verify_sig_hex(&key, &r.hash, &r.sig));
        0
    })));
    out
}

fn pct(v: &mut [f64], p: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    let i = ((v.len() as f64 - 1.0) * p).round() as usize;
    v[i.min(v.len() - 1)]
}

#[derive(Debug, Clone)]
pub struct ProfileRow {
    pub label: String,
    pub records: usize,
    pub delivered: usize,
    pub rec_p50: f64,
    pub rec_p95: f64,
    pub rec_max: f64,
    pub alarm_ms: Option<u64>,
    pub vitals_ratio: f64,
    pub vitals_p50: f64,
    pub bytes_per_record: f64,
    pub retransmits: u64,
    pub drained_s: Option<f64>,
}

/// One profile: `run_ms` of traffic (15 bands at 1 Hz, 1 signed record/s,
/// one alarm at 30 s), then up to 5 min to drain. `phases` switches the
/// profile at given times (outage then recovery).
pub fn profile_run(label: &str, phases: &[(u64, LinkProfile)], run_ms: u64, seed: u64) -> ProfileRow {
    let first = phases[0].1.clone();
    let mut up = LinkSim::new(first.clone(), seed);
    let mut down = LinkSim::new(first.clone(), seed ^ 0xff);
    let mut ship = Endpoint::new(1).with_crc(first.crc);
    let mut ground = Endpoint::new(0).with_crc(first.crc);
    let mut chain = Chain::in_memory("ship");
    let mut sent_at: HashMap<u64, u64> = HashMap::new();
    let mut lat = Vec::new();
    let mut delivered = std::collections::HashSet::new();
    let mut vitals_sent = 0u64;
    let mut vitals_lat = Vec::new();
    let mut alarm_at = None;
    let mut alarm_frame = 0;
    let mut tokens = 0usize;
    let mut p = first.clone();
    let tick = 100u64;
    let v = sample();
    let mut t = 0u64;
    let mut drained = None;
    let end = run_ms + 300_000;
    while t < end {
        if let Some((_, np)) = phases.iter().rev().find(|(at, _)| *at <= t) {
            if np.name != p.name {
                p = np.clone();
                up.set_profile(p.clone());
                down.set_profile(p.clone());
                ship.queue.crc = p.crc;
                ground.queue.crc = p.crc;
            }
        }
        if t < run_ms && t % 1_000 == 0 {
            for s in 0..15u32 {
                let vv = VitalsSample { hr: 70 + ((t / 1000 + s as u64) % 5) as u16, ..v };
                ship.send_vitals(t, s + 1, t / 1000, &vv);
                vitals_sent += 1;
            }
            let seq = chain.append(t, "case", "4B", json!({"text": "step"}));
            let r = &chain.records[seq as usize];
            let mut h = [0u8; 8];
            h.copy_from_slice(&hex::decode(&r.hash).unwrap_or_default()[..8]);
            let msg = RecordMsg { chain_seq: r.seq, ts_ms: r.ts_ms, kind: r.kind.clone(), subject: r.subject.clone(), hash8: h, body: serde_json::to_vec(&json!({"body": r.body})).unwrap_or_default(), sig: hex::decode(&r.sig).unwrap_or_default() };
            ship.send(MsgType::Record, msg.encode(), None);
            sent_at.insert(seq, t);
        }
        // like the real uplink: a ping every 2 s keeps the RTT estimate fresh
        if t % 2_000 == 0 {
            ship.ping(t);
        }
        if t == 30_000 {
            alarm_frame = ship.send(MsgType::Alarm, Alarm { subject: "4B".into(), code: 1, text: String::new() }.encode(), None);
        }
        if p.is_up(t) {
            tokens = (tokens + p.budget(tick)).min(p.mtu.max(256) * 4);
            for d in ship.outgoing(t, tokens, p.mtu.max(64)) {
                tokens = tokens.saturating_sub(d.len());
                up.send(t, d);
            }
            for d in ground.outgoing(t, usize::MAX, p.mtu.max(64)) {
                down.send(t, d);
            }
        } else {
            tokens = 0;
        }
        for d in up.recv(t) {
            for ev in ground.incoming(t, &d) {
                match ev {
                    Delivered::Frame(f) if f.kind == MsgType::Record => {
                        if let Ok(r) = RecordMsg::decode(&f.payload) {
                            if delivered.insert(r.chain_seq) {
                                lat.push((t - sent_at[&r.chain_seq]) as f64);
                            }
                        }
                    }
                    Delivered::Frame(f) if f.kind == MsgType::Alarm && f.seq == alarm_frame && alarm_at.is_none() => alarm_at = Some(t - 30_000),
                    Delivered::Vitals { sample, .. } => vitals_lat.push((t.saturating_sub(sample.ts_s * 1000)) as f64),
                    _ => {}
                }
            }
        }
        for d in down.recv(t) {
            ship.incoming(t, &d);
        }
        t += tick;
        if t >= run_ms && drained.is_none() && delivered.len() == chain.len() && ship.queue.depth().inflight == 0 {
            drained = Some((t - run_ms) as f64 / 1000.0);
            break;
        }
    }
    let n = chain.len();
    let vitals_ratio = vitals_lat.len() as f64 / vitals_sent.max(1) as f64;
    let retx = ship.queue.sent.get(MsgType::Record).retransmits;
    ProfileRow {
        label: label.into(),
        records: n,
        delivered: delivered.len(),
        rec_p50: pct(&mut lat.clone(), 0.5),
        rec_p95: pct(&mut lat.clone(), 0.95),
        rec_max: lat.iter().cloned().fold(0.0, f64::max),
        alarm_ms: alarm_at,
        vitals_ratio,
        vitals_p50: pct(&mut vitals_lat, 0.5),
        bytes_per_record: up.sent_bytes as f64 / n.max(1) as f64,
        retransmits: retx,
        drained_s: drained,
    }
}

pub fn profiles() -> Vec<ProfileRow> {
    let mut rows: Vec<ProfileRow> = [LinkProfile::lan(), LinkProfile::lora(), LinkProfile::near_space(), LinkProfile::storm()].into_iter().enumerate().map(|(i, p)| profile_run(p.label, &[(0, p.clone())], 600_000, 7 + i as u64)).collect();
    rows.push(profile_run("Outage 5 min → near-space", &[(0, LinkProfile::outage()), (300_000, LinkProfile::near_space())], 600_000, 99));
    rows
}

// ------------------------------------------------------------------ server

#[derive(Debug, Default)]
pub struct ServerReport {
    pub secs: u64,
    pub samples: usize,
    pub updates: usize,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub max_ms: f64,
    pub cpu_pct: f64,
    pub rss_mb: f64,
    pub peak_rss_mb: f64,
    pub sse_bytes: u64,
}

fn proc_cpu_ticks(pid: u32) -> Option<u64> {
    let s = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let rest = &s[s.rfind(')')? + 2..];
    let f: Vec<&str> = rest.split_whitespace().collect();
    Some(f.get(11)?.parse::<u64>().ok()? + f.get(12)?.parse::<u64>().ok()?)
}

fn proc_kb(pid: u32, field: &str) -> Option<f64> {
    let s = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    s.lines().find(|l| l.starts_with(field)).and_then(|l| l.split_whitespace().nth(1)).and_then(|v| v.parse().ok())
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").ok().and_then(|l| l.local_addr().ok()).map(|a| a.port()).unwrap_or(18_080)
}

struct Child(std::process::Child);
impl Drop for Child {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn spawn_server(exe: &std::path::Path, port: u16, data: &std::path::Path) -> std::io::Result<Child> {
    std::process::Command::new(exe)
        .args(["serve", "--role", "ship", "--sim", "0", "--host", "127.0.0.1", "--port", &port.to_string(), "--name", "bench", "--offline", "--data"])
        .arg(data)
        .env_remove("GROQ_API_KEY")
        .env("RUST_LOG", "eva=warn")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(Child)
}

async fn wait_http(port: u16, limit: Duration) -> Option<Duration> {
    let t = Instant::now();
    while t.elapsed() < limit {
        if let Ok(mut s) = tokio::net::TcpStream::connect(("127.0.0.1", port)).await {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            if s.write_all(b"GET /healthz HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n").await.is_ok() {
                let mut b = [0u8; 64];
                if let Ok(n) = s.read(&mut b).await {
                    if b[..n].starts_with(b"HTTP/1.1 200") {
                        return Some(t.elapsed());
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(2)).await;
    }
    None
}

/// Median time from spawning `eva serve` to its first HTTP 200, over `n` runs.
pub async fn cold_start(exe: &std::path::Path, n: usize) -> Vec<f64> {
    let mut out = Vec::new();
    for i in 0..n {
        let port = free_port();
        let data = std::env::temp_dir().join(format!("eva-bench-cold-{}-{i}", std::process::id()));
        let _ = std::fs::remove_dir_all(&data);
        let t = Instant::now();
        let Ok(child) = spawn_server(exe, port, &data) else { continue };
        if wait_http(port, Duration::from_secs(10)).await.is_some() {
            out.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        drop(child);
        let _ = std::fs::remove_dir_all(&data);
    }
    out
}

/// 15 bands over Eva Link (one gateway WebSocket, like `eva sim --n 15`) and
/// 3 dashboards on `/sse/dash`, against a separate `eva serve` process.
/// Band 15 carries a probe value (skin 40.0–49.9 °C, never a real reading)
/// that changes every second; latency = sample sent → that value appears in
/// a dashboard's roster event.
pub async fn server(exe: &std::path::Path, secs: u64) -> anyhow::Result<ServerReport> {
    use futures_util::{SinkExt, StreamExt};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio_tungstenite::tungstenite::Message;
    let port = free_port();
    let data = std::env::temp_dir().join(format!("eva-bench-server-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    let child = spawn_server(exe, port, &data)?;
    let pid = child.0.id();
    wait_http(port, Duration::from_secs(15)).await.ok_or_else(|| anyhow::anyhow!("server did not start"))?;
    let sent: std::sync::Arc<std::sync::Mutex<HashMap<String, Instant>>> = Default::default();
    let lat: std::sync::Arc<std::sync::Mutex<Vec<f64>>> = Default::default();
    let bytes = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let updates = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    // 3 dashboards
    let mut dashes = Vec::new();
    for _ in 0..3 {
        let (sent, lat, bytes, updates) = (sent.clone(), lat.clone(), bytes.clone(), updates.clone());
        dashes.push(tokio::spawn(async move {
            let Ok(mut s) = tokio::net::TcpStream::connect(("127.0.0.1", port)).await else { return };
            let _ = s.write_all(b"GET /sse/dash HTTP/1.1\r\nHost: bench\r\nAccept: text/event-stream\r\n\r\n").await;
            let mut buf = vec![0u8; 256 * 1024];
            let mut carry = String::new();
            let mut seen: std::collections::HashSet<String> = Default::default();
            loop {
                let Ok(n) = s.read(&mut buf).await else { break };
                if n == 0 {
                    break;
                }
                let now = Instant::now();
                bytes.fetch_add(n as u64, std::sync::atomic::Ordering::Relaxed);
                carry.push_str(&String::from_utf8_lossy(&buf[..n]));
                while let Some(end) = carry.find("\n\n") {
                    let ev: String = carry.drain(..end + 2).collect();
                    if !ev.contains("event: roster") {
                        continue;
                    }
                    updates.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let probes: Vec<(String, Instant)> = sent.lock().map(|m| m.iter().map(|(k, v)| (k.clone(), *v)).collect()).unwrap_or_default();
                    for (marker, at) in probes {
                        if !seen.contains(&marker) && ev.contains(&format!(">{marker}<")) {
                            seen.insert(marker);
                            if let Ok(mut l) = lat.lock() {
                                l.push(now.duration_since(at).as_secs_f64() * 1000.0);
                            }
                        }
                    }
                }
            }
        }));
    }
    tokio::time::sleep(Duration::from_millis(300)).await;
    // the band gateway
    let (mut ws, _) = tokio_tungstenite::connect_async(format!("ws://127.0.0.1:{port}/link")).await?;
    let mut ep = Endpoint::new(0);
    let hello = ep.queue.control(MsgType::Hello, Hello::new(Role::Band, "bench gateway · 15 bands", 1).encode());
    ws.send(Message::Binary(hello.encode().into())).await?;
    let roster = crate::seed::passengers();
    let mut bands: Vec<(u32, VitalsSample, (u16, u16), i16)> = roster.iter().take(15).map(|p| (p.id, p.vitals, p.usual_hr, p.usual_skin_c10)).collect();
    let mut rng = Rng::new(42);
    let started = Instant::now();
    let cpu0 = proc_cpu_ticks(pid).unwrap_or(0);
    let mut tick = tokio::time::interval(Duration::from_millis(1000));
    let mut k = 0u64;
    let mut samples = 0usize;
    let (mut rx_task_ws_tx, mut ws_rx) = ws.split();
    let drain = tokio::spawn(async move { while let Some(Ok(_)) = ws_rx.next().await {} });
    while started.elapsed() < Duration::from_secs(secs) {
        tick.tick().await;
        let now = started.elapsed().as_millis() as u64;
        k += 1;
        for b in bands.iter_mut() {
            b.1 = crate::sim::baseline(&b.1, b.2, b.3, 1000, &mut rng);
            if b.0 == 15 {
                b.1.skin_c10 = 400 + (k % 100) as i16;
            }
            ep.send_vitals(now, b.0, 50_000 + now / 1000, &b.1);
            samples += 1;
        }
        let marker = format!("{:.1}", (400 + (k % 100) as i16) as f32 / 10.0);
        let datagrams = ep.outgoing(now, usize::MAX, 16_000);
        if let Ok(mut m) = sent.lock() {
            m.insert(marker, Instant::now());
        }
        for d in datagrams {
            rx_task_ws_tx.send(Message::Binary(d.into())).await?;
        }
    }
    let elapsed = started.elapsed().as_secs_f64();
    let cpu1 = proc_cpu_ticks(pid).unwrap_or(cpu0);
    let hz = 100.0; // USER_HZ on Linux
    let rss = proc_kb(pid, "VmRSS:").unwrap_or(0.0) / 1024.0;
    let peak = proc_kb(pid, "VmHWM:").unwrap_or(0.0) / 1024.0;
    tokio::time::sleep(Duration::from_millis(1500)).await;
    drain.abort();
    for d in dashes {
        d.abort();
    }
    drop(child);
    let _ = std::fs::remove_dir_all(&data);
    let mut l = lat.lock().map(|l| l.clone()).unwrap_or_default();
    Ok(ServerReport {
        secs,
        samples,
        updates: updates.load(std::sync::atomic::Ordering::Relaxed),
        p50_ms: pct(&mut l, 0.5),
        p95_ms: pct(&mut l, 0.95),
        max_ms: l.iter().cloned().fold(0.0, f64::max),
        cpu_pct: (cpu1 - cpu0) as f64 / hz / elapsed * 100.0,
        rss_mb: rss,
        peak_rss_mb: peak,
        sse_bytes: bytes.load(std::sync::atomic::Ordering::Relaxed),
    })
}

/// Print everything, as markdown tables.
pub async fn run(server_secs: u64, quick: bool) {
    println!("# eva bench · Eva Link v{} · {} · {} CPUs\n", crate::protocol::frame::VERSION, std::env::consts::ARCH, std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1));
    println!("## Bytes per message (same information in each format)\n");
    println!("| message | Eva Link | JSON | CBOR | JSON / Eva Link |\n|---|---:|---:|---:|---:|");
    for r in sizes() {
        println!("| {} | {} | {} | {} | {:.1}× |", r.name, r.eva, r.json, r.cbor, r.json as f64 / r.eva as f64);
    }
    println!("\n## Encode / decode throughput (one core)\n");
    println!("| operation | ops/s | MB/s |\n|---|---:|---:|");
    for (n, (ops, mbs)) in throughput() {
        println!("| {n} | {} | {} |", fmt_rate(ops), if mbs > 0.0 { format!("{mbs:.0}") } else { "–".into() });
    }
    println!("\n## Link profiles (10 min: 15 bands at 1 Hz, 1 signed record/s, 1 alarm; then up to 5 min to drain)\n");
    println!("| profile | records delivered | record latency p50 / p95 / max | alarm | vitals delivered | vitals age p50 | bytes up / record | record retransmits | drained after |\n|---|---:|---:|---:|---:|---:|---:|---:|---:|");
    for r in profiles() {
        println!(
            "| {} | {}/{} ({:.1}%) | {} / {} / {} | {} | {:.1}% | {} | {:.0} | {} | {} |",
            r.label,
            r.delivered,
            r.records,
            r.delivered as f64 * 100.0 / r.records.max(1) as f64,
            ms(r.rec_p50),
            ms(r.rec_p95),
            ms(r.rec_max),
            r.alarm_ms.map(|a| ms(a as f64)).unwrap_or_else(|| "–".into()),
            r.vitals_ratio * 100.0,
            ms(r.vitals_p50),
            r.bytes_per_record,
            r.retransmits,
            r.drained_s.map(|s| format!("{s:.0} s")).unwrap_or_else(|| "not in 5 min".into())
        );
    }
    let exe = std::env::current_exe().unwrap_or_else(|_| "eva".into());
    if let Ok(m) = std::fs::metadata(&exe) {
        println!("\n## Binary\n\n{}: {:.1} MB", exe.display(), m.len() as f64 / 1e6);
    }
    if quick {
        return;
    }
    let mut cs = cold_start(&exe, 7).await;
    if !cs.is_empty() {
        let med = pct(&mut cs, 0.5);
        println!("\n## Cold start (spawn → first HTTP 200, {} runs)\n\nmedian {:.0} ms · min {:.0} ms · max {:.0} ms", cs.len(), med, cs.first().copied().unwrap_or(0.0), cs.last().copied().unwrap_or(0.0));
    }
    println!("\n## Server: 15 bands over Eva Link + 3 dashboards on SSE ({server_secs} s)\n");
    match server(&exe, server_secs).await {
        Ok(r) => {
            println!("| metric | value |\n|---|---:|");
            println!("| vitals samples in | {} ({} per s) |", r.samples, r.samples as u64 / r.secs.max(1));
            println!("| roster updates received (3 dashboards) | {} |", r.updates);
            println!("| vitals-in → dashboard-update p50 | {:.0} ms |", r.p50_ms);
            println!("| vitals-in → dashboard-update p95 | {:.0} ms |", r.p95_ms);
            println!("| vitals-in → dashboard-update max | {:.0} ms |", r.max_ms);
            println!("| server CPU | {:.1}% of one core |", r.cpu_pct);
            println!("| server RSS (end / peak) | {:.1} MB / {:.1} MB |", r.rss_mb, r.peak_rss_mb);
            println!("| SSE bytes to the 3 dashboards | {:.1} KB/s each |", r.sse_bytes as f64 / 3.0 / r.secs.max(1) as f64 / 1000.0);
        }
        Err(e) => println!("server bench failed: {e}"),
    }
}

fn ms(v: f64) -> String {
    if v.is_nan() {
        "–".into()
    } else if v >= 10_000.0 {
        format!("{:.0} s", v / 1000.0)
    } else if v >= 1_000.0 {
        format!("{:.1} s", v / 1000.0)
    } else {
        format!("{v:.0} ms")
    }
}

fn fmt_rate(v: f64) -> String {
    if v >= 1e6 {
        format!("{:.1} M", v / 1e6)
    } else if v >= 1e3 {
        format!("{:.0} k", v / 1e3)
    } else {
        format!("{v:.0}")
    }
}
