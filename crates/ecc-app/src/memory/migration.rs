//! Migration and export use cases.

use std::path::Path;

use ecc_domain::memory::{MemoryEntry, MemoryId, MemoryTier};
use ecc_domain::memory::export::format_entry_as_md;
use ecc_domain::memory::migration::{parse_action_log_entry, parse_work_item_md};
use ecc_ports::fs::FileSystem;
use ecc_ports::memory_store::MemoryStore;

use crate::memory::crud::MemoryAppError;

/// Result of a migration run.
#[derive(Debug, Default)]
pub struct MigrateResult {
    pub inserted: usize,
    pub skipped_duplicate: usize,
    pub skipped_malformed: usize,
}

/// Migrate legacy work-item markdown files from `source_dir` into the store.
///
/// Idempotent: keyed on source_path (won't insert if already present).
pub fn migrate_work_items(
    store: &dyn MemoryStore,
    fs: &dyn FileSystem,
    source_dir: &Path,
) -> Result<MigrateResult, MemoryAppError> {
    let mut result = MigrateResult::default();

    if !fs.is_dir(source_dir) {
        return Ok(result);
    }

    let files = fs
        .read_dir_recursive(source_dir)
        .map_err(|e| MemoryAppError::InvalidInput(e.to_string()))?;

    for file_path in files {
        if file_path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let path_str = file_path.to_string_lossy().to_string();

        // Idempotency check
        let existing = store
            .get_by_source_path(&path_str)
            .map_err(MemoryAppError::Store)?;
        if existing.is_some() {
            result.skipped_duplicate += 1;
            continue;
        }

        let content = match fs.read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => {
                result.skipped_malformed += 1;
                continue;
            }
        };

        match parse_work_item_md(&content) {
            Some(mut entry) => {
                entry.source_path = Some(path_str);
                store.insert(&entry).map_err(MemoryAppError::Store)?;
                result.inserted += 1;
            }
            None => {
                result.skipped_malformed += 1;
            }
        }
    }

    Ok(result)
}

/// Migrate action-log.json entries into the store.
///
/// Idempotent: keyed on source_path (e.g. "action-log.json:<index>").
pub fn migrate_action_log(
    store: &dyn MemoryStore,
    fs: &dyn FileSystem,
    action_log_path: &Path,
) -> Result<MigrateResult, MemoryAppError> {
    let mut result = MigrateResult::default();

    if !fs.exists(action_log_path) {
        return Ok(result);
    }

    let content = fs
        .read_to_string(action_log_path)
        .map_err(|e| MemoryAppError::InvalidInput(e.to_string()))?;

    let entries: Vec<serde_json::Value> = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => {
            result.skipped_malformed += 1;
            return Ok(result);
        }
    };

    for (i, raw) in entries.iter().enumerate() {
        let source_key = format!("{}:{}", action_log_path.to_string_lossy(), i);

        // Idempotency check
        let existing = store
            .get_by_source_path(&source_key)
            .map_err(MemoryAppError::Store)?;
        if existing.is_some() {
            result.skipped_duplicate += 1;
            continue;
        }

        match parse_action_log_entry(raw) {
            Some(mut entry) => {
                entry.source_path = Some(source_key);
                store.insert(&entry).map_err(MemoryAppError::Store)?;
                result.inserted += 1;
            }
            None => {
                result.skipped_malformed += 1;
            }
        }
    }

    Ok(result)
}

/// Export all entries as individual markdown files, grouped by tier.
///
/// Output structure: `<output_dir>/<tier>/<id>-<sanitized-title>.md`
pub fn export(
    store: &dyn MemoryStore,
    fs: &dyn FileSystem,
    output_dir: &Path,
) -> Result<usize, MemoryAppError> {
    let entries = store
        .list_filtered(None, None, None)
        .map_err(MemoryAppError::Store)?;

    let mut written = 0usize;

    for entry in &entries {
        let tier_dir = output_dir.join(entry.tier.to_string());
        fs.create_dir_all(&tier_dir)
            .map_err(|e| MemoryAppError::InvalidInput(e.to_string()))?;

        let safe_title: String = entry
            .title
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
            .take(50)
            .collect();

        let filename = format!("{}-{}.md", entry.id, safe_title);
        let file_path = tier_dir.join(&filename);
        let md = format_entry_as_md(entry);
        fs.write(&file_path, &md)
            .map_err(|e| MemoryAppError::InvalidInput(e.to_string()))?;

        written += 1;
    }

    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::memory::{MemoryEntry, MemoryId, MemoryTier};
    use ecc_test_support::{InMemoryFileSystem, InMemoryMemoryStore};

    fn make_store() -> InMemoryMemoryStore {
        InMemoryMemoryStore::new()
    }

    fn make_fs() -> InMemoryFileSystem {
        InMemoryFileSystem::new()
    }

    fn insert_entry(
        store: &InMemoryMemoryStore,
        title: &str,
        content: &str,
        tier: MemoryTier,
        source_path: Option<&str>,
    ) -> MemoryId {
        let entry = MemoryEntry::new(
            MemoryId(0),
            tier,
            title,
            content,
            vec![],
            None,
            None,
            1.0,
            "2026-01-01T00:00:00Z",
            "2026-01-01T00:00:00Z",
            false,
            vec![],
            source_path.map(str::to_owned),
        );
        store.insert(&entry).unwrap()
    }

    // PC-041: App `migrate` converts work-item markdown files to episodic entries
    #[test]
    fn test_migrate_work_items_basic() {
        let store = make_store();
        let fs = make_fs();

        let dir = Path::new("/work-items");
        fs.create_dir_all(dir).unwrap();
        fs.write(
            &dir.join("bl-001.md"),
            "# Implement Feature BL-001\n\nSome content here.",
        )
        .unwrap();

        let result = migrate_work_items(&store, &fs, dir).unwrap();
        assert_eq!(result.inserted, 1);
        assert_eq!(result.skipped_malformed, 0);

        let entries = store.list_filtered(None, None, None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].tier, MemoryTier::Episodic);
        assert!(entries[0].title.contains("Implement Feature BL-001"));
    }

    // PC-043: App `migrate` is idempotent (keyed on source_path)
    #[test]
    fn test_migrate_work_items_idempotent() {
        let store = make_store();
        let fs = make_fs();

        let dir = Path::new("/work-items");
        fs.create_dir_all(dir).unwrap();
        fs.write(&dir.join("item.md"), "# My Item\n\nContent.").unwrap();

        migrate_work_items(&store, &fs, dir).unwrap();
        let result2 = migrate_work_items(&store, &fs, dir).unwrap();

        assert_eq!(result2.inserted, 0);
        assert_eq!(result2.skipped_duplicate, 1);
        // Still only one entry
        let entries = store.list_filtered(None, None, None).unwrap();
        assert_eq!(entries.len(), 1);
    }

    // PC-044: App `migrate` populates related_work_items from BL-NNN refs
    #[test]
    fn test_migrate_work_items_populates_bl_refs() {
        let store = make_store();
        let fs = make_fs();

        let dir = Path::new("/work-items");
        fs.create_dir_all(dir).unwrap();
        fs.write(
            &dir.join("bl-042.md"),
            "# Feature BL-042\n\nRelated to BL-010 and BL-020.",
        )
        .unwrap();

        migrate_work_items(&store, &fs, dir).unwrap();
        let entries = store.list_filtered(None, None, None).unwrap();
        assert!(entries[0].related_work_items.contains(&"BL-042".to_owned()));
        assert!(entries[0].related_work_items.contains(&"BL-010".to_owned()));
    }

    // PC-042: App `migrate` converts action-log.json entries to episodic entries
    #[test]
    fn test_migrate_action_log_basic() {
        let store = make_store();
        let fs = make_fs();

        let log_content = r#"[
            {"action": "implement feature", "description": "Did BL-010", "timestamp": "2026-01-01T00:00:00Z"},
            {"action": "fix bug", "description": "Fixed BL-011", "timestamp": "2026-01-02T00:00:00Z"}
        ]"#;
        let log_path = Path::new("/action-log.json");
        fs.write(log_path, log_content).unwrap();

        let result = migrate_action_log(&store, &fs, log_path).unwrap();
        assert_eq!(result.inserted, 2);

        let entries = store.list_filtered(None, None, None).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.tier == MemoryTier::Episodic));
    }

    // PC-043 (action-log variant): idempotent migration
    #[test]
    fn test_migrate_action_log_idempotent() {
        let store = make_store();
        let fs = make_fs();

        let log_content = r#"[{"action": "test action", "description": "desc"}]"#;
        let log_path = Path::new("/action-log.json");
        fs.write(log_path, log_content).unwrap();

        migrate_action_log(&store, &fs, log_path).unwrap();
        let result2 = migrate_action_log(&store, &fs, log_path).unwrap();
        assert_eq!(result2.inserted, 0);
        assert_eq!(result2.skipped_duplicate, 1);
    }

    // PC-045: App `migrate` skips malformed action-log entries, reports count
    #[test]
    fn test_migrate_action_log_skips_malformed_entries() {
        let store = make_store();
        let fs = make_fs();

        let log_content = r#"[
            {"action": "good action", "description": "desc"},
            {"missing_action_field": "bad"},
            {"action": "", "description": "empty action"}
        ]"#;
        let log_path = Path::new("/action-log.json");
        fs.write(log_path, log_content).unwrap();

        let result = migrate_action_log(&store, &fs, log_path).unwrap();
        assert_eq!(result.inserted, 1);
        assert_eq!(result.skipped_malformed, 2);
    }

    // PC-046: App `export` writes individual markdown files grouped by tier
    #[test]
    fn test_export_writes_files_by_tier() {
        let store = make_store();
        let fs = make_fs();

        insert_entry(&store, "Semantic Entry", "content", MemoryTier::Semantic, None);
        insert_entry(&store, "Episodic Entry", "content", MemoryTier::Episodic, None);

        let output_dir = Path::new("/export");
        let written = export(&store, &fs, output_dir).unwrap();
        assert_eq!(written, 2);

        // Check tier subdirectories
        assert!(fs.is_dir(&output_dir.join("semantic")));
        assert!(fs.is_dir(&output_dir.join("episodic")));
    }

    // PC-047: App `export` then re-import is lossless (round-trip)
    #[test]
    fn test_export_produces_valid_markdown() {
        let store = make_store();
        let fs = make_fs();

        insert_entry(&store, "My Memory", "this is content", MemoryTier::Semantic, None);

        let output_dir = Path::new("/export");
        export(&store, &fs, output_dir).unwrap();

        // Verify at least one file was written in semantic subdir
        let files = fs.read_dir_recursive(&output_dir.join("semantic")).unwrap();
        assert_eq!(files.len(), 1);

        // Verify file contains title
        let content = fs.read_to_string(&files[0]).unwrap();
        assert!(content.contains("# My Memory"));
        assert!(content.contains("this is content"));
    }
}
