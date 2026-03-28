use std::collections::HashSet;

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
    GitignoreEntry {
        pattern: ".claude/worktrees/",
        comment: "Agent worktrees (ephemeral, per-session)",
    },
    GitignoreEntry {
        pattern: ".claude/workflow/",
        comment: "Workflow state (ephemeral, per-session)",
    },
    GitignoreEntry {
        pattern: "docs/memory/action-log.json",
        comment: "Cross-session action log (session-specific data)",
    },
    GitignoreEntry {
        pattern: "docs/memory/work-items/",
        comment: "Cross-session work-item records (session-specific data)",
    },
    GitignoreEntry {
        pattern: "docs/interviews/",
        comment: "Interview transcripts (session-specific, may contain sensitive info)",
    },
    GitignoreEntry {
        pattern: ".claude/workflow/.locks/",
        comment: "Workflow lock files (ephemeral, flock-managed)",
    },
];

/// Result of ensuring gitignore entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitignoreResult {
    pub added: Vec<String>,
    pub already_present: Vec<String>,
    pub skipped: bool,
    /// Write error message, if the gitignore file could not be updated.
    pub error: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

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

    // --- ECC_GITIGNORE_ENTRIES ---

    #[test]
    fn entries_count() {
        assert_eq!(ECC_GITIGNORE_ENTRIES.len(), 12);
    }

    #[test]
    fn entries_contain_manifest() {
        assert!(
            ECC_GITIGNORE_ENTRIES
                .iter()
                .any(|e| e.pattern == ".claude/.ecc-manifest.json")
        );
    }

    #[test]
    fn entries_contain_mcp() {
        assert!(
            ECC_GITIGNORE_ENTRIES
                .iter()
                .any(|e| e.pattern == ".mcp.json")
        );
    }
}
