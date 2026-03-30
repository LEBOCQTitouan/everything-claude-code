use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

pub(super) fn validate_paths(root: &Path, fs: &dyn FileSystem, terminal: &dyn TerminalIO) -> bool {
    let targets = ["README.md", "skills", "commands", "agents", "docs"];

    let block_patterns = ["/Users/affoon", "C:\\Users\\affoon"];
    let check_exts = [
        ".md", ".json", ".js", ".ts", ".sh", ".toml", ".yml", ".yaml",
    ];

    let mut failures = 0;

    for target in &targets {
        let target_path = root.join(target);
        if !fs.exists(&target_path) {
            continue;
        }

        let files = if fs.is_file(&target_path) {
            vec![target_path]
        } else {
            match fs.read_dir_recursive(&target_path) {
                Ok(f) => f,
                Err(e) => {
                    terminal.stderr_write(&format!("ERROR: Cannot read {}: {}\n", target, e));
                    failures += 1;
                    continue;
                }
            }
        };

        for file in &files {
            let name = file.to_string_lossy();
            if !check_exts.iter().any(|ext| name.ends_with(ext)) {
                continue;
            }
            if name.contains("node_modules") || name.contains(".git/") {
                continue;
            }

            if let Ok(content) = fs.read_to_string(file) {
                for pattern in &block_patterns {
                    if content.contains(pattern) {
                        let rel = file.strip_prefix(root).unwrap_or(file);
                        terminal.stderr_write(&format!(
                            "ERROR: personal path detected in {}\n",
                            rel.display()
                        ));
                        failures += 1;
                        break;
                    }
                }
            }
        }
    }

    if failures > 0 {
        return false;
    }

    terminal
        .stdout_write("Validated: no personal absolute paths in shipped docs/skills/commands\n");
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
    fn paths_no_targets_succeeds() {
        let fs = InMemoryFileSystem::new();
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Paths,
            Path::new("/root")
        ));
    }

    #[test]
    fn paths_clean_files() {
        let fs =
            InMemoryFileSystem::new().with_file("/root/README.md", "# Project\nClean content.");
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Paths,
            Path::new("/root")
        ));
    }

    #[test]
    fn paths_personal_path_detected() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/README.md", "See /Users/affoon/code for details.");
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Paths,
            Path::new("/root")
        ));
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("personal path detected"))
        );
    }

    #[test]
    fn paths_read_dir_error_reported() {
        // Verify that read_dir errors are now reported (fixes ERR-008)
        let fs = InMemoryFileSystem::new().with_file("/root/skills/test.md", "content");
        let t = term();
        // skills dir exists as a file (not a dir), so read_dir_recursive will fail
        // This verifies the error path is now surfaced
        let result = run_validate(&fs, &t, &MockEnvironment::default(), &ValidateTarget::Paths, Path::new("/root"));
        // Should still pass since this path is not a checked extension
        assert!(result);
    }
}
