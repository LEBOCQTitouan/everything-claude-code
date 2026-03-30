//! Formatting hooks — auto-format JS/TS files after edits.

use tracing::warn;

use crate::hook::{HookPorts, HookResult};
use ecc_ports::env::Platform;
use std::path::Path;

use super::helpers::{detect_formatter, extract_file_path, find_project_root, validate_file_path};

/// post-edit-format: auto-format JS/TS files after edits.
pub fn post_edit_format(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    let file_path = extract_file_path(stdin);
    if file_path.is_empty() {
        return HookResult::passthrough(stdin);
    }

    if let Err(msg) = validate_file_path(&file_path) {
        return HookResult::warn(stdin, &format!("{msg}\n"));
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
            match ports.shell.run_command_in_dir(
                npx,
                &["@biomejs/biome", "format", "--write", &file_path],
                &project_root,
            ) {
                Ok(out) if out.exit_code != 0 => {
                    let msg = format!(
                        "[Hook] biome format failed (exit {}): {}",
                        out.exit_code, out.stderr
                    );
                    warn!("{}", msg);
                    return HookResult::warn(stdin, &format!("{msg}\n"));
                }
                Err(e) => {
                    let msg = format!("[Hook] biome format error: {}", e);
                    warn!("{}", msg);
                }
                _ => {}
            }
        }
        Some("prettier") => {
            match ports.shell.run_command_in_dir(
                npx,
                &["prettier", "--write", &file_path],
                &project_root,
            ) {
                Ok(out) if out.exit_code != 0 => {
                    let msg = format!(
                        "[Hook] prettier format failed (exit {}): {}",
                        out.exit_code, out.stderr
                    );
                    warn!("{}", msg);
                    return HookResult::warn(stdin, &format!("{msg}\n"));
                }
                Err(e) => {
                    let msg = format!("[Hook] prettier format error: {}", e);
                    warn!("{}", msg);
                }
                _ => {}
            }
        }
        _ => {}
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
}
