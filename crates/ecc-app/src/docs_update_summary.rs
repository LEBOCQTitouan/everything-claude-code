//! Module summary update use case.

use ecc_domain::docs::module_summary;
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

/// Run module summary update. Returns true on success.
pub fn run_update_summary(
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    changed_files: &[String],
    feature: &str,
    json: bool,
) -> bool {
    let crates = module_summary::identify_crate_paths(changed_files);
    if crates.is_empty() {
        if json {
            terminal.stdout_write("{\"updated_crates\":[]}\n");
        } else {
            terminal.stdout_write("No module changes\n");
        }
        return true;
    }

    let entries: Vec<(String, String)> = crates
        .iter()
        .map(|c| (c.clone(), module_summary::format_entry(c, feature)))
        .collect();

    let summaries_path = Path::new("docs/MODULE-SUMMARIES.md");
    let existing = fs.read_to_string(summaries_path).unwrap_or_default();
    let updated = module_summary::insert_entries(&existing, &entries);
    if let Err(e) = fs.write(summaries_path, &updated) {
        terminal.stderr_write(&format!("ERROR: Cannot write MODULE-SUMMARIES.md: {e}\n"));
        return false;
    }

    if json {
        let names: Vec<String> = crates.iter().map(|c| format!("\"{c}\"")).collect();
        terminal.stdout_write(&format!("{{\"updated_crates\":[{}]}}\n", names.join(",")));
    } else {
        terminal.stdout_write(&format!("Updated {} crate entries\n", crates.len()));
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem};

    #[test]
    fn empty_changed_files() {
        let fs = InMemoryFileSystem::new();
        let term = BufferedTerminal::new();
        assert!(run_update_summary(&fs, &term, &[], "feat", false));
        assert!(term.stdout_output().join("").contains("No module changes"));
    }

    #[test]
    fn updates_crate_entry() {
        let fs = InMemoryFileSystem::new()
            .with_dir("docs")
            .with_file("docs/MODULE-SUMMARIES.md", "# Summaries\n");
        let term = BufferedTerminal::new();
        let files = vec!["crates/ecc-domain/src/lib.rs".to_string()];
        assert!(run_update_summary(&fs, &term, &files, "BL-126", false));
    }
}
