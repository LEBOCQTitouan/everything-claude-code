//! Tier 3 Hooks — Session management and file I/O hooks.

mod helpers;

use log::warn;

use crate::hook::{HookPorts, HookResult};
use ecc_domain::time::{datetime_from_epoch, format_date, format_datetime, format_time};
use helpers::{
    append_project_detection, build_summary_section, count_files_with_ext, estimate_cost,
    extract_session_summary, find_files_by_suffix, find_last_updated_line, to_u64,
};
use std::path::Path;

/// Log a write failure and append the warning to stderr_parts if provided.
fn log_write_failure(path: &Path, err: &ecc_ports::fs::FsError, stderr_parts: Option<&mut Vec<String>>) {
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

/// session-start: load previous context, detect project type.
pub fn session_start(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    let sessions_dir = home.join(".claude").join("sessions");
    let learned_dir = home.join(".claude").join("learned-skills");

    if let Err(e) = ports.fs.create_dir_all(&sessions_dir) {
        warn!("Cannot create sessions dir: {}", e);
    }
    if let Err(e) = ports.fs.create_dir_all(&learned_dir) {
        warn!("Cannot create learned-skills dir: {}", e);
    }

    let mut stderr_parts: Vec<String> = Vec::new();

    // Find recent session files
    let session_files = find_files_by_suffix(&sessions_dir, "-session.tmp", ports);
    if !session_files.is_empty() {
        stderr_parts.push(format!(
            "[SessionStart] Found {} recent session(s)",
            session_files.len()
        ));

        // Try to load latest session content
        if let Some(latest) = session_files.first() {
            stderr_parts.push(format!("[SessionStart] Latest: {}", latest.display()));
            if let Ok(content) = ports.fs.read_to_string(latest)
                && !content.contains("[Session context goes here]") && !content.trim().is_empty() {
                    // Output previous session summary to stdout
                    let mut out = format!("Previous session summary:\n{}", content);
                    if !stdin.is_empty() {
                        out = format!("{}\n{}", out, stdin);
                    }
                    // Load learned skills count
                    let learned_count = count_files_with_ext(&learned_dir, ".md", ports);
                    if learned_count > 0 {
                        stderr_parts.push(format!(
                            "[SessionStart] {} learned skill(s) available in {}",
                            learned_count,
                            learned_dir.display()
                        ));
                    }

                    // Detect project type
                    append_project_detection(&mut stderr_parts, ports);

                    return HookResult {
                        stdout: out,
                        stderr: format!("{}\n", stderr_parts.join("\n")),
                        exit_code: 0,
                    };
                }
        }
    }

    // Load learned skills count
    let learned_count = count_files_with_ext(&learned_dir, ".md", ports);
    if learned_count > 0 {
        stderr_parts.push(format!(
            "[SessionStart] {} learned skill(s) available in {}",
            learned_count,
            learned_dir.display()
        ));
    }

    // Detect project type
    append_project_detection(&mut stderr_parts, ports);

    if stderr_parts.is_empty() {
        return HookResult::passthrough(stdin);
    }

    HookResult {
        stdout: stdin.to_string(),
        stderr: format!("{}\n", stderr_parts.join("\n")),
        exit_code: 0,
    }
}

/// session-end: persist session summary from transcript.
pub fn session_end(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    // Parse transcript_path from stdin JSON
    let transcript_path = serde_json::from_str::<serde_json::Value>(stdin)
        .ok()
        .and_then(|v| v.get("transcript_path")?.as_str().map(|s| s.to_string()))
        .or_else(|| ports.env.var("CLAUDE_TRANSCRIPT_PATH"));

    let sessions_dir = home.join(".claude").join("sessions");
    if let Err(e) = ports.fs.create_dir_all(&sessions_dir) {
        warn!("Cannot create sessions dir: {}", e);
    }

    let today = format_date(&datetime_from_epoch(epoch_secs()));
    let short_id = ports
        .env
        .var("CLAUDE_SESSION_ID")
        .map(|s| {
            if s.len() > 8 {
                s[..8].to_string()
            } else {
                s
            }
        })
        .unwrap_or_else(|| "unknown".to_string());
    let session_file = sessions_dir.join(format!("{}-{}-session.tmp", today, short_id));
    let current_time = format_time(&datetime_from_epoch(epoch_secs()));

    // Try to extract summary from transcript
    let summary = transcript_path
        .as_deref()
        .and_then(|tp| {
            let path = Path::new(tp);
            if ports.fs.exists(path) {
                ports
                    .fs
                    .read_to_string(path)
                    .ok()
                    .and_then(|content| extract_session_summary(&content))
            } else {
                None
            }
        });

    if ports.fs.exists(&session_file) {
        // Update existing session file
        if let Ok(existing) = ports.fs.read_to_string(&session_file) {
            let mut updated = existing.replace(
                &find_last_updated_line(&existing),
                &format!("**Last Updated:** {}", current_time),
            );

            if let Some(ref summary) = summary {
                let summary_section = build_summary_section(summary);
                // Replace existing summary section
                if let Some(pos) = updated.find("## Session Summary") {
                    updated.truncate(pos);
                    updated.push_str(&summary_section);
                } else if let Some(pos) = updated.find("## Current State") {
                    updated.truncate(pos);
                    updated.push_str(&summary_section);
                } else {
                    updated.push_str(&summary_section);
                }
            }

            if let Err(e) = ports.fs.write(&session_file, &updated) {
                let msg = format!("[Warning] Failed to write session: {}", e);
                warn!("{}", msg);
                return HookResult::warn(stdin, &format!("{msg}\n"));
            }
        }
    } else {
        // Create new session file
        let summary_section = if let Some(ref summary) = summary {
            build_summary_section(summary)
        } else {
            "## Current State\n\n[Session context goes here]\n\n\
             ### Completed\n- [ ]\n\n\
             ### In Progress\n- [ ]\n\n\
             ### Notes for Next Session\n-\n\n\
             ### Context to Load\n```\n[relevant files]\n```"
                .to_string()
        };

        let template = format!(
            "# Session: {today}\n\
             **Date:** {today}\n\
             **Started:** {time}\n\
             **Last Updated:** {time}\n\n\
             ---\n\n\
             {summary}\n",
            today = today,
            time = current_time,
            summary = summary_section,
        );

        if let Err(e) = ports.fs.write(&session_file, &template) {
            let msg = format!("[Warning] Failed to write session: {}", e);
            warn!("{}", msg);
            return HookResult::warn(stdin, &format!("{msg}\n"));
        }
    }

    HookResult::passthrough(stdin)
}

/// pre-compact: save state before context compaction.
pub fn pre_compact(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
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
    let existing = ports
        .fs
        .read_to_string(&compaction_log)
        .unwrap_or_default();
    let new_content = format!("{}[{}] Context compaction triggered\n", existing, timestamp);
    if let Err(e) = ports.fs.write(&compaction_log, &new_content) {
        log_write_failure(&compaction_log, &e, None);
    }

    // Append note to active session
    let session_files = find_files_by_suffix(&sessions_dir, "-session.tmp", ports);
    if let Some(active) = session_files.first()
        && let Ok(content) = ports.fs.read_to_string(active) {
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

/// evaluate-session: count messages and log evaluation hint.
pub fn evaluate_session(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    // Parse transcript_path from stdin JSON
    let transcript_path = serde_json::from_str::<serde_json::Value>(stdin)
        .ok()
        .and_then(|v| v.get("transcript_path")?.as_str().map(|s| s.to_string()))
        .or_else(|| ports.env.var("CLAUDE_TRANSCRIPT_PATH"));

    let transcript_path = match transcript_path {
        Some(tp) => tp,
        None => return HookResult::passthrough(stdin),
    };

    let path = Path::new(&transcript_path);
    if !ports.fs.exists(path) {
        return HookResult::passthrough(stdin);
    }

    let content = match ports.fs.read_to_string(path) {
        Ok(c) => c,
        Err(_) => return HookResult::passthrough(stdin),
    };

    // Count user messages
    let message_count = content
        .lines()
        .filter(|line| line.contains("\"type\"") && line.contains("\"user\""))
        .count();

    let min_session_length: usize = 10;

    if message_count < min_session_length {
        let msg = format!(
            "[ContinuousLearning] Session too short ({} messages), skipping\n",
            message_count
        );
        return HookResult::warn(stdin, &msg);
    }

    let learned_dir = home.join(".claude").join("learned-skills");
    if let Err(e) = ports.fs.create_dir_all(&learned_dir) {
        warn!("Cannot create learned-skills dir: {}", e);
    }

    let msg = format!(
        "[ContinuousLearning] Session has {} messages - evaluate for extractable patterns\n\
         [ContinuousLearning] Save learned skills to: {}\n",
        message_count,
        learned_dir.display()
    );
    HookResult::warn(stdin, &msg)
}

/// cost-tracker: estimate cost and append JSONL metrics.
pub fn cost_tracker(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    let input: serde_json::Value = match serde_json::from_str(stdin) {
        Ok(v) => v,
        Err(_) => return HookResult::passthrough(stdin),
    };

    let usage = input
        .get("usage")
        .or_else(|| input.get("token_usage"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let input_tokens = to_u64(&usage, "input_tokens")
        .or_else(|| to_u64(&usage, "prompt_tokens"))
        .unwrap_or(0);
    let output_tokens = to_u64(&usage, "output_tokens")
        .or_else(|| to_u64(&usage, "completion_tokens"))
        .unwrap_or(0);

    let model = input
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| {
            ports
                .env
                .var("CLAUDE_MODEL")
                .as_deref()
                .unwrap_or("unknown")
                // Can't return a reference to a local, so just use "unknown"
                ;
            "unknown"
        })
        .to_string();

    let session_id = ports
        .env
        .var("CLAUDE_SESSION_ID")
        .unwrap_or_else(|| "default".to_string());

    let metrics_dir = home.join(".claude").join("metrics");
    if let Err(e) = ports.fs.create_dir_all(&metrics_dir) {
        warn!("Cannot create metrics dir: {}", e);
    }

    let cost = estimate_cost(&model, input_tokens, output_tokens);
    let timestamp = format_datetime(&datetime_from_epoch(epoch_secs()));

    let row = serde_json::json!({
        "timestamp": timestamp,
        "session_id": session_id,
        "model": model,
        "input_tokens": input_tokens,
        "output_tokens": output_tokens,
        "estimated_cost_usd": cost,
    });

    let costs_file = metrics_dir.join("costs.jsonl");
    let existing = ports.fs.read_to_string(&costs_file).unwrap_or_default();
    let new_content = format!("{}{}\n", existing, row);
    if let Err(e) = ports.fs.write(&costs_file, &new_content) {
        log_write_failure(&costs_file, &e, None);
    }

    HookResult::passthrough(stdin)
}

/// stop:oath-reflection — summarize session against Programmer's Oath.
pub fn oath_reflection(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    // Parse recent git log to count commit types
    let log_output = ports
        .shell
        .run_command("git", &["log", "--oneline", "-50"]);
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
    let oath_notes_exist = ports
        .fs
        .exists(Path::new("docs/audits/robert-notes.md"));
    let oath_summary = if oath_notes_exist {
        match ports.fs.read_to_string(Path::new("docs/audits/robert-notes.md")) {
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
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    let log_output = ports
        .shell
        .run_command("git", &["log", "--oneline", "-50"]);
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

        if msg.starts_with("feat:") || msg.starts_with("feat(") || msg.starts_with("test:") || msg.starts_with("test(") || msg.starts_with("docs:") || msg.starts_with("docs(") || msg.starts_with("chore(scout)") {
            forward += 1;
        } else if msg.starts_with("fix:") || msg.starts_with("fix(") || msg.starts_with("chore:") || msg.starts_with("chore(") {
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

        // Verify JSONL was written
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
