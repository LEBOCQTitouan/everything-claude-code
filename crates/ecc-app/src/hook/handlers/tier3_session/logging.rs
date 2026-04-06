//! Logging hooks — subagent lifecycle and config change logging.

use tracing::warn;

use crate::hook::{HookPorts, HookResult};
use crate::metrics_mgmt::record_if_enabled;
use crate::metrics_session::resolve_session_id;
use ecc_domain::metrics::{MetricEvent, MetricOutcome};
use ecc_domain::time::{datetime_from_epoch, format_datetime, format_time};

use super::helpers::find_files_by_suffix;
use super::{epoch_secs, log_write_failure};

/// subagent:start:log — Log subagent lifecycle start to session file.
///
/// Parses `agent_type` from stdin JSON. Records an AgentSpawn/Success metric event.
pub fn subagent_start_log(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "subagent_start_log", "executing handler");
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    let parsed = serde_json::from_str::<serde_json::Value>(stdin).ok();
    let agent_type = parsed
        .as_ref()
        .and_then(|v| v.get("agent_type")?.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    let sessions_dir = home.join(".claude").join("sessions");
    let session_files = find_files_by_suffix(&sessions_dir, "-session.tmp", ports);

    if let Some(active) = session_files.first()
        && let Ok(content) = ports.fs.read_to_string(active)
    {
        let timestamp = format_time(&datetime_from_epoch(epoch_secs()));
        let updated = format!(
            "{}\n[{}] [Subagent] Started: {}\n",
            content, timestamp, agent_type
        );
        if let Err(e) = ports.fs.write(active, &updated) {
            log_write_failure(active, &e, None);
        }
    }

    // Record AgentSpawn/Success metric event.
    let disabled = ports.env.var("ECC_METRICS_DISABLED").as_deref() == Some("1");
    if !disabled {
        let session_id = resolve_session_id(ports.env.var("CLAUDE_SESSION_ID").as_deref());
        let timestamp = format_datetime(&datetime_from_epoch(epoch_secs()));
        if let Ok(event) = MetricEvent::agent_spawn(
            session_id,
            timestamp,
            agent_type,
            MetricOutcome::Success,
            None,
        ) {
            let _ = record_if_enabled(ports.metrics_store, &event, false);
        }
    }

    HookResult::passthrough(stdin)
}

/// subagent:stop:log — Log subagent lifecycle completion to session file.
///
/// Parses `agent_type`, `$.error`, `$.exit_code`, `$.retry_count` from stdin JSON.
/// Records an AgentSpawn metric event: Failure if error is non-null or exit_code != 0,
/// Success otherwise. retry_count is Some(n) if present, None otherwise.
pub fn subagent_stop_log(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "subagent_stop_log", "executing handler");
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    let parsed = serde_json::from_str::<serde_json::Value>(stdin).ok();
    let agent_type = parsed
        .as_ref()
        .and_then(|v| v.get("agent_type")?.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    let sessions_dir = home.join(".claude").join("sessions");
    let session_files = find_files_by_suffix(&sessions_dir, "-session.tmp", ports);

    if let Some(active) = session_files.first()
        && let Ok(content) = ports.fs.read_to_string(active)
    {
        let timestamp = format_time(&datetime_from_epoch(epoch_secs()));
        let updated = format!(
            "{}\n[{}] [Subagent] Completed: {}\n",
            content, timestamp, agent_type
        );
        if let Err(e) = ports.fs.write(active, &updated) {
            log_write_failure(active, &e, None);
        }
    }

    // Determine outcome: Failure if $.error is a non-null string or $.exit_code != 0.
    let has_error = parsed
        .as_ref()
        .and_then(|v| v.get("error"))
        .and_then(|e| e.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let bad_exit = parsed
        .as_ref()
        .and_then(|v| v.get("exit_code"))
        .and_then(|c| c.as_i64())
        .map(|c| c != 0)
        .unwrap_or(false);
    let outcome = if has_error || bad_exit {
        MetricOutcome::Failure
    } else {
        MetricOutcome::Success
    };

    // Parse retry_count: Some(u32) if present and valid, None otherwise.
    let retry_count: Option<u32> = parsed
        .as_ref()
        .and_then(|v| v.get("retry_count"))
        .and_then(|r| r.as_u64())
        .and_then(|r| u32::try_from(r).ok());

    // Record AgentSpawn metric event.
    let disabled = ports.env.var("ECC_METRICS_DISABLED").as_deref() == Some("1");
    if !disabled {
        let session_id = resolve_session_id(ports.env.var("CLAUDE_SESSION_ID").as_deref());
        let timestamp = format_datetime(&datetime_from_epoch(epoch_secs()));
        if let Ok(event) = MetricEvent::agent_spawn(
            session_id,
            timestamp,
            agent_type,
            outcome,
            retry_count,
        ) {
            let _ = record_if_enabled(ports.metrics_store, &event, false);
        }
    }

    HookResult::passthrough(stdin)
}

/// config:change:log — Log configuration changes to a dedicated log file.
///
/// Parses `config_key` and `config_value` from stdin JSON.
pub fn config_change_log(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "config_change_log", "executing handler");
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    let parsed = serde_json::from_str::<serde_json::Value>(stdin).ok();
    let config_key = parsed
        .as_ref()
        .and_then(|v| v.get("config_key")?.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string());
    let config_value = parsed
        .as_ref()
        .and_then(|v| v.get("config_value")?.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    let sessions_dir = home.join(".claude").join("sessions");
    if let Err(e) = ports.fs.create_dir_all(&sessions_dir) {
        warn!("Cannot create sessions dir: {}", e);
    }

    let log_file = sessions_dir.join("config-changes.log");
    let timestamp = format_datetime(&datetime_from_epoch(epoch_secs()));
    let existing = ports.fs.read_to_string(&log_file).unwrap_or_default();
    let new_content = format!(
        "{}[{}] Config changed: {} = {}\n",
        existing, timestamp, config_key, config_value
    );

    if let Err(e) = ports.fs.write(&log_file, &new_content) {
        log_write_failure(&log_file, &e, None);
    }

    HookResult::passthrough(stdin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookPorts;
    use ecc_domain::metrics::{MetricEventType, MetricOutcome};
    use ecc_ports::fs::FileSystem;
    use ecc_test_support::{
        BufferedTerminal, InMemoryFileSystem, InMemoryMetricsStore, MockEnvironment, MockExecutor,
    };

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

    fn make_ports_with_metrics<'a>(
        fs: &'a InMemoryFileSystem,
        shell: &'a MockExecutor,
        env: &'a MockEnvironment,
        term: &'a BufferedTerminal,
        store: &'a InMemoryMetricsStore,
    ) -> HookPorts<'a> {
        HookPorts {
            fs,
            shell,
            env,
            terminal: term,
            cost_store: None,
            bypass_store: None,
            metrics_store: Some(store),
        }
    }

    // --- PC-009: subagent_start_log records AgentSpawn/Success ---

    #[test]
    fn subagent_start_records_agent_spawn_success() {
        let fs = InMemoryFileSystem::new().with_file(
            "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            "# Session",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_var("CLAUDE_SESSION_ID", "sess-pc009");
        let term = BufferedTerminal::new();
        let store = InMemoryMetricsStore::new();
        let ports = make_ports_with_metrics(&fs, &shell, &env, &term, &store);

        let stdin = r#"{"agent_type":"code-reviewer"}"#;
        let result = subagent_start_log(stdin, &ports);
        assert_eq!(result.exit_code, 0);

        let events = store.snapshot();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, MetricEventType::AgentSpawn);
        assert_eq!(events[0].outcome, MetricOutcome::Success);
        assert_eq!(events[0].agent_type.as_deref(), Some("code-reviewer"));
    }

    // --- PC-010: subagent_stop_log with $.error records AgentSpawn/Failure ---

    #[test]
    fn subagent_stop_records_agent_spawn_failure() {
        let fs = InMemoryFileSystem::new().with_file(
            "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            "# Session",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_var("CLAUDE_SESSION_ID", "sess-pc010");
        let term = BufferedTerminal::new();
        let store = InMemoryMetricsStore::new();
        let ports = make_ports_with_metrics(&fs, &shell, &env, &term, &store);

        let stdin = r#"{"agent_type":"architect","error":"something went wrong"}"#;
        let result = subagent_stop_log(stdin, &ports);
        assert_eq!(result.exit_code, 0);

        let events = store.snapshot();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, MetricEventType::AgentSpawn);
        assert_eq!(events[0].outcome, MetricOutcome::Failure);
        assert_eq!(events[0].agent_type.as_deref(), Some("architect"));
    }

    // --- PC-011: subagent_stop_log with $.retry_count: 3 records retry_count=Some(3) ---

    #[test]
    fn subagent_stop_parses_retry_count() {
        let fs = InMemoryFileSystem::new().with_file(
            "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            "# Session",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_var("CLAUDE_SESSION_ID", "sess-pc011");
        let term = BufferedTerminal::new();
        let store = InMemoryMetricsStore::new();
        let ports = make_ports_with_metrics(&fs, &shell, &env, &term, &store);

        let stdin = r#"{"agent_type":"tdd-guide","retry_count":3}"#;
        let result = subagent_stop_log(stdin, &ports);
        assert_eq!(result.exit_code, 0);

        let events = store.snapshot();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].retry_count, Some(3));
    }

    // --- PC-012: subagent_stop_log with no failure fields records Success and retry_count=None ---

    #[test]
    fn subagent_stop_defaults_success_none() {
        let fs = InMemoryFileSystem::new().with_file(
            "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            "# Session",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_var("CLAUDE_SESSION_ID", "sess-pc012");
        let term = BufferedTerminal::new();
        let store = InMemoryMetricsStore::new();
        let ports = make_ports_with_metrics(&fs, &shell, &env, &term, &store);

        let stdin = r#"{"agent_type":"planner"}"#;
        let result = subagent_stop_log(stdin, &ports);
        assert_eq!(result.exit_code, 0);

        let events = store.snapshot();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].outcome, MetricOutcome::Success);
        assert_eq!(events[0].retry_count, None);
    }

    // --- PC-013: subagent_start_log with ECC_METRICS_DISABLED=1 records zero events ---

    #[test]
    fn subagent_start_metrics_disabled() {
        let fs = InMemoryFileSystem::new().with_file(
            "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            "# Session",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_var("CLAUDE_SESSION_ID", "sess-pc013")
            .with_var("ECC_METRICS_DISABLED", "1");
        let term = BufferedTerminal::new();
        let store = InMemoryMetricsStore::new();
        let ports = make_ports_with_metrics(&fs, &shell, &env, &term, &store);

        let stdin = r#"{"agent_type":"security-reviewer"}"#;
        let result = subagent_start_log(stdin, &ports);
        assert_eq!(result.exit_code, 0);

        let events = store.snapshot();
        assert_eq!(events.len(), 0, "no events when metrics disabled");
    }

    // --- subagent_start_log ---

    #[test]
    fn subagent_start_log_with_agent_type() {
        let fs = InMemoryFileSystem::new().with_file(
            "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            "# Session",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"agent_type":"code-reviewer"}"#;
        let result = subagent_start_log(stdin, &ports);
        assert_eq!(result.exit_code, 0);

        let content = fs
            .read_to_string(std::path::Path::new(
                "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            ))
            .unwrap();
        assert!(content.contains("[Subagent] Started: code-reviewer"));
    }

    #[test]
    fn subagent_start_log_missing_agent_type() {
        let fs = InMemoryFileSystem::new().with_file(
            "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            "# Session",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = subagent_start_log("{}", &ports);
        assert_eq!(result.exit_code, 0);

        let content = fs
            .read_to_string(std::path::Path::new(
                "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            ))
            .unwrap();
        assert!(content.contains("[Subagent] Started: unknown"));
    }

    // --- subagent_stop_log ---

    #[test]
    fn subagent_stop_log_with_fields() {
        let fs = InMemoryFileSystem::new().with_file(
            "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            "# Session",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"agent_type":"architect","agent_id":"abc123"}"#;
        let result = subagent_stop_log(stdin, &ports);
        assert_eq!(result.exit_code, 0);

        let content = fs
            .read_to_string(std::path::Path::new(
                "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            ))
            .unwrap();
        assert!(content.contains("[Subagent] Completed: architect"));
    }

    #[test]
    fn subagent_stop_log_missing_fields() {
        let fs = InMemoryFileSystem::new().with_file(
            "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            "# Session",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = subagent_stop_log("{}", &ports);
        assert_eq!(result.exit_code, 0);

        let content = fs
            .read_to_string(std::path::Path::new(
                "/home/test/.claude/sessions/2026-01-01-abcd1234-session.tmp",
            ))
            .unwrap();
        assert!(content.contains("[Subagent] Completed: unknown"));
    }

    // --- config_change_log ---

    #[test]
    fn config_change_log_writes_both_fields() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"config_key":"theme","config_value":"dark"}"#;
        let result = config_change_log(stdin, &ports);
        assert_eq!(result.exit_code, 0);

        let log_path = std::path::Path::new("/home/test/.claude/sessions/config-changes.log");
        let content = fs.read_to_string(log_path).unwrap();
        assert!(content.contains("Config changed: theme = dark"));
    }

    #[test]
    fn config_change_log_missing_key_uses_unknown() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = config_change_log("{}", &ports);
        assert_eq!(result.exit_code, 0);

        let log_path = std::path::Path::new("/home/test/.claude/sessions/config-changes.log");
        let content = fs.read_to_string(log_path).unwrap();
        assert!(content.contains("Config changed: unknown = unknown"));
    }

    #[test]
    fn config_change_log_no_home_passthrough() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new(); // no home
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = config_change_log("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn config_change_log_appends_to_existing() {
        let fs = InMemoryFileSystem::new().with_file(
            "/home/test/.claude/sessions/config-changes.log",
            "[old] existing entry\n",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"config_key":"model","config_value":"opus"}"#;
        let result = config_change_log(stdin, &ports);
        assert_eq!(result.exit_code, 0);

        let log_path = std::path::Path::new("/home/test/.claude/sessions/config-changes.log");
        let content = fs.read_to_string(log_path).unwrap();
        assert!(content.contains("[old] existing entry"));
        assert!(content.contains("Config changed: model = opus"));
    }
}
