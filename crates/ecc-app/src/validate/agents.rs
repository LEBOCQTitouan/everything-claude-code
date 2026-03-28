use ecc_domain::config::validate::{VALID_MODELS, extract_frontmatter};
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

pub(super) fn validate_agents(root: &Path, fs: &dyn FileSystem, terminal: &dyn TerminalIO) -> bool {
    let agents_dir = root.join("agents");
    if !fs.exists(&agents_dir) {
        terminal.stdout_write("No agents directory found, skipping validation\n");
        return true;
    }

    let files = match fs.read_dir(&agents_dir) {
        Ok(f) => f,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: Cannot read agents directory: {e}\n"));
            return false;
        }
    };
    let md_files: Vec<_> = files
        .iter()
        .filter(|f| f.to_string_lossy().ends_with(".md"))
        .collect();

    let mut has_errors = false;
    for file in &md_files {
        if !validate_agent_file(file, fs, terminal) {
            has_errors = true;
        }
    }

    if has_errors {
        return false;
    }

    terminal.stdout_write(&format!("Validated {} agent files\n", md_files.len()));
    true
}

fn validate_agent_file(file: &Path, fs: &dyn FileSystem, terminal: &dyn TerminalIO) -> bool {
    let required_fields = ["model", "tools"];

    let content = match fs.read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: {} - {}\n", file.display(), e));
            return false;
        }
    };

    let frontmatter = match extract_frontmatter(&content) {
        Some(fm) => fm,
        None => {
            let name = file.file_name().unwrap_or_default().to_string_lossy();
            terminal.stderr_write(&format!("ERROR: {} - Missing frontmatter\n", name));
            return false;
        }
    };

    let name = file.file_name().unwrap_or_default().to_string_lossy();
    let mut valid = true;

    for field in &required_fields {
        match frontmatter.get(*field) {
            Some(v) if !v.trim().is_empty() => {}
            _ => {
                terminal.stderr_write(&format!(
                    "ERROR: {} - Missing required field: {}\n",
                    name, field
                ));
                valid = false;
            }
        }
    }

    if let Some(model) = frontmatter.get("model")
        && !VALID_MODELS.contains(&model.as_str())
    {
        terminal.stderr_write(&format!(
            "ERROR: {} - Invalid model '{}'. Must be one of: {}\n",
            name,
            model,
            VALID_MODELS.join(", ")
        ));
        valid = false;
    }

    valid
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
    fn agents_no_dir_succeeds() {
        let fs = InMemoryFileSystem::new();
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
    }

    #[test]
    fn agents_valid_file() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/test.md",
            "---\nmodel: sonnet\ntools: Read\n---\n# Agent",
        );
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
    }

    #[test]
    fn agents_missing_frontmatter() {
        let fs = InMemoryFileSystem::new().with_file("/root/agents/bad.md", "# No frontmatter");
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("Missing frontmatter"))
        );
    }

    #[test]
    fn agents_missing_required_field() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/agents/bad.md", "---\nmodel: sonnet\n---\n# Agent");
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("Missing required field"))
        );
    }

    #[test]
    fn agents_invalid_model() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/bad.md",
            "---\nmodel: gpt4\ntools: Read\n---\n# Agent",
        );
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("Invalid model"))
        );
    }
}
