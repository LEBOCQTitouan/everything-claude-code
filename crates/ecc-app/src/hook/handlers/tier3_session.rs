//! Tier 3 Hooks — Session management and file I/O hooks.

use crate::hook::{HookPorts, HookResult};
use ecc_domain::time::{date_string, datetime_string, time_string};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// session-start: load previous context, detect project type.
pub fn session_start(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    let sessions_dir = home.join(".claude").join("sessions");
    let learned_dir = home.join(".claude").join("learned-skills");

    let _ = ports.fs.create_dir_all(&sessions_dir);
    let _ = ports.fs.create_dir_all(&learned_dir);

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
    let _ = ports.fs.create_dir_all(&sessions_dir);

    let today = date_string();
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
    let current_time = time_string();

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

            let _ = ports.fs.write(&session_file, &updated);
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

        let _ = ports.fs.write(&session_file, &template);
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
    let _ = ports.fs.create_dir_all(&sessions_dir);

    let compaction_log = sessions_dir.join("compaction-log.txt");
    let timestamp = datetime_string();

    // Append to compaction log
    let existing = ports
        .fs
        .read_to_string(&compaction_log)
        .unwrap_or_default();
    let new_content = format!("{}[{}] Context compaction triggered\n", existing, timestamp);
    let _ = ports.fs.write(&compaction_log, &new_content);

    // Append note to active session
    let session_files = find_files_by_suffix(&sessions_dir, "-session.tmp", ports);
    if let Some(active) = session_files.first()
        && let Ok(content) = ports.fs.read_to_string(active) {
            let time_str = time_string();
            let updated = format!(
                "{}\n---\n**[Compaction occurred at {}]** - Context was summarized\n",
                content, time_str
            );
            let _ = ports.fs.write(active, &updated);
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
    let _ = ports.fs.create_dir_all(&learned_dir);

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
    let _ = ports.fs.create_dir_all(&metrics_dir);

    let cost = estimate_cost(&model, input_tokens, output_tokens);
    let timestamp = datetime_string();

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
    let _ = ports.fs.write(&costs_file, &new_content);

    HookResult::passthrough(stdin)
}

// --- Helper functions ---

/// Find files in a directory ending with a specific suffix.
fn find_files_by_suffix(dir: &Path, suffix: &str, ports: &HookPorts<'_>) -> Vec<PathBuf> {
    match ports.fs.read_dir(dir) {
        Ok(entries) => {
            let mut files: Vec<PathBuf> = entries
                .into_iter()
                .filter(|p| {
                    p.to_string_lossy().ends_with(suffix)
                })
                .collect();
            files.sort();
            files.reverse(); // Most recent first (assuming date-prefixed names)
            files
        }
        Err(_) => Vec::new(),
    }
}

/// Count files with a specific extension in a directory.
fn count_files_with_ext(dir: &Path, ext: &str, ports: &HookPorts<'_>) -> usize {
    match ports.fs.read_dir(dir) {
        Ok(entries) => entries
            .iter()
            .filter(|p| p.to_string_lossy().ends_with(ext))
            .count(),
        Err(_) => 0,
    }
}

/// Append project detection info to stderr parts.
fn append_project_detection(parts: &mut Vec<String>, ports: &HookPorts<'_>) {
    let cwd = match ports.env.current_dir() {
        Some(d) => d,
        None => return,
    };

    let mut languages = Vec::new();
    let mut frameworks = Vec::new();

    // Detect languages by marker files
    if ports.fs.exists(&cwd.join("package.json")) {
        languages.push("typescript");
    }
    if ports.fs.exists(&cwd.join("Cargo.toml")) {
        languages.push("rust");
    }
    if ports.fs.exists(&cwd.join("go.mod")) {
        languages.push("go");
    }
    if ports.fs.exists(&cwd.join("pyproject.toml"))
        || ports.fs.exists(&cwd.join("setup.py"))
        || ports.fs.exists(&cwd.join("requirements.txt"))
    {
        languages.push("python");
    }

    // Detect frameworks
    if ports.fs.exists(&cwd.join("next.config.js"))
        || ports.fs.exists(&cwd.join("next.config.mjs"))
        || ports.fs.exists(&cwd.join("next.config.ts"))
    {
        frameworks.push("nextjs");
    }

    if languages.is_empty() && frameworks.is_empty() {
        parts.push("[SessionStart] No specific project type detected".to_string());
        return;
    }

    let mut info_parts = Vec::new();
    if !languages.is_empty() {
        info_parts.push(format!("languages: {}", languages.join(", ")));
    }
    if !frameworks.is_empty() {
        info_parts.push(format!("frameworks: {}", frameworks.join(", ")));
    }
    parts.push(format!(
        "[SessionStart] Project detected — {}",
        info_parts.join("; ")
    ));
}

/// Session summary extracted from transcript.
#[derive(Debug)]
struct SessionSummary {
    user_messages: Vec<String>,
    tools_used: Vec<String>,
    files_modified: Vec<String>,
    total_messages: usize,
}

/// Extract a session summary from JSONL transcript content.
fn extract_session_summary(content: &str) -> Option<SessionSummary> {
    let mut user_messages = Vec::new();
    let mut tools_used = BTreeSet::new();
    let mut files_modified = BTreeSet::new();

    for line in content.lines() {
        let entry: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Check for user messages
        let is_user = entry.get("type").and_then(|v| v.as_str()) == Some("user")
            || entry.get("role").and_then(|v| v.as_str()) == Some("user")
            || entry
                .get("message")
                .and_then(|m| m.get("role"))
                .and_then(|r| r.as_str())
                == Some("user");

        if is_user {
            let raw = entry
                .get("message")
                .and_then(|m| m.get("content"))
                .or_else(|| entry.get("content"));

            let text = match raw {
                Some(serde_json::Value::String(s)) => s.clone(),
                Some(serde_json::Value::Array(arr)) => arr
                    .iter()
                    .filter_map(|c| c.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join(" "),
                _ => String::new(),
            };

            let trimmed = text.trim().to_string();
            if !trimmed.is_empty() {
                let truncated = if trimmed.len() > 200 {
                    trimmed[..200].to_string()
                } else {
                    trimmed
                };
                user_messages.push(truncated);
            }
        }

        // Check for tool use
        let tool_name = entry
            .get("tool_name")
            .or_else(|| entry.get("name"))
            .and_then(|v| v.as_str());

        if let Some(name) = tool_name {
            tools_used.insert(name.to_string());
            let file_path = entry
                .get("tool_input")
                .or_else(|| entry.get("input"))
                .and_then(|ti| ti.get("file_path"))
                .and_then(|fp| fp.as_str());
            if let Some(fp) = file_path
                && (name == "Edit" || name == "Write") {
                    files_modified.insert(fp.to_string());
                }
        }

        // Check assistant message content blocks for tool use
        if let Some(blocks) = entry
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_array())
        {
            for block in blocks {
                if block.get("type").and_then(|t| t.as_str()) == Some("tool_use")
                    && let Some(name) = block.get("name").and_then(|n| n.as_str()) {
                        tools_used.insert(name.to_string());
                        if let Some(fp) =
                            block.get("input").and_then(|i| i.get("file_path")).and_then(|f| f.as_str())
                            && (name == "Edit" || name == "Write") {
                                files_modified.insert(fp.to_string());
                            }
                    }
            }
        }
    }

    if user_messages.is_empty() {
        return None;
    }

    let total = user_messages.len();
    Some(SessionSummary {
        user_messages: user_messages.into_iter().rev().take(10).collect::<Vec<_>>().into_iter().rev().collect(),
        tools_used: tools_used.into_iter().take(20).collect(),
        files_modified: files_modified.into_iter().take(30).collect(),
        total_messages: total,
    })
}

/// Build a markdown summary section from a SessionSummary.
fn build_summary_section(summary: &SessionSummary) -> String {
    let mut section = String::from("## Session Summary\n\n");
    section.push_str("### Tasks\n");
    for msg in &summary.user_messages {
        let escaped = msg.replace('\n', " ").replace('`', "\\`");
        section.push_str(&format!("- {}\n", escaped));
    }
    section.push('\n');

    if !summary.files_modified.is_empty() {
        section.push_str("### Files Modified\n");
        for f in &summary.files_modified {
            section.push_str(&format!("- {}\n", f));
        }
        section.push('\n');
    }

    if !summary.tools_used.is_empty() {
        section.push_str(&format!(
            "### Tools Used\n{}\n\n",
            summary.tools_used.join(", ")
        ));
    }

    section.push_str(&format!(
        "### Stats\n- Total user messages: {}\n",
        summary.total_messages
    ));

    section
}

/// Find the **Last Updated:** line in session content.
fn find_last_updated_line(content: &str) -> String {
    for line in content.lines() {
        if line.starts_with("**Last Updated:**") {
            return line.to_string();
        }
    }
    String::new()
}

/// Extract a numeric value from a JSON object.
fn to_u64(value: &serde_json::Value, key: &str) -> Option<u64> {
    value
        .get(key)
        .and_then(|v| v.as_u64().or_else(|| v.as_f64().map(|f| f as u64)))
}

/// Estimate cost based on model and token counts.
fn estimate_cost(model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
    let normalized = model.to_lowercase();
    let (in_rate, out_rate) = if normalized.contains("haiku") {
        (0.8, 4.0)
    } else if normalized.contains("opus") {
        (15.0, 75.0)
    } else {
        // Default to sonnet rates
        (3.0, 15.0)
    };

    let cost =
        (input_tokens as f64 / 1_000_000.0) * in_rate + (output_tokens as f64 / 1_000_000.0) * out_rate;
    (cost * 1_000_000.0).round() / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookPorts;
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

    // --- session_start ---

    #[test]
    fn session_start_detects_project() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/Cargo.toml", "[package]")
            .with_dir("/home/test/.claude/sessions")
            .with_dir("/home/test/.claude/learned-skills");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = session_start("", &ports);
        assert!(result.stderr.contains("rust"));
    }

    #[test]
    fn session_start_loads_previous_session() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/home/test/.claude/sessions/2026-03-14-abc12345-session.tmp",
                "# Previous work\nImplemented feature X",
            )
            .with_dir("/home/test/.claude/learned-skills");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = session_start("", &ports);
        assert!(result.stdout.contains("Previous session summary"));
    }

    #[test]
    fn session_start_basic_path() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/home/test/.claude/sessions")
            .with_dir("/home/test/.claude/learned-skills");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = session_start("stdin", &ports);
        assert_eq!(result.exit_code, 0);
    }

    // --- session_end ---

    #[test]
    fn session_end_creates_new_file() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/home/test/.claude/sessions");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_var("CLAUDE_SESSION_ID", "abc12345");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = session_end("{}", &ports);
        assert_eq!(result.exit_code, 0);

        // Verify session file was created
        let files = ports.fs.read_dir(Path::new("/home/test/.claude/sessions")).unwrap();
        let session_files: Vec<_> = files
            .iter()
            .filter(|f| f.to_string_lossy().ends_with("-session.tmp"))
            .collect();
        assert_eq!(session_files.len(), 1);
    }

    #[test]
    fn session_end_with_transcript() {
        let transcript = r#"{"type":"user","message":{"role":"user","content":"Fix the bug"}}
{"type":"tool_use","tool_name":"Edit","tool_input":{"file_path":"src/main.rs"}}"#;
        let fs = InMemoryFileSystem::new()
            .with_file("/tmp/transcript.jsonl", transcript)
            .with_dir("/home/test/.claude/sessions");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_var("CLAUDE_SESSION_ID", "test1234");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"transcript_path":"/tmp/transcript.jsonl"}"#;
        let result = session_end(stdin, &ports);
        assert_eq!(result.exit_code, 0);

        // Verify the session file contains summary
        let files = ports.fs.read_dir(Path::new("/home/test/.claude/sessions")).unwrap();
        let session_file = files
            .iter()
            .find(|f| f.to_string_lossy().ends_with("-session.tmp"))
            .unwrap();
        let content = ports.fs.read_to_string(session_file).unwrap();
        assert!(content.contains("Session Summary"));
        assert!(content.contains("Fix the bug"));
    }

    // --- pre_compact ---

    #[test]
    fn pre_compact_writes_log() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/home/test/.claude/sessions");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = pre_compact("data", &ports);
        assert!(result.stderr.contains("PreCompact"));

        let log = ports
            .fs
            .read_to_string(Path::new("/home/test/.claude/sessions/compaction-log.txt"))
            .unwrap();
        assert!(log.contains("Context compaction triggered"));
    }

    #[test]
    fn pre_compact_appends_to_active_session() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/home/test/.claude/sessions/2026-03-14-test-session.tmp",
                "# Session\nContent here",
            );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let _ = pre_compact("data", &ports);

        let content = ports
            .fs
            .read_to_string(Path::new(
                "/home/test/.claude/sessions/2026-03-14-test-session.tmp",
            ))
            .unwrap();
        assert!(content.contains("Compaction occurred"));
    }

    // --- evaluate_session ---

    #[test]
    fn evaluate_session_short_session() {
        let transcript = r#"{"type":"user","content":"hello"}
{"type":"user","content":"world"}"#;
        let fs = InMemoryFileSystem::new()
            .with_file("/tmp/transcript.jsonl", transcript);
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"transcript_path":"/tmp/transcript.jsonl"}"#;
        let result = evaluate_session(stdin, &ports);
        assert!(result.stderr.contains("too short"));
    }

    #[test]
    fn evaluate_session_long_enough() {
        let mut transcript_lines = Vec::new();
        for i in 0..15 {
            transcript_lines.push(format!(
                r#"{{"type":"user","content":"message {}"}}"#,
                i
            ));
        }
        let transcript = transcript_lines.join("\n");
        let fs = InMemoryFileSystem::new()
            .with_file("/tmp/transcript.jsonl", &transcript);
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"transcript_path":"/tmp/transcript.jsonl"}"#;
        let result = evaluate_session(stdin, &ports);
        assert!(result.stderr.contains("15 messages"));
        assert!(result.stderr.contains("extractable patterns"));
    }

    #[test]
    fn evaluate_session_no_transcript() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/test");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = evaluate_session("{}", &ports);
        assert!(result.stderr.is_empty());
    }

    // --- cost_tracker ---

    #[test]
    fn cost_tracker_writes_jsonl() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/home/test/.claude/metrics");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_var("CLAUDE_SESSION_ID", "test-session");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"model":"sonnet","usage":{"input_tokens":1000,"output_tokens":500}}"#;
        let result = cost_tracker(stdin, &ports);
        assert_eq!(result.exit_code, 0);

        let content = ports
            .fs
            .read_to_string(Path::new("/home/test/.claude/metrics/costs.jsonl"))
            .unwrap();
        assert!(content.contains("sonnet"));
        assert!(content.contains("estimated_cost_usd"));
    }

    #[test]
    fn cost_tracker_estimates_correctly() {
        // Sonnet: $3/M input, $15/M output
        let cost = estimate_cost("sonnet", 1_000_000, 1_000_000);
        assert!((cost - 18.0).abs() < 0.001);
    }

    #[test]
    fn cost_tracker_haiku_rates() {
        let cost = estimate_cost("haiku", 1_000_000, 1_000_000);
        assert!((cost - 4.8).abs() < 0.001);
    }

    #[test]
    fn cost_tracker_opus_rates() {
        let cost = estimate_cost("opus", 1_000_000, 1_000_000);
        assert!((cost - 90.0).abs() < 0.001);
    }

    // --- extract_session_summary ---

    #[test]
    fn extract_summary_from_transcript() {
        let content = r#"{"type":"user","message":{"role":"user","content":"Fix the bug"}}
{"type":"tool_use","tool_name":"Edit","tool_input":{"file_path":"src/main.rs"}}
{"type":"user","message":{"role":"user","content":"Now add tests"}}"#;

        let summary = extract_session_summary(content).unwrap();
        assert_eq!(summary.total_messages, 2);
        assert!(summary.tools_used.contains(&"Edit".to_string()));
        assert!(summary.files_modified.contains(&"src/main.rs".to_string()));
    }

    #[test]
    fn extract_summary_empty_returns_none() {
        assert!(extract_session_summary("").is_none());
    }

    #[test]
    fn build_summary_section_formats_correctly() {
        let summary = SessionSummary {
            user_messages: vec!["Fix bug".to_string()],
            tools_used: vec!["Edit".to_string(), "Read".to_string()],
            files_modified: vec!["src/main.rs".to_string()],
            total_messages: 1,
        };
        let section = build_summary_section(&summary);
        assert!(section.contains("## Session Summary"));
        assert!(section.contains("Fix bug"));
        assert!(section.contains("Edit, Read"));
        assert!(section.contains("src/main.rs"));
    }
}
