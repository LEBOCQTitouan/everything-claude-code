//! Logging hooks — subagent lifecycle and config change logging.

use tracing::warn;

use crate::hook::{HookPorts, HookResult};
use ecc_domain::time::{datetime_from_epoch, format_datetime, format_time};

use super::helpers::find_files_by_suffix;
use super::{epoch_secs, log_write_failure};

/// subagent:start:log — Log subagent lifecycle start to session file.
///
/// Parses `agent_type` from stdin JSON.
pub fn subagent_start_log(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "subagent_start_log", "executing handler");
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    let agent_type = serde_json::from_str::<serde_json::Value>(stdin)
        .ok()
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

    HookResult::passthrough(stdin)
}

/// subagent:stop:log — Log subagent lifecycle completion to session file.
///
/// Parses `agent_type` from stdin JSON.
pub fn subagent_stop_log(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "subagent_stop_log", "executing handler");
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    let agent_type = serde_json::from_str::<serde_json::Value>(stdin)
        .ok()
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
