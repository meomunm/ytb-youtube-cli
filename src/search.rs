//! Searching YouTube via `yt-dlp` and parsing its output into entries.

use std::process::Command;

use crate::entry::Entry;
use crate::err::AppError;

/// The `--print` template we hand to yt-dlp, one tab-separated row per result.
const PRINT_TEMPLATE: &str = "%(id)s\t%(title)s\t%(duration)s\t%(uploader)s";

/// How many results to fetch, from `YTB_COUNT` (default 20).
pub fn count_from_env() -> usize {
    count_from(std::env::var("YTB_COUNT").ok())
}

/// Pure core of [`count_from_env`]: parse an optional raw value.
fn count_from(raw: Option<String>) -> usize {
    raw.and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|&n| n >= 1)
        .unwrap_or(20)
}

/// Build the `ytsearch{N}:{query}` argument yt-dlp expects.
pub fn build_query(count: usize, query: &str) -> String {
    format!("ytsearch{count}:{query}")
}

/// Parse yt-dlp's stdout into entries, skipping any malformed lines.
pub fn parse_output(stdout: &str) -> Vec<Entry> {
    stdout.lines().filter_map(Entry::from_tsv).collect()
}

/// Run a search and return the parsed entries.
///
/// Progress goes to stderr (so `ytb url 1 | pbcopy` stays clean); only the
/// caller prints the list to stdout.
pub fn run_search(query: &str) -> Result<Vec<Entry>, AppError> {
    let count = count_from_env();
    eprintln!("Đang tìm \"{query}\"...");

    let output = Command::new("yt-dlp")
        .arg(build_query(count, query))
        .arg("--flat-playlist")
        .arg("--no-warnings")
        .arg("--ignore-errors")
        .arg("--print")
        .arg(PRINT_TEMPLATE)
        .output()
        .map_err(|e| AppError::Io(format!("không chạy được 'yt-dlp': {e}")))?;

    if !output.status.success() && output.stdout.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = stderr.lines().last().unwrap_or("yt-dlp lỗi").trim();
        return Err(AppError::SearchFailed(msg.to_string()));
    }

    Ok(parse_output(&String::from_utf8_lossy(&output.stdout)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_defaults_to_20() {
        assert_eq!(count_from(None), 20);
        assert_eq!(count_from(Some("".to_string())), 20);
        assert_eq!(count_from(Some("abc".to_string())), 20);
        assert_eq!(count_from(Some("0".to_string())), 20);
    }

    #[test]
    fn count_reads_valid_value() {
        assert_eq!(count_from(Some("5".to_string())), 5);
        assert_eq!(count_from(Some("  50 ".to_string())), 50);
    }

    #[test]
    fn build_query_shapes_ytsearch() {
        assert_eq!(build_query(20, "lofi beats"), "ytsearch20:lofi beats");
    }

    #[test]
    fn parse_output_skips_garbage() {
        let raw = "id1\tTitle One\t100\tChan A\n\
                   garbage line without tabs\n\
                   id2\tTitle Two\tNA\tChan B\n";
        let out = parse_output(raw);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].id, "id1");
        assert_eq!(out[1].duration, "NA");
    }
}
