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
