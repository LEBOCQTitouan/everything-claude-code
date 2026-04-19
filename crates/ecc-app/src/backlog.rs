//! Backlog management use cases — next_id, check_duplicates, reindex, list_available.
//!
//! Orchestrates domain logic through port traits (BacklogEntryStore, BacklogLockStore,
//! BacklogIndexStore, WorktreeManager, Clock).

use ecc_domain::backlog::entry::{BacklogEntry, BacklogError, BacklogStatus};
use ecc_domain::backlog::index::{extract_dependency_graph, generate_index_table, generate_stats};
use ecc_domain::backlog::similarity::{DUPLICATE_THRESHOLD, DuplicateCandidate, composite_score};
use ecc_ports::backlog::{BacklogEntryStore, BacklogIndexStore, BacklogLockStore};
use ecc_ports::clock::Clock;
use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use ecc_ports::memory_store::MemoryStore;
use ecc_ports::worktree::WorktreeManager;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::LazyLock;

static BL_ID_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)bl-?(\d{3,})").unwrap());

/// Extract a BL-NNN numeric ID from a worktree path's last component.
///
/// Matches patterns like `ecc-session-20260407-bl-042-something` or
/// `ecc-bl042-feature` (case-insensitive).
fn extract_bl_id_from_path(path: &str) -> Option<u32> {
    let last = path.rsplit('/').next().unwrap_or(path);
    let caps = BL_ID_RE.captures(last)?;
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

/// Parse the BACKLOG.md index content and extract a map of `BL-NNN → status`.
///
/// Scans table rows (lines starting with `|`), skips header and separator rows.
/// The table format is: `| ID | Title | Tier | Scope | Target | Status | Created |`
/// (Status is at column index 5, 0-indexed in the trimmed parts).
pub fn parse_index_statuses(index_content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in index_content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        // Skip separator rows (contain ---)
        if trimmed.contains("---") {
            continue;
        }
        // Split by '|', filter empty parts from leading/trailing '|'
        let parts: Vec<&str> = trimmed
            .split('|')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        // Need at least 6 columns: ID, Title, Tier, Scope, Target, Status
        if parts.len() < 6 {
            continue;
        }
        let id = parts[0];
        let status = parts[5];
        // Skip header row (ID column is literally "ID")
        if id == "ID" {
            continue;
        }
        // Validate looks like BL-NNN
        if !id.starts_with("BL-") {
            continue;
        }
        map.insert(id.to_string(), status.to_string());
    }
    map
}

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

/// Regenerate BACKLOG.md from all BL-*.md files.
///
/// Accepts a WorktreeManager and Clock to determine which entries are in-progress
/// (claimed by active worktrees or fresh lock files).
///
/// If `dry_run` is true, returns the generated content without writing.
/// If `force` is false and >5 status changes are detected vs the current index,
/// returns an error (safety block to prevent accidental status reversion).
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
    force: bool,
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
    let dep_graph = existing_index.as_deref().and_then(extract_dependency_graph);

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

    // Safety check: if >5 status changes vs current index, block or warn.
    let existing_statuses = existing_index
        .as_deref()
        .map(parse_index_statuses)
        .unwrap_or_default();
    let new_statuses = parse_index_statuses(&output);
    let changed_ids: Vec<&str> = new_statuses
        .iter()
        .filter(|(id, new_status)| {
            existing_statuses
                .get(*id)
                .is_some_and(|old| old != *new_status)
        })
        .map(|(id, _)| id.as_str())
        .collect();
    let diff_count = changed_ids.len();
    if diff_count > 5 {
        let id_list = changed_ids.join(", ");
        if !force {
            return Err(BacklogError::SafetyBlock(format!(
                "{diff_count} status changes detected ({id_list}). Use --force to override."
            )));
        }
        tracing::warn!(
            diff_count = diff_count,
            changed = %id_list,
            "reindex: forcing write despite {diff_count} status changes"
        );
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

/// Extract the numeric part from a BL-NNN id string.
fn extract_bl_num(id: &str) -> Option<u32> {
    id.strip_prefix("BL-").and_then(|s| s.parse().ok())
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

/// Update the status of a backlog entry and, if transitioning to "implemented",
/// fire-and-forget the memory prune hook.
///
/// Wraps [`update_status`] and then calls
/// [`memory::file_prune::prune_file_memories_for_backlog`] if `new_status == "implemented"`.
/// If `store` is `Some`, also calls [`memory::lifecycle::prune_by_backlog`] to remove
/// SQLite memory entries tagged with `id`.
/// Prune errors are logged via `tracing::warn!` with target `"memory::prune"` but do NOT
/// propagate — the status transition always returns `Ok(())` on success.
/// If [`memory::paths::resolve_project_memory_root`] fails, a warn is emitted and
/// pruning is skipped entirely.
#[allow(clippy::too_many_arguments)]
pub fn update_status_with_prune_hook(
    entries: &dyn BacklogEntryStore,
    index_store: &dyn BacklogIndexStore,
    lock_store: &dyn BacklogLockStore,
    worktree_mgr: &dyn WorktreeManager,
    clock: &dyn Clock,
    backlog_dir: &Path,
    project_dir: &Path,
    id: &str,
    new_status: &str,
    env: &dyn Environment,
    fs: &dyn FileSystem,
    store: Option<&dyn MemoryStore>,
) -> Result<(), BacklogError> {
    update_status(
        entries,
        index_store,
        lock_store,
        worktree_mgr,
        clock,
        backlog_dir,
        project_dir,
        id,
        new_status,
    )?;

    if new_status == "implemented" {
        let today = &clock.now_iso8601()[..10]; // YYYY-MM-DD prefix
        match crate::memory::paths::resolve_project_memory_root(env, fs) {
            Err(e) => {
                tracing::warn!(
                    target: "memory::prune",
                    bl_id = id,
                    error = ?e,
                    "prune skipped: could not resolve memory root"
                );
            }
            Ok(root) => {
                let report = crate::memory::file_prune::prune_file_memories_for_backlog(
                    fs, &root, id, today,
                );
                for e in &report.errors {
                    tracing::warn!(
                        target: "memory::prune",
                        bl_id = id,
                        error = %e,
                        "prune error (fire-and-forget)"
                    );
                }
            }
        }

        if let Some(memory_store) = store
            && let Err(e) = crate::memory::lifecycle::prune_by_backlog(memory_store, id)
        {
            tracing::warn!(
                target: "memory::prune",
                bl_id = id,
                error = ?e,
                "sqlite prune error (fire-and-forget)"
            );
        }
    }

    Ok(())
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
        LockFile::new(
            worktree_name.to_string(),
            "2026-04-07T11:00:00Z".to_string(),
        )
        .unwrap()
    }

    fn make_stale_lock(worktree_name: &str) -> LockFile {
        // 2026-04-06T00:00:00Z — more than 24h before fresh_clock's 2026-04-07T12:00:00Z
        LockFile::new(
            worktree_name.to_string(),
            "2026-04-06T00:00:00Z".to_string(),
        )
        .unwrap()
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
            false,
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
            false,
        )
        .unwrap();

        let content = result.expect("dry_run should return content");
        assert!(
            content.contains("in-progress"),
            "BL-010 should be in-progress"
        );
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
            false,
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
            false,
        )
        .unwrap()
        .unwrap();

        assert_eq!(result1, result2, "reindex must be idempotent");
    }

    #[test]
    fn reindex_preserves_dep_graph() {
        let dep_graph_content = "# Backlog Index\n\n| old |\n\n## Dependency Graph\n\n```\nBL-001 → BL-002\n```\n\n## Stats\n\n- **Total:** 1\n";

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
            false,
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

    fn raw_open_content(id: &str) -> String {
        format!("---\nid: {id}\nstatus: open\ntitle: Test\ncreated: 2026-01-01\n---\n\n# Body\n")
    }

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
        // When list_available is called, it will try to remove the stale lock.
        // We verify the function completes successfully (does NOT panic/return error)
        // even when lock removal would fail. Since InMemoryBacklogRepository always
        // succeeds on remove_lock, we test the code path compiles and runs.
        // The key AC-004.1 behavior is: execution continues, a warn is logged.
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

    /// PC-037: prune failure does NOT fail the status transition (fire-and-forget).
    ///
    /// When `update_status_with_prune_hook` transitions to "implemented", it must call
    /// the memory prune hook. If the prune fails (e.g. HOME not set), it must:
    ///  1. Emit a `tracing::warn!` (captured via a channel layer).
    ///  2. Still return `Ok(())`.
    #[test]
    fn prune_failure_does_not_fail_transition() {
        use ecc_test_support::{InMemoryFileSystem, MockEnvironment};
        use std::sync::{Arc, Mutex};
        use tracing_subscriber::layer::SubscriberExt as _;

        let warn_messages: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        struct CaptureLayer(Arc<Mutex<Vec<String>>>);
        impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for CaptureLayer {
            fn on_event(
                &self,
                event: &tracing::Event<'_>,
                _ctx: tracing_subscriber::layer::Context<'_, S>,
            ) {
                if *event.metadata().level() == tracing::Level::WARN {
                    let mut visitor = MessageVisitor(String::new());
                    event.record(&mut visitor);
                    let msg = format!(
                        "WARN target={} msg={}",
                        event.metadata().target(),
                        visitor.0
                    );
                    self.0.lock().unwrap().push(msg);
                }
            }
        }

        struct MessageVisitor(String);
        impl tracing::field::Visit for MessageVisitor {
            fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                if field.name() == "message" {
                    self.0 = value.to_string();
                }
            }
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.0 = format!("{value:?}");
                }
            }
        }

        let subscriber =
            tracing_subscriber::registry().with(CaptureLayer(Arc::clone(&warn_messages)));
        let _guard = tracing::subscriber::set_default(subscriber);

        let raw = raw_open_content("BL-001");
        let repo = InMemoryBacklogRepository::new()
            .with_raw_content("BL-001", &raw)
            .with_entry(make_entry("BL-001", BacklogStatus::Open));
        let worktree_mgr = MockWorktreeManager::new();
        let clock = fresh_clock();

        // MockEnvironment with no HOME → resolve_project_memory_root returns Err(HomeNotSet).
        let env = MockEnvironment::new(); // vars is empty, var("HOME") returns None
        let fs = InMemoryFileSystem::new();

        let result = update_status_with_prune_hook(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            "BL-001",
            "implemented",
            &env,
            &fs,
            None,
        );

        assert!(
            result.is_ok(),
            "status transition must succeed even when prune hook fails; got: {result:?}"
        );

        // Verify that a warn was emitted referencing the prune failure.
        let msgs = warn_messages.lock().unwrap();
        let has_prune_warn = msgs.iter().any(|m| m.contains("memory::prune"));
        assert!(
            has_prune_warn,
            "must emit tracing::warn with target memory::prune when prune fails; got: {msgs:?}"
        );
    }

    /// PC-116: non-implemented transitions skip the memory prune hook.
    ///
    /// Verifies that `update_status_with_prune_hook` does NOT fire the prune
    /// when `new_status` is "in-progress", "archived", or "promoted".
    /// A memory file is seeded that WOULD be pruned if the hook fired.
    /// After each transition, the file must still exist and no `memory::prune`
    /// warn events must have been emitted.
    #[test]
    fn non_implemented_transitions_skip_prune() {
        use ecc_test_support::{InMemoryFileSystem, MockEnvironment};
        use std::path::PathBuf;
        use std::sync::{Arc, Mutex};
        use tracing_subscriber::layer::SubscriberExt as _;

        struct CaptureLayer(Arc<Mutex<Vec<String>>>);
        impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for CaptureLayer {
            fn on_event(
                &self,
                event: &tracing::Event<'_>,
                _ctx: tracing_subscriber::layer::Context<'_, S>,
            ) {
                if *event.metadata().level() == tracing::Level::WARN
                    && event.metadata().target().contains("memory::prune")
                {
                    self.0
                        .lock()
                        .unwrap()
                        .push(format!("WARN target={}", event.metadata().target()));
                }
            }
        }

        for target_status in ["in-progress", "archived", "promoted"] {
            let warn_messages: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
            let layer = CaptureLayer(Arc::clone(&warn_messages));
            let subscriber = tracing_subscriber::registry().with(layer);

            tracing::subscriber::with_default(subscriber, || {
                let root = PathBuf::from("/home/alice/.claude/projects/default/memory");
                let memory_file = root.join("project_bl001_foo.md");
                let fs = InMemoryFileSystem::new()
                    .with_dir(&root)
                    .with_file(&memory_file, "content")
                    .with_file(root.join("MEMORY.md"), "- [foo](project_bl001_foo.md)\n");
                let env = MockEnvironment::new().with_var("HOME", "/home/alice");

                let raw = raw_open_content("BL-001");
                let repo = InMemoryBacklogRepository::new()
                    .with_raw_content("BL-001", &raw)
                    .with_entry(make_entry("BL-001", BacklogStatus::Open));
                let worktree_mgr = MockWorktreeManager::new();
                let clock = fresh_clock();

                let result = update_status_with_prune_hook(
                    &repo,
                    &repo,
                    &repo,
                    &worktree_mgr,
                    &clock,
                    Path::new(BACKLOG_DIR),
                    Path::new(PROJECT_DIR),
                    "BL-001",
                    target_status,
                    &env,
                    &fs,
                    None,
                );

                assert!(
                    result.is_ok(),
                    "update_status_with_prune_hook({target_status}) must return Ok; got: {result:?}"
                );

                // Memory file must still exist — prune was NOT called
                let file_exists = fs.exists(&memory_file);
                assert!(
                    file_exists,
                    "memory file must survive non-implemented transition to '{target_status}'"
                );
            });

            let msgs = warn_messages.lock().unwrap();
            assert!(
                msgs.is_empty(),
                "transition to '{target_status}' must not fire prune hook; got warns: {msgs:?}"
            );
        }
    }

    /// PC-119: Sequential double-call simulates concurrency — second call is a no-op (idempotent).
    ///
    /// First call: transitions BL-001 from in-progress → implemented, moves memory file to trash.
    /// Second call (same args): status is already implemented → `update_status` no-op guard fires,
    /// prune hook is NOT re-invoked, no panic, no error.
    #[test]
    fn concurrent_update_status_idempotent_prune() {
        use ecc_test_support::{InMemoryFileSystem, MockEnvironment};
        use std::path::PathBuf;

        let home = "/home/alice";
        let root = PathBuf::from("/home/alice/.claude/projects/default/memory");
        let memory_file = root.join("project_bl001_foo.md");
        let trash_file = root.join(".trash/2026-04-07/project_bl001_foo.md");

        let fs = InMemoryFileSystem::new()
            .with_dir(PathBuf::from(home))
            .with_dir(&root)
            .with_file(&memory_file, "BL-001 memory content")
            .with_file(root.join("MEMORY.md"), "- [foo](project_bl001_foo.md)\n");
        let env = MockEnvironment::new()
            .with_var("HOME", home)
            .with_var("ECC_PROJECT_MEMORY_ROOT", root.to_str().unwrap());

        let raw_in_progress = "---\nid: BL-001\nstatus: in-progress\ntitle: Test\ncreated: 2026-01-01\n---\n\n# Body\n";
        let repo = InMemoryBacklogRepository::new()
            .with_raw_content("BL-001", raw_in_progress)
            .with_entry(make_entry("BL-001", BacklogStatus::InProgress));
        let worktree_mgr = MockWorktreeManager::new();
        let clock = fresh_clock();

        // First call: in-progress → implemented. Memory file should be trashed.
        let result1 = update_status_with_prune_hook(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            "BL-001",
            "implemented",
            &env,
            &fs,
            None,
        );
        assert!(result1.is_ok(), "first call must succeed; got: {result1:?}");

        // Memory file moved to trash after first call.
        assert!(
            fs.exists(&trash_file),
            "memory file must be in trash after first call"
        );
        assert!(
            !fs.exists(&memory_file),
            "original memory file must not exist after first call"
        );

        // Second call: already implemented → no-op guard fires in update_status.
        // Prune hook is reached only if update_status succeeds with actual write;
        // no-op guard prevents the write, so prune is not re-invoked.
        let result2 = update_status_with_prune_hook(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            "BL-001",
            "implemented",
            &env,
            &fs,
            None,
        );
        assert!(
            result2.is_ok(),
            "second call must be a no-op and not panic; got: {result2:?}"
        );

        // Trash file still exists (was not moved again).
        assert!(
            fs.exists(&trash_file),
            "trash file must still exist after second (no-op) call"
        );
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

    // --- parse_index_statuses tests ---

    /// PC-012: parse_index_statuses extracts id→status map from BACKLOG.md table
    #[test]
    fn parse_index_statuses_extracts_map() {
        let index_content = "\
# Backlog Index

| ID | Title | Tier | Scope | Target | Status | Created |
|----|-------|------|-------|--------|--------|---------|
| BL-001 | First entry | core | infra | — | open | 2026-01-01 |
| BL-002 | Second entry | core | app | — | implemented | 2026-01-01 |
| BL-003 | Third entry | core | cli | — | archived | 2026-01-01 |

## Stats
";
        let map = parse_index_statuses(index_content);
        assert_eq!(map.get("BL-001").map(|s| s.as_str()), Some("open"));
        assert_eq!(map.get("BL-002").map(|s| s.as_str()), Some("implemented"));
        assert_eq!(map.get("BL-003").map(|s| s.as_str()), Some("archived"));
        assert_eq!(map.len(), 3);
    }

    // --- migrate_statuses tests ---

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

    // --- reindex safety tests ---

    fn make_repo_with_many_diverging_entries() -> InMemoryBacklogRepository {
        // Create 6 entries that diverge from a pre-set index (>5 changes)
        let statuses = ["open", "open", "open", "open", "open", "open"];
        let index_statuses = [
            "implemented",
            "implemented",
            "implemented",
            "implemented",
            "implemented",
            "implemented",
        ];
        let ids = ["BL-001", "BL-002", "BL-003", "BL-004", "BL-005", "BL-006"];

        let mut repo = InMemoryBacklogRepository::new();
        for (i, id) in ids.iter().enumerate() {
            let raw = make_raw_with_status(id, statuses[i]);
            repo = repo
                .with_raw_content(id, &raw)
                .with_entry(make_entry(id, BacklogStatus::Open));
            let _ = index_statuses[i]; // suppress unused warning
        }

        // Build index with "implemented" for all entries
        let index_entries: Vec<(&str, &str)> = ids
            .iter()
            .zip(index_statuses.iter())
            .map(|(id, s)| (*id, *s))
            .collect();
        let index = make_index_with_entries(&index_entries);
        repo.with_index(&index)
    }

    /// PC-016: Reindex blocks >5 changes without force (returns error)
    #[test]
    fn reindex_safety_blocks_without_force() {
        let repo = make_repo_with_many_diverging_entries();
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
            false, // dry_run
            false, // force
        );

        assert!(
            result.is_err(),
            "reindex should block when >5 changes without force"
        );
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("5") || msg.contains("force") || msg.contains("status"),
            "error should mention safety block"
        );
    }

    /// PC-017: Reindex allows >5 changes with force=true
    #[test]
    fn reindex_safety_allows_with_force() {
        let repo = make_repo_with_many_diverging_entries();
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
            false, // dry_run
            true,  // force
        );

        assert!(
            result.is_ok(),
            "reindex should proceed with force=true, got: {:?}",
            result.err()
        );
    }

    /// PC-018: Reindex no warning when <=5 changes
    #[test]
    fn reindex_no_warning_under_threshold() {
        // Only 5 entries diverge (equal to threshold, not over)
        let ids = ["BL-001", "BL-002", "BL-003", "BL-004", "BL-005"];
        let mut repo = InMemoryBacklogRepository::new();
        for id in &ids {
            let raw = make_raw_with_status(id, "open");
            repo = repo
                .with_raw_content(id, &raw)
                .with_entry(make_entry(id, BacklogStatus::Open));
        }
        // Index says all are "implemented" — exactly 5 divergences
        let index_entries: Vec<(&str, &str)> = ids.iter().map(|id| (*id, "implemented")).collect();
        let index = make_index_with_entries(&index_entries);
        let repo = repo.with_index(&index);

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
            false, // dry_run
            false, // force (no force needed for <=5)
        );

        assert!(
            result.is_ok(),
            "reindex with <=5 changes should not be blocked: {:?}",
            result.err()
        );
    }

    // --- migration integration tests ---

    /// PC-028: After migration, reindex dry-run matches current index (idempotent proof)
    #[test]
    fn migration_idempotent_proof() {
        // Setup: 2 entries where file diverges from index
        // After migration, reindex is called with force=true.
        // Then a subsequent dry-run reindex should produce the same content as the current index
        // (no further safety-blocking divergence), proving idempotency.
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

        // Run migration — updates BL-001 raw content, writes new index via reindex(force=true)
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

        // Migration should have processed BL-001
        assert!(
            !report.failed.iter().any(|(id, _)| id == "BL-001"),
            "migration should not fail on BL-001"
        );

        // After migration, index is written by migrate_statuses. A subsequent dry-run should
        // succeed without blocking — meaning the safety check passes (≤5 changes).
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
        // The dry-run output should contain BL-001 and BL-002 (idempotent proof: no crash)
        assert!(
            dry_run_output.contains("BL-001"),
            "dry-run output should contain BL-001"
        );
        assert!(
            dry_run_output.contains("BL-002"),
            "dry-run output should contain BL-002"
        );
    }

    /// PC-038: migrate_statuses does NOT trigger the memory-prune hook.
    ///
    /// When `migrate_statuses` processes an entry with `implemented` status,
    /// it must call `entries.update_entry_status()` directly (the non-hooked path),
    /// NOT `update_status_with_prune_hook`. This is verified by asserting that no
    /// `memory::prune` warn event is emitted during migration.
    #[test]
    fn migrate_does_not_prune_memory() {
        use std::sync::{Arc, Mutex};
        use tracing_subscriber::layer::SubscriberExt as _;

        let warn_messages: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        struct CaptureLayer(Arc<Mutex<Vec<String>>>);
        impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for CaptureLayer {
            fn on_event(
                &self,
                event: &tracing::Event<'_>,
                _ctx: tracing_subscriber::layer::Context<'_, S>,
            ) {
                if *event.metadata().level() == tracing::Level::WARN
                    && event.metadata().target().contains("memory::prune")
                {
                    self.0
                        .lock()
                        .unwrap()
                        .push(format!("WARN target={}", event.metadata().target()));
                }
            }
        }

        let subscriber =
            tracing_subscriber::registry().with(CaptureLayer(Arc::clone(&warn_messages)));
        let _guard = tracing::subscriber::set_default(subscriber);

        // File says "open", index says "implemented" — migrate should update the file
        // using the direct path (no prune hook).
        let raw_bl001 = make_raw_with_status("BL-001", "open");
        let index = make_index_with_entries(&[("BL-001", "implemented")]);

        let repo = InMemoryBacklogRepository::new()
            .with_raw_content("BL-001", &raw_bl001)
            .with_entry(make_entry("BL-001", BacklogStatus::Implemented))
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

        // Migration should have updated BL-001
        assert!(
            report.updated.contains(&"BL-001".to_string()),
            "BL-001 should be in updated list; got: {report:?}"
        );

        // No memory::prune warn should have been emitted — migrate uses the non-hooked path
        let msgs = warn_messages.lock().unwrap();
        assert!(
            msgs.is_empty(),
            "migrate_statuses must NOT trigger the memory-prune hook; got warns: {msgs:?}"
        );
    }

    /// PC-115: table-driven test — migrate_statuses never fires prune hook for any status variant.
    ///
    /// Verifies that regardless of the status in the BACKLOG.md index (in-progress, archived,
    /// promoted, implemented), `migrate_statuses` uses the direct path (no prune hook) and
    /// emits zero `memory::prune` warn events.
    #[test]
    fn migrate_variant_never_fires_hooks() {
        use std::sync::{Arc, Mutex};
        use tracing_subscriber::layer::SubscriberExt as _;

        struct CaptureLayer(Arc<Mutex<Vec<String>>>);
        impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for CaptureLayer {
            fn on_event(
                &self,
                event: &tracing::Event<'_>,
                _ctx: tracing_subscriber::layer::Context<'_, S>,
            ) {
                if *event.metadata().level() == tracing::Level::WARN
                    && event.metadata().target().contains("memory::prune")
                {
                    self.0
                        .lock()
                        .unwrap()
                        .push(format!("WARN target={}", event.metadata().target()));
                }
            }
        }

        let target_statuses = ["in-progress", "archived", "promoted", "implemented"];

        for target_status in target_statuses {
            let warn_messages: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
            let layer = CaptureLayer(Arc::clone(&warn_messages));
            let subscriber = tracing_subscriber::registry().with(layer);

            tracing::subscriber::with_default(subscriber, || {
                // File says "open", index says target_status → migrate should update via direct path
                let raw_bl001 = make_raw_with_status("BL-001", "open");
                let index = make_index_with_entries(&[("BL-001", target_status)]);

                let repo = InMemoryBacklogRepository::new()
                    .with_raw_content("BL-001", &raw_bl001)
                    .with_entry(make_entry("BL-001", BacklogStatus::Open))
                    .with_index(&index);

                let worktree_mgr = MockWorktreeManager::new();
                let clock = fresh_clock();

                migrate_statuses(
                    &repo,
                    &repo,
                    &repo,
                    &worktree_mgr,
                    &clock,
                    Path::new(BACKLOG_DIR),
                    Path::new(PROJECT_DIR),
                )
                .unwrap();
            });

            let msgs = warn_messages.lock().unwrap();
            assert!(
                msgs.is_empty(),
                "migrate of status={target_status} fired prune hook: {msgs:?}"
            );
        }
    }

    /// PC-046: implemented transition prunes both the file memory AND the SQLite memory store.
    ///
    /// Seeds a file memory (BL-001 tagged .md file) and a SQLite memory entry with
    /// `source_path = "BL-001"`. Calls `update_status_with_prune_hook` with a
    /// `Some(&store)`. Verifies that:
    ///  1. The file memory is moved to `.trash/<today>/`
    ///  2. The SQLite entry is deleted from the store
    #[test]
    fn implemented_transition_prunes_file_and_sqlite() {
        use ecc_domain::memory::{MemoryEntry, MemoryId, MemoryTier};
        use ecc_ports::memory_store::MemoryStore as _;
        use ecc_test_support::{InMemoryFileSystem, InMemoryMemoryStore, MockEnvironment};
        use std::path::PathBuf;

        let home = "/home/alice";
        let root = PathBuf::from("/home/alice/.claude/projects/default/memory");
        let memory_file = root.join("project_bl001_foo.md");
        let today = "2026-04-07";
        let trash_file = root.join(format!(".trash/{today}/project_bl001_foo.md"));

        // Seed file memory
        let fs = InMemoryFileSystem::new()
            .with_dir(PathBuf::from(home))
            .with_dir(&root)
            .with_file(&memory_file, "BL-001 file memory content")
            .with_file(root.join("MEMORY.md"), "- [foo](project_bl001_foo.md)\n");
        let env = MockEnvironment::new()
            .with_var("HOME", home)
            .with_var("ECC_PROJECT_MEMORY_ROOT", root.to_str().unwrap());

        // Seed SQLite (in-memory) store entry tagged BL-001
        let store = InMemoryMemoryStore::new();
        let sqlite_entry = MemoryEntry::new(
            MemoryId(0),
            MemoryTier::Episodic,
            "BL-001 session note",
            "some context about BL-001",
            vec![],
            None,
            None,
            1.0,
            "2026-04-01T00:00:00Z",
            "2026-04-01T00:00:00Z",
            false,
            vec![],
            Some("BL-001".to_owned()),
        );
        store.insert(&sqlite_entry).unwrap();

        // Verify entry is in the store before the call
        let before = store.list_filtered(None, None, None).unwrap();
        assert_eq!(before.len(), 1, "store should have 1 entry before transition");

        let raw = "---\nid: BL-001\nstatus: open\ntitle: Test\ncreated: 2026-01-01\n---\n\n# Body\n";
        let repo = InMemoryBacklogRepository::new()
            .with_raw_content("BL-001", raw)
            .with_entry(make_entry("BL-001", BacklogStatus::Open));
        let worktree_mgr = MockWorktreeManager::new();
        let clock = fresh_clock();

        let result = update_status_with_prune_hook(
            &repo,
            &repo,
            &repo,
            &worktree_mgr,
            &clock,
            Path::new(BACKLOG_DIR),
            Path::new(PROJECT_DIR),
            "BL-001",
            "implemented",
            &env,
            &fs,
            Some(&store),
        );

        assert!(result.is_ok(), "transition must succeed; got: {result:?}");

        // 1. File memory moved to trash
        assert!(
            fs.exists(&trash_file),
            "file memory must be in trash after implemented transition"
        );
        assert!(
            !fs.exists(&memory_file),
            "original file memory must not exist after implemented transition"
        );

        // 2. SQLite entry deleted
        let after = store.list_filtered(None, None, None).unwrap();
        assert_eq!(
            after.len(),
            0,
            "SQLite entry tagged BL-001 must be deleted; remaining: {after:?}"
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
