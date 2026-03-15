//! Session manager — types and pure functions for Claude Code session files.
//! Ported from TypeScript `session-manager.ts`.
//!
//! Sessions are stored as markdown files with format:
//! - `YYYY-MM-DD-session.tmp` (legacy, no short ID)
//! - `YYYY-MM-DD-<short-id>-session.tmp` (current)
//!
//! I/O functions (get_all_sessions, get_session_by_id, write_session_content,
//! delete_session) live in `ecc-app::session`.

use regex::Regex;
use std::path::PathBuf;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionFilename {
    pub filename: String,
    pub short_id: String,
    pub date: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SessionMetadata {
    pub title: Option<String>,
    pub date: Option<String>,
    pub started: Option<String>,
    pub last_updated: Option<String>,
    pub completed: Vec<String>,
    pub in_progress: Vec<String>,
    pub notes: String,
    pub context: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionStats {
    pub total_items: usize,
    pub completed_items: usize,
    pub in_progress_items: usize,
    pub line_count: usize,
    pub has_notes: bool,
    pub has_context: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetAllSessionsOptions {
    pub limit: usize,
    pub offset: usize,
    pub date: Option<String>,
    pub search: Option<String>,
}

impl Default for GetAllSessionsOptions {
    fn default() -> Self {
        Self {
            limit: 50,
            offset: 0,
            date: None,
            search: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionListResult {
    pub sessions: Vec<SessionListItem>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionListItem {
    pub filename: String,
    pub short_id: String,
    pub date: String,
    pub session_path: PathBuf,
    pub has_content: bool,
    pub size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionDetail {
    pub filename: String,
    pub short_id: String,
    pub date: String,
    pub session_path: PathBuf,
    pub size: usize,
    pub content: Option<String>,
    pub metadata: Option<SessionMetadata>,
    pub stats: Option<SessionStats>,
}

// ---------------------------------------------------------------------------
// Compiled regexes (static)
// ---------------------------------------------------------------------------

static SESSION_FILENAME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\d{4}-\d{2}-\d{2})(?:-([a-z0-9]{8,}))?-session\.tmp$").expect("BUG: invalid SESSION_FILENAME_RE regex")
});

static TITLE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^#\s+(.+)$").expect("BUG: invalid TITLE_RE regex"));

static DATE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\*Date:\*\*\s*(\d{4}-\d{2}-\d{2})").expect("BUG: invalid regex constant"));

static STARTED_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\*Started:\*\*\s*([\d:]+)").expect("BUG: invalid regex constant"));

static UPDATED_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\*Last Updated:\*\*\s*([\d:]+)").expect("BUG: invalid regex constant"));

static COMPLETED_SECTION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)### Completed\s*\n(.*?)(?:###|\n\n|$)").expect("BUG: invalid regex constant"));

static PROGRESS_SECTION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)### In Progress\s*\n(.*?)(?:###|\n\n|$)").expect("BUG: invalid regex constant"));

static NOTES_SECTION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)### Notes for Next Session\s*\n(.*?)(?:###|\n\n|$)").expect("BUG: invalid regex constant")
});

static CONTEXT_SECTION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)### Context to Load\s*\n```\n(.*?)```").expect("BUG: invalid regex constant"));

static COMPLETED_ITEM_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"- \[x\]\s*(.+)").expect("BUG: invalid regex constant"));

static IN_PROGRESS_ITEM_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"- \[ \]\s*(.+)").expect("BUG: invalid regex constant"));

// ---------------------------------------------------------------------------
// Pure functions
// ---------------------------------------------------------------------------

/// Parse a session filename to extract date and short ID.
///
/// Returns `None` for invalid filenames or invalid dates (month > 12, day > 31).
pub fn parse_session_filename(filename: &str) -> Option<SessionFilename> {
    let caps = SESSION_FILENAME_RE.captures(filename)?;

    let date_str = caps.get(1)?.as_str();
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return None;
    }

    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;

    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let short_id = caps
        .get(2)
        .map_or("no-id", |m| m.as_str())
        .to_string();

    Some(SessionFilename {
        filename: filename.to_string(),
        short_id,
        date: date_str.to_string(),
    })
}

/// Parse markdown session content into structured metadata.
pub fn parse_session_metadata(content: Option<&str>) -> SessionMetadata {
    let Some(content) = content else {
        return SessionMetadata::default();
    };

    let title = TITLE_RE
        .captures(content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string());

    let date = DATE_RE
        .captures(content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string());

    let started = STARTED_RE
        .captures(content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string());

    let last_updated = UPDATED_RE
        .captures(content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string());

    let completed = COMPLETED_SECTION_RE
        .captures(content)
        .and_then(|c| c.get(1))
        .map(|section| {
            COMPLETED_ITEM_RE
                .captures_iter(section.as_str())
                .filter_map(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let in_progress = PROGRESS_SECTION_RE
        .captures(content)
        .and_then(|c| c.get(1))
        .map(|section| {
            IN_PROGRESS_ITEM_RE
                .captures_iter(section.as_str())
                .filter_map(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let notes = NOTES_SECTION_RE
        .captures(content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default();

    let context = CONTEXT_SECTION_RE
        .captures(content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default();

    SessionMetadata {
        title,
        date,
        started,
        last_updated,
        completed,
        in_progress,
        notes,
        context,
    }
}

/// Compute statistics for session content.
pub fn get_session_stats(content: &str) -> SessionStats {
    let metadata = parse_session_metadata(Some(content));

    SessionStats {
        total_items: metadata.completed.len() + metadata.in_progress.len(),
        completed_items: metadata.completed.len(),
        in_progress_items: metadata.in_progress.len(),
        line_count: content.split('\n').count(),
        has_notes: !metadata.notes.is_empty(),
        has_context: !metadata.context.is_empty(),
    }
}

/// Format a byte size into a human-readable string.
pub fn format_session_size(size: usize) -> String {
    if size < 1024 {
        format!("{size} B")
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // parse_session_filename
    // -----------------------------------------------------------------------

    #[test]
    fn parse_filename_with_id() {
        let result =
            parse_session_filename("2024-03-15-abc12345-session.tmp").unwrap();
        assert_eq!(result.short_id, "abc12345");
        assert_eq!(result.date, "2024-03-15");
        assert_eq!(result.filename, "2024-03-15-abc12345-session.tmp");
    }

    #[test]
    fn parse_filename_with_long_id() {
        let result =
            parse_session_filename("2024-01-01-abcdefgh12345678-session.tmp")
                .unwrap();
        assert_eq!(result.short_id, "abcdefgh12345678");
        assert_eq!(result.date, "2024-01-01");
    }

    #[test]
    fn parse_filename_without_id_gives_no_id() {
        let result =
            parse_session_filename("2024-03-15-session.tmp").unwrap();
        assert_eq!(result.short_id, "no-id");
        assert_eq!(result.date, "2024-03-15");
    }

    #[test]
    fn parse_filename_invalid_format_returns_none() {
        assert!(parse_session_filename("random-file.txt").is_none());
        assert!(parse_session_filename("2024-03-15.tmp").is_none());
        assert!(parse_session_filename("session.tmp").is_none());
        assert!(parse_session_filename("").is_none());
    }

    #[test]
    fn parse_filename_invalid_month_13() {
        assert!(
            parse_session_filename("2024-13-15-abc12345-session.tmp").is_none()
        );
    }

    #[test]
    fn parse_filename_invalid_day_32() {
        assert!(
            parse_session_filename("2024-03-32-abc12345-session.tmp").is_none()
        );
    }

    #[test]
    fn parse_filename_invalid_month_00() {
        assert!(
            parse_session_filename("2024-00-15-abc12345-session.tmp").is_none()
        );
    }

    #[test]
    fn parse_filename_invalid_day_00() {
        assert!(
            parse_session_filename("2024-03-00-abc12345-session.tmp").is_none()
        );
    }

    #[test]
    fn parse_filename_edge_date_month_12_day_31() {
        let result =
            parse_session_filename("2024-12-31-abc12345-session.tmp").unwrap();
        assert_eq!(result.date, "2024-12-31");
    }

    #[test]
    fn parse_filename_edge_date_month_01_day_01() {
        let result =
            parse_session_filename("2024-01-01-abc12345-session.tmp").unwrap();
        assert_eq!(result.date, "2024-01-01");
    }

    #[test]
    fn parse_filename_short_id_too_short_is_invalid() {
        // ID must be 8+ characters
        assert!(
            parse_session_filename("2024-03-15-abc-session.tmp").is_none()
        );
    }

    #[test]
    fn parse_filename_id_with_uppercase_is_invalid() {
        assert!(
            parse_session_filename("2024-03-15-ABCDEFGH-session.tmp").is_none()
        );
    }

    // -----------------------------------------------------------------------
    // parse_session_metadata
    // -----------------------------------------------------------------------

    #[test]
    fn metadata_none_content() {
        let meta = parse_session_metadata(None);
        assert_eq!(meta, SessionMetadata::default());
    }

    #[test]
    fn metadata_empty_content() {
        let meta = parse_session_metadata(Some(""));
        assert!(meta.title.is_none());
        assert!(meta.date.is_none());
        assert!(meta.completed.is_empty());
    }

    #[test]
    fn metadata_title() {
        let meta = parse_session_metadata(Some("# My Session Title\n"));
        assert_eq!(meta.title.as_deref(), Some("My Session Title"));
    }

    #[test]
    fn metadata_date() {
        let meta =
            parse_session_metadata(Some("**Date:** 2024-03-15\n"));
        assert_eq!(meta.date.as_deref(), Some("2024-03-15"));
    }

    #[test]
    fn metadata_started() {
        let meta = parse_session_metadata(Some("**Started:** 14:30\n"));
        assert_eq!(meta.started.as_deref(), Some("14:30"));
    }

    #[test]
    fn metadata_last_updated() {
        let meta =
            parse_session_metadata(Some("**Last Updated:** 16:45\n"));
        assert_eq!(meta.last_updated.as_deref(), Some("16:45"));
    }

    #[test]
    fn metadata_completed_tasks() {
        let content = "### Completed\n- [x] Task one\n- [x] Task two\n\n";
        let meta = parse_session_metadata(Some(content));
        assert_eq!(meta.completed, vec!["Task one", "Task two"]);
    }

    #[test]
    fn metadata_in_progress_tasks() {
        let content = "### In Progress\n- [ ] WIP task\n- [ ] Another WIP\n\n";
        let meta = parse_session_metadata(Some(content));
        assert_eq!(meta.in_progress, vec!["WIP task", "Another WIP"]);
    }

    #[test]
    fn metadata_notes_section() {
        let content =
            "### Notes for Next Session\nRemember to check the logs\n\n";
        let meta = parse_session_metadata(Some(content));
        assert_eq!(meta.notes, "Remember to check the logs");
    }

    #[test]
    fn metadata_context_section() {
        let content =
            "### Context to Load\n```\nsome context data\n```\n";
        let meta = parse_session_metadata(Some(content));
        assert_eq!(meta.context, "some context data");
    }

    #[test]
    fn metadata_complex_content() {
        let content = "\
# Development Session

**Date:** 2024-03-15
**Started:** 10:00
**Last Updated:** 12:30

### Completed
- [x] Set up project
- [x] Write tests

### In Progress
- [ ] Implement feature

### Notes for Next Session
Review PR feedback

### Context to Load
```
branch: feature/foo
```
";
        let meta = parse_session_metadata(Some(content));
        assert_eq!(meta.title.as_deref(), Some("Development Session"));
        assert_eq!(meta.date.as_deref(), Some("2024-03-15"));
        assert_eq!(meta.started.as_deref(), Some("10:00"));
        assert_eq!(meta.last_updated.as_deref(), Some("12:30"));
        assert_eq!(meta.completed.len(), 2);
        assert_eq!(meta.in_progress.len(), 1);
        assert_eq!(meta.notes, "Review PR feedback");
        assert_eq!(meta.context, "branch: feature/foo");
    }

    #[test]
    fn metadata_no_tasks_sections() {
        let meta = parse_session_metadata(Some("# Just a title\n"));
        assert!(meta.completed.is_empty());
        assert!(meta.in_progress.is_empty());
    }

    // -----------------------------------------------------------------------
    // get_session_stats
    // -----------------------------------------------------------------------

    #[test]
    fn stats_empty_content() {
        let stats = get_session_stats("");
        assert_eq!(stats.total_items, 0);
        assert_eq!(stats.completed_items, 0);
        assert_eq!(stats.in_progress_items, 0);
        assert_eq!(stats.line_count, 1); // single empty line
        assert!(!stats.has_notes);
        assert!(!stats.has_context);
    }

    #[test]
    fn stats_with_items() {
        let content = "\
### Completed
- [x] Done 1
- [x] Done 2

### In Progress
- [ ] WIP 1

### Notes for Next Session
Some notes

### Context to Load
```
ctx
```
";
        let stats = get_session_stats(content);
        assert_eq!(stats.total_items, 3);
        assert_eq!(stats.completed_items, 2);
        assert_eq!(stats.in_progress_items, 1);
        assert!(stats.has_notes);
        assert!(stats.has_context);
    }

    #[test]
    fn stats_line_count() {
        let content = "line1\nline2\nline3\n";
        let stats = get_session_stats(content);
        assert_eq!(stats.line_count, 4); // trailing newline creates empty 4th
    }

    // -----------------------------------------------------------------------
    // format_session_size
    // -----------------------------------------------------------------------

    #[test]
    fn format_size_bytes() {
        assert_eq!(format_session_size(0), "0 B");
        assert_eq!(format_session_size(512), "512 B");
        assert_eq!(format_session_size(1023), "1023 B");
    }

    #[test]
    fn format_size_kb() {
        assert_eq!(format_session_size(1024), "1.0 KB");
        assert_eq!(format_session_size(1536), "1.5 KB");
        assert_eq!(format_session_size(10240), "10.0 KB");
    }

    #[test]
    fn format_size_mb() {
        assert_eq!(format_session_size(1024 * 1024), "1.0 MB");
        assert_eq!(
            format_session_size(1024 * 1024 + 512 * 1024),
            "1.5 MB"
        );
    }

}
