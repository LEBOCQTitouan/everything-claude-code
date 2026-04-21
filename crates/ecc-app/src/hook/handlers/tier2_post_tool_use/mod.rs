//! Tier 2 PostToolUse hook handlers — refresh heartbeat on every tool use.

use crate::hook::{HookPorts, HookResult};
use crate::worktree::heartbeat;
use std::path::Path;

/// PostToolUse heartbeat refresh — keeps the `.ecc-session` file current.
///
/// Called after every tool invocation so GC can detect live sessions even when
/// Claude Code is actively working but not at a session boundary.
///
/// Fire-and-forget: errors are logged at WARN but never block the tool use.
pub fn post_tool_use_heartbeat(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    let disabled = ports.env.var("ECC_WORKTREE_LIVENESS_DISABLED").as_deref() == Some("1");

    let Some(project_dir) = ports.env.var("CLAUDE_PROJECT_DIR") else {
        return HookResult::passthrough(stdin);
    };
    let worktree_path = Path::new(&project_dir);

    #[cfg(unix)]
    let pid = std::os::unix::process::parent_id();
    #[cfg(not(unix))]
    let pid = std::process::id();

    let clock = ports.clock;
    if let Err(e) = heartbeat::write_heartbeat(
        ports.fs,
        worktree_path,
        pid,
        || clock.now_epoch_secs(),
        disabled,
    ) {
        tracing::warn!(error = ?e, "post-tool-use heartbeat write failed (non-blocking)");
    }

    HookResult::passthrough(stdin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::fs::FileSystem as _;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};

    /// PostToolUse heartbeat writes .ecc-session when CLAUDE_PROJECT_DIR is set.
    #[test]
    fn post_tool_use_writes_heartbeat_when_project_dir_set() {
        let fs = InMemoryFileSystem::new().with_dir("/wt");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_var("CLAUDE_PROJECT_DIR", "/wt")
            .with_home("/home/user");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let result = post_tool_use_heartbeat("{}", &ports);
        assert_eq!(result.exit_code, 0);
        // .ecc-session must exist (heartbeat was written).
        assert!(
            fs.exists(std::path::Path::new("/wt/.ecc-session")),
            ".ecc-session must exist after PostToolUse heartbeat"
        );
    }

    /// PostToolUse heartbeat is a no-op when CLAUDE_PROJECT_DIR is not set.
    #[test]
    fn post_tool_use_noop_without_project_dir() {
        let fs = InMemoryFileSystem::new().with_dir("/wt");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/user");
        // No CLAUDE_PROJECT_DIR set.
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let result = post_tool_use_heartbeat("{}", &ports);
        assert_eq!(result.exit_code, 0);
        // No .ecc-session should be written.
        assert!(
            !fs.exists(std::path::Path::new("/wt/.ecc-session")),
            ".ecc-session must NOT exist when CLAUDE_PROJECT_DIR is unset"
        );
    }

    /// PostToolUse heartbeat is suppressed when kill-switch is active.
    #[test]
    fn post_tool_use_suppressed_by_kill_switch() {
        let fs = InMemoryFileSystem::new().with_dir("/wt");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_var("CLAUDE_PROJECT_DIR", "/wt")
            .with_var("ECC_WORKTREE_LIVENESS_DISABLED", "1")
            .with_home("/home/user");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let result = post_tool_use_heartbeat("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert!(
            !fs.exists(std::path::Path::new("/wt/.ecc-session")),
            ".ecc-session must NOT be written when kill-switch is active"
        );
    }
}
