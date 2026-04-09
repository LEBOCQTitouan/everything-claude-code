/// Extract the command string from JSON stdin (`tool_input.command`).
pub(super) fn extract_command(stdin: &str) -> String {
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
pub(super) fn extract_file_path(stdin: &str) -> String {
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
pub(super) fn extract_tool_output(stdin: &str) -> String {
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
pub(super) fn regex_find_pr_url(text: &str) -> Option<String> {
    // Simple pattern match without regex crate
    let marker = "https://github.com/";
    let start = text.find(marker)?;
    let rest = &text[start..];
    // Find end of URL (whitespace or end of string)
    let end = rest.find(|c: char| c.is_whitespace()).unwrap_or(rest.len());
    let url = &rest[..end];
    // Validate it looks like a PR URL
    if url.contains("/pull/") {
        Some(url.to_string())
    } else {
        None
    }
}

/// Scan a source file for exports and count undocumented ones.
pub(super) fn scan_exports(content: &str, ext: &str) -> (usize, usize) {
    let lines: Vec<&str> = content.lines().collect();
    let mut total = 0;
    let mut undocumented = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let is_export = match ext {
            "ts" | "tsx" | "js" | "jsx" => {
                trimmed.starts_with("export ") && {
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
                }
            }
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
            "rs" => {
                trimmed.starts_with("pub fn ")
                    || trimmed.starts_with("pub struct ")
                    || trimmed.starts_with("pub enum ")
                    || trimmed.starts_with("pub trait ")
                    || trimmed.starts_with("pub type ")
                    || trimmed.starts_with("pub const ")
                    || trimmed.starts_with("pub static ")
                    || trimmed.starts_with("pub mod ")
            }
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
pub(super) fn has_doc_comment(lines: &[&str], export_line: usize, ext: &str) -> bool {
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
    use crate::hook::handlers::tier1_simple::*;
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor}; // --- check_hook_enabled ---

    #[test]
    fn check_hook_enabled_returns_yes_for_standard() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let result = check_hook_enabled("my-hook", &ports);
        assert_eq!(result.stdout, "yes");
    }

    #[test]
    fn check_hook_enabled_returns_no_for_disabled() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("ECC_DISABLED_HOOKS", "my-hook");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let result = check_hook_enabled("my-hook", &ports);
        assert_eq!(result.stdout, "no");
    }

    #[test]
    fn check_hook_enabled_json_input() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("ECC_DISABLED_HOOKS", "target-hook");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

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
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let result = session_end_marker("stdin data", &ports);
        assert_eq!(result.stdout, "stdin data");
        assert_eq!(result.exit_code, 0);
    }

    // --- check_console_log ---

    #[test]
    fn check_console_log_warns_when_found() {
        let fs =
            InMemoryFileSystem::new().with_file("src/main.ts", "console.log('debug');\nlet x = 1;");
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
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let result = check_console_log("input", &ports);
        assert!(result.stderr.contains("console.log found"));
    }

    #[test]
    fn check_console_log_skips_test_files() {
        let fs =
            InMemoryFileSystem::new().with_file("src/main.test.ts", "console.log('test debug');");
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
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let result = check_console_log("input", &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn check_console_log_no_git_repo() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

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
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

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
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

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
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

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
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

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
        let fs =
            InMemoryFileSystem::new().with_file("src/lib.rs", "pub fn foo() {}\npub fn bar() {}\n");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/lib.rs"}}"#;
        let result = doc_coverage_reminder(stdin, &ports);
        assert!(result.stderr.contains("DocCoverage"));
        assert!(result.stderr.contains("2/2"));
    }

    #[test]
    fn doc_coverage_ok_when_documented() {
        let fs =
            InMemoryFileSystem::new().with_file("src/lib.rs", "/// Documented\npub fn foo() {}\n");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/lib.rs"}}"#;
        let result = doc_coverage_reminder(stdin, &ports);
        assert!(result.stderr.is_empty());
    }

    // --- post_edit_console_warn ---

    #[test]
    fn post_edit_console_warn_finds_console_log() {
        let fs =
            InMemoryFileSystem::new().with_file("src/app.ts", "const x = 1;\nconsole.log(x);\n");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/app.ts"}}"#;
        let result = post_edit_console_warn(stdin, &ports);
        assert!(result.stderr.contains("console.log found"));
    }

    #[test]
    fn post_edit_console_warn_ignores_non_js() {
        let fs = InMemoryFileSystem::new().with_file("src/lib.rs", "println!(\"hello\");");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let stdin = r#"{"tool_input":{"file_path":"src/lib.rs"}}"#;
        let result = post_edit_console_warn(stdin, &ports);
        assert!(result.stderr.is_empty());
    }

    // --- suggest_compact ---

    #[test]
    fn suggest_compact_first_call_no_suggestion() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("CLAUDE_SESSION_ID", "test-session");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let result = suggest_compact("{}", &ports);
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn suggest_compact_at_threshold() {
        let fs = InMemoryFileSystem::new().with_file("/tmp/claude-tool-count-test-session", "49");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("CLAUDE_SESSION_ID", "test-session");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let result = suggest_compact("{}", &ports);
        assert!(result.stderr.contains("50 tool calls reached"));
    }

    #[test]
    fn suggest_compact_periodic_reminder() {
        let fs = InMemoryFileSystem::new().with_file("/tmp/claude-tool-count-test-session", "74");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("CLAUDE_SESSION_ID", "test-session");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

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
