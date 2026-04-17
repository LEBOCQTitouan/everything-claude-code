use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

static DOLLAR_ARGUMENTS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[[:space:]]*!.*\$ARGUMENTS").expect("valid regex"));

pub(super) fn validate_commands(
    root: &Path,
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
) -> bool {
    let commands_dir = root.join("commands");
    if !fs.exists(&commands_dir) {
        terminal.stdout_write("No commands directory found, skipping validation\n");
        return true;
    }

    let files = match fs.read_dir(&commands_dir) {
        Ok(f) => f,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: Cannot read commands directory: {e}\n"));
            return false;
        }
    };
    let md_files: Vec<_> = files
        .iter()
        .filter(|f| f.to_string_lossy().ends_with(".md"))
        .collect();

    let mut has_errors = false;

    for file in &md_files {
        let content = match fs.read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                terminal.stderr_write(&format!("ERROR: {} - {}\n", file.display(), e));
                has_errors = true;
                continue;
            }
        };

        let name = file.file_name().unwrap_or_default().to_string_lossy();

        if content.trim().is_empty() {
            terminal.stderr_write(&format!("ERROR: {} - Empty command file\n", name));
            has_errors = true;
        }

        for (idx, line) in content.lines().enumerate() {
            if DOLLAR_ARGUMENTS_RE.is_match(line) {
                let lineno = idx + 1;
                terminal.stderr_write(&format!(
                    "ERROR: {}:{}: !-prefix shell-eval line references $ARGUMENTS — use Bash-tool argv invocation instead\n",
                    file.display(),
                    lineno
                ));
                has_errors = true;
            }
        }
    }

    if has_errors {
        return false;
    }

    terminal.stdout_write(&format!("Validated {} command files\n", md_files.len()));
    true
}

#[cfg(test)]
mod tests {
    use super::super::{ValidateTarget, run_validate};
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment};
    use std::path::Path;

    fn term() -> BufferedTerminal {
        BufferedTerminal::new()
    }

    #[test]
    fn commands_validate_rule_detects_dollar_arguments_fixture() {
        let fixture_content = "# Test Fixture\n\n!ecc-workflow init dev \"$ARGUMENTS\"\n";
        let fs = InMemoryFileSystem::new().with_file("/root/commands/fixture.md", fixture_content);
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Commands,
            Path::new("/root"),
        );
        assert!(
            !result,
            "validate_commands should return false for $ARGUMENTS in !-prefix line"
        );
        let stderr = t.stderr_output().join("");
        assert!(
            stderr.contains("fixture.md"),
            "stderr should contain the filename; got: {stderr}"
        );
        assert!(
            stderr.contains(":3:") || stderr.contains(":3 "),
            "stderr should contain the 1-based line number :3; got: {stderr}"
        );
    }

    #[test]
    fn commands_no_dir_succeeds() {
        let fs = InMemoryFileSystem::new();
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Commands,
            Path::new("/root")
        ));
    }

    #[test]
    fn commands_valid_file() {
        let fs = InMemoryFileSystem::new().with_file("/root/commands/test.md", "# Command");
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Commands,
            Path::new("/root")
        ));
    }

    #[test]
    fn commands_validate_rule_ignores_prose_dollar_arguments() {
        let prose_content =
            "# Audit Scope\n\nScope: $ARGUMENTS (or full codebase if none provided)\n";
        let fs = InMemoryFileSystem::new().with_file("/root/commands/prose.md", prose_content);
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Commands,
            Path::new("/root"),
        );
        assert!(
            result,
            "validate_commands should return true for $ARGUMENTS in prose (no !-prefix)"
        );
        let stderr = t.stderr_output().join("");
        assert!(
            !stderr.contains("prose.md"),
            "stderr should NOT contain prose.md error; got: {stderr}"
        );
    }

    #[test]
    fn commands_validate_rule_flags_fenced_code_examples() {
        // Conservative behavior: the rule flags !-prefix $ARGUMENTS lines
        // even when they appear inside fenced code blocks (```...```).
        // Template authors MUST NOT use illustrative occurrences.
        // Line 4 is the offending line inside the fenced block.
        let fenced_content = "# Example\n\n```bash\n!ecc-workflow init dev \"$ARGUMENTS\"\n```\n";
        let fs = InMemoryFileSystem::new().with_file("/root/commands/fenced.md", fenced_content);
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Commands,
            Path::new("/root"),
        );
        assert!(
            !result,
            "validate_commands should return false for $ARGUMENTS inside fenced code block"
        );
        let stderr = t.stderr_output().join("");
        assert!(
            stderr.contains("fenced.md"),
            "stderr should contain the filename; got: {stderr}"
        );
        assert!(
            stderr.contains(":4:") || stderr.contains(":4 "),
            "stderr should contain the 1-based line number :4; got: {stderr}"
        );
    }

    #[test]
    fn commands_validate_rule_crlf_line_numbers_correct() {
        // CRLF-terminated content simulating Windows checkout or CRLF editor.
        // Line 1: "# Test", Line 2: "" (blank), Line 3: offending !-prefix line.
        let crlf_content = "# Test\r\n\r\n!ecc-workflow init dev \"$ARGUMENTS\"\r\n";
        let fs = InMemoryFileSystem::new().with_file("/root/commands/crlf.md", crlf_content);
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Commands,
            Path::new("/root"),
        );
        assert!(
            !result,
            "validate_commands should return false for CRLF-terminated $ARGUMENTS line"
        );
        let stderr = t.stderr_output().join("");
        assert!(
            stderr.contains("crlf.md"),
            "stderr should contain the filename; got: {stderr}"
        );
        assert!(
            stderr.contains(":3:") || stderr.contains(":3 "),
            "stderr should contain 1-based line number :3 (not off-by-one); got: {stderr}"
        );
    }

    #[test]
    fn commands_empty_file() {
        let fs = InMemoryFileSystem::new().with_file("/root/commands/bad.md", "   ");
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Commands,
            Path::new("/root")
        ));
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("Empty command file"))
        );
    }
}
