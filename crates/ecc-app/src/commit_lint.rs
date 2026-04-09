//! Commit lint use case — runs git diff and detects multi-concern changes.

use ecc_domain::docs::commit_lint;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::terminal::TerminalIO;

/// Run commit lint. Returns exit code (0=pass, 2=warn).
pub fn run_commit_lint(shell: &dyn ShellExecutor, terminal: &dyn TerminalIO, json: bool) -> i32 {
    let output = match shell.run_command_in_dir(
        "git",
        &["diff", "--cached", "--name-only"],
        std::path::Path::new("."),
    ) {
        Ok(o) => o.stdout,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: git diff failed: {e}\n"));
            return 1;
        }
    };

    let files: Vec<String> = output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();

    let result = commit_lint::detect_concerns(&files);

    if json {
        let concerns_json: Vec<String> =
            result.concerns.iter().map(|c| format!("\"{c}\"")).collect();
        terminal.stdout_write(&format!(
            "{{\"concerns\":[{}],\"verdict\":\"{}\"}}\n",
            concerns_json.join(","),
            if result.verdict == commit_lint::ConcernVerdict::Pass {
                "pass"
            } else {
                "warn"
            }
        ));
    } else if !result.concerns.is_empty() {
        for concern in &result.concerns {
            terminal.stderr_write(&format!("WARN: {concern}\n"));
        }
    }

    match result.verdict {
        commit_lint::ConcernVerdict::Pass => 0,
        commit_lint::ConcernVerdict::Warn => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::{BufferedTerminal, MockExecutor};

    #[test]
    fn lint_passes_single_concern() {
        let shell = MockExecutor::new().on(
            "git",
            CommandOutput {
                stdout: "crates/ecc-domain/src/lib.rs\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let term = BufferedTerminal::new();
        assert_eq!(run_commit_lint(&shell, &term, false), 0);
    }

    #[test]
    fn lint_warns_multi_concern() {
        let shell = MockExecutor::new().on(
            "git",
            CommandOutput {
                stdout: "crates/ecc-domain/src/lib.rs\nagents/drift-checker.md\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let term = BufferedTerminal::new();
        assert_eq!(run_commit_lint(&shell, &term, false), 2);
    }
}
