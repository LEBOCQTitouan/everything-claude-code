//! Backlog management use cases — next_id, check_duplicates, reindex, list_available.
//!
//! Orchestrates domain logic through port traits (BacklogEntryStore, BacklogLockStore,
//! BacklogIndexStore, WorktreeManager, Clock).

use ecc_domain::backlog::entry::{BacklogEntry, BacklogError, BacklogStatus};
use ecc_domain::backlog::index::{extract_dependency_graph, generate_index_table, generate_stats};
use ecc_domain::backlog::similarity::{DUPLICATE_THRESHOLD, DuplicateCandidate, composite_score};
use ecc_ports::backlog::{BacklogEntryStore, BacklogIndexStore, BacklogLockStore};
use ecc_ports::clock::Clock;
use ecc_ports::worktree::WorktreeManager;
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;

/// Extract a BL-NNN numeric ID from a worktree path's last component.
///
/// Matches patterns like `ecc-session-20260407-bl-042-something` or
/// `ecc-bl042-feature` (case-insensitive).
fn extract_bl_id_from_path(path: &str) -> Option<u32> {
    let last = path.rsplit('/').next().unwrap_or(path);
    // Match bl-NNN or blNNN (case-insensitive)
    let re = Regex::new(r"(?i)bl-?(\d{3,})").ok()?;
    let caps = re.captures(last)?;
    caps.get(1)?.as_str().parse().ok()
}

/// Compute the next sequential backlog ID from entries via the store port.
///
/// Returns `"BL-NNN"` where NNN is max existing ID + 1, zero-padded to 3 digits.
pub fn next_id(
    entries: &dyn BacklogEntryStore,
    backlog_dir: &Path,
) -> Result<String, BacklogError> {
    entries.next_id(backlog_dir)
}

/// Check for duplicate backlog entries by title similarity.
///
/// Filters to active entries (open/in-progress) only.
/// Returns candidates sorted by score descending, filtered to score >= DUPLICATE_THRESHOLD.
pub fn check_duplicates(
    entries: &dyn BacklogEntryStore,
    backlog_dir: &Path,
    query: &str,
    query_tags: &[String],
) -> Result<Vec<DuplicateCandidate>, BacklogError> {
    if query.is_empty() {
        return Err(BacklogError::EmptyQuery);
    }

    let all_entries = entries.load_entries(backlog_dir)?;
    let mut candidates = Vec::new();

    for entry in &all_entries {
        if !entry.status.is_active() {
            continue;
        }
        let score = composite_score(query, &entry.title, query_tags, &entry.tags);
        if score >= DUPLICATE_THRESHOLD {
            candidates.push(DuplicateCandidate {
                id: entry.id.clone(),
                title: entry.title.clone(),
                score: (score * 100.0).round() / 100.0,
            });
        }
    }

    candidates.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(candidates)
}

/// Regenerate BACKLOG.md from all BL-*.md files.
///
/// Accepts a WorktreeManager and Clock to determine which entries are in-progress
/// (claimed by active worktrees or fresh lock files).
///
/// If `dry_run` is true, returns the generated content without writing.
#[allow(clippy::too_many_arguments)]
pub fn reindex(
    entries: &dyn BacklogEntryStore,
    locks: &dyn BacklogLockStore,
    index: &dyn BacklogIndexStore,
    worktree_mgr: &dyn WorktreeManager,
    clock: &dyn Clock,
    backlog_dir: &Path,
    project_dir: &Path,
    dry_run: bool,
) -> Result<Option<String>, BacklogError> {
    let mut all_entries = entries.load_entries(backlog_dir)?;

    // Collect claimed IDs from worktrees
    let claimed = collect_claimed_ids(locks, worktree_mgr, clock, backlog_dir, project_dir);

    // Override status to InProgress for claimed entries
    for entry in &mut all_entries {
        if let Some(num) = extract_bl_num(&entry.id)
            && claimed.contains(&num)
            && entry.status == BacklogStatus::Open
        {
            entry.status = BacklogStatus::InProgress;
        }
    }

    let table = generate_index_table(&all_entries);
    let stats = generate_stats(&all_entries);

    let existing_index = index.read_index(backlog_dir)?;
    let dep_graph = existing_index
        .as_deref()
        .and_then(extract_dependency_graph);

    let mut output = String::new();
    output.push_str("# Backlog Index\n\n");
    output.push_str(&table);
    output.push_str("\n\n");
    if let Some(graph) = &dep_graph {
        output.push_str(graph);
        output.push_str("\n\n");
    }
    output.push_str(&stats);
    output.push('\n');

    if dry_run {
        return Ok(Some(output));
    }

    index.write_index(backlog_dir, &output)?;
    Ok(None)
}

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

/// Collect the set of BL numeric IDs that are currently claimed (worktrees + fresh locks).
///
/// Stale and orphaned locks are auto-removed as a side-effect.
fn collect_claimed_ids(
    locks: &dyn BacklogLockStore,
    worktree_mgr: &dyn WorktreeManager,
    clock: &dyn Clock,
    backlog_dir: &Path,
    project_dir: &Path,
) -> HashSet<u32> {
    let mut claimed: HashSet<u32> = HashSet::new();

    // Collect IDs claimed by active worktrees
    let worktrees = worktree_mgr
        .list_worktrees(project_dir)
        .unwrap_or_default();
    let worktree_names: HashSet<String> = worktrees
        .iter()
        .map(|wt| {
            wt.path
                .rsplit('/')
                .next()
                .unwrap_or(&wt.path)
                .to_string()
        })
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
            let _ = locks.remove_lock(backlog_dir, &id);
        } else if let Some(num) = extract_bl_num(&id) {
            claimed.insert(num);
        }
    }

    claimed
}

/// Extract the numeric part from a BL-NNN id string.
fn extract_bl_num(id: &str) -> Option<u32> {
    id.strip_prefix("BL-").and_then(|s| s.parse().ok())
}

/// Convert an [`ecc_ports::fs::FsError`] into a [`BacklogError`].
///
/// The orphan rule prevents `impl From<FsError> for BacklogError` here since neither
/// type is defined in this crate. Use this function where conversion is needed.
pub fn backlog_error_from_fs(e: ecc_ports::fs::FsError) -> BacklogError {
    BacklogError::Io {
        path: e.to_string(),
        message: e.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::backlog::entry::BacklogStatus;
    use ecc_domain::backlog::lock::LockFile;
    use ecc_ports::worktree::WorktreeInfo;
    use ecc_test_support::{InMemoryBacklogRepository, MockClock, MockWorktreeManager};
    use std::path::Path;

    const BACKLOG_DIR: &str = "/backlog";
    const PROJECT_DIR: &str = "/project";

    /// Helper: 2026-04-07T10:00:00Z = 1744016400 epoch seconds (approx)
    /// Use a fixed "now" that is recent enough to not be stale.
    fn fresh_clock() -> MockClock {
        // A recent timestamp: 2026-04-07T12:00:00Z
        MockClock::fixed("2026-04-07T12:00:00Z", 1_744_023_600)
    }

    fn make_entry(id: &str, status: BacklogStatus) -> BacklogEntry {
        BacklogEntry {
            id: id.to_string(),
            title: format!("Entry {id}"),
            status,
            created: "2026-04-07".into(),
            tier: None,
            scope: None,
            target: None,
            target_command: None,
            tags: vec![],
        }
    }

    fn make_entry_with_tags(id: &str, status: BacklogStatus, tags: Vec<String>) -> BacklogEntry {
        BacklogEntry {
            id: id.to_string(),
            title: format!("Entry {id}"),
            status,
            created: "2026-04-07".into(),
            tier: None,
            scope: None,
            target: None,
            target_command: None,
            tags,
        }
    }

    fn make_fresh_lock(worktree_name: &str) -> LockFile {
        // Timestamp close to now (within 24h)
        LockFile::new(worktree_name.to_string(), "2026-04-07T11:00:00Z".to_string()).unwrap()
    }

    fn make_stale_lock(worktree_name: &str) -> LockFile {
        // 2026-04-06T00:00:00Z — more than 24h before fresh_clock's 2026-04-07T12:00:00Z
        LockFile::new(worktree_name.to_string(), "2026-04-06T00:00:00Z".to_string()).unwrap()
    }

    // --- next_id tests ---

    #[test]
    fn next_id_sequential() {
        let repo = InMemoryBacklogRepository::new()
            .with_entry(make_entry("BL-001", BacklogStatus::Open))
            .with_entry(make_entry("BL-075", BacklogStatus::Implemented));
        let result = next_id(&repo, Path::new(BACKLOG_DIR)).unwrap();
        assert_eq!(result, "BL-076");
    }

    #[test]
    fn next_id_empty() {
        let repo = InMemoryBacklogRepository::new();
        let result = next_id(&repo, Path::new(BACKLOG_DIR)).unwrap();
        assert_eq!(result, "BL-001");
    }

    // --- check_duplicates tests ---

    #[test]
    fn check_duplicates_finds_similar() {
        let mut entry = make_entry_with_tags(
            "BL-052",
            BacklogStatus::Open,
            vec!["rust".into(), "hooks".into()],
        );
        entry.title = "Replace hooks with Rust binaries".into();
        let repo = InMemoryBacklogRepository::new().with_entry(entry);

        let result = check_duplicates(
            &repo,
            Path::new(BACKLOG_DIR),
            "Replace hooks with compiled Rust",
            &["rust".into(), "hooks".into()],
        )
        .unwrap();

        assert!(!result.is_empty(), "expected at least one candidate");
        assert!(
            result[0].score >= DUPLICATE_THRESHOLD,
            "score {} < {}",
            result[0].score,
            DUPLICATE_THRESHOLD
        );
    }

    // --- reindex tests ---

    #[test]
    fn reindex_marks_in_progress_from_worktree() {
        let repo = InMemoryBacklogRepository::new()
            .with_entry(make_entry("BL-042", BacklogStatus::Open))
            .with_entry(make_entry("BL-043", BacklogStatus::Open));

        // Worktree path contains "bl-042" -> BL-042 should be marked in-progress
        let worktree_mgr = MockWorktreeManager::new().with_worktrees(vec![WorktreeInfo {
            path: "/project/.claude/worktrees/ecc-session-20260407-bl-042-something".to_string(),
            branch: Some("worktree-ecc-session-20260407-bl-042".to_string()),
        }]);
        let clock = fresh_clock();

        let result = reindex(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            true,
        )
        .unwrap();

        let content = result.expect("dry_run should return content");
        assert!(
            content.contains("in-progress"),
            "BL-042 should be in-progress"
        );
        // BL-043 should remain open
        assert!(content.contains("open"), "BL-043 should remain open");
    }

    #[test]
    fn reindex_marks_in_progress_from_lock() {
        let repo = InMemoryBacklogRepository::new()
            .with_entry(make_entry("BL-010", BacklogStatus::Open))
            .with_lock(
                "BL-010",
                make_fresh_lock("ecc-session-20260407-bl-010-work"),
            );

        let worktree_mgr = MockWorktreeManager::new().with_worktrees(vec![WorktreeInfo {
            path: "/project/.claude/worktrees/ecc-session-20260407-bl-010-work".to_string(),
            branch: None,
        }]);
        let clock = fresh_clock();

        let result = reindex(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            true,
        )
        .unwrap();

        let content = result.expect("dry_run should return content");
        assert!(content.contains("in-progress"), "BL-010 should be in-progress");
    }

    #[test]
    fn reindex_idempotent() {
        let repo = InMemoryBacklogRepository::new()
            .with_entry(make_entry("BL-001", BacklogStatus::Open))
            .with_entry(make_entry("BL-002", BacklogStatus::Implemented));

        let worktree_mgr = MockWorktreeManager::new();
        let clock = fresh_clock();

        let result1 = reindex(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            true,
        )
        .unwrap()
        .unwrap();

        let result2 = reindex(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            true,
        )
        .unwrap()
        .unwrap();

        assert_eq!(result1, result2, "reindex must be idempotent");
    }

    #[test]
    fn reindex_preserves_dep_graph() {
        let dep_graph_content =
            "# Backlog Index\n\n| old |\n\n## Dependency Graph\n\n```\nBL-001 → BL-002\n```\n\n## Stats\n\n- **Total:** 1\n";

        let repo = InMemoryBacklogRepository::new()
            .with_entry(make_entry("BL-001", BacklogStatus::Open))
            .with_index(dep_graph_content);

        let worktree_mgr = MockWorktreeManager::new();
        let clock = fresh_clock();

        let result = reindex(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            true,
        )
        .unwrap();

        let content = result.expect("dry_run should return content");
        assert!(
            content.contains("## Dependency Graph"),
            "dep graph section must be preserved"
        );
        assert!(
            content.contains("BL-001 → BL-002"),
            "dep graph content must be preserved"
        );
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
            .with_lock("BL-010", make_fresh_lock("ecc-session-20260407-bl-010-work"));

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
        let remaining_locks = repo
            .list_locks(Path::new(BACKLOG_DIR))
            .unwrap();
        assert!(remaining_locks.is_empty(), "stale lock should be auto-removed");
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

    // --- FsError conversion test ---

    #[test]
    fn fs_error_conversion() {
        use ecc_ports::fs::FsError;
        let fs_err = FsError::NotFound(std::path::PathBuf::from("/some/path"));
        let backlog_err = backlog_error_from_fs(fs_err);
        assert!(
            matches!(backlog_err, BacklogError::Io { .. }),
            "FsError should convert to BacklogError::Io"
        );
    }
}
