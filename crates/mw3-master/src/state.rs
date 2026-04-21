//! In-memory server lists keyed by game `Version`.

use std::collections::BTreeMap;
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};

use mw3_protocol::{ClientListResponse, ServerEntry, MS_CLEANUP_RATE_MS};

/// Key: `(ip as u64) << 32 | q_port`.
#[inline]
pub fn entry_key(ip_host: u32, q_port: u16) -> u64 {
    (ip_host as u64) << 32 | q_port as u64
}

/// `ntohl(sockaddr_in.sin_addr.s_addr)` on little-endian hosts (Windows).
#[inline]
pub fn ipv4_to_host_order(ip: Ipv4Addr) -> u32 {
    u32::from_be_bytes(ip.octets()).swap_bytes()
}

#[derive(Debug, Default)]
pub struct MasterState {
    /// Per-version map: composite key -> last heartbeat time.
    by_version: BTreeMap<u32, BTreeMap<u64, Instant>>,
}

impl MasterState {
    pub fn total_entries(&self) -> usize {
        self.by_version.values().map(|m| m.len()).sum()
    }

    pub fn register_server(&mut self, version: u32, ip_host: u32, q_port: u16) {
        let key = entry_key(ip_host, q_port);
        let now = Instant::now();
        self.by_version.entry(version).or_default().insert(key, now);
    }

    /// Sorted by composite key (`BTreeMap` order).
    pub fn list_for_version(&self, version: u32) -> ClientListResponse {
        let entries = self
            .by_version
            .get(&version)
            .map(|m| {
                m.keys()
                    .map(|k| ServerEntry {
                        ip_address: (*k >> 32) as u32,
                        q_port: *k as u16,
                    })
                    .collect()
            })
            .unwrap_or_default();
        ClientListResponse { entries }
    }

    /// Remove entries not refreshed within `STALE_AFTER` (`MS_CLEANUP_RATE_MS`).
    pub fn purge_stale(&mut self, stale_after: Duration) {
        let now = Instant::now();
        for list in self.by_version.values_mut() {
            list.retain(|_, t| now.duration_since(*t) <= stale_after);
        }
        self.by_version.retain(|_, m| !m.is_empty());
    }
}

/// Default stale interval from protocol (`MW3_MS_CLEANUP_RATE`).
pub fn stale_interval() -> Duration {
    Duration::from_millis(u64::from(MS_CLEANUP_RATE_MS))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_roundtrip() {
        let ip = 0x0101_A8C0_u32;
        let q = 0x6987_u16;
        let k = entry_key(ip, q);
        assert_eq!(k >> 32, ip as u64);
        assert_eq!(k as u16, q);
    }

    #[test]
    fn list_sorted_by_key() {
        let mut st = MasterState::default();
        st.register_server(1, 0xC0A8_0101, 100);
        st.register_server(1, 0xC0A8_0101, 50);
        st.register_server(1, 0x0A00_0001, 200);
        let r = st.list_for_version(1);
        let ips: Vec<u32> = r.entries.iter().map(|e| e.ip_address).collect();
        assert_eq!(ips, vec![0x0A00_0001, 0xC0A8_0101, 0xC0A8_0101]);
        let ports: Vec<u16> = r.entries.iter().map(|e| e.q_port).collect();
        assert_eq!(ports, vec![200, 50, 100]);
    }

    #[test]
    fn purge_removes_old() {
        let mut st = MasterState::default();
        st.register_server(1, 0x0101_0101, 1);
        let k = entry_key(0x0101_0101, 1);
        let old = Instant::now() - Duration::from_secs(120);
        st.by_version.get_mut(&1).unwrap().insert(k, old);
        st.purge_stale(Duration::from_millis(30_000));
        assert!(st.by_version.is_empty());
    }
}
