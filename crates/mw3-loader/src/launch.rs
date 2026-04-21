//! Game command-line fragments for MW3 multiplayer / dedicated / SP flows.

/// Dedicated MP server: `iw5mp_server.exe` arguments.
pub fn dedicated_args(port: &str, use_keys: bool) -> String {
    let port = port.trim();
    let core = format!("+set dedicated 1 +set net_port {port}");
    if use_keys {
        format!("+usekeys {core}")
    } else {
        core
    }
}

/// MP LAN listen: `iw5mp.exe` with optional `+usekeys`.
pub fn client_lan_args(use_keys: bool) -> String {
    if use_keys {
        "+usekeys".to_string()
    } else {
        String::new()
    }
}

/// MP direct connect: `+server {host}:{port}` (`host` should be a resolvable address).
pub fn client_connect_args(host: &str, port: &str) -> String {
    format!("+server {}:{}", host.trim(), port.trim())
}

/// SP coop host: `+server ip:0` (for `iw5sp.exe`).
pub fn sp_coop_host_args(ip: &str) -> String {
    format!("+server {}:0", ip.trim())
}

/// Plain single-player: no extra args (`iw5sp.exe`).
pub fn sp_args() -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedicated_args_format() {
        assert_eq!(
            dedicated_args("27015", true),
            "+usekeys +set dedicated 1 +set net_port 27015"
        );
        assert_eq!(
            dedicated_args("27015", false),
            "+set dedicated 1 +set net_port 27015"
        );
    }

    #[test]
    fn client_connect_matches() {
        assert_eq!(
            client_connect_args("192.0.2.1", "27016"),
            "+server 192.0.2.1:27016"
        );
    }
}
