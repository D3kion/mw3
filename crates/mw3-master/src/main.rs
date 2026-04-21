//! TeknoMW3 master server binary entry point.

use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, Mutex};
use std::thread;

use clap::Parser;
use tracing::{debug, info, warn};

use mw3_master::conn::handle_connection;
use mw3_master::{stale_interval, MasterState};

use mw3_protocol::MS_LISTEN_PORT;

#[derive(Parser, Debug)]
#[command(
    name = "mw3-master",
    version,
    about = "TeknoMW3 TCP master server (Rust)"
)]
struct Cli {
    /// Address to bind (e.g. 0.0.0.0:27017)
    #[arg(long, default_value_t = SocketAddr::from(([0, 0, 0, 0], MS_LISTEN_PORT)))]
    bind: SocketAddr,

    /// Log level (e.g. debug, info)
    #[arg(long, default_value = "info")]
    log_level: String,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&cli.log_level)),
        )
        .init();

    let state = Arc::new(Mutex::new(MasterState::default()));

    let cleanup_state = Arc::clone(&state);
    let cleanup_every = stale_interval();
    thread::spawn(move || loop {
        thread::sleep(cleanup_every);
        let mut g = cleanup_state.lock().expect("state mutex poisoned");
        let before = g.total_entries();
        g.purge_stale(cleanup_every);
        let after = g.total_entries();
        if before != after {
            info!(removed = before - after, "stale server entries purged");
        }
    });

    let listener = TcpListener::bind(cli.bind)?;
    info!(addr = %cli.bind, "listening for master server TCP");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let peer = stream.peer_addr().ok();
                let st = Arc::clone(&state);
                thread::spawn(move || {
                    if let Err(e) = handle_connection(stream, st) {
                        debug!(?peer, error = %e, "connection closed with error");
                    }
                });
            }
            Err(e) => warn!("accept error: {e}"),
        }
    }

    Ok(())
}
