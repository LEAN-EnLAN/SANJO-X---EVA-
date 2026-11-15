//! Ed25519-signed chain: `eva verify` on real files, byte flips, and a ship →
//! ground hand-off through the Eva Link payloads.

use eva::chain::{read_file, Chain, NodeKey};
use eva::clock::MissionClock;
use eva::eva_agent::AgentConfig;
use eva::state::{AppState, NodeRole, ServerConfig, Shared};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("eva-sig-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn node(dir: &Path, role: NodeRole, name: &str) -> Shared {
    let cfg = ServerConfig { role, port: 0, peer: None, data_dir: dir.to_path_buf(), node_name: name.into() };
    AppState::new(cfg, Arc::new(MissionClock::new(96, 13, 58, 212)), AgentConfig::offline_only()).unwrap()
}

fn verify(dir: &Path) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_eva")).args(["verify", "--data"]).arg(dir).output().unwrap();
    (out.status.success(), String::from_utf8_lossy(&out.stdout).to_string())
}

#[test]
fn keypair_created_on_first_run_and_every_record_signed() {
    let dir = tmp("first-run");
    let s = node(&dir, NodeRole::Ship, "ship");
    assert!(dir.join("keys/ship.ed25519").exists());
    assert!(dir.join("keys/ship.pub").exists());
    s.log("alarm", "4B", serde_json::json!({"text": "No pulse"}));
    let c = s.chain.lock().unwrap();
    assert!(c.records.iter().all(|r| r.sig.len() == 128), "every record carries a 64-byte signature");
    let rep = c.verify();
    assert!(rep.ok);
    assert_eq!(rep.signatures, c.len());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(dir.join("keys/ship.ed25519")).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "private key readable by the owner only");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn eva_verify_reports_a_flipped_byte() {
    let dir = tmp("cli");
    {
        let s = node(&dir, NodeRole::Ship, "ship");
        s.log("alarm", "4B", serde_json::json!({"text": "No pulse at the wrist"}));
        s.log("case", "4B", serde_json::json!({"text": "Leo arrived, CPR started"}));
    }
    let (ok, out) = verify(&dir);
    assert!(ok, "{out}");
    assert!(out.contains("chain-ship.jsonl: OK"), "{out}");
    assert!(out.contains("Ed25519 signatures valid"), "{out}");

    let path = dir.join("chain-ship.jsonl");
    let clean = std::fs::read(&path).unwrap();
    let at = clean.windows(3).position(|w| w == b"CPR").unwrap();
    let mut bad = clean.clone();
    bad[at] = b'X'; // "CPR" → "XPR"
    std::fs::write(&path, &bad).unwrap();
    let (ok, out) = verify(&dir);
    assert!(!ok, "tampering must fail the command");
    assert!(out.contains("TAMPERED"), "{out}");

    // swapping in a new key and re-signing is caught by the pinned public key
    std::fs::write(&path, &clean).unwrap();
    let c = read_file(&path).unwrap();
    let forger = NodeKey::generate();
    let mut forged = Chain::open_with_key(&dir.join("forged.tmp"), "ship", forger).unwrap();
    for r in &c.records {
        forged.append(r.ts_ms, &r.kind, &r.subject, r.body.clone());
    }
    drop(forged);
    std::fs::rename(dir.join("forged.tmp"), &path).unwrap();
    let (ok, out) = verify(&dir);
    assert!(!ok, "{out}");
    assert!(out.contains("pinned public key"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn ground_mirror_is_persisted_and_checked_by_eva_verify() {
    use eva::protocol::payload::RecordMsg;
    let ship_dir = tmp("ship");
    let ground_dir = tmp("ground");
    let ship = node(&ship_dir, NodeRole::Ship, "ship");
    for i in 0..3 {
        ship.log("case", "4B", serde_json::json!({"text": format!("step {i}")}));
    }
    let key = ship.chain.lock().unwrap().public_key();
    let msgs: Vec<RecordMsg> = ship
        .chain
        .lock()
        .unwrap()
        .records
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let mut w = serde_json::json!({"body": r.body});
            if i == 0 {
                w["prev"] = serde_json::json!(r.prev);
            }
            let mut h = [0u8; 8];
            h.copy_from_slice(&hex::decode(&r.hash).unwrap()[..8]);
            RecordMsg { chain_seq: r.seq, ts_ms: r.ts_ms, kind: r.kind.clone(), subject: r.subject.clone(), hash8: h, body: serde_json::to_vec(&w).unwrap(), sig: hex::decode(&r.sig).unwrap() }
        })
        .collect();
    let n = msgs.len();
    {
        let ground = node(&ground_dir, NodeRole::Ground, "ground");
        let mut m = ground.mirror.lock().unwrap();
        m.pin_key("ship", key).unwrap();
        for msg in msgs {
            m.insert(msg, "ship", 1);
        }
        assert_eq!(m.verified.len(), n);
    }
    // restart: a fresh process state reads the mirror back
    let ground = node(&ground_dir, NodeRole::Ground, "ground");
    assert_eq!(ground.mirror.lock().unwrap().verified.len(), n, "restart keeps the ground copy");
    drop(ground);
    let (ok, out) = verify(&ground_dir);
    assert!(ok, "{out}");
    assert!(out.contains(&format!("mirror-ground.jsonl: OK · {n} records")), "{out}");
    let _ = std::fs::remove_dir_all(&ship_dir);
    let _ = std::fs::remove_dir_all(&ground_dir);
}
