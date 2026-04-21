//! Multichar literals as `u32` on the wire (little-endian), matching MSVC/GCC TeknoMW3 builds.
//!
//! See `rust/docs/CONTRACTS.md` in the repository (path from this file: `../../../docs/CONTRACTS.md`).

/// `MW3_MS_SERVER_MAGIC4CC` - dedicated server registration (TCP master).
pub const MS_SERVER: u32 = 0x424F4F42; // "BOOB"

/// `MW3_MS_CLIENT_MAGIC4CC` - client server-list request (TCP master).
pub const MS_CLIENT: u32 = 0x434F4B45; // "COKE"

/// `MW3_SERVER_4BB` - same numeric value as `MS_SERVER` (UDP / master update path in DLL).
pub const GAME_SERVER_4BB: u32 = MS_SERVER;

/// `MW3_SERVER_4CC` - UDP server query / info (not TCP master).
pub const GAME_SERVER_4CC: u32 = 0x4C4F4F50; // "POOL"

/// `MW3_SERVER_4CC_old`.
pub const GAME_SERVER_4CC_OLD: u32 = 0x504F4F50; // "POOP"

/// Default master server listen port (`MW3_MS_LISTEN_PORT`).
pub const MS_LISTEN_PORT: u16 = 27017;

/// Inactive server entry cleanup interval in milliseconds (`MW3_MS_CLEANUP_RATE`).
pub const MS_CLEANUP_RATE_MS: u32 = 30_000;
