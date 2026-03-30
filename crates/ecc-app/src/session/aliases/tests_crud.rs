use super::*;
use ecc_domain::session::aliases::{ALIAS_VERSION, AliasEntry, AliasMetadata, AliasesData};
use ecc_test_support::InMemoryFileSystem;
use std::collections::BTreeMap;
use std::path::PathBuf;

const NOW: &str = "2025-01-15T10:00:00.000Z";
const LATER: &str = "2025-01-15T11:00:00.000Z";
const EVEN_LATER: &str = "2025-01-15T12:00:00.000Z";

fn aliases_path() -> PathBuf {
    PathBuf::from("/home/user/.claude/session-aliases.json")
}

fn make_data(entries: &[(&str, &str, &str, Option<&str>)]) -> AliasesData {
    let mut aliases = BTreeMap::new();
    for &(name, sp, created, title) in entries {
        aliases.insert(
            name.to_string(),
            AliasEntry {
                session_path: sp.to_string(),
                created_at: created.to_string(),
                updated_at: None,
                title: title.map(|t| t.to_string()),
            },
        );
    }
    AliasesData {
        version: ALIAS_VERSION.to_string(),
        aliases,
        metadata: AliasMetadata {
            total_count: entries.len(),
            last_updated: NOW.to_string(),
        },
    }
}

// ── load_aliases ───────────────────────────────────────────────────

#[test]
fn load_missing_file_returns_default() {
    let fs = InMemoryFileSystem::new();
    let data = load_aliases(&fs, &aliases_path(), NOW);
    assert!(data.aliases.is_empty());
    assert_eq!(data.version, ALIAS_VERSION);
}

#[test]
fn load_empty_file_returns_default() {
    let fs = InMemoryFileSystem::new().with_file("/home/user/.claude/session-aliases.json", "");
    let data = load_aliases(&fs, &aliases_path(), NOW);
    assert!(data.aliases.is_empty());
}

#[test]
fn load_invalid_json_returns_default() {
    let fs = InMemoryFileSystem::new()
        .with_file("/home/user/.claude/session-aliases.json", "{not valid json");
    let data = load_aliases(&fs, &aliases_path(), NOW);
    assert!(data.aliases.is_empty());
}

#[test]
fn load_valid_file() {
    let content = serde_json::to_string_pretty(&make_data(&[(
        "proj",
        "/sessions/abc",
        NOW,
        Some("My Project"),
    )]))
    .unwrap();
    let fs = InMemoryFileSystem::new()
        .with_file("/home/user/.claude/session-aliases.json", &content);
    let data = load_aliases(&fs, &aliases_path(), NOW);
    assert_eq!(data.aliases.len(), 1);
    assert!(data.aliases.contains_key("proj"));
}

#[test]
fn load_preserves_existing_entries() {
    let content = serde_json::to_string_pretty(&make_data(&[
        ("a", "/s/1", NOW, None),
        ("b", "/s/2", NOW, Some("Title B")),
    ]))
    .unwrap();
    let fs = InMemoryFileSystem::new()
        .with_file("/home/user/.claude/session-aliases.json", &content);
    let data = load_aliases(&fs, &aliases_path(), NOW);
    assert_eq!(data.aliases.len(), 2);
    assert_eq!(data.aliases["b"].title.as_deref(), Some("Title B"));
}

// ── save_aliases ───────────────────────────────────────────────────

#[test]
fn save_writes_json() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();
    let mut data = make_data(&[("test", "/s/123", NOW, None)]);
    assert!(save_aliases(&fs, &path, &mut data, NOW));

    let written = fs.read_to_string(&path).unwrap();
    let parsed: AliasesData = serde_json::from_str(&written).unwrap();
    assert_eq!(parsed.aliases.len(), 1);
    assert!(parsed.aliases.contains_key("test"));
}

#[test]
fn save_updates_metadata() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();
    let mut data = make_data(&[("a", "/s/1", NOW, None), ("b", "/s/2", NOW, None)]);
    save_aliases(&fs, &path, &mut data, LATER);
    assert_eq!(data.metadata.total_count, 2);
    assert_eq!(data.metadata.last_updated, LATER);
}

#[test]
fn save_creates_parent_directories() {
    let fs = InMemoryFileSystem::new();
    let path = PathBuf::from("/new/deep/path/aliases.json");
    let mut data = default_aliases(NOW);
    assert!(save_aliases(&fs, &path, &mut data, NOW));
    assert!(fs.exists(&path));
}

// ── set_alias validation ───────────────────────────────────────────

#[test]
fn set_empty_name_fails() {
    let fs = InMemoryFileSystem::new();
    let result = set_alias(&fs, &aliases_path(), "", "/s/1", None, NOW);
    assert!(!result.success);
    assert!(result.error.unwrap().contains("empty"));
}

#[test]
fn set_too_long_name_fails() {
    let fs = InMemoryFileSystem::new();
    let long_name = "a".repeat(129);
    let result = set_alias(&fs, &aliases_path(), &long_name, "/s/1", None, NOW);
    assert!(!result.success);
    assert!(result.error.unwrap().contains("128"));
}

#[test]
fn set_invalid_chars_fails() {
    let fs = InMemoryFileSystem::new();
    let result = set_alias(&fs, &aliases_path(), "bad name!", "/s/1", None, NOW);
    assert!(!result.success);
    assert!(result.error.unwrap().contains("letters"));
}

#[test]
fn set_reserved_name_fails() {
    let fs = InMemoryFileSystem::new();
    for name in &["list", "help", "remove", "delete", "create", "set"] {
        let result = set_alias(&fs, &aliases_path(), name, "/s/1", None, NOW);
        assert!(!result.success, "should reject reserved name: {name}");
        assert!(result.error.unwrap().contains("reserved"));
    }
}

#[test]
fn set_reserved_case_insensitive() {
    let fs = InMemoryFileSystem::new();
    let result = set_alias(&fs, &aliases_path(), "LIST", "/s/1", None, NOW);
    assert!(!result.success);
    assert!(result.error.unwrap().contains("reserved"));
}

#[test]
fn set_empty_session_path_fails() {
    let fs = InMemoryFileSystem::new();
    let result = set_alias(&fs, &aliases_path(), "valid", "", None, NOW);
    assert!(!result.success);
    assert!(result.error.unwrap().contains("Session path"));
}

#[test]
fn set_whitespace_session_path_fails() {
    let fs = InMemoryFileSystem::new();
    let result = set_alias(&fs, &aliases_path(), "valid", "   ", None, NOW);
    assert!(!result.success);
    assert!(result.error.unwrap().contains("Session path"));
}

// ── set_alias CRUD ─────────────────────────────────────────────────

#[test]
fn set_new_alias() {
    let fs = InMemoryFileSystem::new();
    let result = set_alias(
        &fs,
        &aliases_path(),
        "proj",
        "/sessions/abc",
        Some("My Project"),
        NOW,
    );
    assert!(result.success);
    assert_eq!(result.is_new, Some(true));
    assert_eq!(result.alias.as_deref(), Some("proj"));
    assert_eq!(result.session_path.as_deref(), Some("/sessions/abc"));
    assert_eq!(result.title.as_deref(), Some("My Project"));
}

#[test]
fn set_alias_persists_to_disk() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();
    set_alias(&fs, &path, "proj", "/sessions/abc", None, NOW);

    let data = load_aliases(&fs, &path, NOW);
    assert!(data.aliases.contains_key("proj"));
}

#[test]
fn set_update_existing_alias() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();
    set_alias(&fs, &path, "proj", "/sessions/old", None, NOW);
    let result = set_alias(&fs, &path, "proj", "/sessions/new", Some("Updated"), LATER);
    assert!(result.success);
    assert_eq!(result.is_new, Some(false));
    assert_eq!(result.session_path.as_deref(), Some("/sessions/new"));

    let data = load_aliases(&fs, &path, NOW);
    assert_eq!(data.aliases["proj"].session_path, "/sessions/new");
    // created_at should be preserved from original
    assert_eq!(data.aliases["proj"].created_at, NOW);
    assert_eq!(data.aliases["proj"].updated_at.as_deref(), Some(LATER));
}

#[test]
fn set_alias_with_dashes_and_underscores() {
    let fs = InMemoryFileSystem::new();
    let result = set_alias(&fs, &aliases_path(), "my-project_v2", "/s/1", None, NOW);
    assert!(result.success);
}

#[test]
fn set_alias_max_length_ok() {
    let fs = InMemoryFileSystem::new();
    let name = "a".repeat(128);
    let result = set_alias(&fs, &aliases_path(), &name, "/s/1", None, NOW);
    assert!(result.success);
}

// ── delete_alias ───────────────────────────────────────────────────

#[test]
fn delete_existing_alias() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();
    set_alias(&fs, &path, "proj", "/sessions/abc", None, NOW);

    let result = delete_alias(&fs, &path, "proj", LATER);
    assert!(result.success);
    assert_eq!(result.alias.as_deref(), Some("proj"));
    assert_eq!(
        result.deleted_session_path.as_deref(),
        Some("/sessions/abc")
    );

    let data = load_aliases(&fs, &path, LATER);
    assert!(!data.aliases.contains_key("proj"));
}

#[test]
fn delete_nonexistent_alias() {
    let fs = InMemoryFileSystem::new();
    let result = delete_alias(&fs, &aliases_path(), "nope", NOW);
    assert!(!result.success);
    assert!(result.error.unwrap().contains("not found"));
}

#[test]
fn delete_does_not_affect_other_aliases() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();
    set_alias(&fs, &path, "a", "/s/1", None, NOW);
    set_alias(&fs, &path, "b", "/s/2", None, NOW);

    delete_alias(&fs, &path, "a", LATER);

    let data = load_aliases(&fs, &path, LATER);
    assert!(!data.aliases.contains_key("a"));
    assert!(data.aliases.contains_key("b"));
}

// ── Additional tests ───────────────────────────────────────────────

#[test]
fn set_alias_without_title() {
    let fs = InMemoryFileSystem::new();
    let result = set_alias(&fs, &aliases_path(), "proj", "/s/1", None, NOW);
    assert!(result.success);
    assert!(result.title.is_none());
}

#[test]
fn set_alias_numeric_name() {
    let fs = InMemoryFileSystem::new();
    let result = set_alias(&fs, &aliases_path(), "123", "/s/1", None, NOW);
    assert!(result.success);
}

#[test]
fn set_alias_single_char_name() {
    let fs = InMemoryFileSystem::new();
    let result = set_alias(&fs, &aliases_path(), "x", "/s/1", None, NOW);
    assert!(result.success);
}

#[test]
fn delete_alias_returns_session_path() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();
    set_alias(&fs, &path, "proj", "/sessions/xyz", None, NOW);
    let result = delete_alias(&fs, &path, "proj", LATER);
    assert_eq!(
        result.deleted_session_path.as_deref(),
        Some("/sessions/xyz")
    );
}

#[test]
fn set_alias_dot_in_name_rejected() {
    let fs = InMemoryFileSystem::new();
    let result = set_alias(&fs, &aliases_path(), "has.dot", "/s/1", None, NOW);
    assert!(!result.success);
}

#[test]
fn set_alias_slash_in_name_rejected() {
    let fs = InMemoryFileSystem::new();
    let result = set_alias(&fs, &aliases_path(), "has/slash", "/s/1", None, NOW);
    assert!(!result.success);
}

#[test]
fn set_alias_space_in_name_rejected() {
    let fs = InMemoryFileSystem::new();
    let result = set_alias(&fs, &aliases_path(), "has space", "/s/1", None, NOW);
    assert!(!result.success);
}

#[test]
fn save_and_load_roundtrip() {
    let fs = InMemoryFileSystem::new();
    let path = aliases_path();
    let mut original = make_data(&[
        ("alpha", "/s/1", NOW, Some("Alpha")),
        ("beta", "/s/2", LATER, None),
    ]);
    save_aliases(&fs, &path, &mut original, EVEN_LATER);

    let loaded = load_aliases(&fs, &path, EVEN_LATER);
    assert_eq!(loaded.aliases.len(), 2);
    assert_eq!(loaded.aliases["alpha"].title.as_deref(), Some("Alpha"));
    assert_eq!(loaded.aliases["beta"].session_path, "/s/2");
    assert_eq!(loaded.metadata.total_count, 2);
    assert_eq!(loaded.metadata.last_updated, EVEN_LATER);
}

/// PC-040: Corrupt aliases.json emits tracing::warn!
#[test]
fn corrupt_aliases_warns() {
    use testing_logger;

    testing_logger::setup();

    let fs = InMemoryFileSystem::new().with_file(
        "/home/user/.claude/session-aliases.json",
        "{not valid json {{{{",
    );

    let _data = load_aliases(&fs, &aliases_path(), NOW);

    testing_logger::validate(|captured_logs| {
        let found = captured_logs.iter().any(|log| {
            log.level == log::Level::Warn
                && log.body.contains("load_aliases")
                && log.body.contains("corrupt")
        });
        if !found {
            let messages: Vec<String> = captured_logs
                .iter()
                .map(|l| format!("[{}] {}", l.level, l.body))
                .collect();
            panic!(
                "expected tracing::warn! with 'load_aliases' and 'corrupt' in message.\nCaptured logs: {messages:?}"
            );
        }
    });
}
