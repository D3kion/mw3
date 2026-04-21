//! Send a `MW3_SERVER_QUERY` UDP packet and print the `ServerInfos` string from the response.
#![forbid(unsafe_code)]

use std::io;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clap::{Parser, ValueEnum};
use mw3_protocol::{ServerInfoView, ServerQuery, GAME_SERVER_4CC, GAME_SERVER_4CC_OLD};

/// Default local bind (ephemeral port). For a fixed lab port, use e.g. `0.0.0.0:27057`.
const DEFAULT_BIND: &str = "0.0.0.0:0";

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum QueryMagicArg {
    /// `MW3_SERVER_4CC` ('POOL')
    Pool,
    /// `MW3_SERVER_4CC_old` ('POOP')
    Poop,
}

#[derive(Parser, Debug)]
#[command(
    name = "mw3-query-cli",
    version,
    about = "UDP MW3 server query (nettest-style)"
)]
struct Cli {
    /// Local UDP bind address (default ephemeral port)
    #[arg(long, default_value = DEFAULT_BIND)]
    bind: SocketAddr,

    /// Target game server `host:port` (e.g. 192.0.2.1:33333)
    #[arg(long)]
    target: SocketAddr,

    /// Query magic (POOL is the common default)
    #[arg(long, value_enum, default_value_t = QueryMagicArg::Pool)]
    magic: QueryMagicArg,

    /// Optional timestamp (u32). Default: low 32 bits of Unix millis (similar role to GetTickCount for lab use)
    #[arg(long)]
    timestamp: Option<u32>,

    /// Receive timeout in milliseconds
    #[arg(long, default_value_t = 5000)]
    timeout_ms: u64,

    /// Recv buffer size (bytes)
    #[arg(long, default_value_t = 4096)]
    recv_buf: usize,
}

fn default_timestamp() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u32)
        .unwrap_or(0)
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    let ts = cli.timestamp.unwrap_or_else(default_timestamp);

    let query = match cli.magic {
        QueryMagicArg::Pool => ServerQuery::new_pool(ts),
        QueryMagicArg::Poop => ServerQuery::new_poop(ts),
    };

    let socket = std::net::UdpSocket::bind(cli.bind)?;
    socket.set_read_timeout(Some(Duration::from_millis(cli.timeout_ms)))?;

    let payload = query.encode();
    socket.send_to(&payload, cli.target)?;
    eprintln!(
        "sent {} bytes to {} (magic=0x{:08X}, ts={})",
        payload.len(),
        cli.target,
        query.magic,
        query.timestamp
    );

    let mut buf = vec![0u8; cli.recv_buf.max(512)];
    let (len, from) = match socket.recv_from(&mut buf) {
        Ok(v) => v,
        Err(e) if e.kind() == io::ErrorKind::TimedOut => {
            eprintln!("recvfrom: timed out after {} ms", cli.timeout_ms);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };
    buf.truncate(len);
    eprintln!("recv {} bytes from {}", len, from);

    match ServerInfoView::decode(&buf) {
        Ok(view) => {
            if view.header.magic != GAME_SERVER_4CC && view.header.magic != GAME_SERVER_4CC_OLD {
                eprintln!(
                    "warning: response magic 0x{:08X} (expected POOL or POOP)",
                    view.header.magic
                );
            }
            println!("header: {:#?}", view.header);
            match view.server_infos_cstr() {
                Some(s) => println!("server_infos = {s:?}"),
                None => println!("server_infos: <unavailable or invalid UTF-8 / offset>"),
            }
        }
        Err(e) => {
            eprintln!("decode MW3_SERVER_INFO: {e}");
            eprintln!("first 64 bytes (hex): {}", hex_prefix(&buf, 64));
        }
    }

    Ok(())
}

fn hex_prefix(data: &[u8], max: usize) -> String {
    let n = max.min(data.len());
    data[..n]
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .chunks(16)
        .map(|c| c.join(" "))
        .collect::<Vec<_>>()
        .join(" | ")
}
