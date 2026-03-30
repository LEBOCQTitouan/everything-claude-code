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
                let _ = executor.run_command_in_dir("git", &["branch", "-D", "--", branch], project_dir);
            }
        }
        Ok(o) => {
            result.errors.push(format!("{worktree_name}: remove failed: {}", o.stderr));
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
) -> Result<WorktreeGcResult, anyhow::Error> {
    let list_output =
        executor.run_command_in_dir("git", &["worktree", "list", "--porcelain"], project_dir)?;
    let entries = parse_worktree_list(&list_output.stdout);
    let mut result = WorktreeGcResult::default();
    let now = now_secs();

    for entry in entries {
        let Some(worktree_name) = entry.path.split('/').next_back().map(str::to_owned) else {
            continue;
        };
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

        // The error must downcast to WorktreeError::Shell variant
        assert!(
            err.downcast_ref::<WorktreeError>().is_some(),
            "expected WorktreeError but got a different error type"
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
}
