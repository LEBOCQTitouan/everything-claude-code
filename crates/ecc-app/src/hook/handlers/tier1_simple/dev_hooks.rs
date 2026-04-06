use crate::hook::{HookPorts, HookResult};

use super::helpers::{extract_command, extract_tool_output, regex_find_pr_url};

/// pre-bash-tmux-reminder: suggest tmux for long-running commands.
pub fn pre_bash_tmux_reminder(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "pre_bash_tmux_reminder", "executing handler");
    use ecc_ports::env::Platform;

    if ports.env.platform() == Platform::Windows {
        return HookResult::passthrough(stdin);
    }

    if ports.env.var("TMUX").is_some() {
        return HookResult::passthrough(stdin);
    }

    let cmd = extract_command(stdin);
    let long_running_patterns = [
        "npm install",
        "npm test",
        "pnpm install",
        "pnpm test",
        "yarn install",
        "yarn test",
        "bun install",
        "bun test",
        "cargo build",
        "make",
        "docker",
        "pytest",
        "vitest",
        "playwright",
    ];

    let has_long_running = long_running_patterns.iter().any(|pat| cmd.contains(pat));

    if has_long_running {
        let msg = "[Hook] Consider running in tmux for session persistence\n\
                   [Hook] tmux new -s dev  |  tmux attach -t dev\n";
        return HookResult::warn(stdin, msg);
    }

    HookResult::passthrough(stdin)
}

/// post-bash-pr-created: log PR URL after creation.
pub fn post_bash_pr_created(stdin: &str) -> HookResult {
    tracing::debug!(handler = "post_bash_pr_created", "executing handler");
    let cmd = extract_command(stdin);
    if !cmd.contains("gh") || !cmd.contains("pr") || !cmd.contains("create") {
        return HookResult::passthrough(stdin);
    }

    let output_str = extract_tool_output(stdin);

    // Match GitHub PR URL
    if let Some(caps) = regex_find_pr_url(&output_str) {
        let pr_url = &caps;
        // Extract repo and PR number
        let parts: Vec<&str> = pr_url
            .trim_start_matches("https://github.com/")
            .splitn(4, '/')
            .collect();
        if parts.len() >= 4 {
            let repo = format!("{}/{}", parts[0], parts[1]);
            let pr_num = parts[3];
            let msg = format!(
                "[Hook] PR created: {}\n[Hook] To review: gh pr review {} --repo {}\n",
                pr_url, pr_num, repo
            );
            return HookResult::warn(stdin, &msg);
        }
    }

    HookResult::passthrough(stdin)
}

/// post-bash-build-complete: log build completion.
pub fn post_bash_build_complete(stdin: &str) -> HookResult {
    tracing::debug!(handler = "post_bash_build_complete", "executing handler");
    let cmd = extract_command(stdin);
    let build_patterns = ["npm run build", "pnpm build", "yarn build"];
    if build_patterns.iter().any(|p| cmd.contains(p)) {
        let msg = "[Hook] Build completed - async analysis running in background\n";
        return HookResult::warn(stdin, msg);
    }
    HookResult::passthrough(stdin)
}

/// suggest-compact: suggest compaction at logical intervals.
pub fn suggest_compact(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "suggest_compact", "executing handler");
    let session_id = ports
        .env
        .var("CLAUDE_SESSION_ID")
        .unwrap_or_else(|| "default".to_string());
    let temp_dir = ports.env.temp_dir();
    let counter_file = temp_dir.join(format!("claude-tool-count-{}", session_id));

    let threshold: u64 = ports
        .env
        .var("COMPACT_THRESHOLD")
        .and_then(|v| v.parse().ok())
        .filter(|&v: &u64| v > 0 && v <= 10_000)
        .unwrap_or(50);

    // Read current count
    let count = match ports.fs.read_to_string(&counter_file) {
        Ok(s) => s
            .trim()
            .parse::<u64>()
            .ok()
            .filter(|&v| v > 0 && v <= 1_000_000)
            .map(|v| v + 1)
            .unwrap_or(1),
        Err(_) => 1,
    };

    // Write updated count
    if let Err(e) = ports.fs.write(&counter_file, &count.to_string()) {
        tracing::warn!("Cannot write compact counter: {}", e);
    }

    if count == threshold {
        let msg = format!(
            "[StrategicCompact] {} tool calls reached - consider /compact if transitioning phases\n",
            threshold
        );
        return HookResult::warn(stdin, &msg);
    }

    if count > threshold && (count - threshold).is_multiple_of(25) {
        let msg = format!(
            "[StrategicCompact] {} tool calls - good checkpoint for /compact if context is stale\n",
            count
        );
        return HookResult::warn(stdin, &msg);
    }

    HookResult::passthrough(stdin)
}

/// post:failure:error-context — Suggest recovery commands after tool failures.
///
/// Parses `tool_name` and `error` from stdin JSON. If error contains build/compile
/// or test failure keywords, emits a stderr hint. Otherwise passthrough.
pub fn post_failure_error_context(stdin: &str, _ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "post_failure_error_context", "executing handler");
    let parsed = serde_json::from_str::<serde_json::Value>(stdin).ok();

    let error = parsed
        .as_ref()
        .and_then(|v| v.get("error").and_then(|e| e.as_str()))
        .unwrap_or("");

    if error.is_empty() {
        return HookResult::passthrough(stdin);
    }

    let error_lower = error.to_lowercase();

    let build_keywords = [
        "build failed",
        "compile error",
        "compilation failed",
        "cannot find module",
        "syntax error",
        "type error",
    ];
    let test_keywords = [
        "test failed",
        "assertion failed",
        "expected",
        "test error",
        "failures:",
    ];

    if build_keywords.iter().any(|kw| error_lower.contains(kw)) {
        return HookResult::warn(
            stdin,
            "[Hook] Build failure detected. Consider /build-fix\n",
        );
    }

    if test_keywords.iter().any(|kw| error_lower.contains(kw)) {
        return HookResult::warn(
            stdin,
            "[Hook] Test failure. Check test isolation before retrying\n",
        );
    }

    HookResult::passthrough(stdin)
}

/// pre:prompt:context-inject — Inject context reminders before prompt processing.
///
/// Checks for uncommitted changes and emits a stderr reminder. Passthrough (never blocks).
pub fn pre_prompt_context_inject(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "pre_prompt_context_inject", "executing handler");
    let diff_output = ports
        .shell
        .run_command("git", &["diff", "--stat", "--cached"]);

    let unstaged_output = ports.shell.run_command("git", &["diff", "--stat"]);

    let cached_count = diff_output
        .as_ref()
        .ok()
        .filter(|o| o.success())
        .map(|o| o.stdout.lines().count().saturating_sub(1)) // last line is summary
        .unwrap_or(0);

    let unstaged_count = unstaged_output
        .as_ref()
        .ok()
        .filter(|o| o.success())
        .map(|o| o.stdout.lines().count().saturating_sub(1))
        .unwrap_or(0);

    let total = cached_count + unstaged_count;
    if total > 0 {
        let msg = format!(
            "[Hook] Reminder: {} uncommitted change(s) from previous response\n",
            total
        );
        return HookResult::warn(stdin, &msg);
    }

    HookResult::passthrough(stdin)
}

/// instructions:loaded:validate — Validate that the instructions file exists and has content.
///
/// Parses `instructions_path` from stdin JSON. Warns if file not found or empty.
pub fn instructions_loaded_validate(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(
        handler = "instructions_loaded_validate",
        "executing handler"
    );
    let path_str = serde_json::from_str::<serde_json::Value>(stdin)
        .ok()
        .and_then(|v| v.get("instructions_path")?.as_str().map(|s| s.to_string()));

    let path_str = match path_str {
        Some(p) => p,
        None => return HookResult::passthrough(stdin),
    };

    let path = std::path::Path::new(&path_str);

    if !ports.fs.exists(path) {
        return HookResult::warn(
            stdin,
            &format!("[Hook] Instructions file not found: {}\n", path_str),
        );
    }

    match ports.fs.read_to_string(path) {
        Ok(content) if content.trim().is_empty() => {
            HookResult::warn(stdin, "[Hook] Instructions file is empty\n")
        }
        Ok(content) if content.trim().len() < 10 => HookResult::warn(
            stdin,
            &format!(
                "[Hook] Instructions file suspiciously short ({} chars): {}\n",
                content.trim().len(),
                path_str
            ),
        ),
        Ok(_) => HookResult::passthrough(stdin),
        Err(_) => HookResult::warn(
            stdin,
            &format!("[Hook] Could not read instructions file: {}\n", path_str),
        ),
    }
}

/// post:exit-worktree:cleanup-reminder — Remind about unmerged changes after worktree removal.
///
/// Parses `tool_input.worktree_path` (fallback: `tool_input.name`, then `"unknown"`) from
/// PostToolUse stdin JSON.
pub fn post_exit_worktree_cleanup_reminder(stdin: &str, _ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(
        handler = "post_exit_worktree_cleanup_reminder",
        "executing handler"
    );
    let path = serde_json::from_str::<serde_json::Value>(stdin)
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

    let msg = format!(
        "[Hook] Worktree removed: {}. Check for unmerged changes.\n",
        path
    );
    HookResult::warn(stdin, &msg)
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
            cost_store: None,
            bypass_store: None,
            metrics_store: None,
        }
    }

    // --- post_bash_pr_created ---

    #[test]
    fn post_bash_pr_created_logs_pr_url_and_review_command() {
        let stdin = r#"{"tool_input":{"command":"gh pr create --title t"},"tool_output":{"output":"https://github.com/owner/repo/pull/99\n"}}"#;
        let result = post_bash_pr_created(stdin);
        assert!(result.stderr.contains("PR created"));
        assert!(result.stderr.contains("99"));
        assert!(result.stderr.contains("owner/repo"));
    }

    #[test]
    fn post_bash_pr_created_passthrough_non_pr_command() {
        let stdin = r#"{"tool_input":{"command":"echo hello"}}"#;
        let result = post_bash_pr_created(stdin);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn post_bash_pr_created_passthrough_pr_cmd_without_url_in_output() {
        let stdin =
            r#"{"tool_input":{"command":"gh pr create"},"tool_output":{"output":"Draft saved\n"}}"#;
        let result = post_bash_pr_created(stdin);
        assert!(result.stderr.is_empty());
    }

    // --- post_bash_build_complete ---

    #[test]
    fn post_bash_build_complete_warns_on_npm_run_build() {
        let stdin = r#"{"tool_input":{"command":"npm run build"}}"#;
        let result = post_bash_build_complete(stdin);
        assert!(result.stderr.contains("Build completed"));
    }

    #[test]
    fn post_bash_build_complete_warns_on_pnpm_build() {
        let stdin = r#"{"tool_input":{"command":"pnpm build"}}"#;
        let result = post_bash_build_complete(stdin);
        assert!(result.stderr.contains("Build completed"));
    }

    #[test]
    fn post_bash_build_complete_passthrough_for_non_build_command() {
        let stdin = r#"{"tool_input":{"command":"cargo test"}}"#;
        let result = post_bash_build_complete(stdin);
        assert!(result.stderr.is_empty());
    }

    // --- suggest_compact ---

    #[test]
    fn suggest_compact_no_warning_below_threshold() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("CLAUDE_SESSION_ID", "sess-a");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        // count=1, default threshold=50 → no warning
        let result = suggest_compact("{}", &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn suggest_compact_warns_at_threshold() {
        // Counter file holds "49" → next count = 50 = threshold
        let fs = InMemoryFileSystem::new().with_file("/tmp/claude-tool-count-sess-b", "49");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("CLAUDE_SESSION_ID", "sess-b");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = suggest_compact("{}", &ports);
        assert!(result.stderr.contains("50 tool calls reached"));
    }

    #[test]
    fn suggest_compact_warns_at_periodic_checkpoint() {
        // Counter holds "74" → next = 75 = threshold(50) + 25 → periodic reminder
        let fs = InMemoryFileSystem::new().with_file("/tmp/claude-tool-count-sess-c", "74");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("CLAUDE_SESSION_ID", "sess-c");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = suggest_compact("{}", &ports);
        assert!(result.stderr.contains("75 tool calls"));
        assert!(result.stderr.contains("checkpoint"));
    }

    #[test]
    fn suggest_compact_custom_threshold_triggers_at_that_count() {
        // COMPACT_THRESHOLD=5, counter = "4" → next = 5 → warning
        let fs = InMemoryFileSystem::new().with_file("/tmp/claude-tool-count-sess-d", "4");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_var("CLAUDE_SESSION_ID", "sess-d")
            .with_var("COMPACT_THRESHOLD", "5");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = suggest_compact("{}", &ports);
        assert!(result.stderr.contains("5 tool calls reached"));
    }

    // --- post_failure_error_context ---

    #[test]
    fn post_failure_build_error_hint() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_name":"Bash","error":"Build failed: cannot find module 'foo'"}"#;
        let result = post_failure_error_context(stdin, &ports);
        assert!(result.stderr.contains("Build failure detected"));
        assert!(result.stderr.contains("/build-fix"));
    }

    #[test]
    fn post_failure_test_error_hint() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_name":"Bash","error":"Test failed: assertion failed at line 42"}"#;
        let result = post_failure_error_context(stdin, &ports);
        assert!(result.stderr.contains("Test failure"));
        assert!(result.stderr.contains("test isolation"));
    }

    #[test]
    fn post_failure_generic_error_passthrough() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_name":"Bash","error":"Permission denied"}"#;
        let result = post_failure_error_context(stdin, &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn post_failure_missing_fields_passthrough() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = post_failure_error_context("{}", &ports);
        assert!(result.stderr.is_empty());
    }

    // --- pre_prompt_context_inject ---

    #[test]
    fn pre_prompt_with_uncommitted_changes() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new()
            .on_args(
                "git",
                &["diff", "--stat", "--cached"],
                ecc_ports::shell::CommandOutput {
                    stdout: " src/main.rs | 5 ++---\n 1 file changed\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            )
            .on_args(
                "git",
                &["diff", "--stat"],
                ecc_ports::shell::CommandOutput {
                    stdout: " src/lib.rs | 2 +-\n 1 file changed\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = pre_prompt_context_inject("{}", &ports);
        assert!(result.stderr.contains("uncommitted change(s)"));
    }

    #[test]
    fn pre_prompt_no_git_repo_passthrough() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new(); // git commands not registered
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = pre_prompt_context_inject("{}", &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn pre_prompt_no_uncommitted_changes() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new()
            .on_args(
                "git",
                &["diff", "--stat", "--cached"],
                ecc_ports::shell::CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            )
            .on_args(
                "git",
                &["diff", "--stat"],
                ecc_ports::shell::CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = pre_prompt_context_inject("{}", &ports);
        assert!(result.stderr.is_empty());
    }

    // --- instructions_loaded_validate ---

    #[test]
    fn instructions_loaded_missing_path_passthrough() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = instructions_loaded_validate("{}", &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn instructions_loaded_file_not_found_warns() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"instructions_path":"/tmp/missing.md"}"#;
        let result = instructions_loaded_validate(stdin, &ports);
        assert!(result.stderr.contains("not found"));
        assert!(result.stderr.contains("/tmp/missing.md"));
    }

    #[test]
    fn instructions_loaded_empty_file_warns() {
        let fs = InMemoryFileSystem::new().with_file("/tmp/instructions.md", "   ");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"instructions_path":"/tmp/instructions.md"}"#;
        let result = instructions_loaded_validate(stdin, &ports);
        assert!(result.stderr.contains("empty"));
    }

    #[test]
    fn instructions_loaded_tiny_file_warns() {
        let fs = InMemoryFileSystem::new().with_file("/tmp/instructions.md", "# Hi");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"instructions_path":"/tmp/instructions.md"}"#;
        let result = instructions_loaded_validate(stdin, &ports);
        assert!(result.stderr.contains("suspiciously short"));
    }

    #[test]
    fn instructions_loaded_normal_file_passthrough() {
        let fs = InMemoryFileSystem::new().with_file(
            "/tmp/instructions.md",
            "# Project\n\nThis is a valid instructions file with enough content.",
        );
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"instructions_path":"/tmp/instructions.md"}"#;
        let result = instructions_loaded_validate(stdin, &ports);
        assert!(result.stderr.is_empty());
    }

    // --- post_exit_worktree_cleanup_reminder (PostToolUse format) ---

    #[test]
    fn post_exit_worktree_cleanup_with_path() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin =
            r#"{"tool_name":"ExitWorktree","tool_input":{"worktree_path":"/tmp/wt-feature-x"}}"#;
        let result = post_exit_worktree_cleanup_reminder(stdin, &ports);
        assert!(result.stderr.contains("Worktree removed"));
        assert!(result.stderr.contains("/tmp/wt-feature-x"));
        assert!(result.stderr.contains("unmerged changes"));
    }

    #[test]
    fn post_exit_worktree_cleanup_without_path() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_name":"ExitWorktree","tool_input":{}}"#;
        let result = post_exit_worktree_cleanup_reminder(stdin, &ports);
        assert!(result.stderr.contains("Worktree removed"));
        assert!(result.stderr.contains("unmerged changes"));
    }
}
