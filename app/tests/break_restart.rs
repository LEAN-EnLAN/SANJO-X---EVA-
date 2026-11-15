//! Kill the real binaries mid-case (SIGKILL) and restart them: the ship's
//! chain must stay intact, records left in flight must be re-sent, and the
//! ground's copy must survive its own restart. Uses `target/.../eva`.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct Proc(Child);
impl Drop for Proc {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

fn spawn(args: &[&str]) -> Proc {
    let child = Command::new(env!("CARGO_BIN_EXE_eva"))
        .args(args)
        .env_remove("GROQ_API_KEY")
        .env("RUST_LOG", "eva=warn")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn eva");
    Proc(child)
}

fn ship(port: u16, ground: u16, data: &Path) -> Proc {
    spawn(&["serve", "--role", "ship", "--host", "127.0.0.1", "--port", &port.to_string(), "--data", data.to_str().unwrap(), "--name", "ship", "--peer", &format!("http://127.0.0.1:{ground}"), "--offline", "--sim", "15"])
}
fn ground(port: u16, data: &Path) -> Proc {
    spawn(&["serve", "--role", "ground", "--host", "127.0.0.1", "--port", &port.to_string(), "--data", data.to_str().unwrap(), "--name", "ground", "--offline"])
}

async fn wait_up(c: &reqwest::Client, port: u16) {
    let t = Instant::now();
    while t.elapsed() < Duration::from_secs(20) {
        if c.get(format!("http://127.0.0.1:{port}/healthz")).send().await.is_ok_and(|r| r.status().is_success()) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("node on :{port} did not start");
}

async fn json(c: &reqwest::Client, port: u16, path: &str) -> Value {
    c.get(format!("http://127.0.0.1:{port}{path}")).send().await.unwrap().json().await.unwrap()
}

async fn dev(c: &reqwest::Client, port: u16, action: &str, form: &[(&str, &str)]) {
    let r = c.post(format!("http://127.0.0.1:{port}/api/dev/{action}")).form(form).send().await.unwrap();
    assert!(r.status().is_success(), "dev {action}");
}

async fn until<F: std::future::Future<Output = bool>>(limit: Duration, mut f: impl FnMut() -> F) -> bool {
    let t = Instant::now();
    while t.elapsed() < limit {
        if f().await {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    false
}

fn verify(data: &Path) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_eva")).args(["verify", "--data"]).arg(data).output().unwrap();
    (out.status.success(), String::from_utf8_lossy(&out.stdout).to_string())
}

fn tmp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("eva-restart-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kill_ship_and_ground_mid_case_and_restart() {
    let c = reqwest::Client::builder().timeout(Duration::from_secs(5)).build().unwrap();
    let (sp, gp) = (free_port(), free_port());
    let (sd, gd) = (tmp("ship"), tmp("ground"));
    let g = ground(gp, &gd);
    let mut s = ship(sp, gp, &sd);
    wait_up(&c, gp).await;
    wait_up(&c, sp).await;
    dev(&c, sp, "profile", &[("profile", "lan")]).await;

    // cardiac arrest; fast-forward past the 20 s "no answer" window → the reflex opens the case
    dev(&c, sp, "cardiac", &[("seat", "4B")]).await;
    dev(&c, sp, "ff", &[("minutes", "1")]).await;
    let opened = until(Duration::from_secs(15), || async { json(&c, sp, "/api/state").await["cases"].as_array().is_some_and(|a| a.iter().any(|k| k["open"] == true && k["seat"] == "4B")) }).await;
    assert!(opened, "case opened");
    // more records while the case runs
    for i in 0..5 {
        let _ = c.post(format!("http://127.0.0.1:{sp}/api/eva/tool")).form(&[("seat", "3B"), ("name", "log_water"), ("args", &format!("{{\"ml\": {}}}", 100 + i))]).send().await;
    }
    let before = json(&c, sp, "/api/chain/verify").await;
    let n_before = before["chain"]["records"].as_u64().unwrap();
    assert!(before["chain"]["ok"].as_bool().unwrap());

    // 1. SIGKILL the ship mid-case, with records possibly still in flight
    drop(s);
    let (ok, out) = verify(&sd);
    assert!(ok, "chain intact after kill -9:\n{out}");
    assert!(out.contains("Ed25519 signatures valid"), "{out}");

    // 2. restart on the same data dir: same key, same chain, nothing lost
    s = ship(sp, gp, &sd);
    wait_up(&c, sp).await;
    dev(&c, sp, "profile", &[("profile", "lan")]).await;
    let after = json(&c, sp, "/api/chain/verify").await;
    assert!(after["chain"]["ok"].as_bool().unwrap(), "{after}");
    assert!(after["chain"]["records"].as_u64().unwrap() >= n_before, "no record lost: {n_before} before, {after}");
    let chain = json(&c, sp, "/api/chain?limit=2000").await;
    assert!(chain.as_array().unwrap().iter().any(|r| r["kind"] == "alarm" && r["subject"] == "4B"), "the alarm survived");

    // 3. kill the ground too, restart it: its mirror comes back from disk
    drop(g);
    let g2 = ground(gp, &gd);
    wait_up(&c, gp).await;

    // 4. everything the ship signed gets acknowledged (records left "Sent" are re-sent)
    let all_acked = until(Duration::from_secs(40), || async {
        let st = json(&c, sp, "/api/state").await;
        st["chain"]["signed"] == 0 && st["chain"]["sent"] == 0
    })
    .await;
    let st = json(&c, sp, "/api/state").await;
    assert!(all_acked, "records still waiting: {}", st["chain"]);
    let mirror = json(&c, gp, "/api/chain/verify").await;
    assert_eq!(mirror["mirror"]["failed"].as_array().unwrap().len(), 0, "{mirror}");
    // invariant: nothing pending on the ship → the ground mirrors exactly the ship's chain
    let n_ship = json(&c, sp, "/api/chain/verify").await["chain"]["records"].as_u64().unwrap();
    let n_mirror = mirror["mirror"]["verified"].as_u64().unwrap();
    assert_eq!(mirror["mirror"]["records"].as_u64().unwrap(), n_ship, "ground mirror holds every ship record: {mirror}");
    assert_eq!(n_mirror, n_ship, "and every one verified");
    let gstate = json(&c, gp, "/api/state").await;
    assert!(gstate["cases"].as_array().unwrap().iter().any(|k| k["seat"] == "4B"), "the ground rebuilt the case from its mirror");

    drop(s);
    drop(g2);
    let (ok, out) = verify(&gd);
    assert!(ok, "ground mirror verifies after its restart:\n{out}");
    let _ = std::fs::remove_dir_all(&sd);
    let _ = std::fs::remove_dir_all(&gd);
}
