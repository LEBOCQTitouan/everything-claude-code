use crate::hook::{HookPorts, HookResult};
use ecc_domain::hook_runtime::profiles::{is_hook_enabled, HookEnabledOptions};
use std::path::Path;

use super::helpers::{
    extract_command, extract_file_path, extract_tool_output, regex_find_pr_url,
    scan_exports,
};

/// check-hook-enabled: returns "yes" or "no" based on profile.
pub fn check_hook_enabled(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    // This hook checks if a *different* hook is enabled.
    // The hook_id to check comes from stdin (JSON with hook_id field) or is just the raw stdin.
    let check_id = match serde_json::from_str::<serde_json::Value>(stdin) {
        Ok(v) => v
            .get("hook_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        Err(_) => stdin.trim().to_string(),
    };

    let profile_env = ports.env.var("ECC_HOOK_PROFILE");
    let disabled_env = ports.env.var("ECC_DISABLED_HOOKS");
    let opts = HookEnabledOptions::default();

    let enabled = if check_id.is_empty() {
        true
    } else {
        is_hook_enabled(
            &check_id,
            profile_env.as_deref(),
            disabled_env.as_deref(),
            &opts,
        )
    };

    HookResult {
        stdout: if enabled { "yes" } else { "no" }.to_string(),
        stderr: String::new(),
        exit_code: 0,
    }
}

/// session-end-marker: passthrough stdin (lifecycle marker, non-blocking).
pub fn session_end_marker(stdin: &str, _ports: &HookPorts<'_>) -> HookResult {
    HookResult::passthrough(stdin)
}

/// check-console-log: check modified git files for console.log.
pub fn check_console_log(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    let is_git = ports
        .shell
        .run_command("git", &["rev-parse", "--git-dir"]);
    if is_git.is_err() {
        return HookResult::passthrough(stdin);
    }

    let status = ports
        .shell
        .run_command("git", &["diff", "--name-only", "--diff-filter=ACMR"]);
    let files = match status {
        Ok(ref out) if out.success() => out.stdout.clone(),
        _ => return HookResult::passthrough(stdin),
    };

    let excluded = [
        ".test.", ".spec.", ".config.", "scripts/", "__tests__/", "__mocks__/",
    ];

    let mut warnings = Vec::new();

    for file in files.lines() {
        let file = file.trim();
        if file.is_empty() {
            continue;
        }
        if !file.ends_with(".ts")
            && !file.ends_with(".tsx")
            && !file.ends_with(".js")
            && !file.ends_with(".jsx")
        {
            continue;
        }
        if excluded.iter().any(|pat| file.contains(pat)) {
            continue;
        }

        let path = Path::new(file);
        if let Ok(content) = ports.fs.read_to_string(path)
            && content.contains("console.log") {
                warnings.push(format!("[Hook] WARNING: console.log found in {}", file));
            }
    }

    if warnings.is_empty() {
        return HookResult::passthrough(stdin);
    }

    warnings.push("[Hook] Remove console.log statements before committing".to_string());
    HookResult::warn(stdin, &format!("{}\n", warnings.join("\n")))
}

/// stop-uncommitted-reminder: warn about uncommitted changes.
pub fn stop_uncommitted_reminder(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    let is_git = ports
        .shell
        .run_command("git", &["rev-parse", "--git-dir"]);
    if is_git.is_err() {
        return HookResult::passthrough(stdin);
    }

    let status = ports.shell.run_command("git", &["status", "--porcelain"]);
    let output = match status {
        Ok(ref out) if out.success() => &out.stdout,
        _ => return HookResult::passthrough(stdin),
    };

    let lines: Vec<&str> = output.lines().filter(|l| !l.is_empty()).collect();
    if lines.is_empty() {
        return HookResult::passthrough(stdin);
    }

    let staged = lines
        .iter()
        .filter(|l| {
            let bytes = l.as_bytes();
            !bytes.is_empty() && matches!(bytes[0], b'M' | b'A' | b'D' | b'R' | b'C')
        })
        .count();
    let unstaged = lines.len().saturating_sub(staged);

    let mut msg = String::from("[Hook] REMINDER: You have uncommitted changes.\n");
    if staged > 0 {
        msg.push_str(&format!("[Hook]   Staged: {} file(s)\n", staged));
    }
    if unstaged > 0 {
        msg.push_str(&format!(
            "[Hook]   Unstaged/untracked: {} file(s)\n",
            unstaged
        ));
    }
    msg.push_str("[Hook]   Commit each logical change separately for version history.\n");
    msg.push_str("[Hook]   See: skill atomic-commits, rule git-workflow.md\n");

    HookResult::warn(stdin, &msg)
}

/// pre-bash-git-push-reminder: warn before git push.
pub fn pre_bash_git_push_reminder(stdin: &str) -> HookResult {
    let cmd = extract_command(stdin);
    if cmd.contains("git") && cmd.contains("push") {
        let msg = "[Hook] Review changes before push...\n\
                   [Hook] Continuing with push (remove this hook to add interactive review)\n";
        return HookResult::warn(stdin, msg);
    }
    HookResult::passthrough(stdin)
}

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

/// doc-file-warning: warn about non-standard documentation files.
pub fn doc_file_warning(stdin: &str) -> HookResult {
    let file_path = extract_file_path(stdin);
    if file_path.is_empty() {
        return HookResult::passthrough(stdin);
    }

    // Only check .md and .txt files
    if !file_path.ends_with(".md") && !file_path.ends_with(".txt") {
        return HookResult::passthrough(stdin);
    }

    // Allow standard doc files
    let basename = Path::new(&file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let standard_files = [
        "README.md",
        "CLAUDE.md",
        "AGENTS.md",
        "CONTRIBUTING.md",
        "CHANGELOG.md",
        "LICENSE.md",
        "SKILL.md",
    ];
    let basename_upper = basename.to_uppercase();
    if standard_files
        .iter()
        .any(|s| s.to_uppercase() == basename_upper)
    {
        return HookResult::passthrough(stdin);
    }

    // Allow paths in known directories
    let normalized = file_path.replace('\\', "/");
    if normalized.contains(".claude/plans/")
        || normalized.contains("/docs/")
        || normalized.starts_with("docs/")
        || normalized.contains("/skills/")
        || normalized.starts_with("skills/")
        || normalized.contains("/.history/")
    {
        return HookResult::passthrough(stdin);
    }

    let msg = format!(
        "[Hook] WARNING: Non-standard documentation file detected\n\
         [Hook] File: {}\n\
         [Hook] Consider consolidating into README.md or docs/ directory\n",
        file_path
    );
    HookResult::warn(stdin, &msg)
}

/// doc-coverage-reminder: remind about undocumented exports.
pub fn doc_coverage_reminder(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    let file_path = extract_file_path(stdin);
    if file_path.is_empty() {
        return HookResult::passthrough(stdin);
    }

    let ext = Path::new(&file_path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let source_exts = [
        "ts", "tsx", "js", "jsx", "py", "go", "rs", "java",
    ];
    if !source_exts.contains(&ext.as_str()) {
        return HookResult::passthrough(stdin);
    }

    let skip_patterns = [
        "/node_modules/",
        "/dist/",
        "/build/",
        "/.",
        "/vendor/",
        "/__pycache__/",
    ];
    if skip_patterns.iter().any(|p| file_path.contains(p)) {
        return HookResult::passthrough(stdin);
    }

    let path = Path::new(&file_path);
    let content = match ports.fs.read_to_string(path) {
        Ok(c) => c,
        Err(_) => return HookResult::passthrough(stdin),
    };

    let (total, undocumented) = scan_exports(&content, &ext);
    if total > 0 && undocumented > 0 {
        let basename = Path::new(&file_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let msg = format!(
            "[DocCoverage] {}: {}/{} exported items lack doc comments. \
             Run /doc-generate --comments-only to add them.\n",
            basename, undocumented, total
        );
        return HookResult::warn(stdin, &msg);
    }

    HookResult::passthrough(stdin)
}

/// post-edit-console-warn: warn about console.log after edits.
pub fn post_edit_console_warn(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    let file_path = extract_file_path(stdin);
    if file_path.is_empty() {
        return HookResult::passthrough(stdin);
    }

    let ext = Path::new(&file_path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if !matches!(ext.as_str(), "ts" | "tsx" | "js" | "jsx") {
        return HookResult::passthrough(stdin);
    }

    let path = Path::new(&file_path);
    let content = match ports.fs.read_to_string(path) {
        Ok(c) => c,
        Err(_) => return HookResult::passthrough(stdin),
    };

    let matches: Vec<String> = content
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("console.log"))
        .take(5)
        .map(|(idx, line)| format!("{}: {}", idx + 1, line.trim()))
        .collect();

    if matches.is_empty() {
        return HookResult::passthrough(stdin);
    }

    let mut msg = format!("[Hook] WARNING: console.log found in {}\n", file_path);
    for m in &matches {
        msg.push_str(&format!("{}\n", m));
    }
    msg.push_str("[Hook] Remove console.log before committing\n");

    HookResult::warn(stdin, &msg)
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

    // --- check_hook_enabled ---

    #[test]
    fn check_hook_enabled_enabled_hook_returns_yes() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = check_hook_enabled("some-hook", &ports);
        assert_eq!(result.stdout, "yes");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn check_hook_enabled_disabled_hook_returns_no() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("ECC_DISABLED_HOOKS", "some-hook");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = check_hook_enabled("some-hook", &ports);
        assert_eq!(result.stdout, "no");
    }

    #[test]
    fn check_hook_enabled_empty_stdin_always_returns_yes() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("ECC_DISABLED_HOOKS", "some-hook");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        // empty check_id → always enabled regardless of disabled list
        let result = check_hook_enabled("", &ports);
        assert_eq!(result.stdout, "yes");
    }

    #[test]
    fn check_hook_enabled_parses_json_hook_id() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("ECC_DISABLED_HOOKS", "json-hook");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"hook_id":"json-hook"}"#;
        let result = check_hook_enabled(stdin, &ports);
        assert_eq!(result.stdout, "no");
    }

    // --- session_end_marker ---

    #[test]
    fn session_end_marker_passthrough() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = session_end_marker("session data", &ports);
        assert_eq!(result.stdout, "session data");
        assert!(result.stderr.is_empty());
        assert_eq!(result.exit_code, 0);
    }

    // --- check_console_log ---

    #[test]
    fn check_console_log_passthrough_when_not_git_repo() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new(); // no responses → run_command returns Err
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = check_console_log("stdin", &ports);
        assert_eq!(result.stdout, "stdin");
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn check_console_log_warns_on_console_log_in_ts_file() {
        let fs = InMemoryFileSystem::new()
            .with_file("src/app.ts", "const x = 1;\nconsole.log(x);\n");
        let shell = MockExecutor::new()
            .on_args(
                "git",
                &["rev-parse", "--git-dir"],
                CommandOutput {
                    stdout: ".git".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            )
            .on_args(
                "git",
                &["diff", "--name-only", "--diff-filter=ACMR"],
                CommandOutput {
                    stdout: "src/app.ts\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = check_console_log("stdin", &ports);
        assert!(result.stderr.contains("console.log found in src/app.ts"));
        assert!(result.stderr.contains("Remove console.log"));
    }

    #[test]
    fn check_console_log_skips_spec_files() {
        let fs = InMemoryFileSystem::new()
            .with_file("src/app.spec.ts", "console.log('test');");
        let shell = MockExecutor::new()
            .on_args(
                "git",
                &["rev-parse", "--git-dir"],
                CommandOutput {
                    stdout: ".git".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            )
            .on_args(
                "git",
                &["diff", "--name-only", "--diff-filter=ACMR"],
                CommandOutput {
                    stdout: "src/app.spec.ts\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = check_console_log("stdin", &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn check_console_log_passthrough_when_no_console_log_present() {
        let fs = InMemoryFileSystem::new()
            .with_file("src/app.ts", "const x = 1;\n");
        let shell = MockExecutor::new()
            .on_args(
                "git",
                &["rev-parse", "--git-dir"],
                CommandOutput {
                    stdout: ".git".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            )
            .on_args(
                "git",
                &["diff", "--name-only", "--diff-filter=ACMR"],
                CommandOutput {
                    stdout: "src/app.ts\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = check_console_log("stdin", &ports);
        assert!(result.stderr.is_empty());
    }

    // --- stop_uncommitted_reminder ---

    #[test]
    fn stop_uncommitted_reminder_passthrough_when_not_git() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = stop_uncommitted_reminder("stdin", &ports);
        assert_eq!(result.stdout, "stdin");
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn stop_uncommitted_reminder_passthrough_on_clean_repo() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new()
            .on_args(
                "git",
                &["rev-parse", "--git-dir"],
                CommandOutput {
                    stdout: ".git".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            )
            .on_args(
                "git",
                &["status", "--porcelain"],
                CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = stop_uncommitted_reminder("stdin", &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn stop_uncommitted_reminder_warns_with_staged_and_unstaged() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new()
            .on_args(
                "git",
                &["rev-parse", "--git-dir"],
                CommandOutput {
                    stdout: ".git".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            )
            .on_args(
                "git",
                &["status", "--porcelain"],
                CommandOutput {
                    // M at col 0 = staged, ?? = untracked
                    stdout: "M  src/lib.rs\n?? new.txt\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = stop_uncommitted_reminder("stdin", &ports);
        assert!(result.stderr.contains("uncommitted changes"));
        assert!(result.stderr.contains("Staged: 1"));
        assert!(result.stderr.contains("Unstaged/untracked: 1"));
    }

    // --- pre_bash_git_push_reminder ---

    #[test]
    fn pre_bash_git_push_reminder_warns_on_git_push() {
        let stdin = r#"{"tool_input":{"command":"git push origin main"}}"#;
        let result = pre_bash_git_push_reminder(stdin);
        assert!(result.stderr.contains("Review changes before push"));
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn pre_bash_git_push_reminder_passthrough_for_other_git_commands() {
        let stdin = r#"{"tool_input":{"command":"git commit -m 'msg'"}}"#;
        let result = pre_bash_git_push_reminder(stdin);
        assert!(result.stderr.is_empty());
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

    // --- doc_file_warning ---

    #[test]
    fn doc_file_warning_warns_for_non_standard_md() {
        let stdin = r#"{"tool_input":{"file_path":"scratch.md"}}"#;
        let result = doc_file_warning(stdin);
        assert!(result.stderr.contains("Non-standard documentation"));
        assert!(result.stderr.contains("scratch.md"));
    }

    #[test]
    fn doc_file_warning_passthrough_for_readme() {
        let stdin = r#"{"tool_input":{"file_path":"README.md"}}"#;
        let result = doc_file_warning(stdin);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn doc_file_warning_passthrough_for_claude_md() {
        let stdin = r#"{"tool_input":{"file_path":"CLAUDE.md"}}"#;
        let result = doc_file_warning(stdin);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn doc_file_warning_passthrough_for_docs_dir() {
        let stdin = r#"{"tool_input":{"file_path":"docs/guide.md"}}"#;
        let result = doc_file_warning(stdin);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn doc_file_warning_passthrough_for_non_doc_extension() {
        let stdin = r#"{"tool_input":{"file_path":"src/main.rs"}}"#;
        let result = doc_file_warning(stdin);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn doc_file_warning_passthrough_when_file_path_absent() {
        let result = doc_file_warning("{}");
        assert!(result.stderr.is_empty());
    }

    // --- doc_coverage_reminder ---

    #[test]
    fn doc_coverage_reminder_warns_on_undocumented_exports() {
        let fs = InMemoryFileSystem::new()
            .with_file("src/lib.rs", "pub fn alpha() {}\npub fn beta() {}\n");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/lib.rs"}}"#;
        let result = doc_coverage_reminder(stdin, &ports);
        assert!(result.stderr.contains("DocCoverage"));
        assert!(result.stderr.contains("lib.rs"));
    }

    #[test]
    fn doc_coverage_reminder_passthrough_for_non_source_extension() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"README.md"}}"#;
        let result = doc_coverage_reminder(stdin, &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn doc_coverage_reminder_passthrough_when_file_missing() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/missing.rs"}}"#;
        let result = doc_coverage_reminder(stdin, &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn doc_coverage_reminder_passthrough_when_all_documented() {
        let fs = InMemoryFileSystem::new()
            .with_file("src/lib.rs", "/// Documented\npub fn foo() {}\n");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/lib.rs"}}"#;
        let result = doc_coverage_reminder(stdin, &ports);
        assert!(result.stderr.is_empty());
    }

    // --- post_edit_console_warn ---

    #[test]
    fn post_edit_console_warn_warns_when_console_log_present() {
        let fs = InMemoryFileSystem::new()
            .with_file("src/index.ts", "const y = 2;\nconsole.log(y);\n");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/index.ts"}}"#;
        let result = post_edit_console_warn(stdin, &ports);
        assert!(result.stderr.contains("console.log found in src/index.ts"));
        assert!(result.stderr.contains("Remove console.log"));
    }

    #[test]
    fn post_edit_console_warn_passthrough_for_rust_file() {
        let fs = InMemoryFileSystem::new()
            .with_file("src/lib.rs", "println!(\"hello\");");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/lib.rs"}}"#;
        let result = post_edit_console_warn(stdin, &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn post_edit_console_warn_passthrough_when_no_console_log_in_js() {
        let fs = InMemoryFileSystem::new()
            .with_file("src/app.js", "const x = 1;\n");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/app.js"}}"#;
        let result = post_edit_console_warn(stdin, &ports);
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
