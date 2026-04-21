//! UDP game server query / response layout from `game_server_items.h` (`#pragma pack(1)`).

use crate::error::ProtocolError;
use crate::magic::{GAME_SERVER_4CC, GAME_SERVER_4CC_OLD};

/// `MW3_SERVER_QUERY` size (magic + timestamp).
pub const SERVER_QUERY_LEN: usize = 8;

/// Bytes before `RawData[]` in `MW3_SERVER_INFO`.
pub const SERVER_INFO_HEADER_LEN: usize = 81;

/// `MW3_SERVER_INFO_MAX_RAW_DATA_SIZE` from C++.
pub const SERVER_INFO_MAX_RAW_DATA: usize = 2048;

/// UDP query sent to a game server (`MW3_SERVER_QUERY`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerQuery {
    /// Usually `GAME_SERVER_4CC` (POOL) or `GAME_SERVER_4CC_OLD` (POOP).
    pub magic: u32,
    /// Often filled with a millisecond timestamp (may wrap).
    pub timestamp: u32,
}

impl ServerQuery {
    pub fn new_pool(timestamp: u32) -> Self {
        Self {
            magic: GAME_SERVER_4CC,
            timestamp,
        }
    }

    pub fn new_poop(timestamp: u32) -> Self {
        Self {
            magic: GAME_SERVER_4CC_OLD,
            timestamp,
        }
    }

    pub fn encode(self) -> [u8; SERVER_QUERY_LEN] {
        let mut b = [0u8; SERVER_QUERY_LEN];
        b[0..4].copy_from_slice(&self.magic.to_le_bytes());
        b[4..8].copy_from_slice(&self.timestamp.to_le_bytes());
        b
    }

    pub fn decode(buf: &[u8]) -> Result<Self, ProtocolError> {
        if buf.len() < SERVER_QUERY_LEN {
            return Err(ProtocolError::UnexpectedEof {
                need: SERVER_QUERY_LEN,
                got: buf.len(),
            });
        }
        Ok(Self {
            magic: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
            timestamp: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
        })
    }
}

/// Fixed prefix of `MW3_SERVER_INFO` (everything before `RawData`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerInfoHeader {
    pub magic: u32,
    pub timestamp: u32,
    pub players: i32,
    pub max_players: i32,
    pub password_protected: u8,
    pub dedicated: u32,
    pub server_version: i32,
    pub steam_id: u64,
    pub game_ip_int: u32,
    pub game_ip_ext: u32,
    pub game_port: u16,
    pub query_port: u16,
    pub net_port: u16,
    pub sec_id: [u8; 8],
    pub sec_key: [u8; 16],
    pub map_name_ptr: u16,
    pub server_name_ptr: u16,
    pub server_tags_ptr: u16,
    pub server_infos_ptr: u16,
    pub raw_data_size: u16,
}

impl ServerInfoHeader {
    pub fn decode(buf: &[u8]) -> Result<Self, ProtocolError> {
        if buf.len() < SERVER_INFO_HEADER_LEN {
            return Err(ProtocolError::UnexpectedEof {
                need: SERVER_INFO_HEADER_LEN,
                got: buf.len(),
            });
        }
        let b = buf;
        let mut o = 0usize;
        let read_u32 = |b: &[u8], o: &mut usize| -> u32 {
            let v = u32::from_le_bytes(b[*o..*o + 4].try_into().unwrap());
            *o += 4;
            v
        };
        let read_i32 = |b: &[u8], o: &mut usize| -> i32 { read_u32(b, o) as i32 };
        let read_u16 = |b: &[u8], o: &mut usize| -> u16 {
            let v = u16::from_le_bytes(b[*o..*o + 2].try_into().unwrap());
            *o += 2;
            v
        };
        let read_u64 = |b: &[u8], o: &mut usize| -> u64 {
            let v = u64::from_le_bytes(b[*o..*o + 8].try_into().unwrap());
            *o += 8;
            v
        };
        let read_u8 = |b: &[u8], o: &mut usize| -> u8 {
            let v = b[*o];
            *o += 1;
            v
        };
        let magic = read_u32(b, &mut o);
        let timestamp = read_u32(b, &mut o);
        let players = read_i32(b, &mut o);
        let max_players = read_i32(b, &mut o);
        let password_protected = read_u8(b, &mut o);
        let dedicated = read_u32(b, &mut o);
        let server_version = read_i32(b, &mut o);
        let steam_id = read_u64(b, &mut o);
        let game_ip_int = read_u32(b, &mut o);
        let game_ip_ext = read_u32(b, &mut o);
        let game_port = read_u16(b, &mut o);
        let query_port = read_u16(b, &mut o);
        let net_port = read_u16(b, &mut o);
        let mut sec_id = [0u8; 8];
        sec_id.copy_from_slice(&b[o..o + 8]);
        o += 8;
        let mut sec_key = [0u8; 16];
        sec_key.copy_from_slice(&b[o..o + 16]);
        o += 16;
        let map_name_ptr = read_u16(b, &mut o);
        let server_name_ptr = read_u16(b, &mut o);
        let server_tags_ptr = read_u16(b, &mut o);
        let server_infos_ptr = read_u16(b, &mut o);
        let raw_data_size = read_u16(b, &mut o);
        debug_assert_eq!(o, SERVER_INFO_HEADER_LEN);
        Ok(Self {
            magic,
            timestamp,
            players,
            max_players,
            password_protected,
            dedicated,
            server_version,
            steam_id,
            game_ip_int,
            game_ip_ext,
            game_port,
            query_port,
            net_port,
            sec_id,
            sec_key,
            map_name_ptr,
            server_name_ptr,
            server_tags_ptr,
            server_infos_ptr,
            raw_data_size,
        })
    }
}

/// Borrowed view of a datagram after the fixed `MW3_SERVER_INFO` header.
#[derive(Debug, Clone, Copy)]
pub struct ServerInfoView<'a> {
    pub header: ServerInfoHeader,
    /// `RawData` bytes actually present (may be shorter than `header.raw_data_size` or 2048).
    pub raw_data: &'a [u8],
}

impl<'a> ServerInfoView<'a> {
    pub fn decode(buf: &'a [u8]) -> Result<Self, ProtocolError> {
        let header = ServerInfoHeader::decode(buf)?;
        let raw_data = buf.get(SERVER_INFO_HEADER_LEN..).unwrap_or(&[]);
        Ok(Self { header, raw_data })
    }

    /// C-style string at `RawData[header.server_infos_ptr]`.
    pub fn server_infos_cstr(&self) -> Option<&'a str> {
        let off = self.header.server_infos_ptr as usize;
        if off >= self.raw_data.len() {
            return None;
        }
        let tail = &self.raw_data[off..];
        let end = tail.iter().position(|&b| b == 0).unwrap_or(tail.len());
        core::str::from_utf8(&tail[..end]).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_query_pool_roundtrip() {
        let q = ServerQuery::new_pool(0xDEAD_BEEF);
        let enc = q.encode();
        assert_eq!(enc, [0x50, 0x4F, 0x4F, 0x4C, 0xEF, 0xBE, 0xAD, 0xDE]);
        let d = ServerQuery::decode(&enc).unwrap();
        assert_eq!(d, q);
    }

    #[test]
    fn server_query_poop_magic() {
        let q = ServerQuery::new_poop(1);
        assert_eq!(q.magic, GAME_SERVER_4CC_OLD);
    }

    #[test]
    fn server_info_header_roundtrip_synthetic() {
        let mut buf = vec![0u8; SERVER_INFO_HEADER_LEN + 16];
        buf[0..4].copy_from_slice(&GAME_SERVER_4CC.to_le_bytes());
        buf[77..79].copy_from_slice(&0u16.to_le_bytes()); // server_infos_ptr -> start of RawData
        buf[79..81].copy_from_slice(&16u16.to_le_bytes()); // raw_data_size (declared)
        buf[SERVER_INFO_HEADER_LEN..SERVER_INFO_HEADER_LEN + 3].copy_from_slice(b"hi\0");

        let v = ServerInfoView::decode(&buf).unwrap();
        assert_eq!(v.header.server_infos_ptr, 0);
        assert_eq!(v.server_infos_cstr(), Some("hi"));
    }
}
