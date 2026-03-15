use crate::hook::{HookPorts, HookResult};
use std::path::Path;

use super::helpers::{extract_command, extract_file_path};

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
}
