//! `--tls`: self-signed certificate for installing the PWA on other devices
//! on the LAN (service workers need a secure context; `localhost` is exempt).
//!
//! The certificate covers `localhost`, `127.0.0.1`, `::1`, this machine's LAN
//! address and any `--tls-host` names. It is created once in
//! `<data>/certs/` and reused. Devices must trust it once (see
//! `docs/app/ADR-004-pwa-and-tls.md`; mkcert is the smoother alternative).

use std::net::UdpSocket;
use std::path::{Path, PathBuf};

/// This machine's outward-facing LAN address (no packet is sent).
pub fn lan_ip() -> Option<String> {
    let s = UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect("10.255.255.255:1").ok()?;
    s.local_addr().ok().map(|a| a.ip().to_string()).filter(|ip| ip != "0.0.0.0")
}

pub fn load_or_create(dir: &Path, extra_hosts: &[String]) -> anyhow::Result<(PathBuf, PathBuf)> {
    std::fs::create_dir_all(dir)?;
    let cert = dir.join("eva-cert.pem");
    let key = dir.join("eva-key.pem");
    if cert.exists() && key.exists() {
        return Ok((cert, key));
    }
    let mut names = vec!["localhost".to_string(), "127.0.0.1".to_string(), "::1".to_string()];
    if let Some(ip) = lan_ip() {
        names.push(ip);
    }
    names.extend(extra_hosts.iter().cloned());
    let ck = rcgen::generate_simple_self_signed(names.clone())?;
    std::fs::write(&cert, ck.cert.pem())?;
    std::fs::write(&key, ck.key_pair.serialize_pem())?;
    tracing::info!("tls: created self-signed certificate for {}", names.join(", "));
    Ok((cert, key))
}
