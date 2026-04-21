//! TeknoMW3 binary protocol types: TCP master server messages and related constants.
//!
//! Wire formats are documented in `rust/docs/CONTRACTS.md`.

#![forbid(unsafe_code)]

mod error;
mod magic;
mod master;
mod udp;

pub use error::ProtocolError;
pub use magic::{
    GAME_SERVER_4BB, GAME_SERVER_4CC, GAME_SERVER_4CC_OLD, MS_CLEANUP_RATE_MS, MS_CLIENT,
    MS_LISTEN_PORT, MS_SERVER,
};
pub use master::{
    ClientListRequest, ClientListResponse, ServerEntry, ServerRegisterRequest, CLIENT_REQUEST_LEN,
    MAX_CLIENT_RESPONSE_ENTRIES, SERVER_ENTRY_LEN, SERVER_REQUEST_LEN,
};
pub use udp::{
    ServerInfoHeader, ServerInfoView, ServerQuery, SERVER_INFO_HEADER_LEN,
    SERVER_INFO_MAX_RAW_DATA, SERVER_QUERY_LEN,
};
