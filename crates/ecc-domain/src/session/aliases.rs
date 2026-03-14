//! Session alias management — port of `session-aliases.ts`.
//!
//! All I/O functions take `fs: &dyn FileSystem` and `path: &Path`.
//! All timestamp-dependent functions take `now: &str` for deterministic tests.

use ecc_ports::fs::FileSystem;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::LazyLock;

// ── Constants ──────────────────────────────────────────────────────────

pub const ALIAS_VERSION: &str = "1.0";
pub const MAX_ALIAS_LENGTH: usize = 128;

const RESERVED_NAMES: &[&str] = &["list", "help", "remove", "delete", "create", "set"];

static ALIAS_NAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_-]+$").expect("valid regex"));

// ── Data types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AliasEntry {
    pub session_path: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AliasMetadata {
    pub total_count: usize,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AliasesData {
    pub version: String,
    pub aliases: BTreeMap<String, AliasEntry>,
    pub metadata: AliasMetadata,
}

// ── Result types ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAlias {
    pub alias: String,
    pub session_path: String,
    pub created_at: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetAliasResult {
    pub success: bool,
    pub error: Option<String>,
    pub is_new: Option<bool>,
    pub alias: Option<String>,
    pub session_path: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteAliasResult {
    pub success: bool,
    pub error: Option<String>,
    pub alias: Option<String>,
    pub deleted_session_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameAliasResult {
    pub success: bool,
    pub error: Option<String>,
    pub old_alias: Option<String>,
    pub new_alias: Option<String>,
    pub session_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasInfo {
    pub name: String,
    pub session_path: String,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionAliasInfo {
    pub name: String,
    pub created_at: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupResult {
    pub success: bool,
    pub total_checked: usize,
    pub removed: usize,
    pub removed_aliases: Vec<RemovedAlias>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovedAlias {
    pub name: String,
    pub session_path: String,
}

// ── Validation helpers ─────────────────────────────────────────────────

fn is_valid_alias_name(name: &str) -> bool {
    ALIAS_NAME_RE.is_match(name)
}

fn is_reserved(name: &str) -> bool {
    RESERVED_NAMES.contains(&name.to_lowercase().as_str())
}

fn validate_alias_name(name: &str) -> Option<String> {
    if name.is_empty() {
        return Some("Alias name cannot be empty".to_string());
    }
    if name.len() > MAX_ALIAS_LENGTH {
        return Some("Alias name cannot exceed 128 characters".to_string());
    }
    if !is_valid_alias_name(name) {
        return Some(
            "Alias name must contain only letters, numbers, dashes, and underscores".to_string(),
        );
    }
    if is_reserved(name) {
        return Some(format!("'{name}' is a reserved alias name"));
    }
    None
}

// ── Core functions ─────────────────────────────────────────────────────

/// Create an empty `AliasesData` with the given timestamp.
pub fn default_aliases(now: &str) -> AliasesData {
    AliasesData {
        version: ALIAS_VERSION.to_string(),
        aliases: BTreeMap::new(),
        metadata: AliasMetadata {
            total_count: 0,
            last_updated: now.to_string(),
        },
    }
}

/// Load aliases from the file at `path`. Returns defaults if the file
/// is missing or contains invalid JSON/structure.
pub fn load_aliases(fs: &dyn FileSystem, path: &Path, now: &str) -> AliasesData {
    let content = match fs.read_to_string(path) {
        Ok(c) => c,
        Err(_) => return default_aliases(now),
    };

    if content.is_empty() {
        return default_aliases(now);
    }

    match serde_json::from_str::<AliasesData>(&content) {
        Ok(mut data) => {
            // Ensure version is populated
            if data.version.is_empty() {
                data.version = ALIAS_VERSION.to_string();
            }
            data
        }
        Err(_) => default_aliases(now),
    }
}

/// Save aliases to the file at `path`. Updates metadata before writing.
/// Returns `true` on success.
pub fn save_aliases(
    fs: &dyn FileSystem,
    path: &Path,
    data: &mut AliasesData,
    now: &str,
) -> bool {
    data.metadata = AliasMetadata {
        total_count: data.aliases.len(),
        last_updated: now.to_string(),
    };

    let content = match serde_json::to_string_pretty(data) {
        Ok(c) => c,
        Err(_) => return false,
    };

    if let Some(parent) = path.parent()
        && fs.create_dir_all(parent).is_err()
    {
        return false;
    }

    fs.write(path, &content).is_ok()
}

/// Resolve an alias name to a `ResolvedAlias`. Pure function.
pub fn resolve_alias(data: &AliasesData, alias: &str) -> Option<ResolvedAlias> {
    if alias.is_empty() || !is_valid_alias_name(alias) {
        return None;
    }

    data.aliases.get(alias).map(|entry| ResolvedAlias {
        alias: alias.to_string(),
        session_path: entry.session_path.clone(),
        created_at: entry.created_at.clone(),
        title: entry.title.clone(),
    })
}

/// Set or update an alias. Validates the name, loads from disk, upserts, saves.
pub fn set_alias(
    fs: &dyn FileSystem,
    path: &Path,
    alias: &str,
    session_path: &str,
    title: Option<&str>,
    now: &str,
) -> SetAliasResult {
    if let Some(err) = validate_alias_name(alias) {
        return SetAliasResult {
            success: false,
            error: Some(err),
            is_new: None,
            alias: None,
            session_path: None,
            title: None,
        };
    }

    if session_path.is_empty() || session_path.trim().is_empty() {
        return SetAliasResult {
            success: false,
            error: Some("Session path cannot be empty".to_string()),
            is_new: None,
            alias: None,
            session_path: None,
            title: None,
        };
    }

    let mut data = load_aliases(fs, path, now);
    let existing = data.aliases.get(alias);
    let is_new = existing.is_none();
    let created_at = existing
        .map(|e| e.created_at.clone())
        .unwrap_or_else(|| now.to_string());

    let title_value = title.map(|t| t.to_string());

    data.aliases.insert(
        alias.to_string(),
        AliasEntry {
            session_path: session_path.to_string(),
            created_at,
            updated_at: Some(now.to_string()),
            title: title_value.clone(),
        },
    );

    if save_aliases(fs, path, &mut data, now) {
        SetAliasResult {
            success: true,
            error: None,
            is_new: Some(is_new),
            alias: Some(alias.to_string()),
            session_path: Some(session_path.to_string()),
            title: title_value,
        }
    } else {
        SetAliasResult {
            success: false,
            error: Some("Failed to save alias".to_string()),
            is_new: None,
            alias: None,
            session_path: None,
            title: None,
        }
    }
}

/// List aliases with optional search filter and limit. Pure function.
pub fn list_aliases(
    data: &AliasesData,
    search: Option<&str>,
    limit: Option<usize>,
) -> Vec<AliasInfo> {
    let mut aliases: Vec<AliasInfo> = data
        .aliases
        .iter()
        .map(|(name, entry)| AliasInfo {
            name: name.clone(),
            session_path: entry.session_path.clone(),
            created_at: entry.created_at.clone(),
            updated_at: entry.updated_at.clone(),
            title: entry.title.clone(),
        })
        .collect();

    // Sort by updated_at (or created_at fallback) descending
    aliases.sort_by(|a, b| {
        let a_ts = a.updated_at.as_deref().unwrap_or(&a.created_at);
        let b_ts = b.updated_at.as_deref().unwrap_or(&b.created_at);
        b_ts.cmp(a_ts)
    });

    if let Some(query) = search {
        let lower = query.to_lowercase();
        aliases.retain(|a| {
            a.name.to_lowercase().contains(&lower)
                || a.title
                    .as_ref()
                    .is_some_and(|t| t.to_lowercase().contains(&lower))
        });
    }

    if let Some(n) = limit
        && n > 0
    {
        aliases.truncate(n);
    }

    aliases
}

/// Delete an alias. Returns the deleted session path on success.
pub fn delete_alias(
    fs: &dyn FileSystem,
    path: &Path,
    alias: &str,
    now: &str,
) -> DeleteAliasResult {
    let mut data = load_aliases(fs, path, now);

    let Some(entry) = data.aliases.remove(alias) else {
        return DeleteAliasResult {
            success: false,
            error: Some(format!("Alias '{alias}' not found")),
            alias: None,
            deleted_session_path: None,
        };
    };

    if save_aliases(fs, path, &mut data, now) {
        DeleteAliasResult {
            success: true,
            error: None,
            alias: Some(alias.to_string()),
            deleted_session_path: Some(entry.session_path),
        }
    } else {
        DeleteAliasResult {
            success: false,
            error: Some("Failed to delete alias".to_string()),
            alias: None,
            deleted_session_path: None,
        }
    }
}

/// Rename an alias from `old_alias` to `new_alias`.
pub fn rename_alias(
    fs: &dyn FileSystem,
    path: &Path,
    old_alias: &str,
    new_alias: &str,
    now: &str,
) -> RenameAliasResult {
    let mut data = load_aliases(fs, path, now);

    if !data.aliases.contains_key(old_alias) {
        return RenameAliasResult {
            success: false,
            error: Some(format!("Alias '{old_alias}' not found")),
            old_alias: None,
            new_alias: None,
            session_path: None,
        };
    }

    if let Some(err) = validate_alias_name(new_alias) {
        return RenameAliasResult {
            success: false,
            error: Some(err),
            old_alias: None,
            new_alias: None,
            session_path: None,
        };
    }

    if data.aliases.contains_key(new_alias) {
        return RenameAliasResult {
            success: false,
            error: Some(format!("Alias '{new_alias}' already exists")),
            old_alias: None,
            new_alias: None,
            session_path: None,
        };
    }

    let mut entry = data.aliases.remove(old_alias).expect("checked above");
    entry.updated_at = Some(now.to_string());
    let session_path = entry.session_path.clone();
    data.aliases.insert(new_alias.to_string(), entry);

    if save_aliases(fs, path, &mut data, now) {
        RenameAliasResult {
            success: true,
            error: None,
            old_alias: Some(old_alias.to_string()),
            new_alias: Some(new_alias.to_string()),
            session_path: Some(session_path),
        }
    } else {
        RenameAliasResult {
            success: false,
            error: Some("Failed to save renamed alias".to_string()),
            old_alias: None,
            new_alias: None,
            session_path: None,
        }
    }
}

/// Get all aliases pointing to a specific session path. Pure function.
pub fn get_aliases_for_session(data: &AliasesData, session_path: &str) -> Vec<SessionAliasInfo> {
    data.aliases
        .iter()
        .filter(|(_, entry)| entry.session_path == session_path)
        .map(|(name, entry)| SessionAliasInfo {
            name: name.clone(),
            created_at: entry.created_at.clone(),
            title: entry.title.clone(),
        })
        .collect()
}

/// Remove aliases whose sessions no longer exist. Mutates `data` in place.
pub fn cleanup_aliases(
    data: &mut AliasesData,
    session_exists: &dyn Fn(&str) -> bool,
) -> CleanupResult {
    let total_checked = data.aliases.len();
    let mut removed_aliases = Vec::new();

    data.aliases.retain(|name, entry| {
        if session_exists(&entry.session_path) {
            true
        } else {
            removed_aliases.push(RemovedAlias {
                name: name.clone(),
                session_path: entry.session_path.clone(),
            });
            false
        }
    });

    CleanupResult {
        success: true,
        total_checked,
        removed: removed_aliases.len(),
        removed_aliases,
        error: None,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;
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

    // ── default_aliases ────────────────────────────────────────────────

    #[test]
    fn default_aliases_has_version() {
        let data = default_aliases(NOW);
        assert_eq!(data.version, ALIAS_VERSION);
    }

    #[test]
    fn default_aliases_is_empty() {
        let data = default_aliases(NOW);
        assert!(data.aliases.is_empty());
        assert_eq!(data.metadata.total_count, 0);
    }

    #[test]
    fn default_aliases_uses_now() {
        let data = default_aliases(NOW);
        assert_eq!(data.metadata.last_updated, NOW);
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
        let fs = InMemoryFileSystem::new()
            .with_file("/home/user/.claude/session-aliases.json", "");
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
        let content = serde_json::to_string_pretty(&make_data(&[
            ("proj", "/sessions/abc", NOW, Some("My Project")),
        ]))
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

    // ── resolve_alias (pure) ───────────────────────────────────────────

    #[test]
    fn resolve_found() {
        let data = make_data(&[("proj", "/sessions/abc", NOW, Some("My Project"))]);
        let resolved = resolve_alias(&data, "proj").unwrap();
        assert_eq!(resolved.session_path, "/sessions/abc");
        assert_eq!(resolved.title.as_deref(), Some("My Project"));
    }

    #[test]
    fn resolve_not_found() {
        let data = make_data(&[]);
        assert!(resolve_alias(&data, "nope").is_none());
    }

    #[test]
    fn resolve_empty_alias() {
        let data = make_data(&[("a", "/s/1", NOW, None)]);
        assert!(resolve_alias(&data, "").is_none());
    }

    #[test]
    fn resolve_invalid_format() {
        let data = make_data(&[]);
        assert!(resolve_alias(&data, "has spaces").is_none());
    }

    #[test]
    fn resolve_special_chars_rejected() {
        let data = make_data(&[]);
        assert!(resolve_alias(&data, "a@b").is_none());
        assert!(resolve_alias(&data, "a/b").is_none());
        assert!(resolve_alias(&data, "a.b").is_none());
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
        for name in RESERVED_NAMES {
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
        assert_eq!(result.deleted_session_path.as_deref(), Some("/sessions/abc"));

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

    // ── list_aliases (pure) ────────────────────────────────────────────

    #[test]
    fn list_empty() {
        let data = default_aliases(NOW);
        let list = list_aliases(&data, None, None);
        assert!(list.is_empty());
    }

    #[test]
    fn list_returns_all() {
        let data = make_data(&[("a", "/s/1", NOW, None), ("b", "/s/2", NOW, None)]);
        let list = list_aliases(&data, None, None);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn list_sorted_by_updated_desc() {
        let mut data = make_data(&[
            ("old", "/s/1", "2025-01-01T00:00:00Z", None),
            ("new", "/s/2", "2025-01-03T00:00:00Z", None),
        ]);
        data.aliases
            .get_mut("old")
            .unwrap()
            .updated_at = Some("2025-01-05T00:00:00Z".to_string());

        let list = list_aliases(&data, None, None);
        assert_eq!(list[0].name, "old"); // updated_at is newest
        assert_eq!(list[1].name, "new");
    }

    #[test]
    fn list_sorted_falls_back_to_created_at() {
        let data = make_data(&[
            ("older", "/s/1", "2025-01-01T00:00:00Z", None),
            ("newer", "/s/2", "2025-01-03T00:00:00Z", None),
        ]);
        let list = list_aliases(&data, None, None);
        assert_eq!(list[0].name, "newer");
        assert_eq!(list[1].name, "older");
    }

    #[test]
    fn list_search_by_name() {
        let data = make_data(&[
            ("project-alpha", "/s/1", NOW, None),
            ("project-beta", "/s/2", NOW, None),
            ("other", "/s/3", NOW, None),
        ]);
        let list = list_aliases(&data, Some("alpha"), None);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "project-alpha");
    }

    #[test]
    fn list_search_by_title() {
        let data = make_data(&[
            ("a", "/s/1", NOW, Some("Alpha Project")),
            ("b", "/s/2", NOW, Some("Beta Project")),
        ]);
        let list = list_aliases(&data, Some("beta"), None);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "b");
    }

    #[test]
    fn list_search_case_insensitive() {
        let data = make_data(&[("MyProject", "/s/1", NOW, None)]);
        let list = list_aliases(&data, Some("myproject"), None);
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn list_search_no_match() {
        let data = make_data(&[("a", "/s/1", NOW, None)]);
        let list = list_aliases(&data, Some("zzz"), None);
        assert!(list.is_empty());
    }

    #[test]
    fn list_with_limit() {
        let data = make_data(&[
            ("a", "/s/1", "2025-01-01T00:00:00Z", None),
            ("b", "/s/2", "2025-01-02T00:00:00Z", None),
            ("c", "/s/3", "2025-01-03T00:00:00Z", None),
        ]);
        let list = list_aliases(&data, None, Some(2));
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn list_limit_zero_returns_all() {
        let data = make_data(&[("a", "/s/1", NOW, None), ("b", "/s/2", NOW, None)]);
        let list = list_aliases(&data, None, Some(0));
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn list_limit_exceeding_count() {
        let data = make_data(&[("a", "/s/1", NOW, None)]);
        let list = list_aliases(&data, None, Some(100));
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn list_search_and_limit_combined() {
        let data = make_data(&[
            ("proj-1", "/s/1", "2025-01-01T00:00:00Z", None),
            ("proj-2", "/s/2", "2025-01-02T00:00:00Z", None),
            ("proj-3", "/s/3", "2025-01-03T00:00:00Z", None),
            ("other", "/s/4", NOW, None),
        ]);
        let list = list_aliases(&data, Some("proj"), Some(2));
        assert_eq!(list.len(), 2);
        // Should be the most recently created proj entries
        assert_eq!(list[0].name, "proj-3");
        assert_eq!(list[1].name, "proj-2");
    }

    // ── get_aliases_for_session (pure) ─────────────────────────────────

    #[test]
    fn get_aliases_for_session_found() {
        let data = make_data(&[
            ("a", "/sessions/xyz", NOW, Some("Title A")),
            ("b", "/sessions/xyz", NOW, None),
            ("c", "/sessions/other", NOW, None),
        ]);
        let aliases = get_aliases_for_session(&data, "/sessions/xyz");
        assert_eq!(aliases.len(), 2);
        let names: Vec<&str> = aliases.iter().map(|a| a.name.as_str()).collect();
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
    }

    #[test]
    fn get_aliases_for_session_not_found() {
        let data = make_data(&[("a", "/sessions/abc", NOW, None)]);
        let aliases = get_aliases_for_session(&data, "/sessions/nope");
        assert!(aliases.is_empty());
    }

    #[test]
    fn get_aliases_for_session_preserves_title() {
        let data = make_data(&[("a", "/sessions/abc", NOW, Some("My Title"))]);
        let aliases = get_aliases_for_session(&data, "/sessions/abc");
        assert_eq!(aliases[0].title.as_deref(), Some("My Title"));
    }

    // ── cleanup_aliases (pure) ─────────────────────────────────────────

    #[test]
    fn cleanup_removes_dead_sessions() {
        let mut data = make_data(&[
            ("alive", "/sessions/exists", NOW, None),
            ("dead", "/sessions/gone", NOW, None),
        ]);
        let result = cleanup_aliases(&mut data, &|path| path == "/sessions/exists");
        assert!(result.success);
        assert_eq!(result.total_checked, 2);
        assert_eq!(result.removed, 1);
        assert_eq!(result.removed_aliases.len(), 1);
        assert_eq!(result.removed_aliases[0].name, "dead");
        assert_eq!(result.removed_aliases[0].session_path, "/sessions/gone");
        assert!(data.aliases.contains_key("alive"));
        assert!(!data.aliases.contains_key("dead"));
    }

    #[test]
    fn cleanup_keeps_all_live_sessions() {
        let mut data = make_data(&[
            ("a", "/sessions/1", NOW, None),
            ("b", "/sessions/2", NOW, None),
        ]);
        let result = cleanup_aliases(&mut data, &|_| true);
        assert!(result.success);
        assert_eq!(result.total_checked, 2);
        assert_eq!(result.removed, 0);
        assert!(result.removed_aliases.is_empty());
        assert_eq!(data.aliases.len(), 2);
    }

    #[test]
    fn cleanup_removes_all_dead() {
        let mut data = make_data(&[
            ("a", "/sessions/1", NOW, None),
            ("b", "/sessions/2", NOW, None),
        ]);
        let result = cleanup_aliases(&mut data, &|_| false);
        assert_eq!(result.removed, 2);
        assert!(data.aliases.is_empty());
    }

    #[test]
    fn cleanup_empty_data() {
        let mut data = default_aliases(NOW);
        let result = cleanup_aliases(&mut data, &|_| false);
        assert!(result.success);
        assert_eq!(result.total_checked, 0);
        assert_eq!(result.removed, 0);
    }

    #[test]
    fn cleanup_no_error_on_success() {
        let mut data = default_aliases(NOW);
        let result = cleanup_aliases(&mut data, &|_| true);
        assert!(result.error.is_none());
    }

    // ── Round-trip integration tests ───────────────────────────────────

    #[test]
    fn full_lifecycle_create_resolve_delete() {
        let fs = InMemoryFileSystem::new();
        let path = aliases_path();

        // Create
        let set_result = set_alias(&fs, &path, "myproj", "/sessions/abc", Some("My Project"), NOW);
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

    #[test]
    fn set_alias_without_title() {
        let fs = InMemoryFileSystem::new();
        let result = set_alias(&fs, &aliases_path(), "proj", "/s/1", None, NOW);
        assert!(result.success);
        assert!(result.title.is_none());
    }

    #[test]
    fn list_includes_all_fields() {
        let mut data = make_data(&[("proj", "/s/1", NOW, Some("Title"))]);
        data.aliases.get_mut("proj").unwrap().updated_at = Some(LATER.to_string());

        let list = list_aliases(&data, None, None);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "proj");
        assert_eq!(list[0].session_path, "/s/1");
        assert_eq!(list[0].created_at, NOW);
        assert_eq!(list[0].updated_at.as_deref(), Some(LATER));
        assert_eq!(list[0].title.as_deref(), Some("Title"));
    }

    #[test]
    fn resolve_alias_returns_created_at() {
        let data = make_data(&[("proj", "/s/1", "2025-06-01T00:00:00Z", None)]);
        let resolved = resolve_alias(&data, "proj").unwrap();
        assert_eq!(resolved.created_at, "2025-06-01T00:00:00Z");
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
    fn cleanup_result_has_correct_session_paths() {
        let mut data = make_data(&[
            ("dead1", "/sessions/gone1", NOW, None),
            ("dead2", "/sessions/gone2", NOW, None),
        ]);
        let result = cleanup_aliases(&mut data, &|_| false);
        let paths: Vec<&str> = result
            .removed_aliases
            .iter()
            .map(|r| r.session_path.as_str())
            .collect();
        assert!(paths.contains(&"/sessions/gone1"));
        assert!(paths.contains(&"/sessions/gone2"));
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
}
