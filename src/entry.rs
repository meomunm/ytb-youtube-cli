//! A single YouTube result and its TSV (de)serialization.

/// One entry in the result list.
///
/// The list is persisted as a TSV file with exactly four columns:
/// `id`, `title`, `duration`, `uploader`. The `uploader` column absorbs any
/// remaining tabs (via `splitn(4, ..)`) so a stray tab never shifts columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub id: String,
    pub title: String,
    pub duration: String,
    pub uploader: String,
}

impl Entry {
    /// The canonical watch URL for this entry.
    pub fn watch_url(&self) -> String {
        format!("https://www.youtube.com/watch?v={}", self.id)
    }

    /// Parse one TSV line into an [`Entry`].
    ///
    /// Returns `None` for malformed lines: fewer than four columns, or an
    /// empty `id` (an entry with no id is unplayable, so we drop it).
    pub fn from_tsv(line: &str) -> Option<Entry> {
        let parts: Vec<&str> = line.splitn(4, '\t').collect();
        if parts.len() < 4 {
            return None;
        }
        let id = parts[0].trim();
        if id.is_empty() {
            return None;
        }
        Some(Entry {
            id: id.to_string(),
            title: parts[1].to_string(),
            duration: parts[2].to_string(),
            uploader: parts[3].to_string(),
        })
    }

    /// Serialize this entry to a single TSV line (no trailing newline).
    pub fn to_tsv(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}",
            self.id, self.title, self.duration, self.uploader
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Entry {
        Entry {
            id: "dQw4w9WgXcQ".to_string(),
            title: "Never Gonna Give You Up".to_string(),
            duration: "212".to_string(),
            uploader: "Rick Astley".to_string(),
        }
    }

    #[test]
    fn watch_url_uses_id() {
        assert_eq!(
            sample().watch_url(),
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
        );
    }

    #[test]
    fn tsv_roundtrips() {
        let e = sample();
        let parsed = Entry::from_tsv(&e.to_tsv()).unwrap();
        assert_eq!(parsed, e);
    }

    #[test]
    fn from_tsv_rejects_short_lines() {
        assert!(Entry::from_tsv("id\ttitle\tduration").is_none());
        assert!(Entry::from_tsv("").is_none());
    }

    #[test]
    fn from_tsv_rejects_empty_id() {
        assert!(Entry::from_tsv("\ttitle\t100\tchannel").is_none());
        assert!(Entry::from_tsv("   \ttitle\t100\tchannel").is_none());
    }

    #[test]
    fn uploader_absorbs_extra_tabs() {
        let e = Entry::from_tsv("id\ttitle\t100\tchan\tnel\textra").unwrap();
        assert_eq!(e.id, "id");
        assert_eq!(e.title, "title");
        assert_eq!(e.duration, "100");
        assert_eq!(e.uploader, "chan\tnel\textra");
    }
}
