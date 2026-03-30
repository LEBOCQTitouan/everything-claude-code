//! Reflection hooks — oath reflection and craft velocity.

use tracing::warn;

use crate::hook::{HookPorts, HookResult};
use ecc_domain::time::{datetime_from_epoch, format_datetime};
use std::path::Path;

use super::{epoch_secs, log_write_failure};

/// stop:oath-reflection — summarize session against Programmer's Oath.
pub fn oath_reflection(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "oath_reflection", "executing handler");
    // Parse recent git log to count commit types
    let log_output = ports.shell.run_command("git", &["log", "--oneline", "-50"]);
    let log_lines = match log_output {
        Ok(ref out) if out.success() => out.stdout.clone(),
        _ => return HookResult::passthrough(stdin),
    };

    if log_lines.trim().is_empty() {
        return HookResult::passthrough(stdin);
    }

    let mut feat_count = 0u32;
    let mut fix_count = 0u32;
    let mut scout_count = 0u32;
    let mut test_count = 0u32;
    let mut other_count = 0u32;

    for line in log_lines.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Skip hash prefix (first word)
        let msg = trimmed
            .split_once(' ')
            .map(|(_, rest)| rest)
            .unwrap_or(trimmed);

        if msg.starts_with("feat:") || msg.starts_with("feat(") {
            feat_count += 1;
        } else if msg.starts_with("fix:") || msg.starts_with("fix(") {
            fix_count += 1;
        } else if msg.starts_with("chore(scout)") {
            scout_count += 1;
        } else if msg.starts_with("test:") || msg.starts_with("test(") {
            test_count += 1;
        } else {
            other_count += 1;
        }
    }

    let total = feat_count + fix_count + scout_count + test_count + other_count;
    if total == 0 {
        return HookResult::passthrough(stdin);
    }

    // Check for oath notes file
    let oath_notes_exist = ports.fs.exists(Path::new("docs/audits/robert-notes.md"));
    let oath_summary = if oath_notes_exist {
        match ports
            .fs
            .read_to_string(Path::new("docs/audits/robert-notes.md"))
        {
            Ok(content) => {
                let warning_count = content.matches("WARNING").count();
                let violation_count = content.matches("VIOLATION").count();
                if warning_count + violation_count > 0 {
                    format!("{} oath note(s)", warning_count + violation_count)
                } else {
                    "0 oath notes".to_string()
                }
            }
            Err(_) => "0 oath notes".to_string(),
        }
    } else {
        "0 oath notes".to_string()
    };

    let msg = format!(
        "[Hook] Oath Reflection: {} features shipped, {} tests, {} scout improvements, {} fixes, {}.\n",
        feat_count, test_count, scout_count, fix_count, oath_summary
    );

    if feat_count == 0 && fix_count == 0 && test_count == 0 {
        return HookResult::passthrough(stdin);
    }

    HookResult::warn(stdin, &msg)
}

/// stop:craft-velocity — calculate rework ratio and append to metrics.
pub fn craft_velocity(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "craft_velocity", "executing handler");
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    let log_output = ports.shell.run_command("git", &["log", "--oneline", "-50"]);
    let log_lines = match log_output {
        Ok(ref out) if out.success() => out.stdout.clone(),
        _ => return HookResult::passthrough(stdin),
    };

    if log_lines.trim().is_empty() {
        return HookResult::passthrough(stdin);
    }

    let mut forward = 0u32;
    let mut rework = 0u32;
    let mut neutral = 0u32;

    for line in log_lines.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let msg = trimmed
            .split_once(' ')
            .map(|(_, rest)| rest)
            .unwrap_or(trimmed);

        if msg.starts_with("feat:")
            || msg.starts_with("feat(")
            || msg.starts_with("test:")
            || msg.starts_with("test(")
            || msg.starts_with("docs:")
            || msg.starts_with("docs(")
            || msg.starts_with("chore(scout)")
        {
            forward += 1;
        } else if msg.starts_with("fix:")
            || msg.starts_with("fix(")
            || msg.starts_with("chore:")
            || msg.starts_with("chore(")
        {
            // chore(scout) already captured above
            if !msg.starts_with("chore(scout)") {
                rework += 1;
            }
        } else {
            // refactor, ci, perf, and other types are neutral
            neutral += 1;
        }
    }

    let total = forward + rework + neutral;
    if total == 0 {
        return HookResult::passthrough(stdin);
    }

    let ratio = rework as f64 / total as f64;

    let metrics_dir = home.join(".claude").join("metrics");
    if let Err(e) = ports.fs.create_dir_all(&metrics_dir) {
        warn!("Cannot create metrics dir: {}", e);
    }

    let timestamp = format_datetime(&datetime_from_epoch(epoch_secs()));
    let session_id = ports
        .env
        .var("CLAUDE_SESSION_ID")
        .unwrap_or_else(|| "default".to_string());

    let row = serde_json::json!({
        "timestamp": timestamp,
        "session_id": session_id,
        "forward": forward,
        "rework": rework,
        "neutral": neutral,
        "total": total,
        "rework_ratio": (ratio * 100.0).round() / 100.0,
    });

    let velocity_file = metrics_dir.join("craft-velocity.jsonl");
    let existing = ports.fs.read_to_string(&velocity_file).unwrap_or_default();
    let new_content = format!("{}{}\n", existing, row);
    if let Err(e) = ports.fs.write(&velocity_file, &new_content) {
        log_write_failure(&velocity_file, &e, None);
    }

    HookResult::passthrough(stdin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookPorts;
    use ecc_ports::fs::FileSystem;
    use ecc_ports::shell::CommandOutput;
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
        }
    }

    // --- oath_reflection ---

    #[test]
    fn oath_reflection_summarizes_mixed_commits() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on_args(
            "git",
            &["log", "--oneline", "-50"],
            CommandOutput {
                stdout: "abc1234 feat: add user auth\n\
                         def5678 test: add auth tests\n\
                         ghi9012 fix: handle null token\n\
                         jkl3456 chore(scout): extract constant\n"
                    .to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = oath_reflection("{}", &ports);
        assert!(result.stderr.contains("1 features shipped"));
        assert!(result.stderr.contains("1 tests"));
        assert!(result.stderr.contains("1 scout"));
        assert!(result.stderr.contains("1 fixes"));
    }

    #[test]
    fn oath_reflection_passthrough_no_commits() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on_args(
            "git",
            &["log", "--oneline", "-50"],
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = oath_reflection("{}", &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn oath_reflection_passthrough_no_git() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = oath_reflection("{}", &ports);
        assert!(result.stderr.is_empty());
    }

    // --- craft_velocity ---

    #[test]
    fn craft_velocity_calculates_ratio_and_writes_jsonl() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on_args(
            "git",
            &["log", "--oneline", "-50"],
            CommandOutput {
                stdout: "a feat: add feature\n\
                         b test: add tests\n\
                         c fix: bug fix\n\
                         d refactor: cleanup\n"
                    .to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_var("CLAUDE_SESSION_ID", "test-session");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = craft_velocity("{}", &ports);
        assert_eq!(result.exit_code, 0);

        let velocity_path = Path::new("/home/test/.claude/metrics/craft-velocity.jsonl");
        let content = fs.read_to_string(velocity_path).unwrap();
        assert!(content.contains("\"forward\":2"));
        assert!(content.contains("\"rework\":1"));
        assert!(content.contains("\"neutral\":1"));
    }

    #[test]
    fn craft_velocity_passthrough_no_git() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = craft_velocity("{}", &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn craft_velocity_passthrough_empty_log() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on_args(
            "git",
            &["log", "--oneline", "-50"],
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = craft_velocity("{}", &ports);
        assert!(result.stderr.is_empty());
    }
}
