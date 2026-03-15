use crate::hook::{HookPorts, HookResult};

use super::helpers::{extract_command, extract_tool_output, regex_find_pr_url};

/// pre-bash-tmux-reminder: suggest tmux for long-running commands.
pub fn pre_bash_tmux_reminder(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
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

    let has_long_running = long_running_patterns
        .iter()
        .any(|pat| cmd.contains(pat));

    if has_long_running {
        let msg = "[Hook] Consider running in tmux for session persistence\n\
                   [Hook] tmux new -s dev  |  tmux attach -t dev\n";
        return HookResult::warn(stdin, msg);
    }

    HookResult::passthrough(stdin)
}

/// post-bash-pr-created: log PR URL after creation.
pub fn post_bash_pr_created(stdin: &str) -> HookResult {
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
        log::warn!("Cannot write compact counter: {}", e);
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
        let stdin = r#"{"tool_input":{"command":"gh pr create"},"tool_output":{"output":"Draft saved\n"}}"#;
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
        let fs = InMemoryFileSystem::new()
            .with_file("/tmp/claude-tool-count-sess-b", "49");
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
        let fs = InMemoryFileSystem::new()
            .with_file("/tmp/claude-tool-count-sess-c", "74");
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
        let fs = InMemoryFileSystem::new()
            .with_file("/tmp/claude-tool-count-sess-d", "4");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_var("CLAUDE_SESSION_ID", "sess-d")
            .with_var("COMPACT_THRESHOLD", "5");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = suggest_compact("{}", &ports);
        assert!(result.stderr.contains("5 tool calls reached"));
    }
}
