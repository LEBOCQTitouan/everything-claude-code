use super::*;
use ecc_test_support::InMemoryFileSystem;
use std::path::{Path, PathBuf};

fn sessions_dir() -> PathBuf {
    PathBuf::from("/sessions")
}

// -----------------------------------------------------------------------
// get_all_sessions (moved from domain)
// -----------------------------------------------------------------------

#[test]
fn get_all_sessions_empty_dir() {
    let fs = InMemoryFileSystem::new().with_dir("/sessions");
    let result = get_all_sessions(&fs, &sessions_dir(), &GetAllSessionsOptions::default());
    assert_eq!(result.total, 0);
    assert!(result.sessions.is_empty());
    assert!(!result.has_more);
}

#[test]
fn get_all_sessions_nonexistent_dir() {
    let fs = InMemoryFileSystem::new();
    let result = get_all_sessions(&fs, &sessions_dir(), &GetAllSessionsOptions::default());
    assert_eq!(result.total, 0);
}

#[test]
fn get_all_sessions_with_sessions() {
    let fs = InMemoryFileSystem::new()
        .with_file("/sessions/2024-03-15-abc12345-session.tmp", "content A")
        .with_file("/sessions/2024-03-16-def12345-session.tmp", "content B");
    let result = get_all_sessions(&fs, &sessions_dir(), &GetAllSessionsOptions::default());
    assert_eq!(result.total, 2);
    assert_eq!(result.sessions.len(), 2);
    // Sorted desc by filename — 03-16 first
    assert_eq!(result.sessions[0].date, "2024-03-16");
    assert_eq!(result.sessions[1].date, "2024-03-15");
}

#[test]
fn get_all_sessions_skips_non_tmp() {
    let fs = InMemoryFileSystem::new()
        .with_file("/sessions/2024-03-15-abc12345-session.tmp", "ok")
        .with_file("/sessions/readme.md", "ignore me");
    let result = get_all_sessions(&fs, &sessions_dir(), &GetAllSessionsOptions::default());
    assert_eq!(result.total, 1);
}

#[test]
fn get_all_sessions_skips_invalid_filenames() {
    let fs = InMemoryFileSystem::new()
        .with_file("/sessions/random.tmp", "bad")
        .with_file("/sessions/2024-03-15-abc12345-session.tmp", "good");
    let result = get_all_sessions(&fs, &sessions_dir(), &GetAllSessionsOptions::default());
    assert_eq!(result.total, 1);
}

#[test]
fn get_all_sessions_date_filter() {
    let fs = InMemoryFileSystem::new()
        .with_file("/sessions/2024-03-15-abc12345-session.tmp", "a")
        .with_file("/sessions/2024-03-16-def12345-session.tmp", "b");
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
        .with_file("/sessions/2024-03-15-abc12345-session.tmp", "a")
        .with_file("/sessions/2024-03-16-xyz98765-session.tmp", "b");
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
        .with_file("/sessions/2024-03-15-aaa11111-session.tmp", "a")
        .with_file("/sessions/2024-03-16-bbb22222-session.tmp", "b")
        .with_file("/sessions/2024-03-17-ccc33333-session.tmp", "c");
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
        .with_file("/sessions/2024-03-15-aaa11111-session.tmp", "a")
        .with_file("/sessions/2024-03-16-bbb22222-session.tmp", "b")
        .with_file("/sessions/2024-03-17-ccc33333-session.tmp", "c");
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
        .with_file("/sessions/2024-03-15-abc12345-session.tmp", "hello")
        .with_file("/sessions/2024-03-16-def12345-session.tmp", "");
    let result = get_all_sessions(&fs, &sessions_dir(), &GetAllSessionsOptions::default());
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
    let fs = InMemoryFileSystem::new()
        .with_file("/sessions/2024-03-15-abc12345-session.tmp", content);
    let result = get_all_sessions(&fs, &sessions_dir(), &GetAllSessionsOptions::default());
    assert_eq!(result.sessions[0].size, content.len());
}

// -----------------------------------------------------------------------
// get_session_by_id (moved from domain)
// -----------------------------------------------------------------------

#[test]
fn get_session_by_short_id_prefix() {
    let fs = InMemoryFileSystem::new()
        .with_file("/sessions/2024-03-15-abc12345-session.tmp", "found");
    let result = get_session_by_id(&fs, &sessions_dir(), "abc", false);
    assert!(result.is_some());
    let s = result.unwrap();
    assert_eq!(s.short_id, "abc12345");
    assert_eq!(s.date, "2024-03-15");
}

#[test]
fn get_session_by_full_filename() {
    let fs = InMemoryFileSystem::new()
        .with_file("/sessions/2024-03-15-abc12345-session.tmp", "found");
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
    let fs = InMemoryFileSystem::new()
        .with_file("/sessions/2024-03-15-abc12345-session.tmp", "found");
    let result = get_session_by_id(&fs, &sessions_dir(), "2024-03-15-abc12345-session", false);
    assert!(result.is_some());
}

#[test]
fn get_session_by_id_no_id_session() {
    let fs = InMemoryFileSystem::new().with_file("/sessions/2024-03-15-session.tmp", "legacy");
    let result = get_session_by_id(&fs, &sessions_dir(), "2024-03-15", false);
    assert!(result.is_some());
    assert_eq!(result.unwrap().short_id, "no-id");
}

#[test]
fn get_session_by_id_not_found() {
    let fs = InMemoryFileSystem::new()
        .with_file("/sessions/2024-03-15-abc12345-session.tmp", "found");
    let result = get_session_by_id(&fs, &sessions_dir(), "zzz", false);
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
    let fs = InMemoryFileSystem::new()
        .with_file("/sessions/2024-03-15-abc12345-session.tmp", content);
    let result = get_session_by_id(&fs, &sessions_dir(), "abc", true);
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
    let fs = InMemoryFileSystem::new()
        .with_file("/sessions/2024-03-15-abc12345-session.tmp", "some stuff");
    let result = get_session_by_id(&fs, &sessions_dir(), "abc", false);
    let s = result.unwrap();
    assert!(s.content.is_none());
    assert!(s.metadata.is_none());
    assert!(s.stats.is_none());
}

#[test]
fn get_session_by_id_empty_id_no_match() {
    let fs =
        InMemoryFileSystem::new().with_file("/sessions/2024-03-15-abc12345-session.tmp", "x");
    let result = get_session_by_id(&fs, &sessions_dir(), "", false);
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
    let fs = InMemoryFileSystem::new().with_file("/sessions/test.tmp", "old");
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
    let fs = InMemoryFileSystem::new().with_file("/sessions/test.tmp", "content");
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
    let fs = InMemoryFileSystem::new().with_file("/sessions/test.tmp", "data");
    let path = Path::new("/sessions/test.tmp");
    let _ = delete_session(&fs, path);
    assert!(fs.read_to_string(path).is_err());
}
