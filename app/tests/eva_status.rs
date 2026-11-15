//! `GET /api/eva/status` against the real binary: no key → `no_key`, `--offline` →
//! `offline_flag`, a key from `--env-file` refused by a (local, fake) provider →
//! `bad_key`, and the body never contains the key. A missing `--env-file` is exit 2.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const SECRET: &str = "gsk_STATUSTEST_abc123XYZ";

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

fn get(port: u16, path: &str) -> Option<String> {
    let mut s = TcpStream::connect(("127.0.0.1", port)).ok()?;
    s.set_read_timeout(Some(Duration::from_secs(3))).ok()?;
    write!(s, "GET {path} HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n").ok()?;
    let mut out = String::new();
    s.read_to_string(&mut out).ok()?;
    Some(out)
}

struct Node(Child, #[allow(dead_code)] tempdir::Dir);
impl Drop for Node {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

mod tempdir {
    pub struct Dir(pub std::path::PathBuf);
    impl Dir {
        pub fn new(tag: &str) -> Self {
            let p = std::env::temp_dir().join(format!("eva-status-{tag}-{}-{}", std::process::id(), super::free_port()));
            std::fs::create_dir_all(&p).unwrap();
            Dir(p)
        }
    }
    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

fn start(tag: &str, extra: &[&str], envs: &[(&str, String)]) -> (Node, u16) {
    let dir = tempdir::Dir::new(tag);
    let port = free_port();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_eva"));
    cmd.current_dir(&dir.0).env_remove("GROQ_API_KEY").env_remove("EVA_OFFLINE").env_remove("EVA_CHAT").env_remove("EVA_ENV_FILE").env_remove("GROQ_BASE_URL");
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.args(extra).args(["serve", "--host", "127.0.0.1", "--port", &port.to_string(), "--sim", "0", "--data"]).arg(dir.0.join("data")).stdout(Stdio::null()).stderr(Stdio::null());
    let child = cmd.spawn().expect("spawn eva");
    let node = Node(child, dir);
    let t = Instant::now();
    while t.elapsed() < Duration::from_secs(15) {
        if get(port, "/healthz").is_some_and(|r| r.ends_with("ok")) {
            return (node, port);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("eva did not start");
}

fn status(port: u16) -> (String, serde_json::Value) {
    let raw = get(port, "/api/eva/status").expect("status");
    let body = raw.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    // the router may chunk the body; take the JSON object out of it
    let json = &body[body.find('{').unwrap_or(0)..body.rfind('}').map(|e| e + 1).unwrap_or(body.len())];
    (raw.clone(), serde_json::from_str(json).unwrap_or_else(|e| panic!("{e}: {raw}")))
}

#[test]
fn no_key_reads_no_key() {
    let (_n, port) = start("nokey", &[], &[]);
    let (raw, v) = status(port);
    assert_eq!(v["mode"], "rules");
    assert_eq!(v["reason"], "no_key");
    assert_eq!(v["hint"], "Set GROQ_API_KEY and restart");
    assert_eq!(v["key_source"], "none");
    assert_eq!(v["chat"], false);
    assert_eq!(v["version"], env!("CARGO_PKG_VERSION"));
    assert!(raw.to_ascii_lowercase().contains("cache-control: no-store"), "{raw}");
}

#[test]
fn offline_flag_wins() {
    let (_n, port) = start("offline", &[], &[("GROQ_API_KEY", SECRET.to_string()), ("EVA_OFFLINE", "1".to_string())]);
    let (raw, v) = status(port);
    assert_eq!(v["reason"], "offline_flag", "{raw}");
    assert_eq!(v["hint"], "Started with --offline");
    assert_eq!(v["mode"], "rules");
    assert!(!raw.contains(SECRET));
}

/// A fake provider that refuses every key with 401.
fn refusing_provider() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = l.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for s in l.incoming().flatten() {
            let mut s = s;
            let mut buf = [0u8; 4096];
            let _ = s.read(&mut buf);
            let body = "{\"error\":{\"message\":\"Invalid API Key\"}}";
            let _ = write!(s, "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
        }
    });
    port
}

#[test]
fn env_file_key_refused_reads_bad_key_and_never_leaks() {
    let up = refusing_provider();
    let keydir = tempdir::Dir::new("envfile");
    let file = keydir.0.join("eva.env");
    std::fs::write(&file, format!("\u{feff}# test\r\nexport GROQ_API_KEY=\"{SECRET}\"\r\nGROQ_BASE_URL=http://127.0.0.1:{up}\r\n")).unwrap();
    let (_n, port) = start("badkey", &["--env-file", file.to_str().unwrap()], &[]);
    let t = Instant::now();
    let mut last = serde_json::Value::Null;
    while t.elapsed() < Duration::from_secs(10) {
        let (raw, v) = status(port);
        assert!(!raw.contains(SECRET), "the key leaked: {raw}");
        if v["reason"] == "bad_key" {
            assert_eq!(v["mode"], "rules");
            assert_eq!(v["key_source"], "--env-file");
            assert_eq!(v["hint"], "The Groq key was refused. Check GROQ_API_KEY and restart");
            return;
        }
        last = v;
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("never reached bad_key: {last}");
}

#[test]
fn missing_env_file_is_exit_2() {
    let out = Command::new(env!("CARGO_BIN_EXE_eva")).args(["--env-file", "/definitely/not/here.env", "verify", "--data", "/nonexistent"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("no such file"));
}
