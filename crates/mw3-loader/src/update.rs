//! HTTP update-check endpoint: GET response body `version;url` on the first non-empty line.

use std::time::Duration;

use thiserror::Error;

/// Default TeknoGods MW3 update-check URL.
pub const DEFAULT_MW3_UPDATE_URL: &str = "http://teknogods.com:8080/updatecheck/?project=mw3";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateCheck {
    pub version_label: String,
    pub open_url: String,
}

#[derive(Debug, Error)]
pub enum UpdateParseError {
    #[error("expected 'version;url'")]
    InvalidFormat,
    #[error("empty version or url")]
    EmptyField,
}

#[derive(Debug, Error)]
pub enum UpdateFetchError {
    #[error("HTTP status {0}")]
    Status(u16),
    #[error("transport: {0}")]
    Transport(String),
    #[error("parse: {0}")]
    Parse(#[from] UpdateParseError),
}

/// Strip non-digits and parse to a single integer for coarse version ordering
/// (e.g. `1.2.10` → `1210`).
pub fn numeric_version_key(label: &str) -> Option<u64> {
    let digits: String = label.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse().ok()
}

pub fn is_remote_newer_than_local(remote_label: &str, local_label: &str) -> bool {
    match (
        numeric_version_key(remote_label),
        numeric_version_key(local_label),
    ) {
        (Some(r), Some(l)) => r > l,
        (Some(_), None) => true,
        _ => false,
    }
}

/// Parse the first non-empty line of the response body (ASCII).
pub fn parse_update_response(body: &str) -> Result<UpdateCheck, UpdateParseError> {
    let line = body
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or(body)
        .trim();
    let mut parts = line.splitn(2, ';');
    let version_label = parts.next().unwrap_or("").trim().to_string();
    let raw_url = parts.next().ok_or(UpdateParseError::InvalidFormat)?.trim();
    let open_url: String = raw_url
        .chars()
        .filter(|c| *c != '\r' && *c != '\n')
        .collect();
    if version_label.is_empty() || open_url.is_empty() {
        return Err(UpdateParseError::EmptyField);
    }
    Ok(UpdateCheck {
        version_label,
        open_url,
    })
}

pub fn fetch_mw3_update_check(
    url: &str,
    timeout: Duration,
) -> Result<UpdateCheck, UpdateFetchError> {
    let resp = ureq::get(url)
        .timeout(timeout)
        .call()
        .map_err(|e: ureq::Error| UpdateFetchError::Transport(e.to_string()))?;
    let status = resp.status();
    if !(200..300).contains(&status) {
        return Err(UpdateFetchError::Status(status));
    }
    let body = resp
        .into_string()
        .map_err(|e: std::io::Error| UpdateFetchError::Transport(e.to_string()))?;
    Ok(parse_update_response(&body)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_semicolon_line() {
        let u = parse_update_response("2.1.0;http://example.com/dl\r\n").unwrap();
        assert_eq!(u.version_label, "2.1.0");
        assert_eq!(u.open_url, "http://example.com/dl");
    }

    #[test]
    fn version_key_compare() {
        assert!(is_remote_newer_than_local("2.0.0", "1.9.9"));
        assert!(!is_remote_newer_than_local("1.0.0", "2.0.0"));
        assert_eq!(numeric_version_key("1.2.3"), Some(123));
    }
}
