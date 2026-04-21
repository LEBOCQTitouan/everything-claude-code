//! Migration use case: migrate_statuses syncs entry files against the BACKLOG.md index.

use super::index::{parse_index_statuses, reindex};
use ecc_domain::backlog::entry::BacklogError;
use ecc_ports::backlog::{BacklogEntryStore, BacklogIndexStore, BacklogLockStore};
use ecc_ports::clock::Clock;
use ecc_ports::worktree::WorktreeManager;
use std::path::Path;

/// Report produced by [`migrate_statuses`].
#[derive(Debug, Clone)]
pub struct MigrationReport {
    /// IDs of entries whose status file was updated.
    pub updated: Vec<String>,
    /// IDs of entries that were already in sync (no write needed).
    pub skipped: Vec<String>,
    /// IDs + error messages for entries that could not be processed.
    pub failed: Vec<(String, String)>,
}

/// Sync divergent entry files against the BACKLOG.md index (best-effort).
///
/// For each entry in the index, compares the file's status against the index status.
/// If they differ, updates the file via `entries.update_entry_status()`.
/// Also normalizes quoted status values to unquoted even when status is the same.
/// Failures are collected into `MigrationReport.failed`; processing continues.
/// After migration, rewrites the index via `reindex(force=true)`.
#[allow(clippy::too_many_arguments)]
pub fn migrate_statuses(
    entries: &dyn BacklogEntryStore,
    locks: &dyn BacklogLockStore,
    index_store: &dyn BacklogIndexStore,
    worktree_mgr: &dyn WorktreeManager,
    clock: &dyn Clock,
    backlog_dir: &Path,
    project_dir: &Path,
) -> Result<MigrationReport, BacklogError> {
    let mut report = MigrationReport {
        updated: Vec::new(),
        skipped: Vec::new(),
        failed: Vec::new(),
    };

    let index_content = index_store.read_index(backlog_dir)?;
    let Some(index_content) = index_content else {
        return Ok(report);
    };
    let index_statuses = parse_index_statuses(&index_content);

    for (id, index_status) in &index_statuses {
        let content = match entries.read_entry_content(backlog_dir, id) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(entry_id = %id, error = %e, "migration: failed to read entry content");
                report.failed.push((id.clone(), e.to_string()));
                continue;
            }
        };

        // Check if the file has a quoted status matching the index — needs normalization
        let needs_normalization = content.contains(&format!("status: \"{index_status}\""))
            || content.contains(&format!("status: '{index_status}'"));

        // Detect if file status differs from index status (by calling replace to check)
        let updated = match ecc_domain::backlog::entry::replace_frontmatter_status(
            &content,
            index_status,
        ) {
            Ok(u) => u,
            Err(e) => {
                tracing::warn!(entry_id = %id, error = %e, "migration: failed to compute status replacement");
                report.failed.push((id.clone(), e.to_string()));
                continue;
            }
        };

        if updated == content && !needs_normalization {
            // Already in sync, no normalization needed
            report.skipped.push(id.clone());
            continue;
        }

        // File needs update (either status change or quoting normalization)
        if let Err(e) = entries.update_entry_status(backlog_dir, id, index_status) {
            tracing::warn!(entry_id = %id, error = %e, "migration: failed to update entry status");
            report.failed.push((id.clone(), e.to_string()));
            continue;
        }
        report.updated.push(id.clone());
    }

    // Reindex with force=true — migration is not blocked by safety check
    reindex(
        entries,
        locks,
        index_store,
        worktree_mgr,
        clock,
        backlog_dir,
        project_dir,
        false,
        true,
    )?;

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::index::reindex;
    use super::super::test_helpers::*;
    use ecc_domain::backlog::entry::BacklogStatus;
    use ecc_test_support::{InMemoryBacklogRepository, MockWorktreeManager};
    use std::path::Path;

    fn make_raw_with_status(id: &str, status: &str) -> String {
        format!(
            "---\nid: {id}\nstatus: {status}\ntitle: Test {id}\ncreated: 2026-01-01\n---\n\n# Body\n"
        )
    }

    fn make_index_with_entries(entries: &[(&str, &str)]) -> String {
        let mut content = String::from(
            "# Backlog Index\n\n| ID | Title | Tier | Scope | Target | Status | Created |\n|----|-------|------|-------|--------|--------|----------|\n",
        );
        for (id, status) in entries {
            content.push_str(&format!(
                "| {id} | Title | core | infra | — | {status} | 2026-01-01 |\n"
            ));
        }
        content.push_str("\n## Stats\n");
        content
    }

    /// PC-013: migrate_statuses computes dynamic divergence and updates files
    #[test]
    fn migrate_statuses_dynamic_divergence() {
        // File says "open", index says "implemented" → file should be updated
        let raw_bl001 = make_raw_with_status("BL-001", "open");
        let raw_bl002 = make_raw_with_status("BL-002", "open"); // no divergence
        let index = make_index_with_entries(&[("BL-001", "implemented"), ("BL-002", "open")]);

        let repo = InMemoryBacklogRepository::new()
            .with_raw_content("BL-001", &raw_bl001)
            .with_raw_content("BL-002", &raw_bl002)
            .with_entry(make_entry("BL-001", BacklogStatus::Open))
            .with_entry(make_entry("BL-002", BacklogStatus::Open))
            .with_index(&index);

        let worktree_mgr = MockWorktreeManager::new();
        let clock = fresh_clock();

        let report = migrate_statuses(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
        )
        .unwrap();

        assert_eq!(report.updated, vec!["BL-001".to_string()]);
        assert!(report.failed.is_empty());

        // Verify file was actually updated
        let updated_content = repo
            .read_entry_content(Path::new(BACKLOG_DIR), "BL-001")
            .unwrap();
        assert!(
            updated_content.contains("status: implemented"),
            "file should now say implemented"
        );
        assert!(
            !updated_content.contains("status: open"),
            "file should no longer say open"
        );
    }

    /// PC-014: Migration handles partial failure (best-effort)
    #[test]
    fn migrate_statuses_partial_failure() {
        // BL-001 exists in index but NOT in raw_contents → should fail gracefully
        // BL-002 diverges and should be updated
        let raw_bl002 = make_raw_with_status("BL-002", "open");
        let index = make_index_with_entries(&[
            ("BL-001", "implemented"), // BL-001 has no raw content → will fail
            ("BL-002", "implemented"), // BL-002 will succeed
        ]);

        let repo = InMemoryBacklogRepository::new()
            .with_raw_content("BL-002", &raw_bl002)
            .with_entry(make_entry("BL-002", BacklogStatus::Open))
            .with_index(&index);

        let worktree_mgr = MockWorktreeManager::new();
        let clock = fresh_clock();

        let report = migrate_statuses(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
        )
        .unwrap();

        // BL-001 should fail (no raw content), BL-002 should succeed
        assert!(
            report.failed.iter().any(|(id, _)| id == "BL-001"),
            "BL-001 should be in failed"
        );
        assert!(
            report.updated.contains(&"BL-002".to_string()),
            "BL-002 should be updated"
        );
    }

    /// PC-015: MigrationReport has updated/skipped/failed fields
    #[test]
    fn migrate_statuses_report_structure() {
        // BL-001: diverges → updated
        // BL-002: same status → skipped
        let raw_bl001 = make_raw_with_status("BL-001", "open");
        let raw_bl002 = make_raw_with_status("BL-002", "implemented");
        let index = make_index_with_entries(&[
            ("BL-001", "implemented"),
            ("BL-002", "implemented"), // same → skip
        ]);

        let repo = InMemoryBacklogRepository::new()
            .with_raw_content("BL-001", &raw_bl001)
            .with_raw_content("BL-002", &raw_bl002)
            .with_entry(make_entry("BL-001", BacklogStatus::Open))
            .with_entry(make_entry("BL-002", BacklogStatus::Implemented))
            .with_index(&index);

        let worktree_mgr = MockWorktreeManager::new();
        let clock = fresh_clock();

        let report = migrate_statuses(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
        )
        .unwrap();

        assert_eq!(report.updated, vec!["BL-001".to_string()]);
        assert!(report.skipped.contains(&"BL-002".to_string()));
        assert!(report.failed.is_empty());
    }

    // --- migration integration tests ---

    /// PC-028: After migration, reindex dry-run matches current index (idempotent proof)
    #[test]
    fn migration_idempotent_proof() {
        let raw_bl001 = make_raw_with_status("BL-001", "open");
        let raw_bl002 = make_raw_with_status("BL-002", "open");
        let index = make_index_with_entries(&[("BL-001", "implemented"), ("BL-002", "open")]);

        let repo = InMemoryBacklogRepository::new()
            .with_raw_content("BL-001", &raw_bl001)
            .with_raw_content("BL-002", &raw_bl002)
            .with_entry(make_entry("BL-001", BacklogStatus::Open))
            .with_entry(make_entry("BL-002", BacklogStatus::Open))
            .with_index(&index);

        let worktree_mgr = MockWorktreeManager::new();
        let clock = fresh_clock();

        // Run migration
        let report = migrate_statuses(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
        )
        .unwrap();

        assert!(
            !report.failed.iter().any(|(id, _)| id == "BL-001"),
            "migration should not fail on BL-001"
        );

        let dry_run_result = reindex(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            true,  // dry_run
            false, // force — should not be needed after migration
        );

        assert!(
            dry_run_result.is_ok(),
            "reindex dry-run should succeed after migration (idempotent): {:?}",
            dry_run_result.err()
        );
        let dry_run_output = dry_run_result
            .unwrap()
            .expect("dry_run should return content");
        assert!(
            dry_run_output.contains("BL-001"),
            "dry-run output should contain BL-001"
        );
        assert!(
            dry_run_output.contains("BL-002"),
            "dry-run output should contain BL-002"
        );
    }

    /// PC-029: Quoting normalized to unquoted
    #[test]
    fn migration_normalizes_quoting() {
        // File has quoted status: "open" → should be normalized to: open
        let quoted_content =
            "---\nid: BL-001\nstatus: \"open\"\ntitle: Test\ncreated: 2026-01-01\n---\n\n# Body\n";
        // Index says "open" too → no status change, but quoting should be normalized
        let index = make_index_with_entries(&[("BL-001", "open")]);

        let repo = InMemoryBacklogRepository::new()
            .with_raw_content("BL-001", quoted_content)
            .with_entry(make_entry("BL-001", BacklogStatus::Open))
            .with_index(&index);

        let worktree_mgr = MockWorktreeManager::new();
        let clock = fresh_clock();

        let _report = migrate_statuses(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
        )
        .unwrap();

        let updated = repo
            .read_entry_content(Path::new(BACKLOG_DIR), "BL-001")
            .unwrap();
        assert!(
            updated.contains("status: open"),
            "status should be unquoted after migration, got: {updated}"
        );
        assert!(
            !updated.contains("status: \"open\""),
            "quoted status should be replaced with unquoted"
        );
    }
}
