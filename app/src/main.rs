//! `eva`: one binary for the whole system.
//!
//! ```text
//! eva serve --role ship --port 8080 --sim 15 --peer http://127.0.0.1:8081
//! eva serve --role ground --port 8081
//! eva sim --target ws://127.0.0.1:8080/link --n 15
//! eva demo            # ship :8080 (+15 simulated bands) and ground :8081 in one process
//! eva bench           # Eva Link bytes per message and a simulated near-space run
//! eva verify          # check the hash chain files in ./data
//! ```

use clap::{Parser, Subcommand};
use eva::eva_agent::AgentConfig;
use eva::state::{AppState, NodeRole, ServerConfig, Shared};
use eva::{clock::MissionClock, seed};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "eva", version, about = "Eva by WearMeds: AI health copilot server (astronaut PWA, crew/ground dashboard, Eva Link)")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(clap::Args, Clone)]
struct ServeArgs {
    /// ship (on board, owns the bands) or ground (mirrors records, acknowledges)
    #[arg(long, value_enum, default_value = "ship")]
    role: NodeRole,
    #[arg(long, default_value_t = 8080)]
    port: u16,
    /// address to listen on; 0.0.0.0 lets phones on the LAN connect
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    /// ground node to uplink to (ship role), e.g. http://10.0.0.5:8081
    #[arg(long)]
    peer: Option<String>,
    /// run N simulated wristbands in this process (ship role; 0 = wait for `eva sim`)
    #[arg(long, default_value_t = 15)]
    sim: usize,
    /// where the hash chain and certificates live
    #[arg(long, default_value = "data")]
    data: PathBuf,
    /// node name shown on the mesh and stamped on chain records
    #[arg(long)]
    name: Option<String>,
    /// serve HTTPS with a self-signed certificate (needed to install the PWA on other LAN devices)
    #[arg(long)]
    tls: bool,
    /// extra host names / IPs for the certificate
    #[arg(long = "tls-host")]
    tls_hosts: Vec<String>,
    /// never call the Groq API even if GROQ_API_KEY is set
    #[arg(long)]
    offline: bool,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run a ship or ground node
    Serve(ServeArgs),
    /// Stream N simulated wristbands to a node's /link
    Sim {
        #[arg(long, default_value = "ws://127.0.0.1:8080/link")]
        target: String,
        #[arg(long, default_value_t = 15)]
        n: usize,
        #[arg(long, default_value_t = 1000)]
        interval_ms: u64,
    },
    /// Ship (:port, with simulated bands) and ground (:ground-port) in one process
    Demo {
        #[arg(long, default_value_t = 8080)]
        port: u16,
        #[arg(long, default_value_t = 8081)]
        ground_port: u16,
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        #[arg(long, default_value = "data")]
        data: PathBuf,
        #[arg(long)]
        tls: bool,
        #[arg(long)]
        offline: bool,
    },
    /// Print Eva Link message sizes and a simulated run over each link profile
    Bench,
    /// Verify the hash chain files in a data directory
    Verify {
        #[arg(long, default_value = "data")]
        data: PathBuf,
    },
}

fn agent_config(offline: bool) -> AgentConfig {
    let mut a = AgentConfig::from_env();
    if offline {
        a.offline = true;
    }
    a
}

fn build_node(args: &ServeArgs, clock: Arc<MissionClock>) -> anyhow::Result<Shared> {
    let name = args.name.clone().unwrap_or_else(|| format!("{}-{}", args.role.name(), args.port));
    let cfg = ServerConfig { role: args.role, port: args.port, peer: args.peer.clone(), data_dir: args.data.clone(), node_name: name };
    let state = AppState::new(cfg, clock, agent_config(args.offline))?;
    if args.role == NodeRole::Ground {
        for p in state.world.write().expect("world").passengers.iter_mut() {
            p.last_ms = 0;
        }
    }
    Ok(state)
}

async fn run_node(args: ServeArgs, state: Shared) -> anyhow::Result<()> {
    tokio::spawn(eva::mesh::run_heartbeat(state.clone()));
    if args.role == NodeRole::Ship && args.sim > 0 {
        tokio::spawn(eva::sim::run_in_process(state.clone(), 1000));
    } else {
        tokio::spawn(eva::sim::run_server_tick(state.clone(), 1000));
    }
    if let (NodeRole::Ship, Some(peer)) = (args.role, args.peer.clone()) {
        tokio::spawn(eva::uplink::run(state.clone(), peer));
    }
    let app = eva::web::router(state.clone());
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    let scheme = if args.tls { "https" } else { "http" };
    tracing::info!("{} node '{}' on {scheme}://{addr}  ·  /app  /dash  /link  /api/state  ·  Eva: {}", args.role.name(), state.cfg.node_name, state.agent.mode_label());
    if let Some(ip) = eva::tls::lan_ip() {
        tracing::info!("on the LAN: {scheme}://{ip}:{}/app", args.port);
    }
    if args.tls {
        let (cert, key) = eva::tls::load_or_create(&args.data.join("certs"), &args.tls_hosts)?;
        let cfg = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert, key).await?;
        axum_server::bind_rustls(addr, cfg).serve(app.into_make_service()).await?;
    } else {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
    }
    Ok(())
}

fn bench() {
    use eva::protocol::profile::{LinkProfile, LinkSim};
    use eva::protocol::{frame::MsgType, payload, Delivered, Endpoint};
    println!("Eva Link v{} · bytes per message\n", eva::protocol::frame::VERSION);
    println!("{:<34} {:>7} {:>9}", "message", "bytes", "payload");
    for (n, b, p) in eva::protocol::message_size_table() {
        println!("{n:<34} {b:>7} {p:>9}");
    }
    println!("\nSimulated 10 min: 15 bands at 1 Hz (vitals, latest-wins) + 1 record/s + 1 alarm, per profile\n");
    println!("{:<26} {:>9} {:>10} {:>10} {:>12} {:>12}", "profile", "records", "acked", "alarm ms", "bytes up", "B/record");
    for p in LinkProfile::all() {
        let mut up = LinkSim::new(p.clone(), 1);
        let mut down = LinkSim::new(p.clone(), 2);
        let mut a = Endpoint::new(1).with_crc(p.crc);
        let mut b = Endpoint::new(0).with_crc(p.crc);
        let (mut acked, mut records, mut alarm_at) = (0usize, 0usize, None);
        let mut alarm_seq = 0;
        let mut tokens = 0usize;
        for t in (0..600_000u64).step_by(100) {
            if t % 1000 == 0 {
                for s in 0..15u32 {
                    let v = eva::protocol::delta::VitalsSample { hr: 70 + ((t / 1000 + s as u64) % 5) as u16, spo2: 98, resp: 14, skin_c10: 340, eda_c: 200, motion: 10, quality: 95, base_batt: 70, screen_batt: 70, flags: 8 };
                    a.send_vitals(t, s + 1, t / 1000, &v);
                }
                let r = payload::RecordMsg { chain_seq: records as u64, ts_ms: t, kind: "log".into(), subject: "ship".into(), hash8: [7; 8], body: br#"{"body":{"text":"daily log"}}"#.to_vec() };
                a.send(MsgType::Record, r.encode(), None);
                records += 1;
            }
            if t == 30_000 {
                alarm_seq = a.send(MsgType::Alarm, payload::Alarm { subject: "4B".into(), code: 1, text: String::new() }.encode(), None);
            }
            if p.is_up(t) {
                tokens = (tokens + p.budget(100)).min(p.mtu.max(256) * 4);
                for d in a.outgoing(t, tokens, p.mtu.max(64)) {
                    tokens = tokens.saturating_sub(d.len());
                    up.send(t, d);
                }
                for d in b.outgoing(t, usize::MAX, p.mtu.max(64)) {
                    down.send(t, d);
                }
            }
            for d in up.recv(t) {
                for ev in b.incoming(t, &d) {
                    if let Delivered::Frame(f) = ev {
                        if f.kind == MsgType::Alarm && alarm_at.is_none() {
                            alarm_at = Some(t - 30_000);
                        }
                    }
                }
            }
            for d in down.recv(t) {
                for ev in a.incoming(t, &d) {
                    if let Delivered::Acked(s) = ev {
                        acked += s.iter().filter(|x| **x != alarm_seq).count();
                    }
                }
            }
        }
        let bytes = up.sent_bytes;
        println!("{:<26} {:>9} {:>10} {:>10} {:>12} {:>12}", p.label, records, acked, alarm_at.map(|v| v.to_string()).unwrap_or_else(|| "queued".into()), bytes, if acked > 0 { (bytes as usize / acked).to_string() } else { "-".into() });
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "eva=info,tower_http=warn".into())).with_target(false).init();
    let _ = rustls::crypto::ring::default_provider().install_default();
    let cli = Cli::parse();
    let clock = || Arc::new(MissionClock::new(seed::DEMO_DAY, seed::DEMO_HH, seed::DEMO_MM, seed::MISSION_DAYS));
    match cli.cmd {
        Cmd::Serve(args) => {
            let state = build_node(&args, clock())?;
            run_node(args, state).await
        }
        Cmd::Sim { target, n, interval_ms } => eva::sim::run_remote(target, n, interval_ms).await,
        Cmd::Demo { port, ground_port, host, data, tls, offline } => {
            let c = clock();
            let scheme = if tls { "https" } else { "http" };
            // the ground stays plain HTTP inside the demo process; the ship (the PWA host) gets --tls
            let ground = ServeArgs { role: NodeRole::Ground, port: ground_port, host: host.clone(), peer: None, sim: 0, data: data.clone(), name: Some("ground".into()), tls: false, tls_hosts: vec![], offline };
            let ship = ServeArgs { role: NodeRole::Ship, port, host, peer: Some(format!("http://127.0.0.1:{ground_port}")), sim: 15, data, name: Some("ship".into()), tls, tls_hosts: vec![], offline };
            let gs = build_node(&ground, c.clone())?;
            let ss = build_node(&ship, c)?;
            let g = tokio::spawn(run_node(ground, gs));
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            tracing::info!("demo: open {scheme}://localhost:{port}/app (astronaut), {scheme}://localhost:{port}/dash (crew), http://localhost:{ground_port}/dash (ground)");
            let s = run_node(ship, ss).await;
            g.abort();
            s
        }
        Cmd::Bench => {
            bench();
            Ok(())
        }
        Cmd::Verify { data } => {
            let mut bad = false;
            for entry in std::fs::read_dir(&data)? {
                let path = entry?.path();
                if path.extension().is_some_and(|e| e == "jsonl") {
                    let c = eva::chain::Chain::open(&path, "verify")?;
                    let r = c.verify();
                    println!("{}: {} records · {} · head {}", path.display(), r.records, if r.ok { "OK".to_string() } else { format!("BROKEN at {:?}", r.first_bad) }, &r.head[..16]);
                    bad |= !r.ok;
                }
            }
            if bad {
                std::process::exit(1);
            }
            Ok(())
        }
    }
}
