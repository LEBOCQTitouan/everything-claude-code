//! Compaction hooks — pre-compact and post-compact state saves.

use tracing::warn;

use crate::hook::{HookPorts, HookResult};
use ecc_domain::time::{datetime_from_epoch, format_datetime, format_time};

use super::helpers::find_files_by_suffix;
use super::{epoch_secs, log_write_failure};

/// pre-compact: save state before context compaction.
pub fn pre_compact(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "pre_compact", "executing handler");
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    let sessions_dir = home.join(".claude").join("sessions");
    if let Err(e) = ports.fs.create_dir_all(&sessions_dir) {
        warn!("Cannot create sessions dir: {}", e);
    }

    let compaction_log = sessions_dir.join("compaction-log.txt");
    let timestamp = format_datetime(&datetime_from_epoch(epoch_secs()));

    // Append to compaction log
    let existing = ports.fs.read_to_string(&compaction_log).unwrap_or_default();
    let new_content = format!("{}[{}] Context compaction triggered\n", existing, timestamp);
    if let Err(e) = ports.fs.write(&compaction_log, &new_content) {
        log_write_failure(&compaction_log, &e, None);
    }

    // Append note to active session
    let session_files = find_files_by_suffix(&sessions_dir, "-session.tmp", ports);
    if let Some(active) = session_files.first()
        && let Ok(content) = ports.fs.read_to_string(active)
    {
        let time_str = format_time(&datetime_from_epoch(epoch_secs()));
        let updated = format!(
            "{}\n---\n**[Compaction occurred at {}]** - Context was summarized\n",
            content, time_str
        );
        if let Err(e) = ports.fs.write(active, &updated) {
            log_write_failure(active, &e, None);
        }
    }

    HookResult::warn(stdin, "[PreCompact] State saved before compaction\n")
}

/// post:compact:state-save — Save compaction summary to session log.
///
/// Parses `compact_summary` from stdin JSON. Appends to compaction-log.txt.
pub fn post_compact(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "post_compact", "executing handler");
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    let summary = serde_json::from_str::<serde_json::Value>(stdin)
        .ok()
        .and_then(|v| v.get("compact_summary")?.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "No summary provided".to_string());

    let sessions_dir = home.join(".claude").join("sessions");
    if let Err(e) = ports.fs.create_dir_all(&sessions_dir) {
        warn!("Cannot create sessions dir: {}", e);
    }

    let compaction_log = sessions_dir.join("compaction-log.txt");
    let timestamp = format_datetime(&datetime_from_epoch(epoch_secs()));
    let existing = ports.fs.read_to_string(&compaction_log).unwrap_or_default();
    let new_content = format!("{}[{}] PostCompact: {}\n", existing, timestamp, summary);

    if let Err(e) = ports.fs.write(&compaction_log, &new_content) {
        log_write_failure(&compaction_log, &e, None);
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

    #[test]
    fn post_compact_with_summary() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"compact_summary":"Summarized auth module discussion"}"#;
        let result = post_compact(stdin, &ports);
        assert_eq!(result.exit_code, 0);

        let log_path = std::path::Path::new("/home/test/.claude/sessions/compaction-log.txt");
        let content = fs.read_to_string(log_path).unwrap();
        assert!(content.contains("Summarized auth module discussion"));
        assert!(content.contains("PostCompact"));
    }

    #[test]
    fn post_compact_without_summary() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = post_compact("{}", &ports);
        assert_eq!(result.exit_code, 0);

        let log_path = std::path::Path::new("/home/test/.claude/sessions/compaction-log.txt");
        let content = fs.read_to_string(log_path).unwrap();
        assert!(content.contains("No summary provided"));
    }

    #[test]
    fn post_compact_no_home_passthrough() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new(); // no home
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = post_compact("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }
}
