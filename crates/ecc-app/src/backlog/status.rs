//! Status-related backlog use cases: list_available, update_status, collect_claimed_ids.

use super::{extract_bl_id_from_path, extract_bl_num};
use super::index::reindex;
use ecc_domain::backlog::entry::{BacklogEntry, BacklogError, BacklogStatus};
use ecc_ports::backlog::{BacklogEntryStore, BacklogIndexStore, BacklogLockStore};
use ecc_ports::clock::Clock;
use ecc_ports::worktree::WorktreeManager;
use std::collections::HashSet;
use std::path::Path;

/// Return the open backlog entries not currently claimed by a worktree or fresh lock.
///
/// If `show_all` is true, returns all open entries regardless of claims.
/// Stale and orphaned locks are auto-removed.
pub fn list_available(
    entries: &dyn BacklogEntryStore,
    locks: &dyn BacklogLockStore,
    worktree_mgr: &dyn WorktreeManager,
    clock: &dyn Clock,
    backlog_dir: &Path,
    project_dir: &Path,
    show_all: bool,
) -> Result<Vec<BacklogEntry>, BacklogError> {
    let all_entries = entries.load_entries(backlog_dir)?;
    let open_entries: Vec<BacklogEntry> = all_entries
        .into_iter()
        .filter(|e| e.status == BacklogStatus::Open)
        .collect();

    if show_all {
        return Ok(open_entries);
    }

    let claimed = collect_claimed_ids(locks, worktree_mgr, clock, backlog_dir, project_dir);

    let available = open_entries
        .into_iter()
        .filter(|e| {
            if let Some(num) = extract_bl_num(&e.id) {
                !claimed.contains(&num)
            } else {
                true
            }
        })
        .collect();

    Ok(available)
}

/// Update the status of a backlog entry by ID, then reindex.
///
/// Validates `new_status` against `VALID_STATUSES`. Returns an error if the
/// entry is not found or the status is invalid. If the entry already has the
/// requested status, returns `Ok(())` without writing (no-op guard).
#[allow(clippy::too_many_arguments)]
pub fn update_status(
    entries: &dyn BacklogEntryStore,
    index_store: &dyn BacklogIndexStore,
    lock_store: &dyn BacklogLockStore,
    worktree_mgr: &dyn WorktreeManager,
    clock: &dyn Clock,
    backlog_dir: &Path,
    project_dir: &Path,
    id: &str,
    new_status: &str,
) -> Result<(), BacklogError> {
    use ecc_domain::backlog::entry::VALID_STATUSES;
    // AC-001.4: validate new_status
    if ecc_domain::backlog::entry::BacklogStatus::from_kebab(new_status).is_none() {
        return Err(BacklogError::MalformedYaml(format!(
            "invalid status '{}'; valid values are: {}",
            new_status,
            VALID_STATUSES.join(", ")
        )));
    }
    // AC-001.3: read raw content (propagates Io error if not found)
    let content = entries.read_entry_content(backlog_dir, id)?;
    // AC-001.5: no-op guard — if content unchanged, return early
    let updated = ecc_domain::backlog::entry::replace_frontmatter_status(&content, new_status)?;
    if updated == content {
        return Ok(());
    }
    // AC-001.2: write update then reindex
    entries.update_entry_status(backlog_dir, id, new_status)?;
    reindex(
        entries,
        lock_store,
        index_store,
        worktree_mgr,
        clock,
        backlog_dir,
        project_dir,
        false,
        false,
    )?;
    Ok(())
}

/// Collect the set of BL numeric IDs that are currently claimed (worktrees + fresh locks).
///
/// Stale and orphaned locks are auto-removed as a side-effect.
pub(crate) fn collect_claimed_ids(
    locks: &dyn BacklogLockStore,
    worktree_mgr: &dyn WorktreeManager,
    clock: &dyn Clock,
    backlog_dir: &Path,
    project_dir: &Path,
) -> HashSet<u32> {
    let mut claimed: HashSet<u32> = HashSet::new();

    // Collect IDs claimed by active worktrees
    let worktrees = worktree_mgr.list_worktrees(project_dir).unwrap_or_default();
    let worktree_names: HashSet<String> = worktrees
        .iter()
        .map(|wt| wt.path.rsplit('/').next().unwrap_or(&wt.path).to_string())
        .collect();

    for wt in &worktrees {
        if let Some(num) = extract_bl_id_from_path(&wt.path) {
            claimed.insert(num);
        }
    }

    // Collect IDs claimed by fresh locks, auto-remove stale/orphaned ones
    let now = clock.now_epoch_secs();
    let all_locks = locks.list_locks(backlog_dir).unwrap_or_default();

    for (id, lock) in all_locks {
        let is_orphaned = !worktree_names.contains(&lock.worktree_name);
        let is_stale = lock.is_stale(now);

        if is_stale || is_orphaned {
            if let Err(e) = locks.remove_lock(backlog_dir, &id) {
                tracing::warn!(lock_id = %id, error = %e, "failed to remove stale lock");
            }
        } else if let Some(num) = extract_bl_num(&id) {
            claimed.insert(num);
        }
    }

    claimed
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::test_helpers::*;
    use ecc_domain::backlog::entry::BacklogStatus;
    use ecc_ports::backlog::BacklogIndexStore;
    use ecc_ports::worktree::WorktreeInfo;
    use ecc_test_support::{InMemoryBacklogRepository, MockWorktreeManager};
    use std::path::Path;

    fn raw_open_content(id: &str) -> String {
        format!("---\nid: {id}\nstatus: open\ntitle: Test\ncreated: 2026-01-01\n---\n\n# Body\n")
    }

    // --- list_available tests ---

    #[test]
    fn list_available_excludes_worktree_claims() {
        let repo = InMemoryBacklogRepository::new()
            .with_entry(make_entry("BL-042", BacklogStatus::Open))
            .with_entry(make_entry("BL-043", BacklogStatus::Open));

        let worktree_mgr = MockWorktreeManager::new().with_worktrees(vec![WorktreeInfo {
            path: "/project/.claude/worktrees/ecc-session-20260407-bl-042-something".to_string(),
            branch: None,
        }]);
        let clock = fresh_clock();

        let result = list_available(
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            false,
        )
        .unwrap();

        assert_eq!(result.len(), 1, "only BL-043 should be available");
        assert_eq!(result[0].id, "BL-043");
    }

    #[test]
    fn list_available_excludes_locked() {
        let repo = InMemoryBacklogRepository::new()
            .with_entry(make_entry("BL-010", BacklogStatus::Open))
            .with_entry(make_entry("BL-011", BacklogStatus::Open))
            .with_lock(
                "BL-010",
                make_fresh_lock("ecc-session-20260407-bl-010-work"),
            );

        let worktree_mgr = MockWorktreeManager::new().with_worktrees(vec![WorktreeInfo {
            path: "/project/.claude/worktrees/ecc-session-20260407-bl-010-work".to_string(),
            branch: None,
        }]);
        let clock = fresh_clock();

        let result = list_available(
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            false,
        )
        .unwrap();

        assert_eq!(result.len(), 1, "only BL-011 should be available");
        assert_eq!(result[0].id, "BL-011");
    }

    #[test]
    fn list_available_includes_stale_lock() {
        let repo = InMemoryBacklogRepository::new()
            .with_entry(make_entry("BL-010", BacklogStatus::Open))
            .with_lock("BL-010", make_stale_lock("old-worktree"));

        // No active worktree named "old-worktree" — so lock is orphaned + stale
        let worktree_mgr = MockWorktreeManager::new();
        let clock = fresh_clock();

        let result = list_available(
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            false,
        )
        .unwrap();

        // BL-010 should be available because the lock is stale/orphaned
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "BL-010");

        // The stale lock should have been removed
        let remaining_locks = repo.list_locks(Path::new(BACKLOG_DIR)).unwrap();
        assert!(
            remaining_locks.is_empty(),
            "stale lock should be auto-removed"
        );
    }

    #[test]
    fn list_available_show_all() {
        let repo = InMemoryBacklogRepository::new()
            .with_entry(make_entry("BL-001", BacklogStatus::Open))
            .with_entry(make_entry("BL-002", BacklogStatus::Open))
            .with_entry(make_entry("BL-003", BacklogStatus::Implemented));

        // BL-001 is claimed by worktree
        let worktree_mgr = MockWorktreeManager::new().with_worktrees(vec![WorktreeInfo {
            path: "/project/.claude/worktrees/ecc-session-20260407-bl-001-something".to_string(),
            branch: None,
        }]);
        let clock = fresh_clock();

        let result = list_available(
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            true, // show_all
        )
        .unwrap();

        // show_all returns all open entries including claimed, but not implemented
        assert_eq!(result.len(), 2, "show_all returns all open entries");
        let ids: Vec<&str> = result.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains(&"BL-001"));
        assert!(ids.contains(&"BL-002"));
    }

    #[test]
    fn list_available_empty_result() {
        let repo = InMemoryBacklogRepository::new()
            .with_entry(make_entry("BL-001", BacklogStatus::Implemented))
            .with_entry(make_entry("BL-002", BacklogStatus::Archived));

        let worktree_mgr = MockWorktreeManager::new();
        let clock = fresh_clock();

        let result = list_available(
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            false,
        )
        .unwrap();

        assert!(result.is_empty(), "no open entries means empty result");
    }

    // --- update_status tests ---

    /// PC-008: update_status errors on invalid BL id
    #[test]
    fn update_status_invalid_id() {
        let repo = InMemoryBacklogRepository::new();
        let worktree_mgr = MockWorktreeManager::new();
        let clock = fresh_clock();

        let err = update_status(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            "BL-999",
            "implemented",
        )
        .unwrap_err();

        assert!(
            matches!(err, BacklogError::Io { .. }),
            "expected Io error for missing entry, got: {err:?}"
        );
    }

    /// PC-009: update_status errors on invalid status string
    #[test]
    fn update_status_invalid_status() {
        let repo = InMemoryBacklogRepository::new()
            .with_raw_content("BL-001", &raw_open_content("BL-001"));
        let worktree_mgr = MockWorktreeManager::new();
        let clock = fresh_clock();

        let err = update_status(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            "BL-001",
            "wip",
        )
        .unwrap_err();

        assert!(
            matches!(err, BacklogError::MalformedYaml(_)),
            "expected MalformedYaml for invalid status, got: {err:?}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("wip"),
            "error message should contain the invalid status"
        );
        // AC-001.4: message should list valid statuses
        assert!(
            msg.contains("open"),
            "error message should list valid statuses"
        );
    }

    /// PC-010: update_status succeeds and triggers reindex
    #[test]
    fn update_status_success_triggers_reindex() {
        let repo = InMemoryBacklogRepository::new()
            .with_raw_content("BL-001", &raw_open_content("BL-001"))
            .with_entry(make_entry("BL-001", BacklogStatus::Open));
        let worktree_mgr = MockWorktreeManager::new();
        let clock = fresh_clock();

        // Before: no index
        assert!(
            repo.read_index(Path::new(BACKLOG_DIR)).unwrap().is_none(),
            "index should not exist before update"
        );

        update_status(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            "BL-001",
            "implemented",
        )
        .unwrap();

        // After: index should have been written by reindex
        let index = repo.read_index(Path::new(BACKLOG_DIR)).unwrap();
        assert!(index.is_some(), "reindex should have written the index");
    }

    /// PC-011: update_status no-op for same status
    #[test]
    fn update_status_noop_same_status() {
        // Content already has "status: open"
        let raw = raw_open_content("BL-001");
        let repo = InMemoryBacklogRepository::new()
            .with_raw_content("BL-001", &raw)
            .with_entry(make_entry("BL-001", BacklogStatus::Open));
        let worktree_mgr = MockWorktreeManager::new();
        let clock = fresh_clock();

        let result = update_status(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            "BL-001",
            "open",
        );

        assert!(result.is_ok(), "no-op should return Ok");
        // No index should be written since this is a no-op
        let index = repo.read_index(Path::new(BACKLOG_DIR)).unwrap();
        assert!(
            index.is_none(),
            "no-op should not trigger reindex (no write occurs)"
        );
    }

    /// PC-019: lock removal failure logged via tracing::warn
    #[test]
    fn lock_removal_failure_logged() {
        use ecc_domain::backlog::lock::LockFile;

        // A stale orphaned lock for BL-010 — no active worktree by that name
        let stale_lock = LockFile::new(
            "old-worktree".to_string(),
            "2026-04-06T00:00:00Z".to_string(),
        )
        .unwrap();
        let repo = InMemoryBacklogRepository::new()
            .with_entry(make_entry("BL-010", BacklogStatus::Open))
            .with_lock("BL-010", stale_lock);

        // No active worktree named "old-worktree" => lock is orphaned + stale
        let worktree_mgr = MockWorktreeManager::new();
        let clock = fresh_clock();

        // Should succeed — stale lock handled gracefully
        let result = list_available(
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            false,
        );
        assert!(
            result.is_ok(),
            "list_available should succeed even with stale lock"
        );

        // Verify stale lock was removed (InMemoryBacklogRepository.remove_lock succeeds)
        let remaining = repo.list_locks(Path::new(BACKLOG_DIR)).unwrap();
        assert!(remaining.is_empty(), "stale lock should have been removed");
    }
}
