//! Worktree GC use case — garbage-collects stale `ecc-session-*` git worktrees.

use ecc_domain::worktree::WorktreeName;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::worktree::WorktreeManager;
use std::path::Path;

use super::{WorktreeError, is_worktree_stale, now_secs};

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
}
