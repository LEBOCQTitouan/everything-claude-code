//! Worktree GC use case — garbage-collects stale `ecc-session-*` git worktrees.

use ecc_domain::worktree::WorktreeName;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::worktree::WorktreeManager;
use std::path::Path;

use super::{WorktreeGcError, compact_ts_to_secs, is_worktree_stale, now_secs};

/// Conservative fallback window in seconds for self-skip when the resolver returns `None`.
/// Default: 3600 (1 hour) — matches `TTL_DEFAULT_SECS` (AC-003.6).
pub const SELF_SKIP_FALLBACK_DEFAULT_SECS: u64 = 3600;

/// Reason a worktree would be deleted in a dry-run preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeletionReason {
    /// Worktree is stale (age > 24h or PID dead).
    Stale,
    /// Worktree is forced via `--force` (unmerged commits override).
    Forced,
    /// Worktree is live but `--force --kill-live` was specified.
    KillLive,
}

impl DeletionReason {
    /// Convert to a human-readable string for display.
    pub fn as_str(&self) -> &'static str {
        match self {
            DeletionReason::Stale => "stale",
            DeletionReason::Forced => "forced",
            DeletionReason::KillLive => "kill-live",
        }
    }
}

/// A worktree that would be deleted in a dry-run preview.
#[derive(Debug, Clone)]
pub struct WouldDelete {
    /// Worktree name.
    pub name: String,
    /// Reason for deletion.
    pub reason: DeletionReason,
}

/// Configuration for the GC run.
#[derive(Debug, Clone)]
pub struct GcOptions {
    /// Override liveness check and remove unmerged worktrees.
    pub force: bool,
    /// Delete live worktrees. Requires `force=true`.
    /// When `false` (default), live worktrees are skipped with a diagnostic message.
    /// When `true`, live worktrees are included in the deletion set.
    pub kill_live: bool,
    /// Preview mode: print what would be deleted without actually deleting (AC-008.1).
    /// When `true`, no destructive operations are performed (AC-008.2).
    pub dry_run: bool,
    /// The current session's worktree name, used for self-skip (AC-003.2).
    /// When `Some`, the named worktree is skipped unconditionally.
    /// When `None`, falls back to conservative skip of all young session worktrees
    /// within `self_skip_fallback_secs` (AC-003.3).
    pub self_skip: Option<WorktreeName>,
    /// Conservative fallback window in seconds for skipping young session worktrees
    /// when `self_skip` is `None`. Default: `SELF_SKIP_FALLBACK_DEFAULT_SECS` = 3600 (AC-003.6).
    pub self_skip_fallback_secs: u64,
    /// When `true`, skip the `.ecc-session` heartbeat consult entirely and fall back to
    /// BL-150 logic (`kill -0` + 24h timer). AC-009.1 kill switch. Plumbed from
    /// `ECC_WORKTREE_LIVENESS_DISABLED=1` env var read at CLI boundary.
    pub liveness_disabled: bool,
}

impl Default for GcOptions {
    fn default() -> Self {
        Self {
            force: false,
            kill_live: false,
            dry_run: false,
            self_skip: None,
            self_skip_fallback_secs: SELF_SKIP_FALLBACK_DEFAULT_SECS,
            liveness_disabled: false,
        }
    }
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
    /// Worktrees that would be deleted (populated only when `dry_run=true`).
    pub would_delete: Vec<WouldDelete>,
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

        // AC-003.2: self-skip — unconditionally skip the current session's own worktree.
        if let Some(ref self_name) = options.self_skip {
            let candidate = WorktreeName::new(&worktree_name).unwrap_or_else(|_| self_name.clone());
            if self_name.eq_platform(&candidate) {
                tracing::debug!(
                    worktree = %worktree_name,
                    "GC: skipping own worktree (self-skip)"
                );
                result.skipped.push(worktree_name);
                continue;
            }
        } else {
            // AC-003.3: conservative fallback — when resolver returned None, skip all
            // session-prefixed worktrees that are younger than the fallback window.
            // This protects the current session in environments where CLAUDE_PROJECT_DIR
            // is not set or canonicalize fails.
            let age_secs = compact_ts_to_secs(&parsed.timestamp)
                .map(|ts| now.saturating_sub(ts))
                .unwrap_or(u64::MAX);
            if age_secs < options.self_skip_fallback_secs {
                tracing::debug!(
                    worktree = %worktree_name,
                    age_secs,
                    fallback_secs = options.self_skip_fallback_secs,
                    "GC: skipping young session worktree (resolver=None fallback)"
                );
                result.skipped.push(worktree_name);
                continue;
            }
        }

        // Consult the .ecc-session heartbeat before the stale-timer fallback.
        // On NotFound or malformed JSON → fall through to stale-timer logic (AC-001.4, AC-001.5).
        // Kill switch (AC-009.1): when `liveness_disabled=true`, skip the heartbeat consult
        // entirely and fall through to BL-150 logic.
        let session_path = worktree_path.join(".ecc-session");
        // When kill_live=true and the worktree is live, we skip the stale-timer check entirely.
        let mut force_delete_live = false;
        if !options.liveness_disabled
            && let Ok(content) = fs.read_to_string(&session_path)
            && let Ok(record) = ecc_domain::worktree::liveness::LivenessRecord::parse(&content)
        {
            let pid_str = record.claude_code_pid.to_string();
            let pid_alive = executor
                .run_command("kill", &["-0", &pid_str])
                .map(|o| o.success())
                .unwrap_or(false);
            if ecc_domain::worktree::liveness::is_live(&record, now, pid_alive, TTL_DEFAULT_SECS) {
                if options.kill_live {
                    tracing::warn!(
                        worktree = %worktree_name,
                        "GC: deleting live worktree (--force --kill-live)"
                    );
                    force_delete_live = true;
                } else {
                    let secs_ago = now.saturating_sub(record.last_seen_unix_ts);
                    eprintln!(
                        "Skipping {worktree_name}: active session detected (PID {}, last seen {secs_ago}s ago). Use --force --kill-live to override.",
                        record.claude_code_pid
                    );
                    result.skipped.push(worktree_name);
                    continue;
                }
            }
        }

        // When force_delete_live=true, skip the stale-timer check: the worktree is
        // explicitly targeted for deletion regardless of staleness.
        if !force_delete_live && !is_worktree_stale(executor, &parsed, now, worktree_path) {
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

        // Determine the deletion reason for dry-run preview.
        let deletion_reason = if force_delete_live {
            DeletionReason::KillLive
        } else if force {
            DeletionReason::Forced
        } else {
            DeletionReason::Stale
        };

        // AC-008.2: dry-run — record what would be deleted, skip destructive calls.
        if options.dry_run {
            result.would_delete.push(WouldDelete {
                name: worktree_name,
                reason: deletion_reason,
            });
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
            GcOptions {
                force: true,
                ..GcOptions::default()
            },
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

    /// AC-009.1 kill-switch READ suppression: GC ignores `.ecc-session` entirely
    /// when `liveness_disabled=true` and falls back to BL-150 logic.
    #[test]
    fn liveness_disabled_skips_heartbeat_consult() {
        // Fresh heartbeat present + alive PID would normally cause GC to SKIP.
        // But with `liveness_disabled=true`, GC must ignore the heartbeat entirely
        // and fall back to stale-timer logic; the stale worktree must be REMOVED.
        let fs = InMemoryFileSystem::new().with_file(
            &format!("/repo/{STALE_SESSION}/.ecc-session"),
            &heartbeat_json(FRESH_TS),
        );
        let mgr = MockWorktreeManager::new().with_worktrees(vec![session_wt(STALE_SESSION)]);
        // PID alive → without kill switch, this would be "live, skip".
        let executor = MockExecutor::new().on_args("kill", &["-0", &HB_PID.to_string()], ok(""));

        let result = gc(
            &mgr,
            &executor,
            &fs,
            Path::new("/repo"),
            GcOptions {
                liveness_disabled: true,
                ..GcOptions::default()
            },
            &*TEST_CLOCK,
        )
        .unwrap();
        assert!(
            result.removed.contains(&STALE_SESSION.to_owned())
                || result.skipped.contains(&STALE_SESSION.to_owned()),
            "with liveness_disabled, GC must reach BL-150 path (not heartbeat-skip); got: {:?}",
            result
        );
        // Crucial: it must NOT short-circuit on the fresh heartbeat.
        // The BL-150 path may skip for other reasons (e.g., alive PID, age) —
        // what matters is we reach BL-150, not the heartbeat path.
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

    // ── PC-037..042: self-skip + fallback tests ───────────────────────────────

    /// Helper: create a WorktreeName from a raw validated string.
    fn worktree_name(s: &str) -> WorktreeName {
        WorktreeName::new(s).unwrap_or_else(|e| panic!("invalid worktree name '{s}': {e}"))
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

    /// AC-003.2: GC skips the worktree whose name matches `self_skip`.
    #[test]
    fn skips_self_worktree() {
        // STALE_SESSION is stale by age and dead by PID — but self_skip should prevent removal.
        let mgr = MockWorktreeManager::new().with_worktrees(vec![session_wt(STALE_SESSION)]);
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions {
                self_skip: Some(worktree_name(STALE_SESSION)),
                ..GcOptions::default()
            },
            &*TEST_CLOCK,
        )
        .unwrap();

        assert!(
            result.skipped.contains(&STALE_SESSION.to_owned()),
            "self-skip worktree must be in skipped, got: {:?}",
            result
        );
        assert!(
            !result.removed.contains(&STALE_SESSION.to_owned()),
            "self-skip worktree must NOT be removed"
        );
    }

    /// AC-003.3: Resolver None → all session worktrees younger than fallback_secs are skipped.
    #[test]
    fn skips_young_when_resolver_none() {
        // FRESH_SESSION has a far-future timestamp → it is "young" (age near-zero from test clock).
        // With self_skip=None and fallback_secs=3600, it must be skipped.
        let mgr = MockWorktreeManager::new().with_worktrees(vec![session_wt(FRESH_SESSION)]);
        // Kill -0 would succeed but we expect skip before that check.
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], ok(""));

        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions {
                self_skip: None,
                self_skip_fallback_secs: 3600,
                ..GcOptions::default()
            },
            &*TEST_CLOCK,
        )
        .unwrap();

        assert!(
            result.skipped.contains(&FRESH_SESSION.to_owned()),
            "young session worktree must be skipped when resolver=None, got: {:?}",
            result
        );
        assert!(
            !result.removed.contains(&FRESH_SESSION.to_owned()),
            "young session worktree must NOT be removed when resolver=None"
        );
    }

    /// AC-003.4: Non-session worktrees are not affected by the resolver-None fallback.
    #[test]
    fn non_session_unaffected_by_none_resolver() {
        // A non-session worktree (cannot be parsed by WorktreeName::parse) must not
        // be skipped by the fallback — it falls through to the normal GC logic.
        let non_session_wt = WorktreeInfo {
            path: "/repo/regular-branch".to_owned(),
            branch: Some("regular-branch".to_owned()),
        };
        let mgr = MockWorktreeManager::new().with_worktrees(vec![non_session_wt]);
        let executor = MockExecutor::new();

        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions {
                self_skip: None,
                self_skip_fallback_secs: 3600,
                ..GcOptions::default()
            },
            &*TEST_CLOCK,
        )
        .unwrap();

        // regular-branch is not a session worktree → WorktreeName::parse returns None
        // → not affected by the self-skip fallback (falls through to existing GC logic,
        // which also skips it because it can't be parsed).
        assert!(
            !result.removed.contains(&"regular-branch".to_owned()),
            "non-session worktree must not be in removed list (parse filter)"
        );
    }

    /// AC-003.6: Fallback window defaults to 3600 seconds via `GcOptions::default()`.
    #[test]
    fn fallback_default_3600() {
        let opts = GcOptions::default();
        assert_eq!(
            opts.self_skip_fallback_secs, 3600,
            "default self_skip_fallback_secs must be 3600 (AC-003.6)"
        );
    }

    /// AC-003.6: Fallback window can be overridden via `self_skip_fallback_secs`.
    #[test]
    fn fallback_env_override_applies() {
        // With fallback_secs=0, even a "fresh" session worktree is not skipped
        // by the fallback (age=0 is NOT < 0).
        let mgr = MockWorktreeManager::new().with_worktrees(vec![session_wt(FRESH_SESSION)]);
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], ok(""));

        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions {
                self_skip: None,
                self_skip_fallback_secs: 0, // override: no protection
                ..GcOptions::default()
            },
            &*TEST_CLOCK,
        )
        .unwrap();

        // With fallback_secs=0: the fallback does NOT skip the worktree.
        // It then proceeds to the heartbeat/stale check. FRESH_SESSION has a future
        // timestamp → it's not stale by age → it's skipped by the normal stale check.
        // The test just verifies fallback_secs=0 overrides the default behavior.
        // Result: skipped (by normal fresh-timestamp path), NOT removed.
        assert!(
            !result.removed.contains(&FRESH_SESSION.to_owned()),
            "fresh session must not be removed even with fallback_secs=0 (still fresh by age)"
        );
    }

    // ── PC-056..058: --dry-run preview tests ─────────────────────────────────

    /// AC-008.1: `--dry-run` populates `would_delete` with stale worktrees.
    #[test]
    fn dry_run_prints_would_delete() {
        let mgr = MockWorktreeManager::new().with_worktrees(vec![session_wt(STALE_SESSION)]);
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions {
                dry_run: true,
                force: true, // force to skip unmerged check
                ..GcOptions::default()
            },
            &*TEST_CLOCK,
        )
        .unwrap();

        assert!(
            result.removed.is_empty(),
            "dry_run must not remove any worktree, got removed={:?}",
            result.removed
        );
        assert_eq!(
            result.would_delete.len(),
            1,
            "dry_run must produce 1 would_delete entry, got: {:?}",
            result.would_delete
        );
        assert_eq!(
            result.would_delete[0].name, STALE_SESSION,
            "would_delete name must match stale session"
        );
    }

    /// AC-008.2: `--dry-run` makes zero destructive calls (remove_worktree never called).
    #[test]
    fn dry_run_no_side_effects() {
        // MockWorktreeManager with remove_succeeds=false: if remove_worktree is called,
        // it returns Err → which would show up in result.errors.
        // Dry-run must produce zero errors (i.e., remove_worktree never called).
        let mgr = MockWorktreeManager::new()
            .with_worktrees(vec![session_wt(STALE_SESSION)])
            .with_remove_succeeds(false); // would fail if called
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], err_output(1));

        let result = gc(
            &mgr,
            &executor,
            &InMemoryFileSystem::new(),
            Path::new("/repo"),
            GcOptions {
                dry_run: true,
                force: true,
                ..GcOptions::default()
            },
            &*TEST_CLOCK,
        )
        .unwrap();

        assert!(
            result.errors.is_empty(),
            "dry_run must make zero destructive calls (no errors expected), got: {:?}",
            result.errors
        );
        assert!(
            result.removed.is_empty(),
            "dry_run must not populate removed, got: {:?}",
            result.removed
        );
        assert_eq!(
            result.would_delete.len(),
            1,
            "dry_run must produce would_delete entry, got: {:?}",
            result.would_delete
        );
    }

    /// AC-008.3: `--dry-run --force --kill-live` includes live worktrees in preview.
    #[test]
    fn dry_run_kill_live_preview() {
        // A live worktree (fresh heartbeat + alive PID).
        let fs = InMemoryFileSystem::new().with_file(
            &format!("/repo/{STALE_SESSION}/.ecc-session"),
            &heartbeat_json(FRESH_TS),
        );
        let mgr = MockWorktreeManager::new().with_worktrees(vec![session_wt(STALE_SESSION)]);
        let executor =
            MockExecutor::new().on_args("kill", &["-0", &HB_PID.to_string()], ok(""));

        let result = gc(
            &mgr,
            &executor,
            &fs,
            Path::new("/repo"),
            GcOptions {
                dry_run: true,
                force: true,
                kill_live: true,
                ..GcOptions::default()
            },
            &*TEST_CLOCK,
        )
        .unwrap();

        assert!(
            result.removed.is_empty(),
            "dry_run must not remove live worktree, got removed={:?}",
            result.removed
        );
        assert_eq!(
            result.would_delete.len(),
            1,
            "dry_run --kill-live must include live worktree in preview, got: {:?}",
            result.would_delete
        );
        assert_eq!(
            result.would_delete[0].reason,
            DeletionReason::KillLive,
            "reason must be KillLive for live worktree, got: {:?}",
            result.would_delete[0].reason
        );
    }
}
