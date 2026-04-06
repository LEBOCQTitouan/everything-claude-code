//! Tier 3 Hooks — Session management and file I/O hooks.

mod cartography;
mod compact;
pub mod daily;
mod helpers;
mod lifecycle;
mod logging;
mod reflection;
mod session_merge;
mod tracking;
mod worktree;

use std::path::Path;
use tracing::warn;

pub use cartography::{start_cartography, stop_cartography};
pub use compact::{post_compact, pre_compact};
pub use daily::daily_summary;
pub use lifecycle::{session_end, session_start};
pub use logging::{config_change_log, subagent_start_log, subagent_stop_log};
pub use reflection::{craft_velocity, oath_reflection};
pub use session_merge::session_end_merge;
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

#[cfg(test)]
mod tests {
    use crate::hook::HookPorts;
    use ecc_ports::fs::FileSystem as _;
    use ecc_test_support::{
        BufferedTerminal, InMemoryCostStore, InMemoryFileSystem, MockEnvironment, MockExecutor,
    };

    use super::cost_tracker;

    fn make_ports_no_store<'a>(
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

    /// PC-024: cost_tracker calls CostStore::append when a store is provided.
    #[test]
    fn cost_tracker_uses_cost_store() {
        let fs = InMemoryFileSystem::new().with_dir("/home/test/.claude/metrics");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_var("CLAUDE_SESSION_ID", "sess-pc024");
        let term = BufferedTerminal::new();
        let store = InMemoryCostStore::new();

        let ports = HookPorts {
            fs: &fs,
            shell: &shell,
            env: &env,
            terminal: &term,
            cost_store: Some(&store),
            bypass_store: None,
            metrics_store: None,
        };

        let stdin = r#"{"model":"claude-sonnet-4-6","usage":{"input_tokens":12500,"output_tokens":3200,"thinking_tokens":0}}"#;
        let result = cost_tracker(stdin, &ports);
        assert_eq!(result.exit_code, 0);

        let records = store.snapshot();
        assert_eq!(records.len(), 1, "CostStore::append must be called once");
        let rec = &records[0];
        assert_eq!(rec.model.as_str(), "claude-sonnet-4-6");
        assert_eq!(rec.input_tokens.value(), 12500);
        assert_eq!(rec.output_tokens.value(), 3200);
        assert_eq!(rec.session_id, "sess-pc024");
    }

    /// PC-025: cost_tracker falls back to JSONL when cost_store is None.
    #[test]
    fn cost_tracker_falls_back_to_jsonl() {
        let fs = InMemoryFileSystem::new().with_dir("/home/test/.claude/metrics");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_var("CLAUDE_SESSION_ID", "sess-pc025");
        let term = BufferedTerminal::new();
        let ports = make_ports_no_store(&fs, &shell, &env, &term);

        let stdin =
            r#"{"model":"claude-haiku-4-5","usage":{"input_tokens":5000,"output_tokens":1000}}"#;
        let result = cost_tracker(stdin, &ports);
        assert_eq!(result.exit_code, 0);

        let content = fs
            .read_to_string(std::path::Path::new(
                "/home/test/.claude/metrics/costs.jsonl",
            ))
            .expect("costs.jsonl must exist as JSONL fallback");
        assert!(
            content.contains("claude-haiku-4-5"),
            "JSONL must contain model"
        );
        assert!(
            content.contains("estimated_cost_usd"),
            "JSONL must contain cost field"
        );
    }

    /// PC-026: cost_tracker extracts agent_type from stdin JSON.
    #[test]
    fn cost_tracker_extracts_agent_type() {
        let fs = InMemoryFileSystem::new().with_dir("/home/test/.claude/metrics");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_var("CLAUDE_SESSION_ID", "sess-pc026");
        let term = BufferedTerminal::new();
        let store = InMemoryCostStore::new();

        let ports = HookPorts {
            fs: &fs,
            shell: &shell,
            env: &env,
            terminal: &term,
            cost_store: Some(&store),
            bypass_store: None,
            metrics_store: None,
        };

        let stdin = r#"{"model":"claude-sonnet-4-6","usage":{"input_tokens":1000,"output_tokens":500},"agent_type":"code-reviewer"}"#;
        let _ = cost_tracker(stdin, &ports);

        let records = store.snapshot();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].agent_type, "code-reviewer");
    }

    /// PC-027: cost_tracker extracts thinking_tokens from stdin JSON.
    #[test]
    fn cost_tracker_extracts_thinking_tokens() {
        let fs = InMemoryFileSystem::new().with_dir("/home/test/.claude/metrics");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_var("CLAUDE_SESSION_ID", "sess-pc027");
        let term = BufferedTerminal::new();
        let store = InMemoryCostStore::new();

        let ports = HookPorts {
            fs: &fs,
            shell: &shell,
            env: &env,
            terminal: &term,
            cost_store: Some(&store),
            bypass_store: None,
            metrics_store: None,
        };

        let stdin = r#"{"model":"claude-sonnet-4-6","usage":{"input_tokens":12500,"output_tokens":3200,"thinking_tokens":8000}}"#;
        let _ = cost_tracker(stdin, &ports);

        let records = store.snapshot();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].thinking_tokens.value(), 8000);
    }
}
