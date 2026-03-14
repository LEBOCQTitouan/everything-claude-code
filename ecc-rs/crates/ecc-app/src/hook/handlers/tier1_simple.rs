//! Tier 1 Hooks — Simple passthrough/warn hooks with no external tool spawning.

use crate::hook::{HookPorts, HookResult};
use ecc_domain::hook_runtime::profiles::{is_hook_enabled, HookEnabledOptions};
use std::path::Path;

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
    let _ = ports.fs.write(&counter_file, &count.to_string());

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

// --- Helper functions ---

/// Extract the command string from JSON stdin (`tool_input.command`).
fn extract_command(stdin: &str) -> String {
    serde_json::from_str::<serde_json::Value>(stdin)
        .ok()
        .and_then(|v| {
            v.get("tool_input")
                .and_then(|ti| ti.get("command"))
                .and_then(|c| c.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_default()
}

/// Extract the file_path from JSON stdin (`tool_input.file_path`).
fn extract_file_path(stdin: &str) -> String {
    serde_json::from_str::<serde_json::Value>(stdin)
        .ok()
        .and_then(|v| {
            v.get("tool_input")
                .and_then(|ti| ti.get("file_path"))
                .and_then(|c| c.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_default()
}

/// Extract tool_output.output from JSON stdin.
fn extract_tool_output(stdin: &str) -> String {
    serde_json::from_str::<serde_json::Value>(stdin)
        .ok()
        .and_then(|v| {
            v.get("tool_output")
                .and_then(|to| to.get("output"))
                .and_then(|o| o.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_default()
}

/// Find a GitHub PR URL in text.
fn regex_find_pr_url(text: &str) -> Option<String> {
    // Simple pattern match without regex crate
    let marker = "https://github.com/";
    let start = text.find(marker)?;
    let rest = &text[start..];
    // Find end of URL (whitespace or end of string)
    let end = rest
        .find(|c: char| c.is_whitespace())
        .unwrap_or(rest.len());
    let url = &rest[..end];
    // Validate it looks like a PR URL
    if url.contains("/pull/") {
        Some(url.to_string())
    } else {
        None
    }
}

/// Scan a source file for exports and count undocumented ones.
fn scan_exports(content: &str, ext: &str) -> (usize, usize) {
    let lines: Vec<&str> = content.lines().collect();
    let mut total = 0;
    let mut undocumented = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let is_export = match ext {
            "ts" | "tsx" | "js" | "jsx" => trimmed.starts_with("export ") && {
                let rest = &trimmed[7..];
                rest.starts_with("function ")
                    || rest.starts_with("class ")
                    || rest.starts_with("const ")
                    || rest.starts_with("let ")
                    || rest.starts_with("var ")
                    || rest.starts_with("type ")
                    || rest.starts_with("interface ")
                    || rest.starts_with("enum ")
                    || rest.starts_with("default ")
                    || rest.starts_with("async function")
            },
            "py" => {
                (trimmed.starts_with("def ") || trimmed.starts_with("class "))
                    && !trimmed.starts_with("def _")
                    && !trimmed.starts_with("class _")
            }
            "go" => {
                (trimmed.starts_with("func ")
                    || trimmed.starts_with("type ")
                    || trimmed.starts_with("var ")
                    || trimmed.starts_with("const "))
                    && trimmed
                        .split_whitespace()
                        .nth(1)
                        .is_some_and(|w| w.starts_with(|c: char| c.is_uppercase()))
            }
            "rs" => trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub struct ")
                || trimmed.starts_with("pub enum ")
                || trimmed.starts_with("pub trait ")
                || trimmed.starts_with("pub type ")
                || trimmed.starts_with("pub const ")
                || trimmed.starts_with("pub static ")
                || trimmed.starts_with("pub mod "),
            "java" => {
                trimmed.starts_with("public ")
                    && (trimmed.contains("class ")
                        || trimmed.contains("interface ")
                        || trimmed.contains("enum "))
            }
            _ => false,
        };

        if !is_export {
            continue;
        }
        total += 1;

        let has_doc = has_doc_comment(&lines, i, ext);
        if !has_doc {
            undocumented += 1;
        }
    }

    (total, undocumented)
}

/// Check if a line has a doc comment above it.
fn has_doc_comment(lines: &[&str], export_line: usize, ext: &str) -> bool {
    if export_line == 0 {
        return false;
    }

    match ext {
        "ts" | "tsx" | "js" | "jsx" | "rs" | "java" => {
            for j in (export_line.saturating_sub(5)..export_line).rev() {
                let prev = lines[j].trim();
                if prev.is_empty() || prev.starts_with('@') || prev.starts_with('#') {
                    continue;
                }
                if prev.starts_with("/**")
                    || prev.starts_with("*/")
                    || prev.starts_with('*')
                    || prev.starts_with("///")
                {
                    return true;
                }
                break;
            }
            false
        }
        "py" => {
            if export_line + 1 < lines.len() {
                let next = lines[export_line + 1].trim();
                next.starts_with("\"\"\"") || next.starts_with("'''")
            } else {
                false
            }
        }
        "go" => {
            let prev = lines[export_line - 1].trim();
            prev.starts_with("//")
        }
        _ => false,
    }
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
    fn check_hook_enabled_returns_yes_for_standard() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = check_hook_enabled("my-hook", &ports);
        assert_eq!(result.stdout, "yes");
    }

    #[test]
    fn check_hook_enabled_returns_no_for_disabled() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("ECC_DISABLED_HOOKS", "my-hook");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = check_hook_enabled("my-hook", &ports);
        assert_eq!(result.stdout, "no");
    }

    #[test]
    fn check_hook_enabled_json_input() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("ECC_DISABLED_HOOKS", "target-hook");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = check_hook_enabled(r#"{"hook_id":"target-hook"}"#, &ports);
        assert_eq!(result.stdout, "no");
    }

    // --- session_end_marker ---

    #[test]
    fn session_end_marker_passes_through() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = session_end_marker("stdin data", &ports);
        assert_eq!(result.stdout, "stdin data");
        assert_eq!(result.exit_code, 0);
    }

    // --- check_console_log ---

    #[test]
    fn check_console_log_warns_when_found() {
        let fs = InMemoryFileSystem::new()
            .with_file("src/main.ts", "console.log('debug');\nlet x = 1;");
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
                    stdout: "src/main.ts\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = check_console_log("input", &ports);
        assert!(result.stderr.contains("console.log found"));
    }

    #[test]
    fn check_console_log_skips_test_files() {
        let fs = InMemoryFileSystem::new()
            .with_file("src/main.test.ts", "console.log('test debug');");
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
                    stdout: "src/main.test.ts\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = check_console_log("input", &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn check_console_log_no_git_repo() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = check_console_log("input", &ports);
        assert_eq!(result.stdout, "input");
        assert!(result.stderr.is_empty());
    }

    // --- stop_uncommitted_reminder ---

    #[test]
    fn uncommitted_reminder_warns_on_dirty() {
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
                    stdout: "M  src/lib.rs\n?? new_file.txt\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = stop_uncommitted_reminder("input", &ports);
        assert!(result.stderr.contains("uncommitted changes"));
        assert!(result.stderr.contains("Staged"));
    }

    #[test]
    fn uncommitted_reminder_clean_repo() {
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

        let result = stop_uncommitted_reminder("input", &ports);
        assert!(result.stderr.is_empty());
    }

    // --- pre_bash_git_push_reminder ---

    #[test]
    fn git_push_reminder_triggers() {
        let stdin = r#"{"tool_input":{"command":"git push origin main"}}"#;
        let result = pre_bash_git_push_reminder(stdin);
        assert!(result.stderr.contains("Review changes before push"));
    }

    #[test]
    fn git_push_reminder_ignores_non_push() {
        let stdin = r#"{"tool_input":{"command":"git status"}}"#;
        let result = pre_bash_git_push_reminder(stdin);
        assert!(result.stderr.is_empty());
    }

    // --- pre_bash_tmux_reminder ---

    #[test]
    fn tmux_reminder_triggers_for_long_commands() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"command":"npm install"}}"#;
        let result = pre_bash_tmux_reminder(stdin, &ports);
        assert!(result.stderr.contains("tmux"));
    }

    #[test]
    fn tmux_reminder_skips_in_tmux() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("TMUX", "/tmp/tmux-1001/default,123,0");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"command":"npm install"}}"#;
        let result = pre_bash_tmux_reminder(stdin, &ports);
        assert!(result.stderr.is_empty());
    }

    // --- post_bash_pr_created ---

    #[test]
    fn pr_created_extracts_url() {
        let stdin = r#"{"tool_input":{"command":"gh pr create"},"tool_output":{"output":"https://github.com/user/repo/pull/42\n"}}"#;
        let result = post_bash_pr_created(stdin);
        assert!(result.stderr.contains("PR created"));
        assert!(result.stderr.contains("42"));
    }

    #[test]
    fn pr_created_ignores_non_pr() {
        let stdin = r#"{"tool_input":{"command":"echo hello"}}"#;
        let result = post_bash_pr_created(stdin);
        assert!(result.stderr.is_empty());
    }

    // --- post_bash_build_complete ---

    #[test]
    fn build_complete_triggers() {
        let stdin = r#"{"tool_input":{"command":"npm run build"}}"#;
        let result = post_bash_build_complete(stdin);
        assert!(result.stderr.contains("Build completed"));
    }

    #[test]
    fn build_complete_ignores_non_build() {
        let stdin = r#"{"tool_input":{"command":"npm test"}}"#;
        let result = post_bash_build_complete(stdin);
        assert!(result.stderr.is_empty());
    }

    // --- doc_file_warning ---

    #[test]
    fn doc_file_warning_warns_non_standard() {
        let stdin = r#"{"tool_input":{"file_path":"notes.md"}}"#;
        let result = doc_file_warning(stdin);
        assert!(result.stderr.contains("Non-standard documentation"));
    }

    #[test]
    fn doc_file_warning_allows_readme() {
        let stdin = r#"{"tool_input":{"file_path":"README.md"}}"#;
        let result = doc_file_warning(stdin);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn doc_file_warning_allows_docs_dir() {
        let stdin = r#"{"tool_input":{"file_path":"docs/api.md"}}"#;
        let result = doc_file_warning(stdin);
        assert!(result.stderr.is_empty());
    }

    // --- doc_coverage_reminder ---

    #[test]
    fn doc_coverage_warns_undocumented() {
        let fs = InMemoryFileSystem::new()
            .with_file("src/lib.rs", "pub fn foo() {}\npub fn bar() {}\n");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/lib.rs"}}"#;
        let result = doc_coverage_reminder(stdin, &ports);
        assert!(result.stderr.contains("DocCoverage"));
        assert!(result.stderr.contains("2/2"));
    }

    #[test]
    fn doc_coverage_ok_when_documented() {
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
    fn post_edit_console_warn_finds_console_log() {
        let fs = InMemoryFileSystem::new()
            .with_file("src/app.ts", "const x = 1;\nconsole.log(x);\n");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/app.ts"}}"#;
        let result = post_edit_console_warn(stdin, &ports);
        assert!(result.stderr.contains("console.log found"));
    }

    #[test]
    fn post_edit_console_warn_ignores_non_js() {
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

    // --- suggest_compact ---

    #[test]
    fn suggest_compact_first_call_no_suggestion() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_var("CLAUDE_SESSION_ID", "test-session");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = suggest_compact("{}", &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn suggest_compact_at_threshold() {
        let fs = InMemoryFileSystem::new()
            .with_file("/tmp/claude-tool-count-test-session", "49");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_var("CLAUDE_SESSION_ID", "test-session");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = suggest_compact("{}", &ports);
        assert!(result.stderr.contains("50 tool calls reached"));
    }

    #[test]
    fn suggest_compact_periodic_reminder() {
        let fs = InMemoryFileSystem::new()
            .with_file("/tmp/claude-tool-count-test-session", "74");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_var("CLAUDE_SESSION_ID", "test-session");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = suggest_compact("{}", &ports);
        assert!(result.stderr.contains("75 tool calls"));
    }

    // --- extract helpers ---

    #[test]
    fn extract_command_from_json() {
        let json = r#"{"tool_input":{"command":"git push"}}"#;
        assert_eq!(extract_command(json), "git push");
    }

    #[test]
    fn extract_command_invalid_json() {
        assert_eq!(extract_command("not json"), "");
    }

    #[test]
    fn extract_file_path_from_json() {
        let json = r#"{"tool_input":{"file_path":"src/lib.rs"}}"#;
        assert_eq!(extract_file_path(json), "src/lib.rs");
    }

    // --- scan_exports ---

    #[test]
    fn scan_exports_rust_counts_pub() {
        let code = "pub fn foo() {}\nfn bar() {}\npub struct Baz;";
        let (total, undoc) = scan_exports(code, "rs");
        assert_eq!(total, 2);
        assert_eq!(undoc, 2);
    }

    #[test]
    fn scan_exports_rust_documented() {
        let code = "/// Doc comment\npub fn foo() {}";
        let (total, undoc) = scan_exports(code, "rs");
        assert_eq!(total, 1);
        assert_eq!(undoc, 0);
    }

    #[test]
    fn scan_exports_ts_counts() {
        let code = "export function foo() {}\nexport const bar = 1;\nconst priv = 2;";
        let (total, undoc) = scan_exports(code, "ts");
        assert_eq!(total, 2);
        assert_eq!(undoc, 2);
    }
}
