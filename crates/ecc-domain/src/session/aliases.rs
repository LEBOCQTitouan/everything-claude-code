//! Session alias management — types and pure functions.
//! Ported from TypeScript `session-aliases.ts`.
//!
//! I/O functions (load_aliases, save_aliases, set_alias, delete_alias,
//! rename_alias) live in `ecc-app::session::aliases`.
//! All timestamp-dependent functions take `now: &str` for deterministic tests.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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

/// Validate an alias name. Returns `None` if valid, or `Some(error_message)` if invalid.
pub fn validate_alias_name(name: &str) -> Option<String> {
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

    const NOW: &str = "2025-01-15T10:00:00.000Z";
    const LATER: &str = "2025-01-15T11:00:00.000Z";

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

}
