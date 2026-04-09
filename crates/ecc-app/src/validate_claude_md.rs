//! CLAUDE.md count validation use case.

use ecc_domain::docs::claude_md;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

/// Run CLAUDE.md count validation. Returns true if all match.
pub fn run_validate_claude_md(
    fs: &dyn FileSystem,
    shell: &dyn ShellExecutor,
    terminal: &dyn TerminalIO,
    project_root: &Path,
    json: bool,
) -> bool {
    let claude_path = project_root.join("CLAUDE.md");
    let content = match fs.read_to_string(&claude_path) {
        Ok(c) => c,
        Err(_) => {
            terminal.stderr_write("ERROR: CLAUDE.md not found\n");
            return false;
        }
    };

    let mut claims = claude_md::extract_claims(&content);

    // Gather actual counts
    let mut actuals: Vec<(String, u64)> = Vec::new();

    // Test count via cargo test --list
    if let Ok(output) = shell.run_command_in_dir(
        "cargo",
        &["test", "--", "--list"],
        std::path::Path::new("."),
    ) {
        let test_count = output
            .stdout
            .lines()
            .filter(|l| l.ends_with(": test") || l.ends_with(": benchmark"))
            .count() as u64;
        actuals.push(("tests".to_string(), test_count));
    } else {
        terminal.stderr_write("WARN: cargo test --list unavailable, skipping test count\n");
    }

    // Crate count
    let crates_dir = project_root.join("crates");
    if let Ok(entries) = fs.read_dir(&crates_dir) {
        let crate_count = entries.iter().filter(|e| fs.is_dir(e)).count() as u64;
        actuals.push(("crates".to_string(), crate_count));
    }

    claude_md::compare_claims(&mut claims, &actuals);

    let all_match = claims.iter().all(|c| c.actual.is_none() || c.matches);

    if json {
        let claims_json: Vec<String> = claims
            .iter()
            .map(|c| {
                format!(
                    "{{\"text\":\"{}\",\"claimed\":{},\"actual\":{},\"match\":{}}}",
                    c.text,
                    c.claimed,
                    c.actual
                        .map(|a| a.to_string())
                        .unwrap_or("null".to_string()),
                    c.matches,
                )
            })
            .collect();
        terminal.stdout_write(&format!("{{\"claims\":[{}]}}\n", claims_json.join(",")));
    } else if all_match {
        terminal.stdout_write("All counts valid\n");
    } else {
        for c in &claims {
            if !c.matches
                && let Some(actual) = c.actual
            {
                terminal.stderr_write(&format!(
                    "MISMATCH: \"{}\" — claimed {}, actual {}\n",
                    c.text, c.claimed, actual
                ));
            }
        }
    }

    all_match
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockExecutor};

    #[test]
    fn missing_claude_md() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let term = BufferedTerminal::new();
        assert!(!run_validate_claude_md(
            &fs,
            &shell,
            &term,
            Path::new("/root"),
            false
        ));
        assert!(term.stderr_output().join("").contains("not found"));
    }
}
