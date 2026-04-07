//! Diagram trigger use case.

use ecc_domain::docs::diagram_triggers;
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;

/// Run diagram trigger evaluation. Returns true always (advisory).
pub fn run_diagram_triggers(
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    changed_files: &[String],
    json: bool,
) -> bool {
    // Read file contents for enum analysis
    let file_contents: Vec<(String, String)> = changed_files
        .iter()
        .filter(|f| f.ends_with(".rs"))
        .filter_map(|f| {
            fs.read_to_string(std::path::Path::new(f))
                .ok()
                .map(|content| (f.clone(), content))
        })
        .collect();

    let result = diagram_triggers::evaluate_triggers(changed_files, &file_contents);

    if json {
        let triggers_json = serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string());
        terminal.stdout_write(&format!("{triggers_json}\n"));
    } else if result.triggers.is_empty() {
        terminal.stdout_write("No diagram triggers fired\n");
    } else {
        terminal.stdout_write("Diagram triggers:\n");
        for t in &result.triggers {
            terminal.stdout_write(&format!("  - {t:?}\n"));
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem};

    #[test]
    fn no_triggers_empty_files() {
        let fs = InMemoryFileSystem::new();
        let term = BufferedTerminal::new();
        assert!(run_diagram_triggers(&fs, &term, &[], true));
        let out = term.stdout_output().join("");
        assert!(out.contains("\"triggers\""));
    }
}
