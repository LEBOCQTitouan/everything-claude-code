//! Quality hooks — typecheck and quality gate.

use log::warn;

use crate::hook::{HookPorts, HookResult};
use ecc_ports::env::Platform;
use std::path::Path;

use super::helpers::{extract_file_path, find_ancestor_with, validate_file_path};

/// post-edit-typecheck: run tsc --noEmit after .ts/.tsx edits.
pub fn post_edit_typecheck(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
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

    if let Err(msg) = validate_file_path(&file_path) {
        return HookResult::warn(stdin, &format!("{msg}\n"));
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
            if let Err(e) = ports.shell.run_command("gofmt", &["-w", &file_path]) {
                let msg = format!("[QualityGate] gofmt error: {}", e);
                warn!("{}", msg);
            }
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
}
