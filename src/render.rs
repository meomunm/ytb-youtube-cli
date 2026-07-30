//! Rendering the result list as aligned, optionally-colored columns.

use crate::entry::Entry;

const YELLOW: &str = "\x1b[33m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

const TITLE_WIDTH: usize = 58;

/// Format a raw duration for display.
///
/// yt-dlp emits durations as a number of seconds (possibly fractional), or a
/// placeholder like `NA`/`none` when unknown. Numbers become `m:ss` or
/// `h:mm:ss`; unknown/empty become `-`.
pub fn fmt_duration(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() || t.eq_ignore_ascii_case("na") || t.eq_ignore_ascii_case("none") {
        return "-".to_string();
    }
    if let Ok(secs) = t.parse::<f64>() {
        if secs.is_finite() && secs >= 0.0 {
            let total = secs as u64;
            let (h, m, s) = (total / 3600, (total % 3600) / 60, total % 60);
            return if h > 0 {
                format!("{h}:{m:02}:{s:02}")
            } else {
                format!("{m}:{s:02}")
            };
        }
    }
    t.to_string()
}

/// Truncate `s` to at most `max` characters, marking a cut with `…`.
pub fn truncate(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    if max == 0 {
        return String::new();
    }
    let kept: String = s.chars().take(max - 1).collect();
    format!("{kept}…")
}

/// Render one row: `idx  duration  title  uploader`.
///
/// When `color` is true the index is yellow and the uploader is dimmed. Widths
/// are computed on the plain text so color codes never disturb alignment.
pub fn format_row(idx: usize, e: &Entry, color: bool) -> String {
    let dur = fmt_duration(&e.duration);
    let title = truncate(&e.title, TITLE_WIDTH);
    let idx_plain = format!("{idx:>3}");
    if color {
        format!(
            "{YELLOW}{idx_plain}{RESET}  {dur:<9}  {title:<TITLE_WIDTH$}  {DIM}{up}{RESET}",
            up = e.uploader
        )
    } else {
        format!(
            "{idx_plain}  {dur:<9}  {title:<TITLE_WIDTH$}  {up}",
            up = e.uploader
        )
    }
}

/// Render the whole list, one row per line.
pub fn render_list(entries: &[Entry], color: bool) -> String {
    let mut out = String::new();
    for (i, e) in entries.iter().enumerate() {
        out.push_str(&format_row(i + 1, e, color));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(title: &str, dur: &str, up: &str) -> Entry {
        Entry {
            id: "id".to_string(),
            title: title.to_string(),
            duration: dur.to_string(),
            uploader: up.to_string(),
        }
    }

    #[test]
    fn duration_seconds_to_mmss() {
        assert_eq!(fmt_duration("212"), "3:32");
        assert_eq!(fmt_duration("59"), "0:59");
    }

    #[test]
    fn duration_hours() {
        assert_eq!(fmt_duration("3661"), "1:01:01");
    }

    #[test]
    fn duration_unknown_becomes_dash() {
        assert_eq!(fmt_duration(""), "-");
        assert_eq!(fmt_duration("NA"), "-");
        assert_eq!(fmt_duration("none"), "-");
        assert_eq!(fmt_duration("  "), "-");
    }

    #[test]
    fn truncate_short_is_unchanged() {
        assert_eq!(truncate("hello", 58), "hello");
    }

    #[test]
    fn truncate_long_adds_ellipsis_within_budget() {
        let s = "x".repeat(100);
        let out = truncate(&s, 58);
        assert_eq!(out.chars().count(), 58);
        assert!(out.ends_with('…'));
    }

    #[test]
    fn row_without_color_has_no_escapes() {
        let row = format_row(1, &e("Song", "212", "Chan"), false);
        assert!(!row.contains('\x1b'));
        assert!(row.contains("Song"));
        assert!(row.contains("3:32"));
        assert!(row.trim_start().starts_with('1'));
    }

    #[test]
    fn row_with_color_wraps_index_and_uploader() {
        let row = format_row(1, &e("Song", "212", "Chan"), true);
        assert!(row.contains(YELLOW));
        assert!(row.contains(DIM));
        assert!(row.contains(RESET));
    }

    #[test]
    fn render_list_is_one_line_per_entry() {
        let out = render_list(&[e("a", "1", "c"), e("b", "2", "d")], false);
        assert_eq!(out.lines().count(), 2);
    }
}
