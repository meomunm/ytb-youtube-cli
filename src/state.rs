//! Persisted result list: `${XDG_CACHE_HOME:-$HOME/.cache}/ytb/results.tsv`.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::entry::Entry;

/// Resolve the cache directory from explicit env values (pure, testable).
///
/// Prefers `XDG_CACHE_HOME` when set and non-empty; otherwise `$HOME/.cache`;
/// falls back to `./.cache` if `HOME` is also missing.
fn resolve_cache_dir_from(xdg: Option<OsString>, home: Option<OsString>) -> PathBuf {
    if let Some(x) = xdg {
        if !x.is_empty() {
            let mut p = PathBuf::from(x);
            p.push("ytb");
            return p;
        }
    }
    let base = home
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut p = base;
    p.push(".cache");
    p.push("ytb");
    p
}

/// The cache directory for the current environment.
pub fn resolve_cache_dir() -> PathBuf {
    resolve_cache_dir_from(std::env::var_os("XDG_CACHE_HOME"), std::env::var_os("HOME"))
}

/// Path to the TSV state file.
pub fn state_path() -> PathBuf {
    resolve_cache_dir().join("results.tsv")
}

/// Read entries from a specific file, tolerating a missing file (empty list).
fn read_state_from(path: &Path) -> Vec<Entry> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    content.lines().filter_map(Entry::from_tsv).collect()
}

/// Read the current result list, or an empty list if none exists.
pub fn read_state() -> Vec<Entry> {
    read_state_from(&state_path())
}

/// Write `entries` into `dir/results.tsv`, creating `dir` as needed. Fully
/// overwrites any previous state.
fn write_state_to(dir: &Path, entries: &[Entry]) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    let mut out = String::new();
    for e in entries {
        out.push_str(&e.to_tsv());
        out.push('\n');
    }
    fs::write(dir.join("results.tsv"), out)
}

/// Persist the result list, overwriting any previous state.
pub fn write_state(entries: &[Entry]) -> std::io::Result<()> {
    write_state_to(&resolve_cache_dir(), entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(id: &str) -> Entry {
        Entry {
            id: id.to_string(),
            title: format!("title {id}"),
            duration: "100".to_string(),
            uploader: "chan".to_string(),
        }
    }

    #[test]
    fn xdg_wins_when_set() {
        let p = resolve_cache_dir_from(Some("/x".into()), Some("/home/u".into()));
        assert_eq!(p, PathBuf::from("/x/ytb"));
    }

    #[test]
    fn empty_xdg_falls_back_to_home() {
        let p = resolve_cache_dir_from(Some("".into()), Some("/home/u".into()));
        assert_eq!(p, PathBuf::from("/home/u/.cache/ytb"));
    }

    #[test]
    fn missing_home_falls_back_to_dot() {
        let p = resolve_cache_dir_from(None, None);
        assert_eq!(p, PathBuf::from("./.cache/ytb"));
    }

    #[test]
    fn read_missing_file_is_empty() {
        let dir = std::env::temp_dir().join("ytb-test-missing-xyzzy");
        let _ = fs::remove_dir_all(&dir);
        assert!(read_state_from(&dir.join("results.tsv")).is_empty());
    }

    #[test]
    fn write_then_read_roundtrips_and_creates_dir() {
        let dir = std::env::temp_dir().join("ytb-test-rt-abc123/nested");
        let _ = fs::remove_dir_all(std::env::temp_dir().join("ytb-test-rt-abc123"));
        let items = vec![e("a"), e("b")];
        write_state_to(&dir, &items).unwrap();
        let back = read_state_from(&dir.join("results.tsv"));
        assert_eq!(back, items);
        let _ = fs::remove_dir_all(std::env::temp_dir().join("ytb-test-rt-abc123"));
    }
}
