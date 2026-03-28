use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

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
    }

    if has_errors {
        return false;
    }

    terminal.stdout_write(&format!("Validated {} command files\n", md_files.len()));
    true
}

#[cfg(test)]
mod tests {
    use super::super::{run_validate, ValidateTarget};
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem};
    use std::path::Path;

    fn term() -> BufferedTerminal {
        BufferedTerminal::new()
    }

    #[test]
    fn commands_no_dir_succeeds() {
        let fs = InMemoryFileSystem::new();
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
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
            &ValidateTarget::Commands,
            Path::new("/root")
        ));
    }

    #[test]
    fn commands_empty_file() {
        let fs = InMemoryFileSystem::new().with_file("/root/commands/bad.md", "   ");
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
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
