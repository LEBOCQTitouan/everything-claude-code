//! Worktree status use case — reports status of active `ecc-session-*` git worktrees.

use ecc_domain::worktree::WorktreeName;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::worktree::WorktreeManager;
use std::path::Path;

use super::{WorktreeGcError, compact_ts_to_secs, is_worktree_stale, now_secs};

/// A single row in the worktree status table.
#[derive(Debug, Clone)]
pub struct WorktreeStatusEntry {
    /// Session name (directory base name).
    pub name: String,
    /// Branch checked out in this worktree.
    pub branch: String,
    /// Age of the worktree in seconds (derived from timestamp in name).
    pub age_secs: u64,
    /// Number of commits ahead of main (unmerged).
    pub commits_ahead: u64,
    /// Whether the working tree is clean (no uncommitted or untracked changes).
    pub is_clean: bool,
    /// Whether the worktree has stashed changes.
    pub has_stash: bool,
    /// Whether the branch is fully pushed to remote.
    pub is_pushed: bool,
    /// Overall merge/staleness status.
    pub status: WorktreeStatus,
}

/// Merge/staleness classification for a worktree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorktreeStatus {
    /// All commits merged into main.
    Merged,
    /// Has commits not yet merged into main.
    Unmerged,
    /// Stale by age (> 24 h) and no unmerged commits.
    Stale,
}

/// Return status entries for all active `ecc-session-*` worktrees.
pub fn status(
    worktree_mgr: &dyn WorktreeManager,
    executor: &dyn ShellExecutor,
    project_dir: &Path,
) -> Result<Vec<WorktreeStatusEntry>, WorktreeGcError> {
    let entries = worktree_mgr.list_worktrees(project_dir)?;
    let now = now_secs();
    let mut out = Vec::new();

    for entry in entries {
        let Some(worktree_name) = entry.path.split('/').next_back().map(str::to_owned) else {
            continue;
        };
        // Only include session worktrees (parseable by WorktreeName).
        let Some(parsed) = WorktreeName::parse(&worktree_name) else {
            continue;
        };

        let worktree_path = Path::new(&entry.path);
        let commits_ahead = worktree_mgr
            .unmerged_commit_count(worktree_path, "main")
            .unwrap_or(0);
        let is_clean = !worktree_mgr
            .has_uncommitted_changes(worktree_path)
            .unwrap_or(false)
            && !worktree_mgr
                .has_untracked_files(worktree_path)
                .unwrap_or(false);
        let has_stash = worktree_mgr.has_stash(worktree_path).unwrap_or(false);
        let branch = entry.branch.clone().unwrap_or_default();
        let is_pushed = worktree_mgr
            .is_pushed_to_remote(worktree_path, &branch)
            .unwrap_or(false);

        let age_secs = compact_ts_to_secs(&parsed.timestamp)
            .map(|ts| now.saturating_sub(ts))
            .unwrap_or(0);

        let stale = is_worktree_stale(executor, &parsed, now, worktree_path);

        let wt_status = if commits_ahead > 0 {
            WorktreeStatus::Unmerged
        } else if stale {
            WorktreeStatus::Stale
        } else {
            WorktreeStatus::Merged
        };

        out.push(WorktreeStatusEntry {
            name: worktree_name,
            branch,
            age_secs,
            commits_ahead,
            is_clean,
            has_stash,
            is_pushed,
            status: wt_status,
        });
    }

    Ok(out)
}

/// Format a slice of [`WorktreeStatusEntry`] as a tab-separated table.
///
/// Returns a string with a header row followed by one row per entry.
/// Columns: Name, Branch, Age, Ahead, Clean, Stash, Pushed, Status
pub fn format_status_table(entries: &[WorktreeStatusEntry]) -> String {
    let header = "Name\tBranch\tAge\tAhead\tClean\tStash\tPushed\tStatus";
    let mut lines = vec![header.to_owned()];
    for e in entries {
        let age = format_age(e.age_secs);
        let status_str = match e.status {
            WorktreeStatus::Merged => "merged",
            WorktreeStatus::Unmerged => "unmerged",
            WorktreeStatus::Stale => "stale",
        };
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            e.name,
            e.branch,
            age,
            e.commits_ahead,
            if e.is_clean { "yes" } else { "no" },
            if e.has_stash { "yes" } else { "no" },
            if e.is_pushed { "yes" } else { "no" },
            status_str,
        ));
    }
    lines.join("\n")
}

/// Format age in seconds to a human-readable string.
fn format_age(secs: u64) -> String {
    if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::shell::CommandOutput;
    use ecc_ports::worktree::WorktreeInfo;
    use ecc_test_support::{MockExecutor, MockWorktreeManager};
    use std::path::Path;

    fn ok(stdout: &str) -> CommandOutput {
        CommandOutput {
            stdout: stdout.to_owned(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    fn session_wt(name: &str) -> WorktreeInfo {
        WorktreeInfo {
            path: format!("/repo/{name}"),
            branch: Some(name.to_owned()),
        }
    }

    const FRESH_SESSION: &str = "ecc-session-20990101-000000-new-feature-99999";

    // ── Wave 5: status tests (PC-039..043) ───────────────────────────────────

    #[test]
    fn status_returns_all_columns() {
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![session_wt(FRESH_SESSION)])
            .with_unmerged_commit_count(2)
            .with_uncommitted_changes(true)
            .with_stash(true)
            .with_pushed(false);
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], ok(""));

        let entries = status(&mgr, &executor, Path::new("/repo")).unwrap();
        assert_eq!(entries.len(), 1, "expected 1 status entry");

        let e = &entries[0];
        assert_eq!(e.name, FRESH_SESSION);
        assert_eq!(e.branch, FRESH_SESSION);
        assert_eq!(e.commits_ahead, 2);
        assert!(!e.is_clean, "has uncommitted changes → not clean");
        assert!(e.has_stash);
        assert!(!e.is_pushed);
        assert_eq!(e.status, WorktreeStatus::Unmerged);
    }

    #[test]
    fn status_excludes_non_session() {
        let mgr = MockWorktreeManager::new().with_worktrees(vec![
            WorktreeInfo {
                path: "/repo/main".to_owned(),
                branch: Some("main".to_owned()),
            },
            WorktreeInfo {
                path: "/repo/feature-xyz".to_owned(),
                branch: Some("feature-xyz".to_owned()),
            },
            session_wt(FRESH_SESSION),
        ]);
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], ok(""));

        let entries = status(&mgr, &executor, Path::new("/repo")).unwrap();
        assert_eq!(entries.len(), 1, "only session worktrees must appear");
        assert_eq!(entries[0].name, FRESH_SESSION);
    }

    #[test]
    fn status_table_format() {
        let entry = WorktreeStatusEntry {
            name: "ecc-session-20990101-000000-feat-123".to_owned(),
            branch: "feature-branch".to_owned(),
            age_secs: 3600,
            commits_ahead: 0,
            is_clean: true,
            has_stash: false,
            is_pushed: true,
            status: WorktreeStatus::Merged,
        };
        let table = format_status_table(&[entry]);
        let lines: Vec<&str> = table.lines().collect();
        assert_eq!(lines.len(), 2, "header + 1 data row");
        let header_cols: Vec<&str> = lines[0].split('\t').collect();
        assert_eq!(header_cols.len(), 8, "header must have 8 columns");
        assert_eq!(header_cols[0], "Name");
        assert_eq!(header_cols[7], "Status");
        let data_cols: Vec<&str> = lines[1].split('\t').collect();
        assert_eq!(data_cols.len(), 8, "data row must have 8 columns");
    }

    #[test]
    fn status_snapshot() {
        let entries: Vec<WorktreeStatusEntry> = vec![];
        let table = format_status_table(&entries);
        assert_eq!(
            table, "Name\tBranch\tAge\tAhead\tClean\tStash\tPushed\tStatus",
            "header must match exactly"
        );
    }
}
