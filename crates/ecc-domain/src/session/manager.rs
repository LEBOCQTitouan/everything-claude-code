//! Session manager — CRUD operations for Claude Code session files.
//! Ported from TypeScript `session-manager.ts`.
//!
//! Sessions are stored as markdown files with format:
//! - `YYYY-MM-DD-session.tmp` (legacy, no short ID)
//! - `YYYY-MM-DD-<short-id>-session.tmp` (current)

use ecc_ports::fs::FileSystem;
use regex::Regex;
use std::path::{Path, PathBuf};
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
    Regex::new(r"^(\d{4}-\d{2}-\d{2})(?:-([a-z0-9]{8,}))?-session\.tmp$").unwrap()
});

static TITLE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^#\s+(.+)$").unwrap());

static DATE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\*Date:\*\*\s*(\d{4}-\d{2}-\d{2})").unwrap());

static STARTED_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\*Started:\*\*\s*([\d:]+)").unwrap());

static UPDATED_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\*\*Last Updated:\*\*\s*([\d:]+)").unwrap());

static COMPLETED_SECTION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)### Completed\s*\n(.*?)(?:###|\n\n|$)").unwrap());

static PROGRESS_SECTION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)### In Progress\s*\n(.*?)(?:###|\n\n|$)").unwrap());

static NOTES_SECTION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)### Notes for Next Session\s*\n(.*?)(?:###|\n\n|$)").unwrap()
});

static CONTEXT_SECTION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)### Context to Load\s*\n```\n(.*?)```").unwrap());

static COMPLETED_ITEM_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"- \[x\]\s*(.+)").unwrap());

static IN_PROGRESS_ITEM_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"- \[ \]\s*(.+)").unwrap());

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
// I/O functions (depend on FileSystem port)
// ---------------------------------------------------------------------------

/// List sessions in a directory with optional filtering and pagination.
pub fn get_all_sessions(
    fs: &dyn FileSystem,
    sessions_dir: &Path,
    options: &GetAllSessionsOptions,
) -> SessionListResult {
    let empty = SessionListResult {
        sessions: vec![],
        total: 0,
        offset: options.offset,
        limit: options.limit,
        has_more: false,
    };

    if !fs.exists(sessions_dir) {
        return empty;
    }

    let entries = match fs.read_dir(sessions_dir) {
        Ok(e) => e,
        Err(_) => return empty,
    };

    let mut sessions: Vec<SessionListItem> = Vec::new();

    for entry in &entries {
        let Some(fname) = entry.file_name().and_then(|n| n.to_str().map(String::from)) else {
            continue;
        };

        if !fname.ends_with(".tmp") {
            continue;
        }

        let Some(parsed) = parse_session_filename(&fname) else {
            continue;
        };

        if let Some(ref date_filter) = options.date
            && parsed.date != *date_filter
        {
            continue;
        }

        if let Some(ref search_filter) = options.search
            && !parsed.short_id.contains(search_filter.as_str())
        {
            continue;
        }

        let size = fs
            .read_to_string(entry)
            .map(|c| c.len())
            .unwrap_or(0);

        sessions.push(SessionListItem {
            filename: fname,
            short_id: parsed.short_id,
            date: parsed.date,
            session_path: entry.clone(),
            has_content: size > 0,
            size,
        });
    }

    // Sort by filename descending (newer dates first)
    sessions.sort_by(|a, b| b.filename.cmp(&a.filename));

    let total = sessions.len();
    let offset = options.offset;
    let limit = options.limit.max(1);
    let paginated: Vec<SessionListItem> =
        sessions.into_iter().skip(offset).take(limit).collect();
    let has_more = offset + limit < total;

    SessionListResult {
        sessions: paginated,
        total,
        offset,
        limit,
        has_more,
    }
}

/// Find a session by ID (short ID prefix, filename, or filename without `.tmp`).
pub fn get_session_by_id(
    fs: &dyn FileSystem,
    sessions_dir: &Path,
    id: &str,
    include_content: bool,
) -> Option<SessionDetail> {
    let entries = fs.read_dir(sessions_dir).ok()?;

    for entry in &entries {
        let fname = entry.file_name()?.to_str()?.to_string();

        if !fname.ends_with(".tmp") {
            continue;
        }

        let parsed = parse_session_filename(&fname)?;

        let short_id_match = !id.is_empty()
            && parsed.short_id != "no-id"
            && parsed.short_id.starts_with(id);
        let filename_match = fname == id || fname == format!("{id}.tmp");
        let no_id_match =
            parsed.short_id == "no-id" && fname == format!("{id}-session.tmp");

        if !short_id_match && !filename_match && !no_id_match {
            continue;
        }

        let session_path = sessions_dir.join(&fname);
        let content_str = fs.read_to_string(&session_path).ok();
        let size = content_str.as_ref().map_or(0, String::len);

        let (content, metadata, stats) = if include_content {
            let meta =
                parse_session_metadata(content_str.as_deref());
            let st = get_session_stats(
                content_str.as_deref().unwrap_or(""),
            );
            (content_str, Some(meta), Some(st))
        } else {
            (None, None, None)
        };

        return Some(SessionDetail {
            filename: fname,
            short_id: parsed.short_id,
            date: parsed.date,
            session_path,
            size,
            content,
            metadata,
            stats,
        });
    }

    None
}

/// Write content to a session file. Returns `true` on success.
pub fn write_session_content(
    fs: &dyn FileSystem,
    path: &Path,
    content: &str,
) -> bool {
    fs.write(path, content).is_ok()
}

/// Delete a session file. Returns `true` if the file existed and was removed.
pub fn delete_session(fs: &dyn FileSystem, path: &Path) -> bool {
    if !fs.exists(path) {
        return false;
    }
    fs.remove_file(path).is_ok()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;
    use std::path::Path;

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

    // -----------------------------------------------------------------------
    // get_all_sessions (I/O)
    // -----------------------------------------------------------------------

    fn sessions_dir() -> PathBuf {
        PathBuf::from("/sessions")
    }

    #[test]
    fn get_all_sessions_empty_dir() {
        let fs = InMemoryFileSystem::new().with_dir("/sessions");
        let result =
            get_all_sessions(&fs, &sessions_dir(), &GetAllSessionsOptions::default());
        assert_eq!(result.total, 0);
        assert!(result.sessions.is_empty());
        assert!(!result.has_more);
    }

    #[test]
    fn get_all_sessions_nonexistent_dir() {
        let fs = InMemoryFileSystem::new();
        let result =
            get_all_sessions(&fs, &sessions_dir(), &GetAllSessionsOptions::default());
        assert_eq!(result.total, 0);
    }

    #[test]
    fn get_all_sessions_with_sessions() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/sessions/2024-03-15-abc12345-session.tmp",
                "content A",
            )
            .with_file(
                "/sessions/2024-03-16-def12345-session.tmp",
                "content B",
            );
        let result =
            get_all_sessions(&fs, &sessions_dir(), &GetAllSessionsOptions::default());
        assert_eq!(result.total, 2);
        assert_eq!(result.sessions.len(), 2);
        // Sorted desc by filename — 03-16 first
        assert_eq!(result.sessions[0].date, "2024-03-16");
        assert_eq!(result.sessions[1].date, "2024-03-15");
    }

    #[test]
    fn get_all_sessions_skips_non_tmp() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/sessions/2024-03-15-abc12345-session.tmp",
                "ok",
            )
            .with_file("/sessions/readme.md", "ignore me");
        let result =
            get_all_sessions(&fs, &sessions_dir(), &GetAllSessionsOptions::default());
        assert_eq!(result.total, 1);
    }

    #[test]
    fn get_all_sessions_skips_invalid_filenames() {
        let fs = InMemoryFileSystem::new()
            .with_file("/sessions/random.tmp", "bad")
            .with_file(
                "/sessions/2024-03-15-abc12345-session.tmp",
                "good",
            );
        let result =
            get_all_sessions(&fs, &sessions_dir(), &GetAllSessionsOptions::default());
        assert_eq!(result.total, 1);
    }

    #[test]
    fn get_all_sessions_date_filter() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/sessions/2024-03-15-abc12345-session.tmp",
                "a",
            )
            .with_file(
                "/sessions/2024-03-16-def12345-session.tmp",
                "b",
            );
        let opts = GetAllSessionsOptions {
            date: Some("2024-03-15".to_string()),
            ..Default::default()
        };
        let result = get_all_sessions(&fs, &sessions_dir(), &opts);
        assert_eq!(result.total, 1);
        assert_eq!(result.sessions[0].date, "2024-03-15");
    }

    #[test]
    fn get_all_sessions_search_filter() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/sessions/2024-03-15-abc12345-session.tmp",
                "a",
            )
            .with_file(
                "/sessions/2024-03-16-xyz98765-session.tmp",
                "b",
            );
        let opts = GetAllSessionsOptions {
            search: Some("abc".to_string()),
            ..Default::default()
        };
        let result = get_all_sessions(&fs, &sessions_dir(), &opts);
        assert_eq!(result.total, 1);
        assert_eq!(result.sessions[0].short_id, "abc12345");
    }

    #[test]
    fn get_all_sessions_pagination_limit() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/sessions/2024-03-15-aaa11111-session.tmp",
                "a",
            )
            .with_file(
                "/sessions/2024-03-16-bbb22222-session.tmp",
                "b",
            )
            .with_file(
                "/sessions/2024-03-17-ccc33333-session.tmp",
                "c",
            );
        let opts = GetAllSessionsOptions {
            limit: 2,
            offset: 0,
            ..Default::default()
        };
        let result = get_all_sessions(&fs, &sessions_dir(), &opts);
        assert_eq!(result.total, 3);
        assert_eq!(result.sessions.len(), 2);
        assert!(result.has_more);
    }

    #[test]
    fn get_all_sessions_pagination_offset() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/sessions/2024-03-15-aaa11111-session.tmp",
                "a",
            )
            .with_file(
                "/sessions/2024-03-16-bbb22222-session.tmp",
                "b",
            )
            .with_file(
                "/sessions/2024-03-17-ccc33333-session.tmp",
                "c",
            );
        let opts = GetAllSessionsOptions {
            limit: 2,
            offset: 2,
            ..Default::default()
        };
        let result = get_all_sessions(&fs, &sessions_dir(), &opts);
        assert_eq!(result.total, 3);
        assert_eq!(result.sessions.len(), 1);
        assert!(!result.has_more);
    }

    #[test]
    fn get_all_sessions_has_content_flag() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/sessions/2024-03-15-abc12345-session.tmp",
                "hello",
            )
            .with_file(
                "/sessions/2024-03-16-def12345-session.tmp",
                "",
            );
        let result =
            get_all_sessions(&fs, &sessions_dir(), &GetAllSessionsOptions::default());
        let with_content = result
            .sessions
            .iter()
            .find(|s| s.date == "2024-03-15")
            .unwrap();
        let without_content = result
            .sessions
            .iter()
            .find(|s| s.date == "2024-03-16")
            .unwrap();
        assert!(with_content.has_content);
        assert!(!without_content.has_content);
    }

    #[test]
    fn get_all_sessions_size_is_content_length() {
        let content = "hello world";
        let fs = InMemoryFileSystem::new().with_file(
            "/sessions/2024-03-15-abc12345-session.tmp",
            content,
        );
        let result =
            get_all_sessions(&fs, &sessions_dir(), &GetAllSessionsOptions::default());
        assert_eq!(result.sessions[0].size, content.len());
    }

    // -----------------------------------------------------------------------
    // get_session_by_id (I/O)
    // -----------------------------------------------------------------------

    #[test]
    fn get_session_by_short_id_prefix() {
        let fs = InMemoryFileSystem::new().with_file(
            "/sessions/2024-03-15-abc12345-session.tmp",
            "found",
        );
        let result =
            get_session_by_id(&fs, &sessions_dir(), "abc", false);
        assert!(result.is_some());
        let s = result.unwrap();
        assert_eq!(s.short_id, "abc12345");
        assert_eq!(s.date, "2024-03-15");
    }

    #[test]
    fn get_session_by_full_filename() {
        let fs = InMemoryFileSystem::new().with_file(
            "/sessions/2024-03-15-abc12345-session.tmp",
            "found",
        );
        let result = get_session_by_id(
            &fs,
            &sessions_dir(),
            "2024-03-15-abc12345-session.tmp",
            false,
        );
        assert!(result.is_some());
    }

    #[test]
    fn get_session_by_filename_without_extension() {
        let fs = InMemoryFileSystem::new().with_file(
            "/sessions/2024-03-15-abc12345-session.tmp",
            "found",
        );
        let result = get_session_by_id(
            &fs,
            &sessions_dir(),
            "2024-03-15-abc12345-session",
            false,
        );
        assert!(result.is_some());
    }

    #[test]
    fn get_session_by_id_no_id_session() {
        let fs = InMemoryFileSystem::new().with_file(
            "/sessions/2024-03-15-session.tmp",
            "legacy",
        );
        let result = get_session_by_id(
            &fs,
            &sessions_dir(),
            "2024-03-15",
            false,
        );
        assert!(result.is_some());
        assert_eq!(result.unwrap().short_id, "no-id");
    }

    #[test]
    fn get_session_by_id_not_found() {
        let fs = InMemoryFileSystem::new().with_file(
            "/sessions/2024-03-15-abc12345-session.tmp",
            "found",
        );
        let result =
            get_session_by_id(&fs, &sessions_dir(), "zzz", false);
        assert!(result.is_none());
    }

    #[test]
    fn get_session_by_id_with_content() {
        let content = "\
# Test Session

**Date:** 2024-03-15

### Completed
- [x] Done task

### In Progress
- [ ] WIP task
";
        let fs = InMemoryFileSystem::new().with_file(
            "/sessions/2024-03-15-abc12345-session.tmp",
            content,
        );
        let result =
            get_session_by_id(&fs, &sessions_dir(), "abc", true);
        let s = result.unwrap();
        assert!(s.content.is_some());
        assert!(s.metadata.is_some());
        assert!(s.stats.is_some());

        let meta = s.metadata.unwrap();
        assert_eq!(meta.title.as_deref(), Some("Test Session"));
        assert_eq!(meta.completed.len(), 1);
        assert_eq!(meta.in_progress.len(), 1);

        let stats = s.stats.unwrap();
        assert_eq!(stats.total_items, 2);
    }

    #[test]
    fn get_session_by_id_without_content_has_no_metadata() {
        let fs = InMemoryFileSystem::new().with_file(
            "/sessions/2024-03-15-abc12345-session.tmp",
            "some stuff",
        );
        let result =
            get_session_by_id(&fs, &sessions_dir(), "abc", false);
        let s = result.unwrap();
        assert!(s.content.is_none());
        assert!(s.metadata.is_none());
        assert!(s.stats.is_none());
    }

    #[test]
    fn get_session_by_id_empty_id_no_match() {
        let fs = InMemoryFileSystem::new().with_file(
            "/sessions/2024-03-15-abc12345-session.tmp",
            "x",
        );
        let result =
            get_session_by_id(&fs, &sessions_dir(), "", false);
        assert!(result.is_none());
    }

    // -----------------------------------------------------------------------
    // write_session_content (I/O)
    // -----------------------------------------------------------------------

    #[test]
    fn write_session_content_success() {
        let fs = InMemoryFileSystem::new().with_dir("/sessions");
        let path = Path::new("/sessions/test.tmp");
        assert!(write_session_content(&fs, path, "hello"));
        assert_eq!(fs.read_to_string(path).unwrap(), "hello");
    }

    #[test]
    fn write_session_content_overwrites() {
        let fs = InMemoryFileSystem::new()
            .with_file("/sessions/test.tmp", "old");
        let path = Path::new("/sessions/test.tmp");
        assert!(write_session_content(&fs, path, "new"));
        assert_eq!(fs.read_to_string(path).unwrap(), "new");
    }

    #[test]
    fn write_session_content_verify_content() {
        let fs = InMemoryFileSystem::new().with_dir("/sessions");
        let path = Path::new("/sessions/verify.tmp");
        let content = "# Session\n\n**Date:** 2024-03-15\n";
        write_session_content(&fs, path, content);
        let read_back = fs.read_to_string(path).unwrap();
        assert_eq!(read_back, content);
    }

    // -----------------------------------------------------------------------
    // delete_session (I/O)
    // -----------------------------------------------------------------------

    #[test]
    fn delete_session_exists() {
        let fs = InMemoryFileSystem::new()
            .with_file("/sessions/test.tmp", "content");
        let path = Path::new("/sessions/test.tmp");
        assert!(delete_session(&fs, path));
        assert!(!fs.exists(path));
    }

    #[test]
    fn delete_session_not_exists() {
        let fs = InMemoryFileSystem::new().with_dir("/sessions");
        let path = Path::new("/sessions/nope.tmp");
        assert!(!delete_session(&fs, path));
    }

    #[test]
    fn delete_session_cannot_read_after() {
        let fs = InMemoryFileSystem::new()
            .with_file("/sessions/test.tmp", "data");
        let path = Path::new("/sessions/test.tmp");
        delete_session(&fs, path);
        assert!(fs.read_to_string(path).is_err());
    }
}
