//! TCP master server messages (`MW3_MS_*` from `MasterServerProtocol.hpp`).

use crate::error::ProtocolError;
use crate::magic;

/// Maximum entries in one client response (DoS guard).
pub const MAX_CLIENT_RESPONSE_ENTRIES: usize = 16_384;

/// Size of a full server registration packet.
pub const SERVER_REQUEST_LEN: usize = 10;

/// Size of a full client list-request packet.
pub const CLIENT_REQUEST_LEN: usize = 8;

/// Size of one server entry in the list response.
pub const SERVER_ENTRY_LEN: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServerEntry {
    /// IPv4 address in **host byte order** (same as `DWORD` after `ntohl` in `master_server.cpp`).
    pub ip_address: u32,
    pub q_port: u16,
}

impl ServerEntry {
    pub fn encode(self) -> [u8; SERVER_ENTRY_LEN] {
        let mut b = [0u8; SERVER_ENTRY_LEN];
        b[0..4].copy_from_slice(&self.ip_address.to_le_bytes());
        b[4..6].copy_from_slice(&self.q_port.to_le_bytes());
        b
    }

    pub fn decode(buf: &[u8]) -> Result<Self, ProtocolError> {
        if buf.len() < SERVER_ENTRY_LEN {
            return Err(ProtocolError::UnexpectedEof {
                need: SERVER_ENTRY_LEN,
                got: buf.len(),
            });
        }
        Ok(Self {
            ip_address: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
            q_port: u16::from_le_bytes(buf[4..6].try_into().unwrap()),
        })
    }
}

/// `MW3_MS_SERVER_REQUEST` / `MW3_MASTER_SERVER_UPDATE_MSG` (10 bytes, BOOB magic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerRegisterRequest {
    pub version: u32,
    pub q_port: u16,
}

impl ServerRegisterRequest {
    pub fn encode(self) -> [u8; SERVER_REQUEST_LEN] {
        let mut b = [0u8; SERVER_REQUEST_LEN];
        b[0..4].copy_from_slice(&magic::MS_SERVER.to_le_bytes());
        b[4..8].copy_from_slice(&self.version.to_le_bytes());
        b[8..10].copy_from_slice(&self.q_port.to_le_bytes());
        b
    }

    pub fn decode(buf: &[u8]) -> Result<Self, ProtocolError> {
        if buf.len() < SERVER_REQUEST_LEN {
            return Err(ProtocolError::UnexpectedEof {
                need: SERVER_REQUEST_LEN,
                got: buf.len(),
            });
        }
        let m = u32::from_le_bytes(buf[0..4].try_into().unwrap());
        if m != magic::MS_SERVER {
            return Err(ProtocolError::BadMagic {
                expected: magic::MS_SERVER,
                got: m,
            });
        }
        Ok(Self {
            version: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
            q_port: u16::from_le_bytes(buf[8..10].try_into().unwrap()),
        })
    }
}

/// `MW3_MS_CLIENT_REQUEST` (8 bytes, COKE magic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientListRequest {
    pub version: u32,
}

impl ClientListRequest {
    pub fn encode(self) -> [u8; CLIENT_REQUEST_LEN] {
        let mut b = [0u8; CLIENT_REQUEST_LEN];
        b[0..4].copy_from_slice(&magic::MS_CLIENT.to_le_bytes());
        b[4..8].copy_from_slice(&self.version.to_le_bytes());
        b
    }

    pub fn decode(buf: &[u8]) -> Result<Self, ProtocolError> {
        if buf.len() < CLIENT_REQUEST_LEN {
            return Err(ProtocolError::UnexpectedEof {
                need: CLIENT_REQUEST_LEN,
                got: buf.len(),
            });
        }
        let m = u32::from_le_bytes(buf[0..4].try_into().unwrap());
        if m != magic::MS_CLIENT {
            return Err(ProtocolError::BadMagic {
                expected: magic::MS_CLIENT,
                got: m,
            });
        }
        Ok(Self {
            version: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
        })
    }
}

/// `MW3_MS_CLIENT_RESPONSE`: header plus a sequence of entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientListResponse {
    pub entries: Vec<ServerEntry>,
}

impl ClientListResponse {
    /// Decode a full response (`4 + 6 * N` bytes).
    pub fn decode(buf: &[u8]) -> Result<Self, ProtocolError> {
        if buf.len() < 4 {
            return Err(ProtocolError::UnexpectedEof {
                need: 4,
                got: buf.len(),
            });
        }
        let n = u32::from_le_bytes(buf[0..4].try_into().unwrap());
        let n_usize = usize::try_from(n).map_err(|_| ProtocolError::SizeOverflow)?;
        if n_usize > MAX_CLIENT_RESPONSE_ENTRIES {
            return Err(ProtocolError::TooManyEntries {
                max: MAX_CLIENT_RESPONSE_ENTRIES,
                got: n_usize,
            });
        }
        let body = n_usize
            .checked_mul(SERVER_ENTRY_LEN)
            .ok_or(ProtocolError::SizeOverflow)?;
        let need = 4usize
            .checked_add(body)
            .ok_or(ProtocolError::SizeOverflow)?;
        if buf.len() < need {
            return Err(ProtocolError::ResponseTruncated {
                number_of_entries: n,
                need,
                got: buf.len(),
            });
        }
        let mut entries = Vec::with_capacity(n_usize);
        let mut off = 4;
        for _ in 0..n_usize {
            entries.push(ServerEntry::decode(&buf[off..])?);
            off += SERVER_ENTRY_LEN;
        }
        Ok(Self { entries })
    }

    /// Encode a response. An empty list yields four zero bytes (`NumberOfEntries == 0`).
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let n = self.entries.len();
        if n > MAX_CLIENT_RESPONSE_ENTRIES {
            return Err(ProtocolError::TooManyEntries {
                max: MAX_CLIENT_RESPONSE_ENTRIES,
                got: n,
            });
        }
        let n_u32 = u32::try_from(n).map_err(|_| ProtocolError::TooManyEntries {
            max: MAX_CLIENT_RESPONSE_ENTRIES,
            got: n,
        })?;
        let body = n
            .checked_mul(SERVER_ENTRY_LEN)
            .ok_or(ProtocolError::SizeOverflow)?;
        let total = 4usize
            .checked_add(body)
            .ok_or(ProtocolError::SizeOverflow)?;
        let mut v = Vec::with_capacity(total);
        v.extend_from_slice(&n_u32.to_le_bytes());
        for e in &self.entries {
            v.extend_from_slice(&e.encode());
        }
        debug_assert_eq!(v.len(), total);
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BOOB, version 1, qport 27015 (`0x6987` LE → bytes `87 69`).
    const FIXTURE_SERVER_REG: &[u8] = &[0x42, 0x4F, 0x4F, 0x42, 0x01, 0x00, 0x00, 0x00, 0x87, 0x69];

    /// COKE (`0x434F4B45` on the wire as LE: `45 4B 4F 43`), version `0xDEADBEEF`.
    const FIXTURE_CLIENT_REQ: &[u8] = &[0x45, 0x4B, 0x4F, 0x43, 0xEF, 0xBE, 0xAD, 0xDE];

    #[test]
    fn server_register_roundtrip() {
        let d = ServerRegisterRequest::decode(FIXTURE_SERVER_REG).unwrap();
        assert_eq!(
            d,
            ServerRegisterRequest {
                version: 1,
                q_port: 27015
            }
        );
        assert_eq!(d.encode().as_slice(), FIXTURE_SERVER_REG);
    }

    #[test]
    fn client_list_roundtrip() {
        let d = ClientListRequest::decode(FIXTURE_CLIENT_REQ).unwrap();
        assert_eq!(
            d,
            ClientListRequest {
                version: 0xDEAD_BEEF
            }
        );
        assert_eq!(d.encode().as_slice(), FIXTURE_CLIENT_REQ);
    }

    #[test]
    fn client_response_empty() {
        let r = ClientListResponse { entries: vec![] };
        let enc = r.encode().unwrap();
        assert_eq!(enc, vec![0, 0, 0, 0]);
        let dec = ClientListResponse::decode(&enc).unwrap();
        assert_eq!(dec.entries.len(), 0);
    }

    #[test]
    fn client_response_two_entries_fixture() {
        // Two entries; ip 192.168.1.1 LE = 01 01 A8 C0; qport 0x1111
        let raw: Vec<u8> = vec![
            0x02, 0x00, 0x00, 0x00, // count
            0x01, 0x01, 0xA8, 0xC0, 0x11, 0x11, 0, 0, 0, 0, 0, 0,
        ];
        let d = ClientListResponse::decode(&raw).unwrap();
        assert_eq!(d.entries.len(), 2);
        assert_eq!(
            d.entries[0],
            ServerEntry {
                ip_address: 0xC0A8_0101,
                q_port: 0x1111
            }
        );
        assert_eq!(
            d.entries[1],
            ServerEntry {
                ip_address: 0,
                q_port: 0
            }
        );
        assert_eq!(d.encode().unwrap(), raw);
    }

    #[test]
    fn truncated_server_request() {
        let err = ServerRegisterRequest::decode(&FIXTURE_SERVER_REG[..6]).unwrap_err();
        assert_eq!(err, ProtocolError::UnexpectedEof { need: 10, got: 6 });
    }

    #[test]
    fn bad_magic_server() {
        let mut b = FIXTURE_SERVER_REG.to_vec();
        b[0] = 0xFF;
        let err = ServerRegisterRequest::decode(&b).unwrap_err();
        assert!(matches!(err, ProtocolError::BadMagic { .. }));
    }

    #[test]
    fn truncated_client_response() {
        let raw = vec![0x02, 0x00, 0x00, 0x00, 0x01, 0x02]; // need 16 bytes total
        let err = ClientListResponse::decode(&raw).unwrap_err();
        assert!(matches!(
            err,
            ProtocolError::ResponseTruncated { .. } | ProtocolError::UnexpectedEof { .. }
        ));
    }

    #[test]
    fn too_many_entries_decode() {
        let n = (MAX_CLIENT_RESPONSE_ENTRIES as u32) + 1;
        let mut raw = Vec::new();
        raw.extend_from_slice(&n.to_le_bytes());
        let err = ClientListResponse::decode(&raw).unwrap_err();
        assert!(matches!(err, ProtocolError::TooManyEntries { .. }));
    }
}
