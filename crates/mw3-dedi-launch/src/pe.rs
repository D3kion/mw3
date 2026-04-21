//! PE `.rdata` scan: locate the `steam_api.dll` string for the dedicated-server patch.

use goblin::pe::PE;
use thiserror::Error;

/// Tail window size (`0xB000` bytes) over `.rdata` raw bytes.
pub const RDATA_TAIL_SCAN: usize = 0xB000;

pub const TEKNO_MARK_SUBSTR: &[u8] = br"pc\iw5mp_server";
pub const STEAM_API_DLL: &[u8] = b"steam_api.dll";

#[derive(Debug, Error)]
pub enum PeLocateError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("PE parse: {0}")]
    Goblin(String),
    #[error("no suitable .rdata section (or too small)")]
    NoRdata,
    #[error("marker {0:?} not found in .rdata tail")]
    MarkerNotFound(&'static str),
    #[error("'{0}' not found in .rdata tail (after marker check)")]
    SteamApiNotFound(&'static str),
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn is_rdata_section_name(name: Result<&str, goblin::error::Error>) -> bool {
    match name {
        Ok(n) => {
            let n = n.trim_end_matches('\0');
            n == ".rdata" || n.starts_with(".rdata")
        }
        Err(_) => false,
    }
}

/// Scan one mapped `.rdata` slice for `pc\iw5mp_server` then `steam_api.dll`; returns VA = image_base + RVA.
fn steam_api_va_from_section_raw(
    section_raw: &[u8],
    raw_ptr: usize,
    virtual_address: u32,
    pointer_to_raw_data: u32,
    image_base: u64,
) -> Result<u64, PeLocateError> {
    let take = RDATA_TAIL_SCAN.min(section_raw.len());
    let win_start = section_raw.len() - take;
    let tail = &section_raw[win_start..];
    if find_subslice(tail, TEKNO_MARK_SUBSTR).is_none() {
        return Err(PeLocateError::MarkerNotFound("pc\\iw5mp_server"));
    }
    let rel_in_tail = find_subslice(tail, STEAM_API_DLL)
        .ok_or(PeLocateError::SteamApiNotFound("steam_api.dll"))?;
    let rel = win_start + rel_in_tail;
    let file_off = raw_ptr + rel;
    let rva = virtual_address as u64 + (file_off as u64 - u64::from(pointer_to_raw_data));
    Ok(image_base.saturating_add(rva))
}

/// Read `path`, find `steam_api.dll` in `.rdata` tail, return **virtual address** (ImageBase + RVA).
///
/// Tries every section named `.rdata` or `.rdata*` (some linkers split read-only data); the first hit wins.
pub fn locate_steam_api_dll_va(path: &std::path::Path) -> Result<u64, PeLocateError> {
    let bytes = std::fs::read(path)?;
    let pe = PE::parse(&bytes).map_err(|e| PeLocateError::Goblin(e.to_string()))?;
    let image_base = pe.image_base as u64;
    let mut last_err: Option<PeLocateError> = None;
    let mut any_eligible = false;

    for section in pe
        .sections
        .iter()
        .filter(|s| is_rdata_section_name(s.name()))
    {
        let raw_ptr = section.pointer_to_raw_data as usize;
        let raw_sz = section.size_of_raw_data as usize;
        if raw_sz < 0x1000 {
            continue;
        }
        let end = match raw_ptr.checked_add(raw_sz) {
            Some(e) => e,
            None => {
                last_err = Some(PeLocateError::NoRdata);
                continue;
            }
        };
        if end > bytes.len() {
            last_err = Some(PeLocateError::Goblin(
                "section raw range out of file".into(),
            ));
            continue;
        }
        any_eligible = true;
        let section_raw = &bytes[raw_ptr..end];
        match steam_api_va_from_section_raw(
            section_raw,
            raw_ptr,
            section.virtual_address,
            section.pointer_to_raw_data,
            image_base,
        ) {
            Ok(va) => return Ok(va),
            Err(e) => last_err = Some(e),
        }
    }

    if !any_eligible {
        return Err(PeLocateError::NoRdata);
    }
    Err(last_err.unwrap_or(PeLocateError::NoRdata))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_tail(section_raw: &[u8], needle: &[u8]) -> Option<usize> {
        if section_raw.is_empty() {
            return None;
        }
        let take = RDATA_TAIL_SCAN.min(section_raw.len());
        let start = section_raw.len() - take;
        let tail = &section_raw[start..];
        find_subslice(tail, TEKNO_MARK_SUBSTR)?;
        let rel = find_subslice(tail, needle)?;
        Some(start + rel)
    }

    #[test]
    fn tail_search_requires_both_markers() {
        let mut raw = vec![0u8; 0x100];
        raw.extend_from_slice(TEKNO_MARK_SUBSTR);
        raw.extend_from_slice(STEAM_API_DLL);
        let i = find_tail(&raw, STEAM_API_DLL).expect("found");
        assert_eq!(&raw[i..i + STEAM_API_DLL.len()], STEAM_API_DLL);
    }

    #[test]
    fn tail_search_fails_without_pc_mark() {
        let raw = vec![b'x'; 0x200];
        assert!(find_tail(&raw, STEAM_API_DLL).is_none());
    }

    #[test]
    fn section_raw_va_matches_expected_rva_math() {
        let mut raw = vec![0u8; 0x100];
        raw.extend_from_slice(TEKNO_MARK_SUBSTR);
        raw.extend_from_slice(STEAM_API_DLL);
        raw.resize(0x2000, 0);
        let raw_ptr = 0x2000usize;
        let virtual_address = 0x5000u32;
        let pointer_to_raw_data = 0x2000u32;
        let image_base = 0x1400_0000u64;
        let va = steam_api_va_from_section_raw(
            &raw,
            raw_ptr,
            virtual_address,
            pointer_to_raw_data,
            image_base,
        )
        .expect("markers in tail");
        let take = RDATA_TAIL_SCAN.min(raw.len());
        let win_start = raw.len() - take;
        let tail = &raw[win_start..];
        let rel_in_tail = find_subslice(tail, STEAM_API_DLL).expect("dll in tail");
        let rel = win_start + rel_in_tail;
        let file_off = raw_ptr + rel;
        let rva = u64::from(virtual_address) + (file_off as u64 - u64::from(pointer_to_raw_data));
        assert_eq!(va, image_base.saturating_add(rva));
    }
}
