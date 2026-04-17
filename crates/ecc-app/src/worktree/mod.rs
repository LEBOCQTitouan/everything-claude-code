//! Worktree use cases — GC and status for `ecc-session-*` git worktrees.
//!
//! Orchestrates [`ecc_domain::worktree`] through [`ecc_ports::shell::ShellExecutor`]
//! and [`ecc_ports::worktree::WorktreeManager`].

pub mod gc;
pub mod shell_manager;
pub mod status;

pub use gc::{WorktreeGcResult, gc};
pub use shell_manager::ShellWorktreeManager;
pub use status::{WorktreeStatus, WorktreeStatusEntry, format_status_table, status};

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
pub(crate) fn now_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub(crate) const STALE_SECS: u64 = 24 * 3600;

/// Determine whether a worktree entry is stale (old enough or PID dead).
pub(crate) fn is_worktree_stale(
    executor: &dyn ShellExecutor,
    parsed: &ParsedWorktreeName,
    now: u64,
) -> bool {
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
pub enum WorktreeGcError {
    /// A shell command failed or was not found.
    #[error("worktree operation failed: {0}")]
    Shell(String),
    /// A worktree manager port error.
    #[error("worktree manager error: {0}")]
    Manager(#[from] ecc_ports::worktree::WorktreeError),
}
