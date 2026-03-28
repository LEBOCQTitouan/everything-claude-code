use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

pub(super) fn validate_rules(root: &Path, fs: &dyn FileSystem, terminal: &dyn TerminalIO) -> bool {
    let rules_dir = root.join("rules");
    if !fs.exists(&rules_dir) {
        terminal.stdout_write("No rules directory found, skipping validation\n");
        return true;
    }

    let files = match fs.read_dir_recursive(&rules_dir) {
        Ok(f) => f,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: Cannot read rules directory: {e}\n"));
            return false;
        }
    };
    let md_files: Vec<_> = files
        .iter()
        .filter(|f| f.to_string_lossy().ends_with(".md"))
        .collect();

    let mut has_errors = false;
    let mut validated = 0;

    for file in &md_files {
        match fs.read_to_string(file) {
            Ok(c) if c.trim().is_empty() => {
                let rel = file.strip_prefix(root).unwrap_or(file);
                terminal.stderr_write(&format!("ERROR: {} - Empty rule file\n", rel.display()));
                has_errors = true;
            }
            Err(e) => {
                let rel = file.strip_prefix(root).unwrap_or(file);
                terminal.stderr_write(&format!("ERROR: {} - {}\n", rel.display(), e));
                has_errors = true;
            }
            Ok(_) => {
                validated += 1;
            }
        }
    }

    if has_errors {
        return false;
    }

    terminal.stdout_write(&format!("Validated {} rule files\n", validated));
    true
}

#[cfg(test)]
mod tests {
    use super::super::{ValidateTarget, run_validate};
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem};
    use std::path::Path;

    fn term() -> BufferedTerminal {
        BufferedTerminal::new()
    }

    #[test]
    fn rules_no_dir_succeeds() {
        let fs = InMemoryFileSystem::new();
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &ValidateTarget::Rules,
            Path::new("/root")
        ));
    }

    #[test]
    fn rules_valid_file() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/rules")
            .with_file("/root/rules/common/test.md", "# Rule");
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &ValidateTarget::Rules,
            Path::new("/root")
        ));
    }

    #[test]
    fn rules_empty_file() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/rules")
            .with_file("/root/rules/common/bad.md", "  ");
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &ValidateTarget::Rules,
            Path::new("/root")
        ));
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("Empty rule file"))
        );
    }
}
