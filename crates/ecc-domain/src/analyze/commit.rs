//! Conventional commit parser.
//!
//! Parses commit messages into structured `ConventionalCommit` values.
//! Non-conventional messages return `None`.

/// Standard conventional commit types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommitType {
    Feat,
    Fix,
    Refactor,
    Docs,
    Test,
    Chore,
    Perf,
    Ci,
    Style,
    Build,
    Unknown(String),
}

impl CommitType {
    /// Human-readable label for changelog sections.
    pub fn label(&self) -> &str {
        match self {
            Self::Feat => "Features",
            Self::Fix => "Bug Fixes",
            Self::Refactor => "Refactoring",
            Self::Docs => "Documentation",
            Self::Test => "Testing",
            Self::Chore => "Maintenance",
            Self::Perf => "Performance",
            Self::Ci => "CI/CD",
            Self::Style => "Style",
            Self::Build => "Build",
            Self::Unknown(s) => s.as_str(),
        }
    }

    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "feat" => Self::Feat,
            "fix" => Self::Fix,
            "refactor" => Self::Refactor,
            "docs" => Self::Docs,
            "test" | "tests" => Self::Test,
            "chore" => Self::Chore,
            "perf" => Self::Perf,
            "ci" => Self::Ci,
            "style" => Self::Style,
            "build" => Self::Build,
            other => Self::Unknown(other.to_string()),
        }
    }
}

/// A parsed conventional commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConventionalCommit {
    pub commit_type: CommitType,
    pub scope: Option<String>,
    pub breaking: bool,
    pub description: String,
    pub hash: String,
    pub author: String,
}

/// Parse a commit message into a `ConventionalCommit`.
///
/// Expected format: `type[(scope)][!]: description`
/// Returns `None` if the message doesn't match conventional commit format.
pub fn parse_conventional_commit(
    hash: &str,
    author: &str,
    message: &str,
) -> Option<ConventionalCommit> {
    let message = message.trim();
    if message.is_empty() {
        return None;
    }

    // Find the colon separator
    let colon_pos = message.find(':')?;
    let prefix = &message[..colon_pos];
    let description = message[colon_pos + 1..].trim().to_string();

    if description.is_empty() {
        return None;
    }

    // Parse prefix: type[(scope)][!]
    let (type_str, scope, bang) = parse_prefix(prefix)?;

    // Check for BREAKING CHANGE in footer (multi-line messages)
    let breaking = bang || message.contains("BREAKING CHANGE:");

    Some(ConventionalCommit {
        commit_type: CommitType::from_str(type_str),
        scope: scope.map(String::from),
        breaking,
        description,
        hash: hash.to_string(),
        author: author.to_string(),
    })
}

/// Parse the prefix before the colon: `type[(scope)][!]`
fn parse_prefix(prefix: &str) -> Option<(&str, Option<&str>, bool)> {
    let prefix = prefix.trim();

    // Check for bang
    let (prefix, bang) = if let Some(stripped) = prefix.strip_suffix('!') {
        (stripped, true)
    } else {
        (prefix, false)
    };

    // Check for scope in parentheses
    if let Some(paren_start) = prefix.find('(') {
        let type_str = &prefix[..paren_start];
        if type_str.is_empty() || !type_str.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return None;
        }
        let rest = &prefix[paren_start + 1..];
        let paren_end = rest.find(')')?;
        let scope = &rest[..paren_end];
        // Ensure nothing after closing paren
        if rest[paren_end + 1..].is_empty() {
            Some((type_str, Some(scope), bang))
        } else {
            None
        }
    } else {
        // No scope — just type
        if prefix.is_empty() || !prefix.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return None;
        }
        Some((prefix, None, bang))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // PC-001: parse feat with scope
    #[test]
    fn parse_feat_with_scope() {
        let commit =
            parse_conventional_commit("abc123", "alice", "feat(cli): add analyze command").unwrap();
        assert_eq!(commit.commit_type, CommitType::Feat);
        assert_eq!(commit.scope, Some("cli".to_string()));
        assert_eq!(commit.description, "add analyze command");
        assert!(!commit.breaking);
    }

    // PC-002: parse breaking fix
    #[test]
    fn parse_breaking_fix() {
        let commit =
            parse_conventional_commit("abc123", "alice", "feat!: remove legacy API").unwrap();
        assert!(commit.breaking);
        assert_eq!(commit.commit_type, CommitType::Feat);
    }

    // PC-003 + PC-053: non-conventional returns None
    #[test]
    fn parse_non_conventional_returns_none() {
        assert!(parse_conventional_commit("abc123", "alice", "update readme").is_none());
        assert!(parse_conventional_commit("abc123", "alice", "Merge branch 'feat'").is_none());
    }

    // PC-004: parse all standard types
    #[test]
    fn parse_all_commit_types() {
        let types = [
            ("feat", CommitType::Feat),
            ("fix", CommitType::Fix),
            ("refactor", CommitType::Refactor),
            ("docs", CommitType::Docs),
            ("test", CommitType::Test),
            ("chore", CommitType::Chore),
            ("perf", CommitType::Perf),
            ("ci", CommitType::Ci),
            ("style", CommitType::Style),
            ("build", CommitType::Build),
        ];
        for (prefix, expected) in types {
            let msg = format!("{prefix}: do something");
            let commit = parse_conventional_commit("abc", "alice", &msg).unwrap();
            assert_eq!(commit.commit_type, expected, "Failed for type: {prefix}");
        }
    }

    // PC-005: unknown type
    #[test]
    fn parse_unknown_type() {
        let commit = parse_conventional_commit("abc", "alice", "wip: stuff").unwrap();
        assert_eq!(commit.commit_type, CommitType::Unknown("wip".to_string()));
    }

    // PC-042: BREAKING CHANGE footer
    #[test]
    fn parse_breaking_change_footer() {
        let msg = "feat: add new API\n\nBREAKING CHANGE: old API removed";
        let commit = parse_conventional_commit("abc", "alice", msg).unwrap();
        assert!(commit.breaking);
    }

    #[test]
    fn parse_empty_message() {
        assert!(parse_conventional_commit("abc", "alice", "").is_none());
        assert!(parse_conventional_commit("abc", "alice", "   ").is_none());
    }

    #[test]
    fn parse_colon_only() {
        assert!(parse_conventional_commit("abc", "alice", ": description").is_none());
    }

    #[test]
    fn parse_no_description_after_colon() {
        assert!(parse_conventional_commit("abc", "alice", "feat:").is_none());
        assert!(parse_conventional_commit("abc", "alice", "feat:   ").is_none());
    }
}
