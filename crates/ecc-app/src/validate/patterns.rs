use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

pub(super) fn validate_patterns(root: &Path, fs: &dyn FileSystem, terminal: &dyn TerminalIO) -> bool {
    let patterns_dir = root.join("patterns");
    if !fs.exists(&patterns_dir) {
        terminal.stdout_write("No patterns directory found, skipping validation\n");
        return true;
    }

    let categories = match fs.read_dir(&patterns_dir) {
        Ok(entries) => entries
            .into_iter()
            .filter(|p| fs.is_dir(p))
            .collect::<Vec<_>>(),
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: Cannot read patterns directory: {e}\n"));
            return false;
        }
    };

    let mut file_count: usize = 0;
    let mut category_count: usize = 0;

    for category in &categories {
        let files = match fs.read_dir(category) {
            Ok(entries) => entries
                .into_iter()
                .filter(|p| {
                    p.extension()
                        .map(|ext| ext.eq_ignore_ascii_case("md"))
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>(),
            Err(_) => continue,
        };
        if !files.is_empty() {
            category_count += 1;
            file_count += files.len();
        }
    }

    terminal.stdout_write(&format!(
        "Validated {} pattern files across {} categories\n",
        file_count, category_count
    ));
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
    fn no_patterns_dir_succeeds() {
        let fs = InMemoryFileSystem::new();
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(result);
        assert!(t
            .stdout_output()
            .iter()
            .any(|s| s.contains("skipping validation")));
    }

    #[test]
    fn empty_dir_succeeds() {
        let fs = InMemoryFileSystem::new().with_dir("/root/patterns");
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Patterns,
            Path::new("/root"),
        );
        assert!(result);
        assert!(t
            .stdout_output()
            .iter()
            .any(|s| s.contains("0 pattern files across 0 categories")));
    }
}
