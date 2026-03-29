use super::*;
use ecc_domain::session::aliases::{get_aliases_for_session, resolve_alias};
use ecc_test_support::InMemoryFileSystem;
use std::path::PathBuf;

const NOW: &str = "2025-01-15T10:00:00.000Z";
const LATER: &str = "2025-01-15T11:00:00.000Z";

fn aliases_path() -> PathBuf {
    PathBuf::from("/home/user/.claude/session-aliases.json")
}

// ── rename_alias ───────────────────────────────────────────────────

#[test]
fn rename_success() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();
    set_alias(&fs, &path, "old", "/sessions/abc", Some("Title"), NOW);

    let result = rename_alias(&fs, &path, "old", "new-name", LATER);
    assert!(result.success);
    assert_eq!(result.old_alias.as_deref(), Some("old"));
    assert_eq!(result.new_alias.as_deref(), Some("new-name"));
    assert_eq!(result.session_path.as_deref(), Some("/sessions/abc"));

    let data = load_aliases(&fs, &path, LATER);
    assert!(!data.aliases.contains_key("old"));
    assert!(data.aliases.contains_key("new-name"));
    assert_eq!(data.aliases["new-name"].title.as_deref(), Some("Title"));
}

#[test]
fn rename_old_not_found() {
    let fs = InMemoryFileSystem::new();
    let result = rename_alias(&fs, &aliases_path(), "nope", "new", NOW);
    assert!(!result.success);
    assert!(result.error.unwrap().contains("not found"));
}

#[test]
fn rename_new_already_exists() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();
    set_alias(&fs, &path, "a", "/s/1", None, NOW);
    set_alias(&fs, &path, "b", "/s/2", None, NOW);

    let result = rename_alias(&fs, &path, "a", "b", LATER);
    assert!(!result.success);
    assert!(result.error.unwrap().contains("already exists"));
}

#[test]
fn rename_new_invalid_name() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();
    set_alias(&fs, &path, "old", "/s/1", None, NOW);

    let result = rename_alias(&fs, &path, "old", "bad name!", LATER);
    assert!(!result.success);
    assert!(result.error.unwrap().contains("letters"));
}

#[test]
fn rename_new_reserved_name() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();
    set_alias(&fs, &path, "old", "/s/1", None, NOW);

    let result = rename_alias(&fs, &path, "old", "list", LATER);
    assert!(!result.success);
    assert!(result.error.unwrap().contains("reserved"));
}

#[test]
fn rename_new_empty_name() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();
    set_alias(&fs, &path, "old", "/s/1", None, NOW);

    let result = rename_alias(&fs, &path, "old", "", LATER);
    assert!(!result.success);
    assert!(result.error.unwrap().contains("empty"));
}

#[test]
fn rename_new_too_long() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();
    set_alias(&fs, &path, "old", "/s/1", None, NOW);

    let long = "x".repeat(129);
    let result = rename_alias(&fs, &path, "old", &long, LATER);
    assert!(!result.success);
    assert!(result.error.unwrap().contains("128"));
}

#[test]
fn rename_updates_timestamp() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();
    set_alias(&fs, &path, "old", "/s/1", None, NOW);

    rename_alias(&fs, &path, "old", "new", LATER);

    let data = load_aliases(&fs, &path, LATER);
    assert_eq!(data.aliases["new"].updated_at.as_deref(), Some(LATER));
}

#[test]
fn rename_preserves_session_path() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();
    set_alias(&fs, &path, "old", "/sessions/abc", None, NOW);
    let result = rename_alias(&fs, &path, "old", "new", LATER);
    assert_eq!(result.session_path.as_deref(), Some("/sessions/abc"));

    let data = load_aliases(&fs, &path, LATER);
    assert_eq!(data.aliases["new"].session_path, "/sessions/abc");
}

#[test]
fn rename_preserves_created_at() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();
    set_alias(&fs, &path, "old", "/s/1", None, NOW);

    rename_alias(&fs, &path, "old", "new", LATER);

    let data = load_aliases(&fs, &path, LATER);
    assert_eq!(data.aliases["new"].created_at, NOW);
}

// ── Round-trip integration tests ───────────────────────────────────

#[test]
fn full_lifecycle_create_resolve_delete() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();

    // Create
    let set_result = set_alias(
        &fs,
        &path,
        "myproj",
        "/sessions/abc",
        Some("My Project"),
        NOW,
    );
    assert!(set_result.success);
    assert_eq!(set_result.is_new, Some(true));

    // Resolve
    let data = load_aliases(&fs, &path, NOW);
    let resolved = resolve_alias(&data, "myproj").unwrap();
    assert_eq!(resolved.session_path, "/sessions/abc");
    assert_eq!(resolved.title.as_deref(), Some("My Project"));

    // Delete
    let del_result = delete_alias(&fs, &path, "myproj", LATER);
    assert!(del_result.success);

    // Verify gone
    let data = load_aliases(&fs, &path, LATER);
    assert!(resolve_alias(&data, "myproj").is_none());
}

#[test]
fn full_lifecycle_create_rename_resolve() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();

    set_alias(&fs, &path, "old-name", "/sessions/abc", None, NOW);
    rename_alias(&fs, &path, "old-name", "new-name", LATER);

    let data = load_aliases(&fs, &path, LATER);
    assert!(resolve_alias(&data, "old-name").is_none());
    let resolved = resolve_alias(&data, "new-name").unwrap();
    assert_eq!(resolved.session_path, "/sessions/abc");
}

#[test]
fn multiple_aliases_same_session() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();

    set_alias(&fs, &path, "alias1", "/sessions/shared", None, NOW);
    set_alias(&fs, &path, "alias2", "/sessions/shared", None, NOW);

    let data = load_aliases(&fs, &path, NOW);
    let session_aliases = get_aliases_for_session(&data, "/sessions/shared");
    assert_eq!(session_aliases.len(), 2);
}
