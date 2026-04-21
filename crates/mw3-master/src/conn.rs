//! One-shot TCP client handling (read up to 512 bytes, then respond or update state).

use std::io::{Read, Write};
use std::net::{IpAddr, TcpStream};
use std::sync::{Arc, Mutex};

use tracing::{debug, info, warn};

use mw3_protocol::{
    ClientListRequest, ServerRegisterRequest, CLIENT_REQUEST_LEN, MS_CLIENT, MS_SERVER,
    SERVER_REQUEST_LEN,
};

use crate::state::{ipv4_to_host_order, MasterState};

const RECV_BUF_MAX: usize = 512;

/// Handle a single accepted TCP connection: one read, one response, then close.
pub fn handle_connection(
    mut stream: TcpStream,
    state: Arc<Mutex<MasterState>>,
) -> std::io::Result<()> {
    let peer = stream.peer_addr()?;
    let ip4 = match peer.ip() {
        IpAddr::V4(v4) => v4,
        IpAddr::V6(_) => {
            warn!(%peer, "rejecting non-IPv4 peer");
            return Ok(());
        }
    };
    let ip_host = ipv4_to_host_order(ip4);

    let mut buf = [0u8; RECV_BUF_MAX];
    let n = stream.read(&mut buf)?;
    if n < 4 {
        debug!(%peer, n, "packet too short for magic");
        return Ok(());
    }

    let magic = u32::from_le_bytes(buf[0..4].try_into().unwrap());

    match magic {
        MS_SERVER => {
            if n < SERVER_REQUEST_LEN {
                debug!(%peer, n, "server register packet truncated");
                return Ok(());
            }
            match ServerRegisterRequest::decode(&buf[..n]) {
                Ok(req) => {
                    info!(
                        version = req.version,
                        q_port = req.q_port,
                        ip = format!("{ip_host:08X}"),
                        "server register"
                    );
                    let mut g = state.lock().expect("state mutex poisoned");
                    g.register_server(req.version, ip_host, req.q_port);
                }
                Err(e) => debug!(%peer, error = %e, "invalid server register decode"),
            }
        }
        MS_CLIENT => {
            if n < CLIENT_REQUEST_LEN {
                debug!(%peer, n, "client list packet truncated");
                return Ok(());
            }
            match ClientListRequest::decode(&buf[..n]) {
                Ok(req) => {
                    info!(version = req.version, "client list request");
                    let resp = {
                        let g = state.lock().expect("state mutex poisoned");
                        g.list_for_version(req.version)
                    };
                    let payload = match resp.encode() {
                        Ok(p) => p,
                        Err(e) => {
                            warn!(error = %e, "failed to encode client response");
                            return Ok(());
                        }
                    };
                    stream.write_all(&payload)?;
                }
                Err(e) => debug!(%peer, error = %e, "invalid client request decode"),
            }
        }
        _ => {
            debug!(%peer, magic = format!("{magic:08X}"), "unknown magic");
        }
    }

    Ok(())
}
