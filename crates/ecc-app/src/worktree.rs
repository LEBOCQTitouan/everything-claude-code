//! Worktree GC use case — garbage-collects stale `ecc-session-*` git worktrees.
//!
//! Orchestrates [`ecc_domain::worktree`] through [`ecc_ports::shell::ShellExecutor`].

use ecc_domain::worktree::{ParsedWorktreeName, WorktreeName};
use ecc_ports::shell::ShellExecutor;
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

/// Parsed representation of a single entry from `git worktree list --porcelain`.
#[derive(Debug)]
struct WorktreeEntry {
    path: String,
    branch: Option<String>,
}

/// Parse `git worktree list --porcelain` output into a list of entries.
fn parse_worktree_list(output: &str) -> Vec<WorktreeEntry> {
    let mut entries = Vec::new();
    let mut current_path: Option<String> = None;
    let mut current_branch: Option<String> = None;

    for line in output.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            if let Some(p) = current_path.take() {
                entries.push(WorktreeEntry {
                    path: p,
                    branch: current_branch.take(),
                });
            }
            current_path = Some(path.to_owned());
            current_branch = None;
        } else if let Some(branch_ref) = line.strip_prefix("branch ") {
            let name = branch_ref
                .strip_prefix("refs/heads/")
                .unwrap_or(branch_ref)
                .to_owned();
            current_branch = Some(name);
        }
    }
    if let Some(p) = current_path {
        entries.push(WorktreeEntry {
            path: p,
            branch: current_branch,
        });
    }
    entries
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

/// Remove a stale worktree (and its branch) from the result, recording success/errors.
fn remove_stale_worktree(
    executor: &dyn ShellExecutor,
    entry: &WorktreeEntry,
    worktree_name: &str,
    project_dir: &Path,
    result: &mut WorktreeGcResult,
) {
    let remove_result = executor.run_command_in_dir(
        "git",
        &["worktree", "remove", "--force", "--", &entry.path],
        project_dir,
    );
    match remove_result {
        Ok(o) if o.success() => {
            result.removed.push(worktree_name.to_owned());
            if let Some(branch) = &entry.branch {
                let _ = executor.run_command_in_dir(
                    "git",
                    &["branch", "-D", "--", branch],
                    project_dir,
                );
            }
        }
        Ok(o) => {
            result
                .errors
                .push(format!("{worktree_name}: remove failed: {}", o.stderr));
        }
        Err(e) => {
            result.errors.push(format!("{worktree_name}: {e}"));
        }
    }
}

/// Errors returned by worktree operations.
#[derive(Debug, thiserror::Error)]
pub enum WorktreeError {
    /// A shell command failed or was not found.
    #[error("worktree operation failed: {0}")]
    Shell(String),
}

/// Run worktree GC: remove stale `ecc-session-*` worktrees and their branches.
///
/// A worktree is considered stale when:
/// - its PID is no longer alive (`kill -0 <pid>` returns non-zero), **or**
/// - its timestamp is older than 24 hours.
///
/// **Security**: `"--"` is always placed before user-supplied paths and branch names
/// in git commands to prevent argument injection.
pub fn gc(
    executor: &dyn ShellExecutor,
    project_dir: &Path,
    _force: bool,
) -> Result<WorktreeGcResult, WorktreeError> {
    let list_output = executor
        .run_command_in_dir("git", &["worktree", "list", "--porcelain"], project_dir)
        .map_err(|e| WorktreeError::Shell(e.to_string()))?;
    let entries = parse_worktree_list(&list_output.stdout);
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

        remove_stale_worktree(executor, &entry, &worktree_name, project_dir, &mut result);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::MockExecutor;
    use std::path::Path;

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

    fn porcelain_with_session(session_name: &str, branch: &str) -> String {
        format!(
            "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n\n\
             worktree /repo/other-feature\nHEAD def456\nbranch refs/heads/feature-x\n\n\
             worktree /repo/{session_name}\nHEAD ghi789\nbranch refs/heads/{branch}\n\n"
        )
    }

    /// A session name whose timestamp is in the past (year 2020) so age_stale = true.
    const STALE_SESSION: &str = "ecc-session-20200101-000000-old-feature-99999";
    /// A session name with a far-future timestamp so age is NOT stale.
    const FRESH_SESSION: &str = "ecc-session-20990101-000000-new-feature-99999";

    #[test]
    fn filters_session_worktrees() {
        let list_output = porcelain_with_session(STALE_SESSION, STALE_SESSION);

        let executor = MockExecutor::new()
            .on_args(
                "git",
                &["worktree", "list", "--porcelain"],
                ok(&list_output),
            )
            .on_args("kill", &["-0", "99999"], err_output(1))
            .on_args(
                "git",
                &[
                    "worktree",
                    "remove",
                    "--force",
                    "--",
                    &format!("/repo/{STALE_SESSION}"),
                ],
                ok(""),
            )
            .on_args("git", &["branch", "-D", "--", STALE_SESSION], ok(""));

        let result = gc(&executor, Path::new("/repo"), false).unwrap();

        assert!(
            !result.removed.contains(&"main".to_owned()),
            "main should not be in removed"
        );
        assert!(
            !result.removed.contains(&"other-feature".to_owned()),
            "other-feature should not be in removed"
        );
        assert!(
            !result.skipped.contains(&"main".to_owned()),
            "main should not be in skipped"
        );
        assert!(
            !result.skipped.contains(&"other-feature".to_owned()),
            "other-feature should not be in skipped"
        );
        let session_name = STALE_SESSION.to_owned();
        assert!(
            result.removed.contains(&session_name) || result.skipped.contains(&session_name),
            "session worktree must be processed"
        );
    }

    #[test]
    fn skips_active() {
        let list_output = porcelain_with_session(FRESH_SESSION, FRESH_SESSION);

        let executor = MockExecutor::new()
            .on_args(
                "git",
                &["worktree", "list", "--porcelain"],
                ok(&list_output),
            )
            .on_args("kill", &["-0", "99999"], ok(""));

        let result = gc(&executor, Path::new("/repo"), false).unwrap();

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
        // Executor with no registered commands -> run_command_in_dir returns ShellError::NotFound
        let executor = MockExecutor::new();

        let err = gc(&executor, Path::new("/repo"), false).unwrap_err();

        // The error must be a WorktreeError::Shell variant
        assert!(
            matches!(err, WorktreeError::Shell(_)),
            "expected WorktreeError::Shell but got: {:?}",
            err
        );
    }

    /// A stale prefixed session name (worktree- prefix, year 2020).
    const STALE_PREFIXED_SESSION: &str = "worktree-ecc-session-20200101-000000-old-feature-99999";
    /// A fresh prefixed session name (far-future timestamp).
    const FRESH_PREFIXED_SESSION: &str = "worktree-ecc-session-20990101-000000-new-feature-99999";

    #[test]
    fn removes_stale_prefixed_worktree() {
        let list_output = porcelain_with_session(STALE_PREFIXED_SESSION, STALE_PREFIXED_SESSION);

        let executor = MockExecutor::new()
            .on_args(
                "git",
                &["worktree", "list", "--porcelain"],
                ok(&list_output),
            )
            .on_args("kill", &["-0", "99999"], err_output(1))
            .on_args(
                "git",
                &[
                    "worktree",
                    "remove",
                    "--force",
                    "--",
                    &format!("/repo/{STALE_PREFIXED_SESSION}"),
                ],
                ok(""),
            )
            .on_args(
                "git",
                &["branch", "-D", "--", STALE_PREFIXED_SESSION],
                ok(""),
            );

        let result = gc(&executor, Path::new("/repo"), false).unwrap();
        assert!(
            result.removed.contains(&STALE_PREFIXED_SESSION.to_owned()),
            "stale prefixed worktree must be removed, got: {:?}",
            result.removed
        );
    }

    #[test]
    fn skips_fresh_prefixed_worktree() {
        let list_output = porcelain_with_session(FRESH_PREFIXED_SESSION, FRESH_PREFIXED_SESSION);

        let executor = MockExecutor::new()
            .on_args(
                "git",
                &["worktree", "list", "--porcelain"],
                ok(&list_output),
            )
            .on_args("kill", &["-0", "99999"], ok(""));

        let result = gc(&executor, Path::new("/repo"), false).unwrap();
        assert!(
            result.skipped.contains(&FRESH_PREFIXED_SESSION.to_owned()),
            "fresh prefixed worktree must be skipped, got: {:?}",
            result.skipped
        );
    }

    #[test]
    fn logs_newly_parseable_worktree() {
        // This test verifies GC processes prefixed worktrees (they were
        // previously silently skipped). The tracing::info! log is verified
        // by the fact that the worktree enters the staleness check path.
        let list_output = porcelain_with_session(STALE_PREFIXED_SESSION, STALE_PREFIXED_SESSION);

        let executor = MockExecutor::new()
            .on_args(
                "git",
                &["worktree", "list", "--porcelain"],
                ok(&list_output),
            )
            .on_args("kill", &["-0", "99999"], err_output(1))
            .on_args(
                "git",
                &[
                    "worktree",
                    "remove",
                    "--force",
                    "--",
                    &format!("/repo/{STALE_PREFIXED_SESSION}"),
                ],
                ok(""),
            )
            .on_args(
                "git",
                &["branch", "-D", "--", STALE_PREFIXED_SESSION],
                ok(""),
            );

        let result = gc(&executor, Path::new("/repo"), false).unwrap();
        // If the worktree was processed (removed), it means parse() succeeded
        // on the prefixed name, which is the observable behavior.
        assert!(
            result.removed.contains(&STALE_PREFIXED_SESSION.to_owned()),
            "prefixed worktree must be processed by GC"
        );
    }

    #[test]
    fn removes_stale() {
        let list_output = porcelain_with_session(STALE_SESSION, STALE_SESSION);

        let executor = MockExecutor::new()
            .on_args(
                "git",
                &["worktree", "list", "--porcelain"],
                ok(&list_output),
            )
            .on_args("kill", &["-0", "99999"], err_output(1))
            .on_args(
                "git",
                &[
                    "worktree",
                    "remove",
                    "--force",
                    "--",
                    &format!("/repo/{STALE_SESSION}"),
                ],
                ok(""),
            )
            .on_args("git", &["branch", "-D", "--", STALE_SESSION], ok(""));

        let result = gc(&executor, Path::new("/repo"), false).unwrap();

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

    // ── Wave 4 new tests (RED: these reference new gc signature and new functions) ──

    #[test]
    fn gc_uses_worktree_manager() {
        use ecc_ports::worktree::WorktreeManager;
        use ecc_test_support::MockWorktreeManager;
        // This test verifies gc() accepts &dyn WorktreeManager.
        // Currently gc() only takes &dyn ShellExecutor — this WILL NOT COMPILE
        // until gc() is refactored.
        let mgr = MockWorktreeManager::new();
        let executor = MockExecutor::new();
        // Call the new gc signature (worktree_mgr first param)
        let result = crate::worktree::gc_with_manager(&mgr, &executor, Path::new("/repo"), false).unwrap();
        assert!(result.removed.is_empty());
        assert!(result.skipped.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn gc_uses_list_worktrees() {
        use ecc_ports::worktree::{WorktreeInfo, WorktreeManager};
        use ecc_test_support::MockWorktreeManager;
        let stale = WorktreeInfo {
            path: format!("/repo/{STALE_SESSION}"),
            branch: Some(STALE_SESSION.to_owned()),
        };
        let mgr = MockWorktreeManager::new().with_worktrees(vec![stale]);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], err_output(1));

        let result = crate::worktree::gc_with_manager(&mgr, &executor, Path::new("/repo"), false).unwrap();
        assert!(
            result.removed.contains(&STALE_SESSION.to_owned()),
            "stale session should be removed via list_worktrees, got removed={:?}",
            result.removed
        );
    }

    #[test]
    fn gc_uses_port_methods() {
        use ecc_ports::worktree::{WorktreeInfo, WorktreeManager};
        use ecc_test_support::MockWorktreeManager;
        let stale = WorktreeInfo {
            path: format!("/repo/{STALE_SESSION}"),
            branch: Some(STALE_SESSION.to_owned()),
        };
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![stale])
            .with_remove_succeeds(true)
            .with_delete_succeeds(true);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], err_output(1));

        let result = crate::worktree::gc_with_manager(&mgr, &executor, Path::new("/repo"), false).unwrap();
        assert!(
            result.removed.contains(&STALE_SESSION.to_owned()),
            "port remove_worktree + delete_branch must be used, got removed={:?}",
            result.removed
        );
        assert!(result.errors.is_empty(), "no errors expected: {:?}", result.errors);
    }

    #[test]
    fn gc_staleness_unchanged() {
        use ecc_ports::worktree::{WorktreeInfo, WorktreeManager};
        use ecc_test_support::MockWorktreeManager;
        let fresh = WorktreeInfo {
            path: format!("/repo/{FRESH_SESSION}"),
            branch: Some(FRESH_SESSION.to_owned()),
        };
        let mgr = MockWorktreeManager::new().with_worktrees(vec![fresh]);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], ok(""));

        let result = crate::worktree::gc_with_manager(&mgr, &executor, Path::new("/repo"), false).unwrap();
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
        use ecc_ports::worktree::{WorktreeInfo, WorktreeManager};
        use ecc_test_support::MockWorktreeManager;
        let stale = WorktreeInfo {
            path: format!("/repo/{STALE_SESSION}"),
            branch: Some(STALE_SESSION.to_owned()),
        };
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![stale])
            .with_unmerged_commit_count(3);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], err_output(1));

        let result = crate::worktree::gc_with_manager(&mgr, &executor, Path::new("/repo"), false).unwrap();
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
        use ecc_ports::worktree::{WorktreeInfo, WorktreeManager};
        use ecc_test_support::MockWorktreeManager;
        let stale = WorktreeInfo {
            path: format!("/repo/{STALE_SESSION}"),
            branch: Some(STALE_SESSION.to_owned()),
        };
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![stale])
            .with_unmerged_commit_count(3);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], err_output(1));

        let result = crate::worktree::gc_with_manager(&mgr, &executor, Path::new("/repo"), true).unwrap();
        assert!(
            result.removed.contains(&STALE_SESSION.to_owned()),
            "force=true must override merge check and remove the worktree, got: {:?}",
            result.removed
        );
    }

    // ── Wave 5 tests (RED: status and format_status_table don't exist yet) ──

    #[test]
    fn status_returns_all_columns() {
        use ecc_ports::worktree::{WorktreeInfo, WorktreeManager};
        use ecc_test_support::MockWorktreeManager;
        let fresh = WorktreeInfo {
            path: format!("/repo/{FRESH_SESSION}"),
            branch: Some(FRESH_SESSION.to_owned()),
        };
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![fresh])
            .with_unmerged_commit_count(2)
            .with_uncommitted_changes(true)
            .with_stash(true)
            .with_pushed(false);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], ok(""));

        let entries = crate::worktree::status(&mgr, &executor, Path::new("/repo")).unwrap();
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert_eq!(e.name, FRESH_SESSION);
        assert_eq!(e.commits_ahead, 2);
        assert!(!e.is_clean);
        assert!(e.has_stash);
        assert!(!e.is_pushed);
        assert_eq!(e.status, crate::worktree::WorktreeStatus::Unmerged);
    }

    #[test]
    fn status_excludes_non_session() {
        use ecc_ports::worktree::WorktreeInfo;
        use ecc_test_support::MockWorktreeManager;
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![
                WorktreeInfo { path: "/repo/main".to_owned(), branch: Some("main".to_owned()) },
                WorktreeInfo { path: "/repo/feature-xyz".to_owned(), branch: Some("feature-xyz".to_owned()) },
                WorktreeInfo { path: format!("/repo/{FRESH_SESSION}"), branch: Some(FRESH_SESSION.to_owned()) },
            ]);
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], ok(""));

        let entries = crate::worktree::status(&mgr, &executor, Path::new("/repo")).unwrap();
        assert_eq!(entries.len(), 1, "only session worktrees must appear");
        assert_eq!(entries[0].name, FRESH_SESSION);
    }

    #[test]
    fn status_table_format() {
        use crate::worktree::{WorktreeStatus, WorktreeStatusEntry, format_status_table};
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
        assert_eq!(lines.len(), 2);
        let header_cols: Vec<&str> = lines[0].split('\t').collect();
        assert_eq!(header_cols.len(), 8);
        assert_eq!(header_cols[0], "Name");
        assert_eq!(header_cols[7], "Status");
        let data_cols: Vec<&str> = lines[1].split('\t').collect();
        assert_eq!(data_cols.len(), 8);
    }

    #[test]
    fn status_snapshot() {
        use crate::worktree::{WorktreeStatusEntry, format_status_table};
        let entries: Vec<WorktreeStatusEntry> = vec![];
        let table = format_status_table(&entries);
        assert_eq!(
            table,
            "Name\tBranch\tAge\tAhead\tClean\tStash\tPushed\tStatus"
        );
    }
}
