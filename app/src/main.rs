//! `eva`: one binary for the whole system.
//!
//! ```text
//! eva serve --role ship --port 8080 --sim 15 --peer http://127.0.0.1:8081
//! eva serve --role ground --port 8081
//! eva sim --target ws://127.0.0.1:8080/link --n 15
//! eva demo            # ship :8080 (+15 simulated bands, 1 medical station) and ground :8081 in one process
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
#[command(name = "eva", version, about = "Eva by WearMeds: Super Intelligence health copilot server (astronaut PWA, crew/ground dashboard, Eva Link)")]
struct Cli {
    /// read KEY=VALUE lines (e.g. GROQ_API_KEY=…) from this file; default: ./.env, then .env or
    /// eva.env next to the eva program. Variables already set in the environment win.
    #[arg(long, global = true, env = "EVA_ENV_FILE")]
    env_file: Option<PathBuf>,
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
    /// run one simulated medical station as a client of this hub (ship role; its screen is /station)
    #[arg(long)]
    station: bool,
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
    /// Benchmarks: bytes per message (Eva Link vs JSON vs CBOR), codec throughput,
    /// delivery per link profile, cold start, and a live server run (15 bands + 3 dashboards)
    Bench {
        /// seconds of the live server run
        #[arg(long, default_value_t = 30)]
        server_secs: u64,
        /// skip the process-level parts (cold start, live server)
        #[arg(long)]
        quick: bool,
    },
    /// Verify the hash chain files in a data directory
    Verify {
        #[arg(long, default_value = "data")]
        data: PathBuf,
    },
}

fn agent_config(offline: bool) -> AgentConfig {
    let mut a = AgentConfig::from_env();
    if offline {
        a.set_offline(true);
    }
    // --offline or EVA_OFFLINE: /api/eva/status says "offline_flag", not "no_key"
    if a.offline {
        a.health.mark_offline_flag();
    }
    a
}

/// One startup line about Eva's mode. Never the key.
fn log_eva_mode(env: &eva::envfile::Report, offline: bool) {
    let offline = offline || std::env::var("EVA_OFFLINE").is_ok_and(|v| v == "1" || v == "true");
    let has_key = std::env::var("GROQ_API_KEY").is_ok_and(|k| !k.trim().is_empty());
    if offline {
        tracing::info!("Eva: on-board rules · --offline");
    } else if has_key {
        tracing::info!("Eva: online · key from {}", env.where_words);
    } else {
        let looked: Vec<String> = env.looked.iter().map(|p| p.display().to_string()).collect();
        tracing::warn!("Eva: on-board rules · no GROQ_API_KEY found (looked in {}) · create one of them with GROQ_API_KEY=… and restart", looked.join(", "));
    }
}

fn build_node(args: &ServeArgs, clock: Arc<MissionClock>) -> anyhow::Result<Shared> {
    let name = args.name.clone().unwrap_or_else(|| format!("{}-{}", args.role.name(), args.port));
    let cfg = ServerConfig { role: args.role, port: args.port, peer: args.peer.clone(), data_dir: args.data.clone(), node_name: name };
    let state = AppState::new(cfg, clock, agent_config(args.offline))?;
    if args.role == NodeRole::Ground {
        for p in state.world.write().expect("world").passengers.iter_mut() {
            p.last_ms = 0;
        }
        eva::link::restore_ground(&state);
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
    if args.role == NodeRole::Ship && args.station {
        eva::station::spawn_local(&state);
    }
    tokio::spawn(eva::eva_agent::status::run_monitor(state.clone()));
    let app = eva::web::router(state.clone());
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    let scheme = if args.tls { "https" } else { "http" };
    tracing::info!("{} node '{}' on {scheme}://{addr}  ·  /app  /dash{}  /link  /api/state  ·  Eva: {}", args.role.name(), state.cfg.node_name, if args.station && args.role == NodeRole::Ship { "  /station" } else { "" }, state.agent.mode_label());
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

/// `eva verify`: every chain file (hash chain + Ed25519 signatures, against
/// the node's pinned public key) and every ground mirror. Returns false if
/// anything was tampered with.
fn verify_dir(data: &std::path::Path) -> anyhow::Result<bool> {
    let keys = data.join("keys");
    let mut all_ok = true;
    let mut files: Vec<PathBuf> = std::fs::read_dir(data)?.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "jsonl")).collect();
    files.sort();
    if files.is_empty() {
        println!("no chain files in {}", data.display());
    }
    for path in files {
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        if let Some(node) = name.strip_prefix("chain-").and_then(|n| n.strip_suffix(".jsonl")) {
            let pinned = eva::chain::read_pub_file(&eva::chain::NodeKey::pub_path(&keys, node));
            let r = eva::chain::verify_file(&path, pinned)?;
            let head = &r.head[..16.min(r.head.len())];
            if r.ok {
                println!("{name}: OK · {} records · {} Ed25519 signatures valid · head {head}", r.records, r.signatures);
            } else {
                println!("{name}: TAMPERED · {} · first bad record {:?} · {} records", r.reason.clone().unwrap_or_default(), r.first_bad, r.records);
            }
            all_ok &= r.ok;
        } else if name.starts_with("mirror-") && !name.contains(".epoch") {
            let (n, ok, failed) = eva::link::verify_mirror_file(&path, &keys)?;
            if failed.is_empty() {
                println!("{name}: OK · {n} records from the ship · {ok} verified (hash chain + signature) · {} waiting for earlier records", n - ok);
            } else {
                println!("{name}: TAMPERED · {} record(s) failed: {}", failed.len(), failed.iter().map(|(s, why)| format!("#{s} {why}")).collect::<Vec<_>>().join(", "));
                all_ok = false;
            }
        }
    }
    Ok(all_ok)
}

/// Plain `fn main`: (1) parse the CLI, (2) load the env file and set its variables while this
/// is still the only thread (`set_var` is not thread-safe), (3) then build the tokio runtime.
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let env = match eva::envfile::load(cli.env_file.as_deref()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("eva: {e}");
            std::process::exit(2);
        }
    };
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "eva=info,tower_http=warn".into())).with_target(false).init();
    let _ = rustls::crypto::ring::default_provider().install_default();
    match &cli.cmd {
        Cmd::Serve(a) => log_eva_mode(&env, a.offline),
        Cmd::Demo { offline, .. } => log_eva_mode(&env, *offline),
        _ => {}
    }
    tokio::runtime::Builder::new_multi_thread().enable_all().build()?.block_on(run(cli))
}

async fn run(cli: Cli) -> anyhow::Result<()> {
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
            let ground = ServeArgs { role: NodeRole::Ground, port: ground_port, host: host.clone(), peer: None, sim: 0, data: data.clone(), name: Some("ground".into()), tls: false, tls_hosts: vec![], offline, station: false };
            let ship = ServeArgs { role: NodeRole::Ship, port, host, peer: Some(format!("http://127.0.0.1:{ground_port}")), sim: 15, data, name: Some("ship".into()), tls, tls_hosts: vec![], offline, station: true };
            let gs = build_node(&ground, c.clone())?;
            let ss = build_node(&ship, c)?;
            let g = tokio::spawn(run_node(ground, gs));
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            tracing::info!("demo: open {scheme}://localhost:{port}/app (astronaut), {scheme}://localhost:{port}/dash (crew), {scheme}://localhost:{port}/station (medical station), http://localhost:{ground_port}/dash (ground)");
            let s = run_node(ship, ss).await;
            g.abort();
            s
        }
        Cmd::Bench { server_secs, quick } => {
            eva::bench::run(server_secs, quick).await;
            Ok(())
        }
        Cmd::Verify { data } => {
            if verify_dir(&data)? {
                Ok(())
            } else {
                std::process::exit(1);
            }
        }
    }
}
