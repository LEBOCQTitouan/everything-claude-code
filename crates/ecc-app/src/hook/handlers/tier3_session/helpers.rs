use crate::hook::HookPorts;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Find files in a directory ending with a specific suffix.
pub(super) fn find_files_by_suffix(
    dir: &Path,
    suffix: &str,
    ports: &HookPorts<'_>,
) -> Vec<PathBuf> {
    match ports.fs.read_dir(dir) {
        Ok(entries) => {
            let mut files: Vec<PathBuf> = entries
                .into_iter()
                .filter(|p| p.to_string_lossy().ends_with(suffix))
                .collect();
            files.sort();
            files.reverse(); // Most recent first (assuming date-prefixed names)
            files
        }
        Err(e) => {
            tracing::warn!("find_files_by_suffix: cannot read {}: {e}", dir.display());
            Vec::new()
        }
    }
}

/// Count files with a specific extension in a directory.
pub(super) fn count_files_with_ext(dir: &Path, ext: &str, ports: &HookPorts<'_>) -> usize {
    match ports.fs.read_dir(dir) {
        Ok(entries) => entries
            .iter()
            .filter(|p| p.to_string_lossy().ends_with(ext))
            .count(),
        Err(e) => {
            tracing::warn!("count_files_with_ext: cannot read {}: {e}", dir.display());
            0
        }
    }
}

/// Append project detection info to stderr parts.
pub(super) fn append_project_detection(parts: &mut Vec<String>, ports: &HookPorts<'_>) {
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
pub(super) struct SessionSummary {
    pub user_messages: Vec<String>,
    pub tools_used: Vec<String>,
    pub files_modified: Vec<String>,
    pub total_messages: usize,
}

/// Extract a session summary from JSONL transcript content.
pub(super) fn extract_session_summary(content: &str) -> Option<SessionSummary> {
    let mut user_messages = Vec::new();
    let mut tools_used = BTreeSet::new();
    let mut files_modified = BTreeSet::new();

    for line in content.lines() {
        let entry: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("extract_session_summary: skipping malformed JSON line: {e}");
                continue;
            }
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
                && (name == "Edit" || name == "Write")
            {
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
                    && let Some(name) = block.get("name").and_then(|n| n.as_str())
                {
                    tools_used.insert(name.to_string());
                    if let Some(fp) = block
                        .get("input")
                        .and_then(|i| i.get("file_path"))
                        .and_then(|f| f.as_str())
                        && (name == "Edit" || name == "Write")
                    {
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
        user_messages: user_messages
            .into_iter()
            .rev()
            .take(10)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect(),
        tools_used: tools_used.into_iter().take(20).collect(),
        files_modified: files_modified.into_iter().take(30).collect(),
        total_messages: total,
    })
}

/// Build a markdown summary section from a SessionSummary.
pub(super) fn build_summary_section(summary: &SessionSummary) -> String {
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
pub(super) fn find_last_updated_line(content: &str) -> String {
    for line in content.lines() {
        if line.starts_with("**Last Updated:**") {
            return line.to_string();
        }
    }
    String::new()
}

/// Extract a numeric value from a JSON object.
pub(super) fn to_u64(value: &serde_json::Value, key: &str) -> Option<u64> {
    value
        .get(key)
        .and_then(|v| v.as_u64().or_else(|| v.as_f64().map(|f| f as u64)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookPorts;
    use crate::hook::handlers::tier3_session::{
        cost_tracker, evaluate_session, pre_compact, session_end, session_start,
    };
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
        let env = MockEnvironment::new().with_home("/home/test");
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
        let fs = InMemoryFileSystem::new().with_dir("/home/test/.claude/sessions");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_home("/home/test")
            .with_var("CLAUDE_SESSION_ID", "abc12345");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = session_end("{}", &ports);
        assert_eq!(result.exit_code, 0);

        // Verify session file was created
        let files = ports
            .fs
            .read_dir(Path::new("/home/test/.claude/sessions"))
            .unwrap();
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
        let files = ports
            .fs
            .read_dir(Path::new("/home/test/.claude/sessions"))
            .unwrap();
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
        let fs = InMemoryFileSystem::new().with_dir("/home/test/.claude/sessions");
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
        let fs = InMemoryFileSystem::new().with_file(
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
        let fs = InMemoryFileSystem::new().with_file("/tmp/transcript.jsonl", transcript);
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
            transcript_lines.push(format!(r#"{{"type":"user","content":"message {}"}}"#, i));
        }
        let transcript = transcript_lines.join("\n");
        let fs = InMemoryFileSystem::new().with_file("/tmp/transcript.jsonl", &transcript);
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
        let fs = InMemoryFileSystem::new().with_dir("/home/test/.claude/metrics");
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
