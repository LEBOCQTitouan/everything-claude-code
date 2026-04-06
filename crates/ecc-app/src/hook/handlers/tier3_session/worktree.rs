//! Worktree hooks — worktree creation logging.

use crate::hook::{HookPorts, HookResult};
use ecc_domain::time::{datetime_from_epoch, format_time};

use super::helpers::find_files_by_suffix;
use super::{epoch_secs, log_write_failure};

/// post:enter-worktree:session-log — Log worktree creation to active session file.
///
/// Parses `tool_input.worktree_path` (fallback: `tool_input.name`, then `"unknown"`) from
/// PostToolUse stdin JSON.
pub fn post_enter_worktree_session_log(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(
        handler = "post_enter_worktree_session_log",
        "executing handler"
    );
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    let worktree_path = serde_json::from_str::<serde_json::Value>(stdin)
        .ok()
        .and_then(|v| {
            let tool_input = v.get("tool_input")?;
            tool_input
                .get("worktree_path")
                .or_else(|| tool_input.get("name"))
                .and_then(|p| p.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    let sessions_dir = home.join(".claude").join("sessions");
    let session_files = find_files_by_suffix(&sessions_dir, "-session.tmp", ports);

    if let Some(active) = session_files.first()
        && let Ok(content) = ports.fs.read_to_string(active)
    {
        let timestamp = format_time(&datetime_from_epoch(epoch_secs()));
        let updated = format!(
            "{}\n[{}] [Worktree] Created: {}\n",
            content, timestamp, worktree_path
        );
        if let Err(e) = ports.fs.write(active, &updated) {
            log_write_failure(active, &e, None);
        }
    }

    HookResult::passthrough(stdin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookPorts;
    use ecc_ports::fs::FileSystem;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};

    fn make_ports<'a>(
        fs: &'a InMemoryFileSystem,
        shell: &'a MockExecutor,
        env: &'a MockEnvironment,
        term: &'a BufferedTerminal,
    ) -> HookPorts<'a> {
        HookPorts {
            fs,
            shell,
            env,
            terminal: term,
            cost_store: None,
            bypass_store: None,
            metrics_store: None,
        }
    }

    // --- post_enter_worktree_session_log (PostToolUse format) ---

    #[test]
    fn post_enter_worktree_session_log_logs_from_tool_input() {
        let fs = InMemoryFileSystem::new().with_file(
            "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            "# Session",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_name":"EnterWorktree","tool_input":{"worktree_path":"/tmp/wt"}}"#;
        let result = post_enter_worktree_session_log(stdin, &ports);
        assert_eq!(result.exit_code, 0);

        let content = fs
            .read_to_string(std::path::Path::new(
                "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            ))
            .unwrap();
        assert!(content.contains("[Worktree] Created: /tmp/wt"));
    }

    #[test]
    fn post_enter_worktree_session_log_fallback_name() {
        let fs = InMemoryFileSystem::new().with_file(
            "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            "# Session",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_name":"EnterWorktree","tool_input":{"name":"feature-branch"}}"#;
        let result = post_enter_worktree_session_log(stdin, &ports);
        assert_eq!(result.exit_code, 0);

        let content = fs
            .read_to_string(std::path::Path::new(
                "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            ))
            .unwrap();
        assert!(content.contains("[Worktree] Created: feature-branch"));
    }

    #[test]
    fn post_enter_worktree_session_log_fallback_unknown() {
        let fs = InMemoryFileSystem::new().with_file(
            "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            "# Session",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_name":"EnterWorktree","tool_input":{}}"#;
        let result = post_enter_worktree_session_log(stdin, &ports);
        assert_eq!(result.exit_code, 0);

        let content = fs
            .read_to_string(std::path::Path::new(
                "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            ))
            .unwrap();
        assert!(content.contains("[Worktree] Created: unknown"));
    }

    #[test]
    fn post_enter_worktree_session_log_no_session_passthrough() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_name":"EnterWorktree","tool_input":{"worktree_path":"/tmp/wt"}}"#;
        let result = post_enter_worktree_session_log(stdin, &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }
}
