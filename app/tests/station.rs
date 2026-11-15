//! The ship's medical station as a client of the hub, over a real WebSocket:
//! the cardiac flow (01 lights for the AED runner, "AED taken" reaches the case
//! timeline, 01 clears when the case closes) and the hub-down fallback (a
//! wristband's LoRa alarm lights 01 by itself, the red button reaches every app
//! over the air, the buffer is uploaded when the hub returns, the chain verifies).

use eva::chain::Record;
use eva::clock::MissionClock;
use eva::eva_agent::{self, AgentConfig};
use eva::link::Mirror;
use eva::model::Scenario;
use eva::protocol::frame::{decode_batch, Frame, MsgType};
use eva::protocol::payload::{alarm_code, Alarm, Hello, RecordMsg, Role};
use eva::state::{AppState, NodeRole, ServerConfig, Shared};
use eva::station::{self, Door, StationHandle};
use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio_tungstenite::tungstenite::Message;

async fn server(tag: &str) -> (Shared, String) {
    let dir = std::env::temp_dir().join(format!("eva-station-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let cfg = ServerConfig { role: NodeRole::Ship, port: 0, peer: None, data_dir: dir, node_name: format!("st-{tag}") };
    let s = AppState::new(cfg, Arc::new(MissionClock::new(96, 13, 58, 212)), AgentConfig::offline_only()).unwrap();
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = l.local_addr().unwrap().to_string();
    let app = eva::web::router(s.clone());
    tokio::spawn(async move { axum::serve(l, app).await.unwrap() });
    tokio::spawn(eva::mesh::run_heartbeat(s.clone()));
    (s, addr)
}

/// A medical station on its own connection to the hub's `/link`.
async fn station_client(s: &Shared, addr: &str, silent_after_ms: u64) -> Arc<StationHandle> {
    let h = Arc::new(StationHandle::new(s.clock.clone(), s.air.clone(), s.tx.clone()));
    h.dev.lock().unwrap().silent_after_ms = silent_after_ms;
    let _ = s.station_device.set(h.clone());
    let (to_hub, from_hub) = station::ws_pipe(&format!("ws://{addr}/link")).await.unwrap();
    tokio::spawn(station::run_device(h.clone(), to_hub, from_hub));
    until("station joins the mesh", 5_000, || h.dev.lock().unwrap().node_id.is_some() && s.world.read().unwrap().station.node.is_some()).await;
    h
}

/// A phone running the app: collects every frame it receives.
async fn app_client(addr: &str) -> Arc<Mutex<Vec<Frame>>> {
    let (mut ws, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/link?seat=3B")).await.unwrap();
    let h = Frame::new(MsgType::Hello, 1, 0, Hello::new(Role::App, "App · 3B", 1).encode());
    ws.send(Message::Binary(h.encode().into())).await.unwrap();
    let got = Arc::new(Mutex::new(Vec::new()));
    let g = got.clone();
    tokio::spawn(async move {
        while let Some(Ok(m)) = ws.next().await {
            if let Message::Binary(b) = m {
                if let Ok(fs) = decode_batch(&b) {
                    g.lock().unwrap().extend(fs);
                }
            }
        }
    });
    got
}

async fn until(what: &str, ms: u64, mut f: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_millis(ms);
    while Instant::now() < end {
        if f() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("timed out waiting for: {what}");
}

/// Ana's heart stops: no pulse, no movement, no answer on her band.
fn arrest(s: &Shared, seat: &str) {
    let now = s.now();
    let mut w = s.world.write().unwrap();
    let p = w.passenger_mut(seat).unwrap();
    p.scenario = Scenario::CardiacArrest { since_ms: now - 30_000 };
    let raw = p.vitals;
    eva::sim::ingest(p, raw, now);
}

fn steps(s: &Shared, seat: &str) -> Vec<String> {
    s.world.read().unwrap().cases.iter().filter(|c| c.seat == seat).flat_map(|c| c.steps.iter().map(|x| x.title.clone())).collect()
}

fn station_events(s: &Shared) -> Vec<(String, serde_json::Value)> {
    s.chain.lock().unwrap().records.iter().filter(|r| r.kind.starts_with("station")).map(|r| (r.body["event"].as_str().unwrap_or("").to_string(), r.body.clone())).collect()
}

/// The ground's check: every record of the ship's chain verifies in a fresh mirror.
fn ground_verifies(s: &Shared) -> (usize, usize) {
    let c = s.chain.lock().unwrap();
    let mut m = Mirror::default();
    m.pin_key(&c.records[0].origin, c.public_key()).unwrap();
    let to_msg = |r: &Record| {
        let mut hash8 = [0u8; 8];
        hash8.copy_from_slice(&hex::decode(&r.hash).unwrap()[..8]);
        RecordMsg { chain_seq: r.seq, ts_ms: r.ts_ms, kind: r.kind.clone(), subject: r.subject.clone(), hash8, body: serde_json::to_vec(&serde_json::json!({"body": r.body})).unwrap(), sig: hex::decode(&r.sig).unwrap() }
    };
    for r in c.records.iter() {
        m.insert(to_msg(r), &r.origin, 1);
    }
    (c.records.len(), m.verified.len())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cardiac_flow_lights_01_and_aed_taken_reaches_the_case() {
    let (s, addr) = server("cardiac").await;
    let h = station_client(&s, &addr, station::HUB_SILENT_MS).await;
    until("the station reports its status (battery)", 5_000, || s.world.read().unwrap().station.battery.is_some()).await;
    assert!(station_events(&s).iter().any(|(e, _)| e == "online"), "'station online' is a signed record");

    // cardiac arrest → the hub opens the case and assigns the AED runner → 01 lights
    arrest(&s, "4B");
    eva_agent::on_tick(&s);
    let case = s.world.read().unwrap().open_case_for("4B").cloned().expect("case open");
    assert!(case.helpers.iter().any(|h| h.task == "AED" && h.seat == "1B"));
    until("compartment 01 lit with 'Take the AED'", 5_000, || {
        let d = h.dev.lock().unwrap();
        d.comp(1).unwrap().state == Door::Lit && d.message.starts_with("Take the AED")
    })
    .await;
    assert!(h.dev.lock().unwrap().comps.iter().skip(1).all(|c| c.state == Door::Idle), "only 01 is lit");
    until("the hub knows 01 is lit", 5_000, || s.world.read().unwrap().station.lit == 1).await;

    // someone opens 01 and takes the AED → "AED taken at the station · mm:ss" on the case
    h.tap(1);
    until("AED taken on the case timeline", 5_000, || steps(&s, "4B").iter().any(|t| t.starts_with("AED taken at the station · "))).await;
    let t = steps(&s, "4B").into_iter().find(|t| t.starts_with("AED taken")).unwrap();
    let mmss = t.rsplit(" · ").next().unwrap();
    assert!(mmss.len() == 5 && mmss.as_bytes()[2] == b':', "mm:ss, got {mmss}");
    assert!(steps(&s, "4B").iter().any(|t| t.starts_with("Door 01 opened at the station")));
    let evs = station_events(&s);
    let taken = evs.iter().find(|(e, _)| e == "item_taken").expect("item taken is a chain record");
    assert_eq!(taken.1["compartment"], 1);
    assert!(taken.1["case_step"].as_str().unwrap().starts_with("AED taken at the station"), "the ground mirrors the timeline line too");
    // the runner then taps "I have it": no second line
    eva_agent::helper_answer(&s, "1B", true);
    assert_eq!(steps(&s, "4B").iter().filter(|t| t.starts_with("AED taken")).count(), 1);

    // the case closes → 01 clears
    eva_agent::close_case(&s, "4B");
    until("compartment 01 clears", 5_000, || h.dev.lock().unwrap().comp(1).unwrap().state == Door::Idle).await;
    assert!(s.chain.lock().unwrap().verify().ok);
    let (n, ok) = ground_verifies(&s);
    assert_eq!(n, ok, "every record, station reports included, verifies on the ground");
}

#[tokio::test]
async fn the_runner_tapping_i_have_it_marks_the_aed_taken() {
    let (s, _addr) = server("runner").await;
    arrest(&s, "4B");
    eva_agent::on_tick(&s);
    eva_agent::helper_answer(&s, "1B", true);
    let st = steps(&s, "4B");
    assert!(st.iter().any(|t| t.starts_with("AED taken at the station · 00:")), "{st:?}");
    let c = s.chain.lock().unwrap();
    assert!(c.records.iter().any(|r| r.kind == "case" && r.body["source"] == "runner" && r.body["text"].as_str().unwrap().starts_with("AED taken")));
    drop(c);
    // a later station report of the same AED does not add a second line
    assert!(station::aed_taken(&s, "station", s.now(), None).is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn hub_down_the_station_runs_on_lora_and_uploads_its_buffer() {
    let (s, addr) = server("down").await;
    // 3 s of silence (the hub pings every 2 s) makes the station standalone
    let h = station_client(&s, &addr, 3_000).await;
    let phone = app_client(&addr).await;
    let http = reqwest::Client::new();
    let dev = |action: &str| http.post(format!("http://{addr}/api/dev/{action}")).form(&[("seat", "4B")]).send();
    until("the station is online", 5_000, || h.dev.lock().unwrap().mode_label() == "Hub online").await;
    let before = s.chain.lock().unwrap().len();

    // 1. the dev panel takes the hub down
    assert!(dev("hub_down").await.unwrap().status().is_success());
    assert!(s.is_hub_down());
    until("the station notices and goes standalone", 8_000, || h.dev.lock().unwrap().standalone).await;
    assert_eq!(h.dev.lock().unwrap().mode_label(), "Standalone");

    // 2. Ana's band sees no pulse + no movement + no answer and broadcasts over LoRa;
    //    the station hears it directly and lights 01 by itself
    arrest(&s, "4B");
    eva_agent::on_tick(&s);
    until("01 lit by the station itself", 5_000, || {
        let d = h.dev.lock().unwrap();
        d.comp(1).unwrap().state == Door::Lit && d.message.starts_with("Emergency · Take the AED")
    })
    .await;
    assert!(s.world.read().unwrap().open_case_for("4B").is_none(), "the hub's rules are down: no case at the hub");

    // 3. the red button reaches every app over the air
    h.press_button();
    until("the phone gets the station's alarm", 5_000, || {
        phone.lock().unwrap().iter().any(|f| f.kind == MsgType::Alarm && Alarm::decode(&f.payload).is_ok_and(|a| a.code == alarm_code::STATION_BUTTON))
    })
    .await;

    // 4. someone takes the AED; everything waits in the station's buffer
    h.tap(1);
    let buffered = h.dev.lock().unwrap().buffered();
    assert!(buffered >= 5, "alarm heard, self lit, button, door, taken: {buffered} buffered");
    assert_eq!(s.chain.lock().unwrap().len(), before, "nothing reached the hub's chain while it was down");

    // 5. the hub comes back: the station uploads its buffer, in order, signed into the chain
    assert!(dev("hub_down").await.unwrap().status().is_success());
    assert!(!s.is_hub_down());
    until("the buffer is uploaded", 8_000, || {
        let d = h.dev.lock().unwrap();
        !d.standalone && d.buffered() == 0
    })
    .await;
    until("the hub logged every station report", 5_000, || {
        let evs: Vec<String> = station_events(&s).into_iter().map(|(e, _)| e).collect();
        ["alarm_heard", "self_lit", "red_button", "door_opened", "item_taken"].iter().all(|e| evs.iter().any(|x| x == e))
    })
    .await;
    let evs = station_events(&s);
    let online = evs.iter().rev().find(|(e, _)| e == "online").expect("back online record");
    assert!(online.1["count"].as_u64().unwrap() >= 5, "it says how many buffered events it uploaded: {}", online.1);
    let heard = evs.iter().position(|(e, _)| e == "alarm_heard").unwrap();
    let taken = evs.iter().position(|(e, _)| e == "item_taken").unwrap();
    assert!(heard < taken, "uploaded in the order they happened");

    // the case exists at the hub now, dated from when the station heard the band
    let st = steps(&s, "4B");
    assert!(st.iter().any(|t| t.starts_with("Station heard the wristband's LoRa alarm")), "{st:?}");
    assert!(st.iter().any(|t| t.starts_with("AED taken at the station · ")), "{st:?}");
    assert!(s.world.read().unwrap().alerts.iter().any(|a| a.what.starts_with("Emergency button")));
    assert!(s.world.read().unwrap().station.uploaded >= 1);

    // and the chain stays verified, on board and on the ground
    let rep = s.chain.lock().unwrap().verify();
    assert!(rep.ok, "{rep:?}");
    let (n, ok) = ground_verifies(&s);
    assert_eq!(n, ok, "every record verifies on the ground");
}

#[tokio::test]
async fn station_screen_and_dashboard_tile_render() {
    let (s, addr) = server("pages").await;
    let http = reqwest::Client::new();
    let page = http.get(format!("http://{addr}/station")).send().await.unwrap().text().await.unwrap();
    assert!(page.contains("No station on this node"), "no station running yet");
    let h = station_client(&s, &addr, station::HUB_SILENT_MS).await;
    let page = http.get(format!("http://{addr}/station")).send().await.unwrap().text().await.unwrap();
    for want in ["01", "AED", "Breathing", "Injectables", "Hygiene", "Emergency", "Hub online"] {
        assert!(page.contains(want), "/station shows {want}");
    }
    let r = http.post(format!("http://{addr}/api/station/tap")).form(&[("n", "7")]).send().await.unwrap();
    assert!(r.status().is_success());
    assert!(r.text().await.unwrap().contains(r#"data-state="taken""#));
    assert_eq!(h.dev.lock().unwrap().comp(7).unwrap().items, 7);
    assert_eq!(http.post(format!("http://{addr}/api/station/tap")).form(&[("n", "13")]).send().await.unwrap().status(), 400);
    until("the hub hears about it", 5_000, || s.world.read().unwrap().station.last_event.as_ref().is_some_and(|(_, t)| t.contains("medication pack"))).await;
    let dash = http.get(format!("http://{addr}/dash")).send().await.unwrap().text().await.unwrap();
    assert!(dash.contains("Medical station") && dash.contains("medication pack") && dash.contains("Battery"), "dashboard tile");
}
