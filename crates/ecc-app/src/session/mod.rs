//! Session use cases — I/O operations for session management.
//!
//! Pure types and functions live in `ecc-domain::session::manager`.
//! This module provides the I/O layer that depends on `FileSystem`.

pub mod aliases;

use ecc_domain::session::manager::{
    get_session_stats, parse_session_filename, parse_session_metadata, GetAllSessionsOptions,
    SessionDetail, SessionListItem, SessionListResult,
};
use ecc_ports::fs::FileSystem;
use std::path::Path;

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

/// Write content to a session file.
pub fn write_session_content(
    fs: &dyn FileSystem,
    path: &Path,
    content: &str,
) -> Result<(), ecc_ports::fs::FsError> {
    fs.write(path, content)
}

/// Delete a session file. Returns `Ok(true)` if deleted, `Ok(false)` if not found.
pub fn delete_session(fs: &dyn FileSystem, path: &Path) -> Result<bool, ecc_ports::fs::FsError> {
    if !fs.exists(path) {
        return Ok(false);
    }
    fs.remove_file(path)?;
    Ok(true)
}

// ---------------------------------------------------------------------------
// Convenience wrappers (preserve original app API)
// ---------------------------------------------------------------------------

/// List sessions in a directory with pagination and filtering.
pub fn list_sessions(
    fs: &dyn FileSystem,
    sessions_dir: &Path,
    options: &GetAllSessionsOptions,
) -> SessionListResult {
    get_all_sessions(fs, sessions_dir, options)
}

/// Get a single session by ID (short ID, date, or full filename).
pub fn get_session(
    fs: &dyn FileSystem,
    sessions_dir: &Path,
    id: &str,
    include_content: bool,
) -> Option<SessionDetail> {
    get_session_by_id(fs, sessions_dir, id, include_content)
}

/// Write session content to a file.
pub fn write_session(fs: &dyn FileSystem, path: &Path, content: &str) -> Result<(), ecc_ports::fs::FsError> {
    write_session_content(fs, path, content)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;
    use std::path::{Path, PathBuf};

    fn sessions_dir() -> PathBuf {
        PathBuf::from("/sessions")
    }

    fn sessions_fs() -> InMemoryFileSystem {
        InMemoryFileSystem::new()
            .with_dir("/sessions")
            .with_file(
                "/sessions/2026-03-14-abc12345-session.tmp",
                "# Session\n## Tasks\n- [x] Task 1\n- [ ] Task 2\n",
            )
            .with_file(
                "/sessions/2026-03-13-def45678-session.tmp",
                "# Session 2\n## Tasks\n- [x] Done\n",
            )
    }

    // -----------------------------------------------------------------------
    // Wrapper tests (original app tests)
    // -----------------------------------------------------------------------

    #[test]
    fn list_sessions_returns_all() {
        let fs = sessions_fs();
        let options = GetAllSessionsOptions::default();
        let result = list_sessions(&fs, Path::new("/sessions"), &options);
        assert_eq!(result.total, 2);
        assert_eq!(result.sessions.len(), 2);
    }

    #[test]
    fn list_sessions_empty_dir() {
        let fs = InMemoryFileSystem::new().with_dir("/sessions");
        let options = GetAllSessionsOptions::default();
        let result = list_sessions(&fs, Path::new("/sessions"), &options);
        assert_eq!(result.total, 0);
    }

    #[test]
    fn get_session_found() {
        let fs = sessions_fs();
        let session = get_session(&fs, Path::new("/sessions"), "abc12345", true);
        assert!(session.is_some());
        let detail = session.unwrap();
        assert_eq!(detail.short_id, "abc12345");
        assert!(detail.content.is_some());
    }

    #[test]
    fn get_session_not_found() {
        let fs = sessions_fs();
        let session = get_session(&fs, Path::new("/sessions"), "nonexistent", false);
        assert!(session.is_none());
    }

    #[test]
    fn get_session_without_content() {
        let fs = sessions_fs();
        let session = get_session(&fs, Path::new("/sessions"), "abc12345", false);
        assert!(session.is_some());
        let detail = session.unwrap();
        assert!(detail.content.is_none());
    }

    #[test]
    fn delete_session_success() {
        let fs = sessions_fs();
        let path = Path::new("/sessions/2026-03-14-abc12345-session.tmp");
        assert!(delete_session(&fs, path).unwrap());
        assert!(!fs.exists(path));
    }

    #[test]
    fn delete_session_not_found() {
        let fs = InMemoryFileSystem::new();
        assert!(!delete_session(&fs, Path::new("/sessions/nonexistent.tmp")).unwrap());
    }

    #[test]
    fn write_session_success() {
        let fs = InMemoryFileSystem::new().with_dir("/sessions");
        let path = Path::new("/sessions/test.tmp");
        write_session(&fs, path, "# New content").unwrap();
        assert_eq!(
            fs.read_to_string(path).unwrap(),
            "# New content"
        );
    }

    // -----------------------------------------------------------------------
    // get_all_sessions (moved from domain)
    // -----------------------------------------------------------------------

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
    // get_session_by_id (moved from domain)
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
    // write_session_content (moved from domain)
    // -----------------------------------------------------------------------

    #[test]
    fn write_session_content_success() {
        let fs = InMemoryFileSystem::new().with_dir("/sessions");
        let path = Path::new("/sessions/test.tmp");
        write_session_content(&fs, path, "hello").unwrap();
        assert_eq!(fs.read_to_string(path).unwrap(), "hello");
    }

    #[test]
    fn write_session_content_overwrites() {
        let fs = InMemoryFileSystem::new()
            .with_file("/sessions/test.tmp", "old");
        let path = Path::new("/sessions/test.tmp");
        write_session_content(&fs, path, "new").unwrap();
        assert_eq!(fs.read_to_string(path).unwrap(), "new");
    }

    #[test]
    fn write_session_content_verify_content() {
        let fs = InMemoryFileSystem::new().with_dir("/sessions");
        let path = Path::new("/sessions/verify.tmp");
        let content = "# Session\n\n**Date:** 2024-03-15\n";
        write_session_content(&fs, path, content).unwrap();
        let read_back = fs.read_to_string(path).unwrap();
        assert_eq!(read_back, content);
    }

    // -----------------------------------------------------------------------
    // delete_session (moved from domain)
    // -----------------------------------------------------------------------

    #[test]
    fn delete_session_exists() {
        let fs = InMemoryFileSystem::new()
            .with_file("/sessions/test.tmp", "content");
        let path = Path::new("/sessions/test.tmp");
        assert!(delete_session(&fs, path).unwrap());
        assert!(!fs.exists(path));
    }

    #[test]
    fn delete_session_not_exists() {
        let fs = InMemoryFileSystem::new().with_dir("/sessions");
        let path = Path::new("/sessions/nope.tmp");
        assert!(!delete_session(&fs, path).unwrap());
    }

    #[test]
    fn delete_session_cannot_read_after() {
        let fs = InMemoryFileSystem::new()
            .with_file("/sessions/test.tmp", "data");
        let path = Path::new("/sessions/test.tmp");
        let _ = delete_session(&fs, path);
        assert!(fs.read_to_string(path).is_err());
    }
}
