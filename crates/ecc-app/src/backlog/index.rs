//! Index-related backlog use cases: parse_index_statuses and reindex.

use super::{collect_claimed_ids, extract_bl_num};
use ecc_domain::backlog::entry::{BacklogError, BacklogStatus};
use ecc_domain::backlog::index::{extract_dependency_graph, generate_index_table, generate_stats};
use ecc_ports::backlog::{BacklogEntryStore, BacklogIndexStore, BacklogLockStore};
use ecc_ports::clock::Clock;
use ecc_ports::worktree::WorktreeManager;
use std::collections::HashMap;
use std::path::Path;

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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::test_helpers::*;
    use ecc_domain::backlog::entry::BacklogStatus;
    use ecc_test_support::{InMemoryBacklogRepository, MockWorktreeManager};
    use ecc_ports::worktree::WorktreeInfo;
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
}
