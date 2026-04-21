//! Worktree use cases — GC and status for `ecc-session-*` git worktrees.
//!
//! Orchestrates [`ecc_domain::worktree`] through [`ecc_ports::shell::ShellExecutor`]
//! and [`ecc_ports::worktree::WorktreeManager`].

pub mod checker;
pub mod gc;
pub mod heartbeat;
pub mod self_identity;
pub mod shell_manager;
pub mod status;

pub use checker::{LivenessChecker, LivenessVerdict};
pub use gc::{GcOptions, WorktreeGcResult, gc};
pub use shell_manager::ShellWorktreeManager;
pub use status::{
    WorktreeStatus, WorktreeStatusEntry, format_status_json, format_status_table, status,
};

use ecc_domain::worktree::ParsedWorktreeName;
use ecc_ports::shell::ShellExecutor;

/// Parse a compact timestamp `YYYYMMDD-HHMMSS` to seconds since epoch (approximate).
pub(crate) fn compact_ts_to_secs(ts: &str) -> Option<u64> {
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
pub(crate) fn now_secs(clock: &dyn ecc_ports::clock::Clock) -> u64 {
    clock.now_epoch_secs()
}

pub(crate) const STALE_SECS: u64 = 24 * 3600;

/// Worktrees modified within this window are never considered stale.
pub(crate) const RECENCY_SECS: u64 = 30 * 60;

/// Determine whether a worktree entry is stale.
///
/// Formula: `stale = (age > 24h OR pid_dead) AND modified > 30min`
///
/// A worktree is stale only when BOTH the existing staleness condition
/// (age or PID) AND the recency condition (not recently modified) are true.
/// The recency check uses `stat` via the executor for hexagonal purity.
pub(crate) fn is_worktree_stale(
    executor: &dyn ShellExecutor,
    parsed: &ParsedWorktreeName,
    now: u64,
    worktree_path: &std::path::Path,
) -> bool {
    let age_stale = compact_ts_to_secs(&parsed.timestamp)
        .map(|ts| now.saturating_sub(ts) > STALE_SECS)
        .unwrap_or(false);
    let pid_str = parsed.pid.to_string();
    let pid_alive = executor
        .run_command("kill", &["-0", &pid_str])
        .map(|o| o.success())
        .unwrap_or(false);

    let base_stale = age_stale || !pid_alive;
    if !base_stale {
        return false;
    }

    // Defense-in-depth: skip if the worktree was recently modified.
    let recently_modified = is_recently_modified(executor, worktree_path, now);
    base_stale && !recently_modified
}

/// Check if a worktree's `.git` file/dir was modified within RECENCY_SECS.
/// Uses `stat` via ShellExecutor. Returns false on any failure (fail-open).
fn is_recently_modified(
    executor: &dyn ShellExecutor,
    worktree_path: &std::path::Path,
    now: u64,
) -> bool {
    let git_path = worktree_path.join(".git");
    let git_path_str = git_path.to_string_lossy();

    // Try macOS stat format first, then Linux fallback.
    let mtime = executor
        .run_command("stat", &["-f", "%m", &git_path_str])
        .or_else(|_| executor.run_command("stat", &["-c", "%Y", &git_path_str]))
        .ok()
        .filter(|o| o.success())
        .and_then(|o| o.stdout.trim().parse::<u64>().ok());

    match mtime {
        Some(t) => now.saturating_sub(t) < RECENCY_SECS,
        None => false, // Stat failed or malformed output → fail-open (skip recency guard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::worktree::ParsedWorktreeName;
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
            stderr: "error".to_owned(),
            exit_code: code,
        }
    }

    fn stale_parsed() -> ParsedWorktreeName {
        ParsedWorktreeName {
            timestamp: "20200101-000000".to_owned(),
            slug: "old".to_owned(),
            pid: 99999,
        }
    }

    #[allow(dead_code)]
    fn far_past() -> u64 {
        // 2020-01-01 epoch seconds
        1_577_836_800
    }

    fn now_test() -> u64 {
        // 2026-04-17 ~epoch seconds
        1_776_000_000
    }

    #[test]
    fn recently_modified_worktree_is_not_stale() {
        let recent_mtime = now_test() - 600; // 10 minutes ago
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], err_output(1)) // PID dead
            .on_args(
                "stat",
                &["-f", "%m", "/wt/.git"],
                ok(&recent_mtime.to_string()),
            );
        let parsed = stale_parsed();
        assert!(
            !is_worktree_stale(&executor, &parsed, now_test(), Path::new("/wt")),
            "worktree modified 10min ago must NOT be stale even with dead PID"
        );
    }

    #[test]
    fn old_unmodified_worktree_is_stale() {
        let old_mtime = now_test() - 7200; // 2 hours ago
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], err_output(1))
            .on_args(
                "stat",
                &["-f", "%m", "/wt/.git"],
                ok(&old_mtime.to_string()),
            );
        let parsed = stale_parsed();
        assert!(
            is_worktree_stale(&executor, &parsed, now_test(), Path::new("/wt")),
            "worktree modified 2h ago + dead PID + old age must be stale"
        );
    }

    #[test]
    fn stat_failure_preserves_existing_behavior() {
        // Stat fails → recency guard not applied → falls back to PID+age
        let executor = MockExecutor::new().on_args("kill", &["-0", "99999"], err_output(1));
        // No stat mock → stat returns Err → fail-open
        let parsed = stale_parsed();
        assert!(
            is_worktree_stale(&executor, &parsed, now_test(), Path::new("/wt")),
            "stat failure must not protect an otherwise-stale worktree"
        );
    }

    #[test]
    fn malformed_stat_output_treated_as_failure() {
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], err_output(1))
            .on_args("stat", &["-f", "%m", "/wt/.git"], ok("not-a-number"));
        let parsed = stale_parsed();
        assert!(
            is_worktree_stale(&executor, &parsed, now_test(), Path::new("/wt")),
            "malformed stat output must be treated as stat failure (fail-open)"
        );
    }

    #[test]
    fn live_pid_overrides_old_modification() {
        let old_mtime = now_test() - 7200;
        let executor = MockExecutor::new()
            .on_args("kill", &["-0", "99999"], ok("")) // PID alive
            .on_args(
                "stat",
                &["-f", "%m", "/wt/.git"],
                ok(&old_mtime.to_string()),
            );
        // PID alive → base_stale = false (age is also old but PID check short-circuits)
        // Actually: age_stale = true (2020 timestamp), pid_alive = true
        // base_stale = age_stale || !pid_alive = true || false = true
        // Hmm — that means live PID doesn't override old age.
        // The formula is: (age > 24h OR pid_dead) AND modified > 30min
        // With PID alive but age > 24h: base_stale = true
        // With old modification: recently_modified = false
        // So stale = true. But the test name says "live PID overrides old modification"
        // Let me fix: use a fresh timestamp so age is NOT stale.
        let fresh_parsed = ParsedWorktreeName {
            timestamp: "20260417-000000".to_owned(),
            slug: "fresh".to_owned(),
            pid: 99999,
        };
        assert!(
            !is_worktree_stale(&executor, &fresh_parsed, now_test(), Path::new("/wt")),
            "live PID + fresh age must not be stale even with old modification"
        );
    }
}

/// Errors returned by worktree operations.
#[derive(Debug, thiserror::Error)]
pub enum WorktreeGcError {
    /// A shell command failed or was not found.
    #[error("worktree operation failed: {0}")]
    Shell(String),
    /// A worktree manager port error.
    #[error("worktree manager error: {0}")]
    Manager(#[from] ecc_ports::worktree::WorktreeError),
}
