//! Thin start:cartography hook — counts pending deltas and prints a reminder.

use std::path::PathBuf;

use crate::hook::{HookPorts, HookResult};

/// start:cartography — counts pending deltas and prints a reminder if any exist.
///
/// Uses CWD to find `.claude/cartography/` (no `CLAUDE_PROJECT_DIR` needed).
/// If pending delta files exist, prints a reminder to stderr suggesting the user
/// run `/doc-suite --phase=cartography` to process them.
pub fn start_cartography(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "start_cartography", "executing handler");

    // Use CWD to find .claude/cartography/ (no CLAUDE_PROJECT_DIR needed)
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cartography_dir = cwd.join(".claude").join("cartography");

    // Count pending-delta-*.json files
    let count = match ports.fs.read_dir(&cartography_dir) {
        Ok(entries) => entries
            .iter()
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("pending-delta-") && n.ends_with(".json"))
                    .unwrap_or(false)
            })
            .count(),
        Err(_) => 0,
    };

    if count == 0 {
        return HookResult::passthrough(stdin);
    }

    // Print reminder to stderr
    let msg = format!(
        "{} pending cartography deltas — run `/doc-suite --phase=cartography` to process\n",
        count
    );
    HookResult {
        stdout: stdin.to_string(),
        stderr: msg,
        exit_code: 0,
    }
}
