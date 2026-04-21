//! Locate the `steam_api.dll` string in `iw5mp_server.exe` and compute the virtual address for the in-process patch.
//!
//! Windows-only process spawning lives in [`launch_suspended_patch_resume`] when built for `cfg(windows)`.

mod pe;

pub use pe::{
    locate_steam_api_dll_va, PeLocateError, RDATA_TAIL_SCAN, STEAM_API_DLL, TEKNO_MARK_SUBSTR,
};

/// Maximum DLL name length written into the remote image (including trailing NUL), matching `char g_DllName[13]`.
pub const MAX_DLL_NAME_BYTES: usize = 13;

/// Build the 13-byte DLL name buffer for `WriteProcessMemory` (NUL-padded).
pub fn dll_name_bytes(name: &str) -> Result<[u8; MAX_DLL_NAME_BYTES], String> {
    let b = name.as_bytes();
    if b.len() >= MAX_DLL_NAME_BYTES {
        return Err(format!(
            "DLL name must fit in {} bytes including NUL (got {} bytes)",
            MAX_DLL_NAME_BYTES,
            b.len()
        ));
    }
    let mut out = [0u8; MAX_DLL_NAME_BYTES];
    out[..b.len()].copy_from_slice(b);
    Ok(out)
}

#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::{launch_suspended_patch_resume, LaunchError, LaunchOptions};
