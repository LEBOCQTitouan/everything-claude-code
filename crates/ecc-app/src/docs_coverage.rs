//! Doc coverage use case — walks source files and counts doc comments.

use ecc_domain::docs::coverage::{self, ModuleCoverage};
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

/// Run doc coverage analysis. Returns true on success.
pub fn run_docs_coverage(
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    scope: &Path,
    json: bool,
) -> bool {
    if !fs.exists(scope) {
        terminal.stderr_write(&format!("ERROR: Path does not exist: {}\n", scope.display()));
        return false;
    }

    let files = match fs.read_dir_recursive(scope) {
        Ok(f) => f,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: Cannot read directory: {e}\n"));
            return false;
        }
    };

    let rs_files: Vec<_> = files
        .iter()
        .filter(|f| f.extension().map(|e| e == "rs").unwrap_or(false))
        .collect();

    let mut modules: Vec<ModuleCoverage> = Vec::new();
    for file_path in &rs_files {
        if let Ok(content) = fs.read_to_string(file_path) {
            let name = file_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let cov = coverage::count_doc_coverage(&name, &content);
            if cov.total_pub_items > 0 {
                modules.push(cov);
            }
        }
    }

    if json {
        let json_modules: Vec<String> = modules
            .iter()
            .map(|m| {
                format!(
                    "{{\"name\":\"{}\",\"total\":{},\"documented\":{},\"pct\":{:.1}}}",
                    m.name, m.total_pub_items, m.documented, m.pct
                )
            })
            .collect();
        terminal.stdout_write(&format!("{{\"modules\":[{}]}}\n", json_modules.join(",")));
    } else {
        terminal.stdout_write("| Module | Total | Documented | Coverage |\n");
        terminal.stdout_write("|--------|-------|------------|----------|\n");
        for m in &modules {
            terminal.stdout_write(&format!(
                "| {} | {} | {} | {:.1}% |\n",
                m.name, m.total_pub_items, m.documented, m.pct
            ));
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem};

    #[test]
    fn coverage_nonexistent_path() {
        let fs = InMemoryFileSystem::new();
        let term = BufferedTerminal::new();
        assert!(!run_docs_coverage(&fs, &term, Path::new("/nope"), false));
    }

    #[test]
    fn coverage_json_output() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/src")
            .with_file("/src/lib.rs", "/// Doc.\npub fn foo() {}");
        let term = BufferedTerminal::new();
        assert!(run_docs_coverage(&fs, &term, Path::new("/src"), true));
        let out = term.stdout_output().join("");
        assert!(out.contains("\"modules\""));
    }
}
