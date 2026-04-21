//! Launcher utilities: `teknogods.ini` parsing, game command-line builders, and HTTP update-check.
#![forbid(unsafe_code)]

pub mod ini;
pub mod launch;
pub mod update;

pub use ini::{IniLoadError, TeknogodsIni};
pub use launch::{
    client_connect_args, client_lan_args, dedicated_args, sp_args, sp_coop_host_args,
};
pub use update::{
    fetch_mw3_update_check, is_remote_newer_than_local, numeric_version_key, parse_update_response,
    UpdateCheck, UpdateFetchError, UpdateParseError, DEFAULT_MW3_UPDATE_URL,
};
