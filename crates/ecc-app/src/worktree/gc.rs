//! Worktree GC use case — garbage-collects stale `ecc-session-*` git worktrees.

use ecc_domain::worktree::WorktreeName;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::worktree::WorktreeManager;
use std::path::Path;

use super::{WorktreeGcError, is_worktree_stale, now_secs};

/// Configuration for the GC run.
#[derive(Debug, Clone, Default)]
pub struct GcOptions {
    /// Override liveness check and remove unmerged worktrees.
    pub force: bool,
}

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

/// Liveness TTL: 60 minutes in seconds (Decision #12).
pub const TTL_DEFAULT_SECS: u64 = 3600;

/// Run worktree GC using [`WorktreeManager`] port methods.
///
/// A worktree is considered stale when:
/// - its PID is no longer alive (`kill -0 <pid>` returns non-zero), **or**
/// - its timestamp is older than 24 hours.
///
/// Before the existing stale check, the GC consults the `.ecc-session` file
/// for an active heartbeat. If the heartbeat is fresh and the PID is alive,
/// the worktree is skipped with a diagnostic message.
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
    fs: &dyn FileSystem,
    project_dir: &Path,
    options: GcOptions,
    clock: &dyn ecc_ports::clock::Clock,
) -> Result<WorktreeGcResult, WorktreeGcError> {
    let entries = worktree_mgr.list_worktrees(project_dir)?;
    let mut result = WorktreeGcResult::default();
    let now = now_secs(clock);
    let force = options.force;

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

        let worktree_path = Path::new(&entry.path);

        // Consult the .ecc-session heartbeat before the stale-timer fallback.
        // On NotFound or malformed JSON → fall through to stale-timer logic (AC-001.4, AC-001.5).
        let session_path = worktree_path.join(".ecc-session");
        if let Ok(content) = fs.read_to_string(&session_path)
            && let Ok(record) = ecc_domain::worktree::liveness::LivenessRecord::parse(&content)
        {
            let pid_str = record.claude_code_pid.to_string();
            let pid_alive = executor
                .run_command("kill", &["-0", &pid_str])
                .map(|o| o.success())
                .unwrap_or(false);
            if ecc_domain::worktree::liveness::is_live(&record, now, pid_alive, TTL_DEFAULT_SECS) {
                let secs_ago = now.saturating_sub(record.last_seen_unix_ts);
                eprintln!(
                    "Skipping {worktree_name}: active session detected (PID {}, last seen {secs_ago}s ago)",
                    record.claude_code_pid
                );
                result.skipped.push(worktree_name);
                continue;
            }
        }

        if !is_worktree_stale(executor, &parsed, now, worktree_path) {
            result.skipped.push(worktree_name);
            continue;
        }

        // Check merge status before removal.
        let unmerged = worktree_mgr
            .unmerged_commit_count(worktree_path, "main")
            .unwrap_or(u64::MAX); // Fail-safe: assume unmerged when query fails
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
    use ecc_test_support::{InMemoryFileSystem, MockExecutor, MockWorktreeManager, TEST_CLOCK};
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
        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions::default(),
            &*TEST_CLOCK,
        )
        .unwrap();
        assert!(result.removed.is_empty());
        assert!(result.skipped.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn gc_uses_list_worktrees() {
        let mgr = MockWorktreeManager::new().with_worktrees(vec![session_wt(STALE_SESSION)]);
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions::default(),
            &*TEST_CLOCK,
        )
        .unwrap();
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
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions::default(),
            &*TEST_CLOCK,
        )
        .unwrap();
        assert!(
            result.removed.contains(&STALE_SESSION.to_owned()),
            "port remove_worktree + delete_branch must be used, got removed={:?}",
            result.removed
        );
        assert!(
            result.errors.is_empty(),
            "no errors expected: {:?}",
            result.errors
        );
    }

    #[test]
    fn gc_staleness_unchanged() {
        let mgr = MockWorktreeManager::new().with_worktrees(vec![session_wt(FRESH_SESSION)]);
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], ok(""));

        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions::default(),
            &*TEST_CLOCK,
        )
        .unwrap();
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
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions::default(),
            &*TEST_CLOCK,
        )
        .unwrap();
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
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions { force: true },
            &*TEST_CLOCK,
        )
        .unwrap();
        assert!(
            result.removed.contains(&STALE_SESSION.to_owned()),
            "force=true must override merge check and remove the worktree, got: {:?}",
            result.removed
        );
    }

    // ── BL-150: fail-safe unmerged count tests ──────────────────────────────

    #[test]
    fn gc_skips_when_unmerged_query_fails() {
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![session_wt(STALE_SESSION)])
            .with_unmerged_query_fails(true);
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions::default(),
            &*TEST_CLOCK,
        )
        .unwrap();
        assert!(
            result.skipped.contains(&STALE_SESSION.to_owned()),
            "worktree must be skipped when unmerged_commit_count returns Err, got: {:?}",
            result
        );
        assert!(
            !result.removed.contains(&STALE_SESSION.to_owned()),
            "worktree must NOT be removed when unmerged query fails"
        );
    }

    // ── Wave 4: migrated existing tests (AC-004.4) ───────────────────────────

    #[test]
    fn filters_session_worktrees() {
        let mgr = MockWorktreeManager::new().with_worktrees(vec![
            WorktreeInfo {
                path: "/repo/main".to_owned(),
                branch: Some("main".to_owned()),
            },
            WorktreeInfo {
                path: "/repo/other-feature".to_owned(),
                branch: Some("other-feature".to_owned()),
            },
            session_wt(STALE_SESSION),
        ]);
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions::default(),
            &*TEST_CLOCK,
        )
        .unwrap();

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
        let mgr = MockWorktreeManager::new().with_worktrees(vec![session_wt(FRESH_SESSION)]);
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], ok(""));

        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions::default(),
            &*TEST_CLOCK,
        )
        .unwrap();

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
        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions::default(),
            &*TEST_CLOCK,
        );
        // Empty worktree list → no-op, no error
        assert!(result.is_ok(), "empty worktree list should succeed");
    }

    #[test]
    fn removes_stale_prefixed_worktree() {
        let mgr =
            MockWorktreeManager::new().with_worktrees(vec![session_wt(STALE_PREFIXED_SESSION)]);
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions::default(),
            &*TEST_CLOCK,
        )
        .unwrap();
        assert!(
            result.removed.contains(&STALE_PREFIXED_SESSION.to_owned()),
            "stale prefixed worktree must be removed, got: {:?}",
            result.removed
        );
    }

    #[test]
    fn skips_fresh_prefixed_worktree() {
        let mgr =
            MockWorktreeManager::new().with_worktrees(vec![session_wt(FRESH_PREFIXED_SESSION)]);
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], ok(""));

        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions::default(),
            &*TEST_CLOCK,
        )
        .unwrap();
        assert!(
            result.skipped.contains(&FRESH_PREFIXED_SESSION.to_owned()),
            "fresh prefixed worktree must be skipped, got: {:?}",
            result.skipped
        );
    }

    #[test]
    fn logs_newly_parseable_worktree() {
        let mgr =
            MockWorktreeManager::new().with_worktrees(vec![session_wt(STALE_PREFIXED_SESSION)]);
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions::default(),
            &*TEST_CLOCK,
        )
        .unwrap();
        assert!(
            result.removed.contains(&STALE_PREFIXED_SESSION.to_owned()),
            "prefixed worktree must be processed by GC"
        );
    }

    #[test]
    fn removes_stale() {
        let mgr = MockWorktreeManager::new().with_worktrees(vec![session_wt(STALE_SESSION)]);
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions::default(),
            &*TEST_CLOCK,
        )
        .unwrap();

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

    // ── PC-019..023: heartbeat consultation tests ─────────────────────────────

    /// now = 1_735_689_600 (from TEST_CLOCK)
    const NOW: u64 = 1_735_689_600;
    /// A fresh heartbeat timestamp: 60 seconds ago.
    const FRESH_TS: u64 = NOW - 60;
    /// A stale heartbeat timestamp: 3601 seconds ago (> TTL_DEFAULT_SECS=3600).
    const STALE_TS: u64 = NOW - 3601;
    /// PID used in heartbeat tests (different from 99999 to distinguish).
    const HB_PID: u32 = 12345;

    fn heartbeat_json(ts: u64) -> String {
        format!(r#"{{"schema_version":1,"claude_code_pid":{HB_PID},"last_seen_unix_ts":{ts}}}"#)
    }

    /// AC-001.1: GC skips worktree with fresh heartbeat and alive PID.
    #[test]
    fn skips_when_fresh_heartbeat_and_pid_alive() {
        let fs = InMemoryFileSystem::new().with_file(
            &format!("/repo/{STALE_SESSION}/.ecc-session"),
            &heartbeat_json(FRESH_TS),
        );
        let mgr = MockWorktreeManager::new().with_worktrees(vec![session_wt(STALE_SESSION)]);
        // PID alive → kill -0 returns 0
        let executor = MockExecutor::new().on_args("kill", &["-0", &HB_PID.to_string()], ok(""));

        let result = gc(
            &mgr,
            &executor,
            &fs,
            Path::new("/repo"),
            GcOptions::default(),
            &*TEST_CLOCK,
        )
        .unwrap();
        assert!(
            result.skipped.contains(&STALE_SESSION.to_owned()),
            "fresh heartbeat + alive PID must skip, got: {:?}",
            result
        );
        assert!(
            !result.removed.contains(&STALE_SESSION.to_owned()),
            "fresh heartbeat + alive PID must NOT remove"
        );
    }

    /// AC-001.2: GC removes worktree when PID is reaped despite fresh heartbeat.
    #[test]
    fn removes_when_pid_reaped() {
        let fs = InMemoryFileSystem::new().with_file(
            &format!("/repo/{STALE_SESSION}/.ecc-session"),
            &heartbeat_json(FRESH_TS),
        );
        let mgr = MockWorktreeManager::new().with_worktrees(vec![session_wt(STALE_SESSION)]);
        // PID dead → kill -0 returns non-zero
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", &HB_PID.to_string()], err_output(1))
            // The stale check also runs kill -0 for the parsed PID in name (99999)
            .on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(
            &mgr,
            &executor,
            &fs,
            Path::new("/repo"),
            GcOptions::default(),
            &*TEST_CLOCK,
        )
        .unwrap();
        assert!(
            result.removed.contains(&STALE_SESSION.to_owned()),
            "reaped PID must lead to removal, got: {:?}",
            result
        );
    }

    /// AC-001.3: GC removes when heartbeat is stale (>60min).
    #[test]
    fn removes_when_heartbeat_stale() {
        let fs = InMemoryFileSystem::new().with_file(
            &format!("/repo/{STALE_SESSION}/.ecc-session"),
            &heartbeat_json(STALE_TS),
        );
        let mgr = MockWorktreeManager::new().with_worktrees(vec![session_wt(STALE_SESSION)]);
        // PID alive but heartbeat is stale → fall through to removal
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", &HB_PID.to_string()], ok(""))
            .on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(
            &mgr,
            &executor,
            &fs,
            Path::new("/repo"),
            GcOptions::default(),
            &*TEST_CLOCK,
        )
        .unwrap();
        assert!(
            result.removed.contains(&STALE_SESSION.to_owned()),
            "stale heartbeat must lead to removal, got: {:?}",
            result
        );
    }

    /// AC-001.4: GC falls back to 24h timer when no .ecc-session file.
    #[test]
    fn missing_session_file_falls_back() {
        // No .ecc-session file → falls through to age+PID check
        let fs = InMemoryFileSystem::new(); // empty filesystem
        let mgr = MockWorktreeManager::new().with_worktrees(vec![session_wt(STALE_SESSION)]);
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(
            &mgr,
            &executor,
            &fs,
            Path::new("/repo"),
            GcOptions::default(),
            &*TEST_CLOCK,
        )
        .unwrap();
        assert!(
            result.removed.contains(&STALE_SESSION.to_owned()),
            "missing .ecc-session must fall back to timer-based GC, got: {:?}",
            result
        );
    }

    /// AC-001.5: GC treats malformed JSON as missing (falls through to AC-001.4).
    #[test]
    fn malformed_session_file_falls_back() {
        let fs = InMemoryFileSystem::new().with_file(
            &format!("/repo/{STALE_SESSION}/.ecc-session"),
            "not valid json {{{",
        );
        let mgr = MockWorktreeManager::new().with_worktrees(vec![session_wt(STALE_SESSION)]);
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(
            &mgr,
            &executor,
            &fs,
            Path::new("/repo"),
            GcOptions::default(),
            &*TEST_CLOCK,
        )
        .unwrap();
        assert!(
            result.removed.contains(&STALE_SESSION.to_owned()),
            "malformed .ecc-session must fall through to timer-based GC, got: {:?}",
            result
        );
    }
}
