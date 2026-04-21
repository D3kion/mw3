//! Simple INI layout used by TeknoGods MW3: `[Section]`, `key=value`, keys before the first section
//! under `ROOT`. Saved with CRLF line endings for broad Windows tool compatibility.

use std::fs;
use std::path::Path;

use indexmap::IndexMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IniLoadError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("line {line}: {message}")]
    Parse { line: usize, message: String },
}

/// In-memory `teknogods.ini` (and similar) with stable read order for round-trips.
#[derive(Debug, Clone, Default)]
pub struct TeknogodsIni {
    sections: IndexMap<String, IndexMap<String, String>>,
}

impl TeknogodsIni {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(path: &Path) -> Result<Self, IniLoadError> {
        let s = fs::read_to_string(path)?;
        Self::parse(&s)
    }

    pub fn parse(contents: &str) -> Result<Self, IniLoadError> {
        let mut out = TeknogodsIni::new();
        let mut current = String::from("ROOT");

        for (idx, raw_line) in contents.lines().enumerate() {
            let line_no = idx + 1;
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') && line.len() >= 2 {
                current = line[1..line.len() - 1].to_string();
                out.sections.entry(current.clone()).or_default();
                continue;
            }
            let mut split = line.splitn(2, '=');
            let key = split
                .next()
                .ok_or_else(|| IniLoadError::Parse {
                    line: line_no,
                    message: "empty line after trim".into(),
                })?
                .trim();
            if key.is_empty() {
                return Err(IniLoadError::Parse {
                    line: line_no,
                    message: "empty key".into(),
                });
            }
            let value = split.next().map(str::trim).unwrap_or("");
            out.sections
                .entry(current.clone())
                .or_default()
                .insert(key.to_string(), value.to_string());
        }

        Ok(out)
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        fs::write(path, self.to_string_crlf())
    }

    /// Serialize with `\r\n` line endings and a blank line after each section.
    pub fn to_string_crlf(&self) -> String {
        let mut out = String::new();
        for (section, keys) in &self.sections {
            out.push('[');
            out.push_str(section);
            out.push(']');
            out.push_str("\r\n");
            for (k, v) in keys {
                out.push_str(k);
                out.push('=');
                out.push_str(v);
                out.push_str("\r\n");
            }
            out.push_str("\r\n");
        }
        out
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.sections
            .get(section)
            .and_then(|m| m.get(key))
            .map(String::as_str)
    }

    /// Missing key returns an empty string.
    pub fn get_or_empty(&self, section: &str, key: &str) -> &str {
        self.get(section, key).unwrap_or("")
    }

    pub fn set(&mut self, section: &str, key: &str, value: &str) {
        self.sections
            .entry(section.to_string())
            .or_default()
            .insert(key.to_string(), value.to_string());
    }

    pub fn sections(&self) -> impl Iterator<Item = (&str, &IndexMap<String, String>)> {
        self.sections.iter().map(|(s, m)| (s.as_str(), m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_preserves_order() {
        let src = "[Settings]\r\nName=foo\r\nFOV=80\r\n[Network]\r\nMasterServer=host:27017\r\n";
        let ini = TeknogodsIni::parse(src).unwrap();
        assert_eq!(ini.get_or_empty("Settings", "Name"), "foo");
        assert_eq!(ini.get_or_empty("Settings", "FOV"), "80");
        assert_eq!(ini.get_or_empty("Network", "MasterServer"), "host:27017");
        let again = TeknogodsIni::parse(&ini.to_string_crlf()).unwrap();
        assert_eq!(again.get_or_empty("Settings", "Name"), "foo");
    }

    #[test]
    fn root_keys_before_first_section() {
        let src = "orphan=value\r\n[Settings]\r\nName=x\r\n";
        let ini = TeknogodsIni::parse(src).unwrap();
        assert_eq!(ini.get_or_empty("ROOT", "orphan"), "value");
        assert_eq!(ini.get_or_empty("Settings", "Name"), "x");
    }

    #[test]
    fn key_without_equals() {
        let src = "[S]\r\nflag\r\n";
        let ini = TeknogodsIni::parse(src).unwrap();
        assert_eq!(ini.get_or_empty("S", "flag"), "");
    }
}
