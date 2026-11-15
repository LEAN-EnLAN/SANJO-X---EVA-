//! Break tests on a real socket: 50 clients connecting and disconnecting in
//! a loop, slow SSE readers, and oversized Eva Link datagrams.

use eva::clock::MissionClock;
use eva::eva_agent::AgentConfig;
use eva::protocol::frame::{decode_batch, Frame, MsgType};
use eva::protocol::payload::{self, Hello, Role};
use eva::state::{AppState, NodeRole, ServerConfig, Shared, UiEvent};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::tungstenite::Message;

async fn server(tag: &str) -> (Shared, String) {
    let dir = std::env::temp_dir().join(format!("eva-net-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let cfg = ServerConfig { role: NodeRole::Ship, port: 0, peer: None, data_dir: dir, node_name: format!("net-{tag}") };
    let s = AppState::new(cfg, Arc::new(MissionClock::new(96, 13, 58, 212)), AgentConfig::offline_only()).unwrap();
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = l.local_addr().unwrap().to_string();
    let app = eva::web::router(s.clone());
    tokio::spawn(async move { axum::serve(l, app).await.unwrap() });
    tokio::spawn(eva::mesh::run_heartbeat(s.clone()));
    (s, addr)
}

async fn hello(addr: &str, role: Role, name: &str) -> anyhow::Result<u32> {
    let (mut ws, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/link?seat=4B")).await?;
    let h = Frame::new(MsgType::Hello, 1, 0, Hello::new(role, name, 1).encode());
    ws.send(Message::Binary(h.encode().into())).await?;
    let id = loop {
        match tokio::time::timeout(Duration::from_secs(5), ws.next()).await? {
            Some(Ok(Message::Binary(b))) => {
                if let Some(f) = decode_batch(&b)?.into_iter().find(|f| f.kind == MsgType::Welcome) {
                    break payload::Welcome::decode(&f.payload)?.node_id;
                }
            }
            Some(Ok(_)) => {}
            other => anyhow::bail!("no welcome: {other:?}"),
        }
    };
    let ping = Frame::new(MsgType::Ping, 2, id, payload::encode_time(5));
    ws.send(Message::Binary(ping.encode().into())).await?;
    ws.close(None).await.ok();
    Ok(id)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fifty_clients_connect_and_disconnect_in_a_loop() {
    let (s, addr) = server("churn").await;
    let t = Instant::now();
    let mut tasks = Vec::new();
    for c in 0..50 {
        let addr = addr.clone();
        tasks.push(tokio::spawn(async move {
            let mut ok = 0;
            for i in 0..20 {
                let role = if c % 5 == 0 { Role::Dash } else { Role::App };
                if hello(&addr, role, &format!("client {c}/{i}")).await.is_ok() {
                    ok += 1;
                }
                // half the loops also open and drop an SSE stream
                if i % 2 == 0 {
                    if let Ok(mut tcp) = tokio::net::TcpStream::connect(&addr).await {
                        let _ = tcp.write_all(format!("GET /sse/app?seat=4B HTTP/1.1\r\nHost: {addr}\r\n\r\n").as_bytes()).await;
                        let mut buf = [0u8; 256];
                        let _ = tokio::time::timeout(Duration::from_millis(200), tcp.read(&mut buf)).await;
                    }
                }
            }
            ok
        }));
    }
    let mut total = 0;
    for t in tasks {
        total += t.await.unwrap();
    }
    assert_eq!(total, 1000, "every handshake succeeded");
    // everyone left: the registry drains
    let deadline = Instant::now() + Duration::from_secs(5);
    while s.mesh.count() > 0 && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(s.mesh.count(), 0, "no ghost nodes after 1,000 connect/disconnect cycles");
    let health = reqwest::get(format!("http://{addr}/healthz")).await.unwrap().text().await.unwrap();
    assert_eq!(health, "ok");
    println!("churn: 50 clients × 20 cycles in {:.1} s", t.elapsed().as_secs_f64());
}

/// Reads one SSE stream until an event named `name` shows up.
async fn wait_event(tcp: &mut tokio::net::TcpStream, name: &str, limit: Duration) -> Option<Duration> {
    let t = Instant::now();
    let mut acc = Vec::new();
    let mut buf = vec![0u8; 64 * 1024];
    while t.elapsed() < limit {
        match tokio::time::timeout(limit - t.elapsed(), tcp.read(&mut buf)).await {
            Ok(Ok(n)) if n > 0 => {
                acc.extend_from_slice(&buf[..n]);
                if acc.windows(name.len() + 7).any(|w| w == format!("event: {name}").as_bytes()) {
                    return Some(t.elapsed());
                }
            }
            _ => return None,
        }
    }
    None
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn slow_sse_clients_do_not_stall_the_others() {
    let (s, addr) = server("slow").await;
    // 5 stalled readers: they never read their socket
    let mut slow = Vec::new();
    for _ in 0..5 {
        let mut tcp = tokio::net::TcpStream::connect(&addr).await.unwrap();
        tcp.write_all(format!("GET /sse/dash HTTP/1.1\r\nHost: {addr}\r\n\r\n").as_bytes()).await.unwrap();
        slow.push(tcp);
    }
    let mut fast = tokio::net::TcpStream::connect(&addr).await.unwrap();
    fast.write_all(format!("GET /sse/dash HTTP/1.1\r\nHost: {addr}\r\n\r\n").as_bytes()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    // flood: 3,000 dashboard refreshes (each renders 6 fragments, ~tens of KB per event set)
    let t = Instant::now();
    for _ in 0..3_000 {
        s.emit(UiEvent::Tick);
        if t.elapsed() > Duration::from_secs(20) {
            break;
        }
        tokio::task::yield_now().await;
    }
    // the healthy dashboard still gets fresh updates right away
    s.emit(UiEvent::Tick);
    let got = wait_event(&mut fast, "kpis", Duration::from_secs(5)).await;
    assert!(got.is_some(), "a healthy dashboard keeps receiving updates while 5 others are stalled");
    // and the server still answers
    let r = reqwest::get(format!("http://{addr}/api/state")).await.unwrap();
    assert!(r.status().is_success());
    // a stalled reader that wakes up gets a current view (lagged events become one refresh), not a crash
    let mut late = slow.pop().unwrap();
    s.emit(UiEvent::Tick);
    assert!(wait_event(&mut late, "kpis", Duration::from_secs(10)).await.is_some());
    println!("slow SSE: flood of 3,000 events in {:.2} s; healthy client updated", t.elapsed().as_secs_f64());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn oversized_and_garbage_link_datagrams_close_only_that_client() {
    let (s, addr) = server("big").await;
    // garbage before Hello: dropped within the hello timeout, never registered
    let (mut ws, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/link")).await.unwrap();
    ws.send(Message::Binary(vec![0xff; 100].into())).await.unwrap();
    let closed = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            match ws.next().await {
                Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break true,
                _ => {}
            }
        }
    })
    .await
    .unwrap_or(false);
    assert!(closed, "a client that does not say Hello is closed");
    // a registered client sends 1 MB in one message (limit 256 KB)
    let (mut ws, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/link")).await.unwrap();
    ws.send(Message::Binary(Frame::new(MsgType::Hello, 1, 0, Hello::new(Role::App, "big", 1).encode()).encode().into())).await.unwrap();
    let _ = ws.next().await;
    let _ = ws.send(Message::Binary(vec![0u8; 1024 * 1024].into())).await;
    let dropped = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match ws.next().await {
                Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break true,
                _ => {}
            }
        }
    })
    .await
    .unwrap_or(false);
    assert!(dropped, "an oversized datagram closes that connection");
    // everyone else is fine
    assert!(hello(&addr, Role::App, "after").await.is_ok());
    let deadline = Instant::now() + Duration::from_secs(5);
    while s.mesh.count() > 0 && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(s.mesh.count(), 0);
}
