//! Session alias I/O operations.
//!
//! Pure types and functions live in `ecc-domain::session::aliases`.
//! This module provides I/O functions that depend on `FileSystem`.

use ecc_domain::session::aliases::{
    ALIAS_VERSION, AliasEntry, AliasMetadata, AliasesData, DeleteAliasResult, RenameAliasResult,
    SetAliasResult, default_aliases, validate_alias_name,
};
use ecc_ports::fs::FileSystem;
use std::path::Path;

// ---------------------------------------------------------------------------
// I/O functions (depend on FileSystem port)
// ---------------------------------------------------------------------------

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
        Err(e) => {
            log::warn!("load_aliases: corrupt JSON at {}: {e}", path.display());
            default_aliases(now)
        }
    }
}

/// Save aliases to the file at `path`. Updates metadata before writing.
/// Returns `true` on success.
pub fn save_aliases(fs: &dyn FileSystem, path: &Path, data: &mut AliasesData, now: &str) -> bool {
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

/// Delete an alias. Returns the deleted session path on success.
pub fn delete_alias(fs: &dyn FileSystem, path: &Path, alias: &str, now: &str) -> DeleteAliasResult {
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "tests_crud.rs"]
mod tests_crud;

#[cfg(test)]
#[path = "tests_rename.rs"]
mod tests_rename;
