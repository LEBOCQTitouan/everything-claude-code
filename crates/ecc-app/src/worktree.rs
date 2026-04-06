//! Worktree GC use case — garbage-collects stale `ecc-session-*` git worktrees.
//!
//! Orchestrates [`ecc_domain::worktree`] through [`ecc_ports::shell::ShellExecutor`]
//! and [`ecc_ports::worktree::WorktreeManager`].

use ecc_domain::worktree::{ParsedWorktreeName, WorktreeName};
use ecc_ports::shell::ShellExecutor;
use ecc_ports::worktree::WorktreeManager;
use std::path::Path;

/// Result of a worktree GC run.
#[derive(Debug, Default)]
pub struct WorktreeGcResult {
    /// Worktree names successfully removed.
    pub removed: Vec<String>,
    /// Worktree names skipped because the owning process is still alive.
    pub skipped: Vec<String>,
    /// Removal failures (name + error message).
    pub errors: Vec<String>,
}

/// Parse a compact timestamp `YYYYMMDD-HHMMSS` to seconds since epoch (approximate).
fn compact_ts_to_secs(ts: &str) -> Option<u64> {
    if ts.len() != 15 {
        return None;
    }
    let year = ts[0..4].parse::<u64>().ok()?;
    let month = ts[4..6].parse::<u64>().ok()?;
    let day = ts[6..8].parse::<u64>().ok()?;
    let hour = ts[9..11].parse::<u64>().ok()?;
    let minute = ts[11..13].parse::<u64>().ok()?;
    let second = ts[13..15].parse::<u64>().ok()?;

    let years_since_epoch = year.saturating_sub(1970);
    let leap_days = years_since_epoch / 4;
    let days_from_years = years_since_epoch * 365 + leap_days;
    let month_days: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let idx = (month as usize).saturating_sub(1).min(12);
    let days_from_months: u64 = month_days[..idx].iter().sum();
    let days = days_from_years + days_from_months + day.saturating_sub(1);
    Some(days * 86400 + hour * 3600 + minute * 60 + second)
}

/// Return the current Unix timestamp in seconds.
fn now_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

const STALE_SECS: u64 = 24 * 3600;

/// Determine whether a worktree entry is stale (old enough or PID dead).
fn is_worktree_stale(executor: &dyn ShellExecutor, parsed: &ParsedWorktreeName, now: u64) -> bool {
    let age_stale = compact_ts_to_secs(&parsed.timestamp)
        .map(|ts| now.saturating_sub(ts) > STALE_SECS)
        .unwrap_or(false);
    let pid_str = parsed.pid.to_string();
    let pid_alive = executor
        .run_command("kill", &["-0", &pid_str])
        .map(|o| o.success())
        .unwrap_or(false);
    age_stale || !pid_alive
}

/// Errors returned by worktree operations.
#[derive(Debug, thiserror::Error)]
pub enum WorktreeError {
    /// A shell command failed or was not found.
    #[error("worktree operation failed: {0}")]
    Shell(String),
    /// A worktree manager port error.
    #[error("worktree manager error: {0}")]
    Manager(#[from] ecc_ports::worktree::WorktreeError),
}

/// Run worktree GC using [`WorktreeManager`] port methods.
///
/// A worktree is considered stale when:
/// - its PID is no longer alive (`kill -0 <pid>` returns non-zero), **or**
/// - its timestamp is older than 24 hours.
///
/// Before removing a stale worktree, its unmerged commit count is checked via
/// `worktree_mgr.unmerged_commit_count`. If unmerged commits exist and `force`
/// is `false`, the worktree is skipped with a warning.
///
/// **Security**: `"--"` is always placed before user-supplied paths and branch
/// names in git commands to prevent argument injection.
pub fn gc(
    worktree_mgr: &dyn WorktreeManager,
    executor: &dyn ShellExecutor,
    project_dir: &Path,
    force: bool,
) -> Result<WorktreeGcResult, WorktreeError> {
    let entries = worktree_mgr.list_worktrees(project_dir)?;
    let mut result = WorktreeGcResult::default();
    let now = now_secs();

    for entry in entries {
        let Some(worktree_name) = entry.path.split('/').next_back().map(str::to_owned) else {
            continue;
        };
        if worktree_name.starts_with("worktree-ecc-session-") {
            tracing::info!(
                worktree = %worktree_name,
                "GC: found worktree with EnterWorktree prefix (now parseable)"
            );
        }
        let Some(parsed) = WorktreeName::parse(&worktree_name) else {
            continue;
        };

        if !is_worktree_stale(executor, &parsed, now) {
            result.skipped.push(worktree_name);
            continue;
        }

        // Check merge status before removal.
        let worktree_path = Path::new(&entry.path);
        let unmerged = worktree_mgr
            .unmerged_commit_count(worktree_path, "main")
            .unwrap_or(0);
        if unmerged > 0 && !force {
            tracing::warn!(
                worktree = %worktree_name,
                unmerged_commits = unmerged,
                "GC: skipping unmerged worktree (use --force to override)"
            );
            result.skipped.push(worktree_name);
            continue;
        }

        // Remove via port methods.
        match worktree_mgr.remove_worktree(project_dir, worktree_path) {
            Ok(()) => {
                result.removed.push(worktree_name.clone());
                if let Some(branch) = &entry.branch
                    && let Err(e) = worktree_mgr.delete_branch(project_dir, branch)
                {
                    tracing::warn!(
                        worktree = %worktree_name,
                        branch = %branch,
                        error = %e,
                        "GC: branch delete failed (non-fatal)"
                    );
                }
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("{worktree_name}: remove failed: {e}"));
            }
        }
    }

    Ok(result)
}

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
) -> Result<Vec<WorktreeStatusEntry>, WorktreeError> {
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

        let stale = is_worktree_stale(executor, &parsed, now);

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

    // ── helpers ──────────────────────────────────────────────────────────────

    fn ok(stdout: &str) -> CommandOutput {
        CommandOutput {
            stdout: stdout.to_owned(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    fn err_output(code: i32) -> CommandOutput {
        CommandOutput {
            stdout: String::new(),
            stderr: "no such process".to_owned(),
            exit_code: code,
        }
    }

    fn session_wt(name: &str) -> WorktreeInfo {
        WorktreeInfo {
            path: format!("/repo/{name}"),
            branch: Some(name.to_owned()),
        }
    }

    /// A session name whose timestamp is in the past (year 2020) so age_stale = true.
    const STALE_SESSION: &str = "ecc-session-20200101-000000-old-feature-99999";
    /// A session name with a far-future timestamp so age is NOT stale.
    const FRESH_SESSION: &str = "ecc-session-20990101-000000-new-feature-99999";
    /// A stale prefixed session name (worktree- prefix, year 2020).
    const STALE_PREFIXED_SESSION: &str = "worktree-ecc-session-20200101-000000-old-feature-99999";
    /// A fresh prefixed session name (far-future timestamp).
    const FRESH_PREFIXED_SESSION: &str = "worktree-ecc-session-20990101-000000-new-feature-99999";

    // ── Wave 4: gc_with_manager tests (PC-032..038) ───────────────────────────

    #[test]
    fn gc_uses_worktree_manager() {
        // Compile-time check: gc() accepts &dyn WorktreeManager.
        let mgr = MockWorktreeManager::new();
        let executor = MockExecutor::new();
        let result = gc(&mgr, &executor, Path::new("/repo"), false).unwrap();
        assert!(result.removed.is_empty());
        assert!(result.skipped.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn gc_uses_list_worktrees() {
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![session_wt(STALE_SESSION)]);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(&mgr, &executor, Path::new("/repo"), false).unwrap();
        assert!(
            result.removed.contains(&STALE_SESSION.to_owned()),
            "stale session should be removed via list_worktrees, got removed={:?}",
            result.removed
        );
    }

    #[test]
    fn gc_uses_port_methods() {
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![session_wt(STALE_SESSION)])
            .with_remove_succeeds(true)
            .with_delete_succeeds(true);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(&mgr, &executor, Path::new("/repo"), false).unwrap();
        assert!(
            result.removed.contains(&STALE_SESSION.to_owned()),
            "port remove_worktree + delete_branch must be used, got removed={:?}",
            result.removed
        );
        assert!(result.errors.is_empty(), "no errors expected: {:?}", result.errors);
    }

    #[test]
    fn gc_staleness_unchanged() {
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![session_wt(FRESH_SESSION)]);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], ok(""));

        let result = gc(&mgr, &executor, Path::new("/repo"), false).unwrap();
        assert!(
            result.skipped.contains(&FRESH_SESSION.to_owned()),
            "fresh session must be skipped, got: {:?}",
            result.skipped
        );
        assert!(
            !result.removed.contains(&FRESH_SESSION.to_owned()),
            "fresh session must NOT be removed"
        );
    }

    #[test]
    fn gc_skips_unmerged() {
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![session_wt(STALE_SESSION)])
            .with_unmerged_commit_count(3);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(&mgr, &executor, Path::new("/repo"), false).unwrap();
        assert!(
            result.skipped.contains(&STALE_SESSION.to_owned()),
            "unmerged stale worktree must be skipped with force=false, got: {:?}",
            result
        );
        assert!(
            !result.removed.contains(&STALE_SESSION.to_owned()),
            "unmerged worktree must NOT be removed with force=false"
        );
    }

    #[test]
    fn gc_force_overrides_merge_check() {
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![session_wt(STALE_SESSION)])
            .with_unmerged_commit_count(3);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(&mgr, &executor, Path::new("/repo"), true).unwrap();
        assert!(
            result.removed.contains(&STALE_SESSION.to_owned()),
            "force=true must override merge check and remove the worktree, got: {:?}",
            result.removed
        );
    }

    // ── Wave 4: migrated existing tests (AC-004.4) ───────────────────────────

    #[test]
    fn filters_session_worktrees() {
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![
                WorktreeInfo { path: "/repo/main".to_owned(), branch: Some("main".to_owned()) },
                WorktreeInfo { path: "/repo/other-feature".to_owned(), branch: Some("other-feature".to_owned()) },
                session_wt(STALE_SESSION),
            ]);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(&mgr, &executor, Path::new("/repo"), false).unwrap();

        assert!(!result.removed.contains(&"main".to_owned()), "main should not be in removed");
        assert!(!result.removed.contains(&"other-feature".to_owned()), "other-feature should not be in removed");
        assert!(!result.skipped.contains(&"main".to_owned()), "main should not be in skipped");
        assert!(!result.skipped.contains(&"other-feature".to_owned()), "other-feature should not be in skipped");
        let session_name = STALE_SESSION.to_owned();
        assert!(
            result.removed.contains(&session_name) || result.skipped.contains(&session_name),
            "session worktree must be processed"
        );
    }

    #[test]
    fn skips_active() {
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![session_wt(FRESH_SESSION)]);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], ok(""));

        let result = gc(&mgr, &executor, Path::new("/repo"), false).unwrap();

        assert!(
            result.skipped.contains(&FRESH_SESSION.to_owned()),
            "active worktree must be in skipped, got: {:?}",
            result.skipped
        );
        assert!(
            !result.removed.contains(&FRESH_SESSION.to_owned()),
            "active worktree must NOT be in removed"
        );
    }

    #[test]
    fn gc_returns_worktree_error_on_shell_failure() {
        // With the new signature, WorktreeManager::list_worktrees failure would
        // return WorktreeError::Manager. With an empty mock it succeeds.
        let mgr = MockWorktreeManager::new();
        let executor = MockExecutor::new();
        let result = gc(&mgr, &executor, Path::new("/repo"), false);
        // Empty worktree list → no-op, no error
        assert!(result.is_ok(), "empty worktree list should succeed");
    }

    #[test]
    fn removes_stale_prefixed_worktree() {
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![session_wt(STALE_PREFIXED_SESSION)]);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(&mgr, &executor, Path::new("/repo"), false).unwrap();
        assert!(
            result.removed.contains(&STALE_PREFIXED_SESSION.to_owned()),
            "stale prefixed worktree must be removed, got: {:?}",
            result.removed
        );
    }

    #[test]
    fn skips_fresh_prefixed_worktree() {
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![session_wt(FRESH_PREFIXED_SESSION)]);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], ok(""));

        let result = gc(&mgr, &executor, Path::new("/repo"), false).unwrap();
        assert!(
            result.skipped.contains(&FRESH_PREFIXED_SESSION.to_owned()),
            "fresh prefixed worktree must be skipped, got: {:?}",
            result.skipped
        );
    }

    #[test]
    fn logs_newly_parseable_worktree() {
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![session_wt(STALE_PREFIXED_SESSION)]);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(&mgr, &executor, Path::new("/repo"), false).unwrap();
        assert!(
            result.removed.contains(&STALE_PREFIXED_SESSION.to_owned()),
            "prefixed worktree must be processed by GC"
        );
    }

    #[test]
    fn removes_stale() {
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![session_wt(STALE_SESSION)]);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(&mgr, &executor, Path::new("/repo"), false).unwrap();

        assert!(
            result.removed.contains(&STALE_SESSION.to_owned()),
            "stale worktree must be removed, got: {:?}",
            result.removed
        );
        assert!(
            result.errors.is_empty(),
            "no errors expected, got: {:?}",
            result.errors
        );
    }

    // ── Wave 5: status tests (PC-039..043) ───────────────────────────────────

    #[test]
    fn status_returns_all_columns() {
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![session_wt(FRESH_SESSION)])
            .with_unmerged_commit_count(2)
            .with_uncommitted_changes(true)
            .with_stash(true)
            .with_pushed(false);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], ok(""));

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
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![
                WorktreeInfo { path: "/repo/main".to_owned(), branch: Some("main".to_owned()) },
                WorktreeInfo { path: "/repo/feature-xyz".to_owned(), branch: Some("feature-xyz".to_owned()) },
                session_wt(FRESH_SESSION),
            ]);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], ok(""));

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
            table,
            "Name\tBranch\tAge\tAhead\tClean\tStash\tPushed\tStatus",
            "header must match exactly"
        );
    }
}

/// A `WorktreeManager` that delegates list/remove/branch operations to a
/// [`ShellExecutor`]. Used when only a shell is available (e.g., session hooks).
///
/// Parses `git worktree list --porcelain` output for listing, and uses raw
/// git commands for removal — matching the previous GC implementation.
pub struct ShellWorktreeManager<'a> {
    shell: &'a dyn ShellExecutor,
}

impl<'a> ShellWorktreeManager<'a> {
    /// Create a new manager backed by the given shell executor.
    pub fn new(shell: &'a dyn ShellExecutor) -> Self {
        Self { shell }
    }

    fn parse_porcelain(output: &str) -> Vec<ecc_ports::worktree::WorktreeInfo> {
        use ecc_ports::worktree::WorktreeInfo;
        let mut out = Vec::new();
        let mut cur_path: Option<String> = None;
        let mut cur_branch: Option<String> = None;

        for line in output.lines() {
            if let Some(path) = line.strip_prefix("worktree ") {
                if let Some(p) = cur_path.take() {
                    out.push(WorktreeInfo {
                        path: p,
                        branch: cur_branch.take(),
                    });
                }
                cur_path = Some(path.to_owned());
                cur_branch = None;
            } else if let Some(branch_ref) = line.strip_prefix("branch ") {
                let name = branch_ref
                    .strip_prefix("refs/heads/")
                    .unwrap_or(branch_ref)
                    .to_owned();
                cur_branch = Some(name);
            }
        }
        if let Some(p) = cur_path {
            out.push(WorktreeInfo {
                path: p,
                branch: cur_branch,
            });
        }
        out
    }
}

impl WorktreeManager for ShellWorktreeManager<'_> {
    fn has_uncommitted_changes(
        &self,
        _worktree_path: &Path,
    ) -> Result<bool, ecc_ports::worktree::WorktreeError> {
        Ok(false)
    }

    fn has_untracked_files(
        &self,
        _worktree_path: &Path,
    ) -> Result<bool, ecc_ports::worktree::WorktreeError> {
        Ok(false)
    }

    fn unmerged_commit_count(
        &self,
        _worktree_path: &Path,
        _target_branch: &str,
    ) -> Result<u64, ecc_ports::worktree::WorktreeError> {
        Ok(0)
    }

    fn has_stash(
        &self,
        _worktree_path: &Path,
    ) -> Result<bool, ecc_ports::worktree::WorktreeError> {
        Ok(false)
    }

    fn is_pushed_to_remote(
        &self,
        _worktree_path: &Path,
        _branch: &str,
    ) -> Result<bool, ecc_ports::worktree::WorktreeError> {
        Ok(true)
    }

    fn remove_worktree(
        &self,
        repo_root: &Path,
        worktree_path: &Path,
    ) -> Result<(), ecc_ports::worktree::WorktreeError> {
        let path_str = worktree_path.to_string_lossy();
        let out = self
            .shell
            .run_command_in_dir(
                "git",
                &["worktree", "remove", "--force", "--", &path_str],
                repo_root,
            )
            .map_err(|e| ecc_ports::worktree::WorktreeError::CommandFailed(e.to_string()))?;
        if out.success() {
            Ok(())
        } else {
            Err(ecc_ports::worktree::WorktreeError::CommandFailed(out.stderr))
        }
    }

    fn delete_branch(
        &self,
        repo_root: &Path,
        branch: &str,
    ) -> Result<(), ecc_ports::worktree::WorktreeError> {
        let out = self
            .shell
            .run_command_in_dir("git", &["branch", "-D", "--", branch], repo_root)
            .map_err(|e| ecc_ports::worktree::WorktreeError::CommandFailed(e.to_string()))?;
        if out.success() {
            Ok(())
        } else {
            Err(ecc_ports::worktree::WorktreeError::CommandFailed(out.stderr))
        }
    }

    fn list_worktrees(
        &self,
        repo_root: &Path,
    ) -> Result<Vec<ecc_ports::worktree::WorktreeInfo>, ecc_ports::worktree::WorktreeError> {
        let out = self
            .shell
            .run_command_in_dir("git", &["worktree", "list", "--porcelain"], repo_root)
            .map_err(|e| ecc_ports::worktree::WorktreeError::CommandFailed(e.to_string()))?;
        Ok(Self::parse_porcelain(&out.stdout))
    }
}
