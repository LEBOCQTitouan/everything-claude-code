use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use std::collections::HashSet;
use std::path::Path;

/// ECC section header in .gitignore.
pub const ECC_SECTION_HEADER: &str = "# ECC (Everything Claude Code) generated files";

/// ECC section footer in .gitignore.
pub const ECC_SECTION_FOOTER: &str = "# End ECC generated files";

/// A gitignore entry with a pattern and a comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitignoreEntry {
    pub pattern: &'static str,
    pub comment: &'static str,
}

/// Default entries ECC should add to .gitignore.
pub const ECC_GITIGNORE_ENTRIES: &[GitignoreEntry] = &[
    GitignoreEntry {
        pattern: ".claude/settings.local.json",
        comment: "Claude Code local settings (machine-specific)",
    },
    GitignoreEntry {
        pattern: ".claude/.ecc-manifest.json",
        comment: "ECC installation manifest",
    },
    GitignoreEntry {
        pattern: "docs/CODEMAPS/",
        comment: "Generated architecture docs (regeneratable via /update-codemaps)",
    },
    GitignoreEntry {
        pattern: ".claude/plans/",
        comment: "Autonomous loop plans (ephemeral)",
    },
    GitignoreEntry {
        pattern: ".mcp.json",
        comment: "MCP server config (may contain API keys)",
    },
    GitignoreEntry {
        pattern: "CLAUDE.local.md",
        comment: "Personal Claude Code instructions (never commit)",
    },
];

/// Result of ensuring gitignore entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitignoreResult {
    pub added: Vec<String>,
    pub already_present: Vec<String>,
    pub skipped: bool,
}

/// Check if a directory is inside a git repository.
pub fn is_git_repo(shell: &dyn ShellExecutor, dir: &Path) -> bool {
    shell
        .run_command_in_dir("git", &["rev-parse", "--git-dir"], dir)
        .is_ok_and(|out| out.success())
}

/// Parse existing .gitignore content and extract all non-comment, non-empty patterns.
pub fn parse_gitignore_patterns(content: &str) -> HashSet<String> {
    content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect()
}

/// Build the ECC section lines to append to .gitignore.
pub fn build_gitignore_section(entries: &[&GitignoreEntry]) -> String {
    let mut lines = Vec::new();
    lines.push(String::new());
    lines.push(ECC_SECTION_HEADER.to_string());
    for entry in entries {
        lines.push(format!("# {}", entry.comment));
        lines.push(entry.pattern.to_string());
    }
    lines.push(ECC_SECTION_FOOTER.to_string());
    lines.push(String::new());
    lines.join("\n")
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
        };
    }

    let gitignore_path = dir.join(".gitignore");
    let existing_content = fs
        .read_to_string(&gitignore_path)
        .unwrap_or_default();

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
        };
    }

    let section = build_gitignore_section(&to_add);
    let new_content = format!("{}\n{}", existing_content.trim_end(), section);
    let _ = fs.write(&gitignore_path, &new_content);

    GitignoreResult {
        added,
        already_present,
        skipped: false,
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
        } else if let Ok(out) = shell.run_command_in_dir(
            "git",
            &["ls-files", "--error-unmatch", entry.pattern],
            dir,
        )
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

    // --- parse_gitignore_patterns ---

    #[test]
    fn parse_empty() {
        assert!(parse_gitignore_patterns("").is_empty());
    }

    #[test]
    fn parse_filters_comments() {
        let patterns = parse_gitignore_patterns("# comment\nnode_modules\n# another\n.env");
        assert_eq!(patterns.len(), 2);
        assert!(patterns.contains("node_modules"));
        assert!(patterns.contains(".env"));
    }

    #[test]
    fn parse_trims_whitespace() {
        let patterns = parse_gitignore_patterns("  node_modules  \n  .env  ");
        assert!(patterns.contains("node_modules"));
        assert!(patterns.contains(".env"));
    }

    #[test]
    fn parse_filters_empty_lines() {
        let patterns = parse_gitignore_patterns("\n\n\nfoo\n\nbar\n\n");
        assert_eq!(patterns.len(), 2);
    }

    // --- build_gitignore_section ---

    #[test]
    fn build_section_format() {
        let entries = vec![&ECC_GITIGNORE_ENTRIES[0]];
        let section = build_gitignore_section(&entries);
        assert!(section.contains(ECC_SECTION_HEADER));
        assert!(section.contains(ECC_SECTION_FOOTER));
        assert!(section.contains(".claude/settings.local.json"));
        assert!(section.contains("# Claude Code local settings"));
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

    // --- ECC_GITIGNORE_ENTRIES ---

    #[test]
    fn entries_count() {
        assert_eq!(ECC_GITIGNORE_ENTRIES.len(), 6);
    }

    #[test]
    fn entries_contain_manifest() {
        assert!(ECC_GITIGNORE_ENTRIES.iter().any(|e| e.pattern == ".claude/.ecc-manifest.json"));
    }

    #[test]
    fn entries_contain_mcp() {
        assert!(ECC_GITIGNORE_ENTRIES.iter().any(|e| e.pattern == ".mcp.json"));
    }
}
