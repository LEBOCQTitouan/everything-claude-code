//! Tier 3 Hooks — Session management and file I/O hooks.

mod compact;
pub mod daily;
mod helpers;
mod lifecycle;
mod logging;
mod reflection;
mod tracking;
mod worktree;

use std::path::Path;
use tracing::warn;

pub use compact::{post_compact, pre_compact};
pub use daily::daily_summary;
pub use lifecycle::{session_end, session_start};
pub use logging::{config_change_log, subagent_start_log, subagent_stop_log};
pub use reflection::{craft_velocity, oath_reflection};
pub use tracking::{cost_tracker, evaluate_session};
pub use worktree::post_enter_worktree_session_log;

/// Log a write failure and append the warning to stderr_parts if provided.
fn log_write_failure(
    path: &Path,
    err: &ecc_ports::fs::FsError,
    stderr_parts: Option<&mut Vec<String>>,
) {
    let msg = format!("[Warning] Failed to write {}: {}", path.display(), err);
    warn!("{}", msg);
    if let Some(parts) = stderr_parts {
        parts.push(msg);
    }
}

fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
