//! Tier 2 Hooks — External tool spawning (formatter, typecheck, quality gate, dev server block).

use crate::hook::{HookPorts, HookResult};
use ecc_ports::env::Platform;
use std::path::Path;

/// pre-bash-dev-server-block: block dev servers outside tmux.
pub fn pre_bash_dev_server_block(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    if ports.env.platform() == Platform::Windows {
        return HookResult::passthrough(stdin);
    }

    let cmd = extract_command(stdin);
    if cmd.is_empty() {
        return HookResult::passthrough(stdin);
    }

    let segments = split_shell_segments(&cmd);
    let dev_pattern = [
        "npm run dev",
        "pnpm dev",
        "pnpm run dev",
        "yarn dev",
        "bun run dev",
    ];

    let has_blocked_dev = segments.iter().any(|seg| {
        let is_dev = dev_pattern.iter().any(|p| seg.contains(p));
        let is_tmux_launched = seg.trim_start().starts_with("tmux ");
        is_dev && !is_tmux_launched
    });

    if has_blocked_dev {
        let msg = "[Hook] BLOCKED: Dev server must run in tmux for log access\n\
                   [Hook] Use: tmux new-session -d -s dev \"npm run dev\"\n\
                   [Hook] Then: tmux attach -t dev\n";
        return HookResult::block(stdin, msg);
    }

    HookResult::passthrough(stdin)
}

/// post-edit-format: auto-format JS/TS files after edits.
pub fn post_edit_format(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
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

    // Find project root (look for package.json)
    let resolved = Path::new(&file_path);
    let project_root = find_project_root(resolved, ports);

    // Detect formatter
    let formatter = detect_formatter(&project_root, ports);
    if formatter.is_none() {
        return HookResult::passthrough(stdin);
    }

    let npx = if ports.env.platform() == Platform::Windows {
        "npx.cmd"
    } else {
        "npx"
    };

    match formatter.as_deref() {
        Some("biome") => {
            let _ = ports.shell.run_command_in_dir(
                npx,
                &["@biomejs/biome", "format", "--write", &file_path],
                &project_root,
            );
        }
        Some("prettier") => {
            let _ = ports.shell.run_command_in_dir(
                npx,
                &["prettier", "--write", &file_path],
                &project_root,
            );
        }
        _ => {}
    }

    HookResult::passthrough(stdin)
}

/// post-edit-typecheck: run tsc --noEmit after .ts/.tsx edits.
pub fn post_edit_typecheck(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    let file_path = extract_file_path(stdin);
    if file_path.is_empty() {
        return HookResult::passthrough(stdin);
    }

    let ext = Path::new(&file_path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if !matches!(ext.as_str(), "ts" | "tsx") {
        return HookResult::passthrough(stdin);
    }

    let resolved = Path::new(&file_path);
    if !ports.fs.exists(resolved) {
        return HookResult::passthrough(stdin);
    }

    // Walk up to find tsconfig.json
    let tsconfig_dir = find_ancestor_with(resolved, "tsconfig.json", ports);
    let tsconfig_dir = match tsconfig_dir {
        Some(d) => d,
        None => return HookResult::passthrough(stdin),
    };

    let npx = if ports.env.platform() == Platform::Windows {
        "npx.cmd"
    } else {
        "npx"
    };

    let result = ports.shell.run_command_in_dir(
        npx,
        &["tsc", "--noEmit", "--pretty", "false"],
        &tsconfig_dir,
    );

    if let Ok(output) = result
        && output.exit_code != 0 {
            let all_output = format!("{}{}", output.stdout, output.stderr);
            let basename = Path::new(&file_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let relevant: Vec<&str> = all_output
                .lines()
                .filter(|line| line.contains(&file_path) || line.contains(&basename))
                .take(10)
                .collect();

            if !relevant.is_empty() {
                let mut msg = format!("[Hook] TypeScript errors in {}:\n", basename);
                for line in relevant {
                    msg.push_str(&format!("{}\n", line));
                }
                return HookResult::warn(stdin, &msg);
            }
        }

    HookResult::passthrough(stdin)
}

/// quality-gate: multi-language quality checks.
pub fn quality_gate(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    let file_path = extract_file_path(stdin);
    if file_path.is_empty() || !ports.fs.exists(Path::new(&file_path)) {
        return HookResult::passthrough(stdin);
    }

    let ext = Path::new(&file_path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let fix = ports
        .env
        .var("ECC_QUALITY_GATE_FIX")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);
    let strict = ports
        .env
        .var("ECC_QUALITY_GATE_STRICT")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    let cwd = ports
        .env
        .current_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    match ext.as_str() {
        "ts" | "tsx" | "js" | "jsx" | "json" | "md" => {
            // Check for biome
            let biome_json = cwd.join("biome.json");
            let biome_jsonc = cwd.join("biome.jsonc");
            if ports.fs.exists(&biome_json) || ports.fs.exists(&biome_jsonc) {
                let mut args = vec!["biome", "check", file_path.as_str()];
                if fix {
                    args.push("--write");
                }
                let result = ports.shell.run_command("npx", &args);
                if let Ok(out) = result
                    && out.exit_code != 0 && strict {
                        let msg = format!(
                            "[QualityGate] Biome check failed for {}\n",
                            file_path
                        );
                        return HookResult::warn(stdin, &msg);
                    }
                return HookResult::passthrough(stdin);
            }

            // Fall back to prettier
            let action = if fix { "--write" } else { "--check" };
            let result = ports
                .shell
                .run_command("npx", &["prettier", action, &file_path]);
            if let Ok(out) = result
                && out.exit_code != 0 && strict {
                    let msg = format!(
                        "[QualityGate] Prettier check failed for {}\n",
                        file_path
                    );
                    return HookResult::warn(stdin, &msg);
                }
        }
        "go" if fix => {
            let _ = ports.shell.run_command("gofmt", &["-w", &file_path]);
        }
        "py" => {
            let mut args = vec!["format"];
            if !fix {
                args.push("--check");
            }
            args.push(&file_path);
            let result = ports.shell.run_command("ruff", &args);
            if let Ok(out) = result
                && out.exit_code != 0 && strict {
                    let msg = format!(
                        "[QualityGate] Ruff check failed for {}\n",
                        file_path
                    );
                    return HookResult::warn(stdin, &msg);
                }
        }
        _ => {}
    }

    HookResult::passthrough(stdin)
}

// --- Helper functions ---

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

/// Split a shell command into segments by ; && || &
fn split_shell_segments(command: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let chars: Vec<char> = command.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        if let Some(q) = quote {
            if ch == q {
                quote = None;
            }
            current.push(ch);
            i += 1;
            continue;
        }

        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            current.push(ch);
            i += 1;
            continue;
        }

        let next = chars.get(i + 1).copied().unwrap_or('\0');

        if ch == ';' || ch == '&' || (ch == '|' && next == '|') {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                segments.push(trimmed);
            }
            current.clear();
            if (ch == '&' && next == '&') || (ch == '|' && next == '|') {
                i += 1;
            }
            i += 1;
            continue;
        }

        current.push(ch);
        i += 1;
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        segments.push(trimmed);
    }
    segments
}

/// Find project root by walking up from a file path looking for package.json.
fn find_project_root(file_path: &Path, ports: &HookPorts<'_>) -> std::path::PathBuf {
    let mut dir = file_path.parent().unwrap_or(file_path).to_path_buf();
    for _ in 0..20 {
        if ports.fs.exists(&dir.join("package.json")) {
            return dir;
        }
        match dir.parent() {
            Some(p) if p != dir => dir = p.to_path_buf(),
            _ => break,
        }
    }
    file_path
        .parent()
        .unwrap_or(file_path)
        .to_path_buf()
}

/// Find an ancestor directory containing a specific file.
fn find_ancestor_with(
    file_path: &Path,
    target: &str,
    ports: &HookPorts<'_>,
) -> Option<std::path::PathBuf> {
    let mut dir = file_path.parent()?.to_path_buf();
    for _ in 0..20 {
        if ports.fs.exists(&dir.join(target)) {
            return Some(dir);
        }
        match dir.parent() {
            Some(p) if p != dir => dir = p.to_path_buf(),
            _ => break,
        }
    }
    None
}

/// Detect the configured formatter (biome or prettier) in a project.
fn detect_formatter(project_root: &Path, ports: &HookPorts<'_>) -> Option<String> {
    let biome_configs = ["biome.json", "biome.jsonc"];
    for cfg in &biome_configs {
        if ports.fs.exists(&project_root.join(cfg)) {
            return Some("biome".to_string());
        }
    }

    let prettier_configs = [
        ".prettierrc",
        ".prettierrc.json",
        ".prettierrc.js",
        ".prettierrc.cjs",
        ".prettierrc.mjs",
        ".prettierrc.yml",
        ".prettierrc.yaml",
        ".prettierrc.toml",
        "prettier.config.js",
        "prettier.config.cjs",
        "prettier.config.mjs",
    ];
    for cfg in &prettier_configs {
        if ports.fs.exists(&project_root.join(cfg)) {
            return Some("prettier".to_string());
        }
    }

    None
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

    // --- pre_bash_dev_server_block ---

    #[test]
    fn dev_server_blocks_npm_run_dev() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"command":"npm run dev"}}"#;
        let result = pre_bash_dev_server_block(stdin, &ports);
        assert_eq!(result.exit_code, 2);
        assert!(result.stderr.contains("BLOCKED"));
    }

    #[test]
    fn dev_server_allows_tmux_wrapped() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"command":"tmux new-session -d -s dev \"npm run dev\""}}"#;
        let result = pre_bash_dev_server_block(stdin, &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn dev_server_allows_non_dev_commands() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"command":"npm test"}}"#;
        let result = pre_bash_dev_server_block(stdin, &ports);
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn dev_server_passthrough_on_windows() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_platform(ecc_ports::env::Platform::Windows);
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"command":"npm run dev"}}"#;
        let result = pre_bash_dev_server_block(stdin, &ports);
        assert_eq!(result.exit_code, 0);
    }

    // --- post_edit_format ---

    #[test]
    fn format_detects_biome() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/biome.json", "{}")
            .with_file("/project/package.json", "{}")
            .with_file("/project/src/app.ts", "code");
        let shell = MockExecutor::new().on(
            "npx",
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"/project/src/app.ts"}}"#;
        let result = post_edit_format(stdin, &ports);
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn format_ignores_non_js_files() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/lib.rs"}}"#;
        let result = post_edit_format(stdin, &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    // --- post_edit_typecheck ---

    #[test]
    fn typecheck_warns_on_errors() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/tsconfig.json", "{}")
            .with_file("/project/src/app.ts", "let x: number = 'oops';");
        let shell = MockExecutor::new().on(
            "npx",
            CommandOutput {
                stdout: "/project/src/app.ts(1,5): error TS2322\n".to_string(),
                stderr: String::new(),
                exit_code: 1,
            },
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"/project/src/app.ts"}}"#;
        let result = post_edit_typecheck(stdin, &ports);
        assert!(result.stderr.contains("TypeScript errors"));
    }

    #[test]
    fn typecheck_ignores_non_ts() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/app.js"}}"#;
        let result = post_edit_typecheck(stdin, &ports);
        assert!(result.stderr.is_empty());
    }

    // --- quality_gate ---

    #[test]
    fn quality_gate_runs_biome_when_configured() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/biome.json", "{}")
            .with_file("/project/src/app.ts", "code");
        let shell = MockExecutor::new().on(
            "npx",
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"/project/src/app.ts"}}"#;
        let result = quality_gate(stdin, &ports);
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn quality_gate_runs_ruff_for_python() {
        let fs = InMemoryFileSystem::new().with_file("main.py", "import os");
        let shell = MockExecutor::new().on(
            "ruff",
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"main.py"}}"#;
        let result = quality_gate(stdin, &ports);
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn quality_gate_strict_mode_warns() {
        let fs = InMemoryFileSystem::new().with_file("main.py", "import os");
        let shell = MockExecutor::new().on(
            "ruff",
            CommandOutput {
                stdout: String::new(),
                stderr: "format error".to_string(),
                exit_code: 1,
            },
        );
        let env = MockEnvironment::new()
            .with_var("ECC_QUALITY_GATE_STRICT", "true");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"main.py"}}"#;
        let result = quality_gate(stdin, &ports);
        assert!(result.stderr.contains("QualityGate"));
    }

    // --- split_shell_segments ---

    #[test]
    fn split_simple_command() {
        let segs = split_shell_segments("echo hello");
        assert_eq!(segs, vec!["echo hello"]);
    }

    #[test]
    fn split_and_chain() {
        let segs = split_shell_segments("npm run build && npm test");
        assert_eq!(segs, vec!["npm run build", "npm test"]);
    }

    #[test]
    fn split_semicolon() {
        let segs = split_shell_segments("cd dir; npm run dev");
        assert_eq!(segs, vec!["cd dir", "npm run dev"]);
    }

    #[test]
    fn split_preserves_quotes() {
        let segs = split_shell_segments(r#"echo "hello && world""#);
        assert_eq!(segs, vec![r#"echo "hello && world""#]);
    }
}
