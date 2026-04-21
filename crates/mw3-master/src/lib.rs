//! TeknoMW3 TCP master server library (BOOB / COKE protocol).

#![forbid(unsafe_code)]

pub mod conn;
pub mod state;

pub use state::{ipv4_to_host_order, stale_interval, MasterState};
