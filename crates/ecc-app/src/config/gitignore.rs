use ecc_domain::config::gitignore::{
    ECC_GITIGNORE_ENTRIES, GitignoreEntry, GitignoreResult, build_gitignore_section,
    parse_gitignore_patterns,
};
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use std::path::Path;

/// Check if a directory is inside a git repository.
pub fn is_git_repo(shell: &dyn ShellExecutor, dir: &Path) -> bool {
    shell
        .run_command_in_dir("git", &["rev-parse", "--git-dir"], dir)
        .is_ok_and(|out| out.success())
}

/// Ensure ECC entries are present in .gitignore.
/// Creates .gitignore if it doesn't exist (only in git repos).
pub fn ensure_gitignore_entries(
    fs: &dyn FileSystem,
    shell: &dyn ShellExecutor,
    dir: &Path,
    entries: Option<&[GitignoreEntry]>,
) -> GitignoreResult {
    let entries = entries.unwrap_or(ECC_GITIGNORE_ENTRIES);

    if !is_git_repo(shell, dir) {
        return GitignoreResult {
            added: vec![],
            already_present: vec![],
            skipped: true,
            error: None,
        };
    }

    let gitignore_path = dir.join(".gitignore");
    let existing_content = fs.read_to_string(&gitignore_path).unwrap_or_default();

    let existing_patterns = parse_gitignore_patterns(&existing_content);
    let mut added = Vec::new();
    let mut already_present = Vec::new();
    let mut to_add = Vec::new();

    for entry in entries {
        if existing_patterns.contains(entry.pattern) {
            already_present.push(entry.pattern.to_string());
        } else {
            to_add.push(entry);
            added.push(entry.pattern.to_string());
        }
    }

    if to_add.is_empty() {
        return GitignoreResult {
            added,
            already_present,
            skipped: false,
            error: None,
        };
    }

    let section = build_gitignore_section(&to_add);
    let new_content = format!("{}\n{}", existing_content.trim_end(), section);
    let write_error = match fs.write(&gitignore_path, &new_content) {
        Ok(()) => None,
        Err(e) => {
            tracing::warn!("Failed to write .gitignore: {}", e);
            Some(format!("Failed to write .gitignore: {e}"))
        }
    };

    GitignoreResult {
        added,
        already_present,
        skipped: false,
        error: write_error,
    }
}

/// Find ECC-generated files that are currently tracked by git.
pub fn find_tracked_ecc_files(
    shell: &dyn ShellExecutor,
    fs: &dyn FileSystem,
    dir: &Path,
) -> Vec<String> {
    if !is_git_repo(shell, dir) {
        return vec![];
    }

    let mut tracked = Vec::new();
    for entry in ECC_GITIGNORE_ENTRIES {
        if entry.pattern.ends_with('/') {
            // Directory — check if any files inside are tracked
            let full_path = dir.join(entry.pattern);
            if fs.exists(&full_path)
                && let Ok(out) = shell.run_command_in_dir("git", &["ls-files", entry.pattern], dir)
                && out.success()
                && !out.stdout.trim().is_empty()
            {
                tracked.push(entry.pattern.to_string());
            }
        } else if let Ok(out) =
            shell.run_command_in_dir("git", &["ls-files", "--error-unmatch", entry.pattern], dir)
            && out.success()
        {
            tracked.push(entry.pattern.to_string());
        }
    }
    tracked
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::{InMemoryFileSystem, MockExecutor};

    fn git_success() -> CommandOutput {
        CommandOutput {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    fn git_executor() -> MockExecutor {
        MockExecutor::new().on("git", git_success())
    }

    // --- is_git_repo ---

    #[test]
    fn is_git_repo_true() {
        let shell = git_executor();
        assert!(is_git_repo(&shell, Path::new("/project")));
    }

    #[test]
    fn is_git_repo_false() {
        let shell = MockExecutor::new().on(
            "git",
            CommandOutput {
                stdout: String::new(),
                stderr: "not a git repo".into(),
                exit_code: 128,
            },
        );
        assert!(!is_git_repo(&shell, Path::new("/project")));
    }

    // --- ensure_gitignore_entries ---

    #[test]
    fn ensure_skips_non_git_repo() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on(
            "git",
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 128,
            },
        );
        let result = ensure_gitignore_entries(&fs, &shell, Path::new("/project"), None);
        assert!(result.skipped);
        assert!(result.added.is_empty());
    }

    #[test]
    fn ensure_adds_all_entries_to_new_gitignore() {
        let fs = InMemoryFileSystem::new();
        let shell = git_executor();
        let result = ensure_gitignore_entries(&fs, &shell, Path::new("/project"), None);
        assert!(!result.skipped);
        assert_eq!(result.added.len(), ECC_GITIGNORE_ENTRIES.len());
        assert!(result.already_present.is_empty());
    }

    #[test]
    fn ensure_detects_already_present() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/.gitignore", ".claude/settings.local.json\n");
        let shell = git_executor();
        let result = ensure_gitignore_entries(&fs, &shell, Path::new("/project"), None);
        assert_eq!(result.already_present.len(), 1);
        assert_eq!(result.added.len(), ECC_GITIGNORE_ENTRIES.len() - 1);
    }

    #[test]
    fn ensure_all_present_adds_nothing() {
        let content = ECC_GITIGNORE_ENTRIES
            .iter()
            .map(|e| e.pattern)
            .collect::<Vec<_>>()
            .join("\n");
        let fs = InMemoryFileSystem::new().with_file("/project/.gitignore", &content);
        let shell = git_executor();
        let result = ensure_gitignore_entries(&fs, &shell, Path::new("/project"), None);
        assert!(result.added.is_empty());
        assert_eq!(result.already_present.len(), ECC_GITIGNORE_ENTRIES.len());
    }
}
