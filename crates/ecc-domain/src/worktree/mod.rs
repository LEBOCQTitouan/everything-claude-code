pub mod liveness;

/// Value object for worktree names used in session isolation.
/// Format: `ecc-session-{YYYYMMDD-HHMMSS}-{slug}-{pid}`
///
/// Slug is sanitized: lowercase, `[a-z0-9-]` only, max 40 chars.
/// Names validated against allowlist: `[a-zA-Z0-9/_-]` only, max 255 bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeName {
    /// The validated worktree name string.
    name: String,
}

/// Components extracted from a parsed worktree name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedWorktreeName {
    /// The timestamp portion in YYYYMMDD-HHMMSS format.
    pub timestamp: String,
    /// The slug portion (sanitized feature description).
    pub slug: String,
    /// The process ID portion.
    pub pid: u32,
}

/// Errors that can occur when creating or validating a `WorktreeName`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorktreeNameError {
    /// The name exceeds the maximum length of 255 bytes.
    TooLong(usize),
    /// The name contains invalid characters outside the allowlist.
    InvalidChars(String),
}

impl std::fmt::Display for WorktreeNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorktreeNameError::TooLong(len) => {
                write!(f, "worktree name too long: {len} bytes (max 255)")
            }
            WorktreeNameError::InvalidChars(name) => {
                write!(f, "worktree name contains invalid characters: {name}")
            }
        }
    }
}

impl WorktreeName {
    /// Generate a new worktree name from concern, feature description, and PID.
    pub fn generate(concern: &str, feature: &str, pid: u32) -> Result<Self, WorktreeNameError> {
        let _ = concern; // concern reserved for future use
        let slug = make_slug(feature);
        let ts = utc_timestamp_compact();
        let name = format!("ecc-session-{ts}-{slug}-{pid}");
        Self::validate(&name)?;
        Ok(Self { name })
    }

    /// Validate a name against the allowlist `[a-zA-Z0-9/_-]`, max 255 bytes.
    pub fn validate(name: &str) -> Result<(), WorktreeNameError> {
        if name.len() > 255 {
            return Err(WorktreeNameError::TooLong(name.len()));
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '_' || c == '-')
        {
            return Err(WorktreeNameError::InvalidChars(name.to_owned()));
        }
        Ok(())
    }

    /// Parse a worktree name back to its components.
    pub fn parse(name: &str) -> Option<ParsedWorktreeName> {
        // Strip optional worktree- prefix (added by EnterWorktree tool).
        // Only strip once to reject double-prefix like worktree-worktree-ecc-session-*.
        let name = name.strip_prefix("worktree-").unwrap_or(name);
        let rest = name.strip_prefix("ecc-session-")?;
        // Timestamp is YYYYMMDD-HHMMSS = 15 chars
        if rest.len() < 16 {
            return None;
        }
        let ts = &rest[..15];
        let rest = &rest[16..]; // skip the '-' after timestamp
        // Find last '-' for PID
        let last_dash = rest.rfind('-')?;
        let slug = &rest[..last_dash];
        let pid_str = &rest[last_dash + 1..];
        let pid = pid_str.parse::<u32>().ok()?;
        Some(ParsedWorktreeName {
            timestamp: ts.to_owned(),
            slug: slug.to_owned(),
            pid,
        })
    }

    /// Return the name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.name
    }

    /// Platform-normalized equality: case-insensitive on macOS (HFS+/APFS), case-sensitive elsewhere.
    pub fn eq_platform(&self, other: &Self) -> bool {
        #[cfg(target_os = "macos")]
        {
            self.name.to_lowercase() == other.name.to_lowercase()
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.name == other.name
        }
    }
}

/// Build a slug from a description: lowercase, `[a-z0-9-]` only, max 40 chars.
fn make_slug(desc: &str) -> String {
    let raw: String = desc
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let joined = raw
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let truncated: String = joined.chars().take(40).collect();
    truncated.trim_end_matches('-').to_owned()
}

/// Return the current UTC time as `YYYYMMDD-HHMMSS`.
fn utc_timestamp_compact() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let dt = crate::time::datetime_from_epoch(secs);
    format!(
        "{:04}{:02}{:02}-{:02}{:02}{:02}",
        dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second
    )
}

/// Input data for the worktree safety assessment — all values gathered by the caller via I/O.
#[derive(Debug, Clone)]
pub struct WorktreeSafetyInput {
    /// Whether the worktree has uncommitted changes.
    pub has_uncommitted_changes: bool,
    /// Whether the worktree has untracked files.
    pub has_untracked_files: bool,
    /// Number of unmerged commits in the worktree.
    pub unmerged_commit_count: u64,
    /// Whether the worktree has stashed changes.
    pub has_stash: bool,
    /// Whether all commits are pushed to the remote.
    pub is_pushed_to_remote: bool,
}

/// Reasons a worktree is not safe to delete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafetyViolation {
    /// The worktree has uncommitted changes.
    UncommittedChanges,
    /// The worktree has untracked files.
    UntrackedFiles,
    /// The worktree has unmerged commits.
    UnmergedCommits {
        /// Number of unmerged commits.
        count: u64,
    },
    /// The worktree has stashed changes.
    StashedChanges,
    /// The worktree has unpushed commits.
    UnpushedCommits,
}

/// Assess all safety violations for a worktree. Pure function — no I/O.
/// Returns an empty vec if the worktree is safe to delete.
pub fn assess_safety(input: &WorktreeSafetyInput) -> Vec<SafetyViolation> {
    let mut violations = Vec::new();
    if input.has_uncommitted_changes {
        violations.push(SafetyViolation::UncommittedChanges);
    }
    if input.has_untracked_files {
        violations.push(SafetyViolation::UntrackedFiles);
    }
    if input.unmerged_commit_count > 0 {
        violations.push(SafetyViolation::UnmergedCommits {
            count: input.unmerged_commit_count,
        });
    }
    if input.has_stash {
        violations.push(SafetyViolation::StashedChanges);
    }
    if !input.is_pushed_to_remote {
        violations.push(SafetyViolation::UnpushedCommits);
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clean_input() -> WorktreeSafetyInput {
        WorktreeSafetyInput {
            has_uncommitted_changes: false,
            has_untracked_files: false,
            unmerged_commit_count: 0,
            has_stash: false,
            is_pushed_to_remote: true,
        }
    }

    #[test]
    fn assess_uncommitted_changes() {
        let input = WorktreeSafetyInput {
            has_uncommitted_changes: true,
            ..clean_input()
        };
        let result = assess_safety(&input);
        assert!(
            result.contains(&SafetyViolation::UncommittedChanges),
            "expected UncommittedChanges violation"
        );
    }

    #[test]
    fn assess_untracked_files() {
        let input = WorktreeSafetyInput {
            has_untracked_files: true,
            ..clean_input()
        };
        let result = assess_safety(&input);
        assert!(
            result.contains(&SafetyViolation::UntrackedFiles),
            "expected UntrackedFiles violation"
        );
    }

    #[test]
    fn assess_unmerged_commits() {
        let input = WorktreeSafetyInput {
            unmerged_commit_count: 3,
            ..clean_input()
        };
        let result = assess_safety(&input);
        assert!(
            result.contains(&SafetyViolation::UnmergedCommits { count: 3 }),
            "expected UnmergedCommits violation with count=3"
        );
    }

    #[test]
    fn assess_stashed_changes() {
        let input = WorktreeSafetyInput {
            has_stash: true,
            ..clean_input()
        };
        let result = assess_safety(&input);
        assert!(
            result.contains(&SafetyViolation::StashedChanges),
            "expected StashedChanges violation"
        );
    }

    #[test]
    fn assess_unpushed_commits() {
        let input = WorktreeSafetyInput {
            is_pushed_to_remote: false,
            ..clean_input()
        };
        let result = assess_safety(&input);
        assert!(
            result.contains(&SafetyViolation::UnpushedCommits),
            "expected UnpushedCommits violation"
        );
    }

    #[test]
    fn assess_all_clean() {
        let input = clean_input();
        let result = assess_safety(&input);
        assert!(result.is_empty(), "expected no violations, got: {result:?}");
    }

    #[test]
    fn assess_safety_is_pure() {
        // Verify that the worktree module source does not import I/O libs.
        // Check for `use std::process`, `use std::fs`, `use std::net` import patterns only.
        let source = include_str!("mod.rs");
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ") || trimmed.starts_with("extern ") {
                let has_io = trimmed.contains("::process")
                    || trimmed.contains("::fs")
                    || trimmed.contains("::net");
                assert!(
                    !has_io,
                    "domain worktree must not import I/O modules, found: {trimmed}"
                );
            }
        }
    }

    #[test]
    fn assess_collects_all_failures() {
        // All 5 unsafe conditions simultaneously
        let input = WorktreeSafetyInput {
            has_uncommitted_changes: true,
            has_untracked_files: true,
            unmerged_commit_count: 2,
            has_stash: true,
            is_pushed_to_remote: false,
        };
        let result = assess_safety(&input);
        assert_eq!(
            result.len(),
            5,
            "expected all 5 violations, got: {result:?}"
        );
        assert!(result.contains(&SafetyViolation::UncommittedChanges));
        assert!(result.contains(&SafetyViolation::UntrackedFiles));
        assert!(result.contains(&SafetyViolation::UnmergedCommits { count: 2 }));
        assert!(result.contains(&SafetyViolation::StashedChanges));
        assert!(result.contains(&SafetyViolation::UnpushedCommits));
    }

    #[test]
    fn rejects_injection() {
        // semicolon
        assert!(WorktreeName::validate("ecc-session-20260328-150000-feat;rm-1234").is_err());
        // dollar sign
        assert!(WorktreeName::validate("ecc-session-20260328-150000-feat$var-1234").is_err());
        // backtick
        assert!(WorktreeName::validate("ecc-session-20260328-150000-feat`cmd`-1234").is_err());
        // double dot
        assert!(WorktreeName::validate("ecc-session-20260328-150000-feat..path-1234").is_err());
        // parentheses
        assert!(WorktreeName::validate("ecc-session-20260328-150000-feat(x)-1234").is_err());
        // curly braces
        assert!(WorktreeName::validate("ecc-session-20260328-150000-feat{x}-1234").is_err());
        // space
        assert!(WorktreeName::validate("ecc-session-20260328-150000-feat x-1234").is_err());
        // pipe
        assert!(WorktreeName::validate("ecc-session-20260328-150000-feat|cmd-1234").is_err());
        // redirect
        assert!(WorktreeName::validate("ecc-session-20260328-150000-feat>out-1234").is_err());
        assert!(WorktreeName::validate("ecc-session-20260328-150000-feat<in-1234").is_err());
        // valid name should pass
        assert!(WorktreeName::validate("ecc-session-20260328-150000-my-feature-1234").is_ok());
    }

    #[test]
    fn generates_correct_format() {
        let name = WorktreeName::generate("dev", "my feature", 12345).unwrap();
        let s = name.as_str();
        // must start with ecc-session-
        assert!(s.starts_with("ecc-session-"), "got: {s}");
        // must end with the pid
        assert!(s.ends_with("-12345"), "got: {s}");
        // slug must be sanitized
        assert!(s.contains("my-feature"), "got: {s}");
        // timestamp portion: digits only in YYYYMMDD-HHMMSS pattern
        let parts: Vec<&str> = s.splitn(4, '-').collect();
        // parts[0]="ecc", parts[1]="session", parts[2]=YYYYMMDD, rest
        assert_eq!(parts[0], "ecc");
        assert_eq!(parts[1], "session");
        // timestamp is 8 digits
        assert_eq!(parts[2].len(), 8, "date part len: {}", parts[2].len());
        assert!(
            parts[2].chars().all(|c: char| c.is_ascii_digit()),
            "date: {}",
            parts[2]
        );
    }

    #[test]
    fn parses_name() {
        let parsed = WorktreeName::parse("ecc-session-20260328-150000-my-feature-12345");
        assert!(parsed.is_some(), "expected Some, got None");
        let p = parsed.unwrap();
        assert_eq!(p.timestamp, "20260328-150000");
        assert_eq!(p.slug, "my-feature");
        assert_eq!(p.pid, 12345);
    }

    #[test]
    fn parses_prefixed_name() {
        let parsed = WorktreeName::parse("worktree-ecc-session-20260404-150000-my-feature-12345");
        assert!(parsed.is_some(), "expected Some for worktree-prefixed name");
        let p = parsed.unwrap();
        assert_eq!(p.timestamp, "20260404-150000");
        assert_eq!(p.slug, "my-feature");
        assert_eq!(p.pid, 12345);
    }

    #[test]
    fn rejects_random_name() {
        assert!(WorktreeName::parse("random-branch-name").is_none());
    }

    #[test]
    fn rejects_double_prefix() {
        assert!(
            WorktreeName::parse("worktree-worktree-ecc-session-20260404-150000-my-feature-12345")
                .is_none()
        );
    }

    #[test]
    fn rejects_prefixed_non_session() {
        assert!(WorktreeName::parse("worktree-feature-x").is_none());
    }

    #[test]
    fn prefixed_and_unprefixed_produce_identical_fields() {
        let unprefixed =
            WorktreeName::parse("ecc-session-20260404-150000-my-feature-12345").unwrap();
        let prefixed =
            WorktreeName::parse("worktree-ecc-session-20260404-150000-my-feature-12345").unwrap();
        assert_eq!(unprefixed.timestamp, prefixed.timestamp);
        assert_eq!(unprefixed.slug, prefixed.slug);
        assert_eq!(unprefixed.pid, prefixed.pid);
    }

    #[test]
    fn eq_platform_case_normalized() {
        // Two names that differ only in case.
        let lower = WorktreeName {
            name: "ecc-session-20260404-150000-my-feature-12345".to_owned(),
        };
        let upper = WorktreeName {
            name: "ECC-SESSION-20260404-150000-MY-FEATURE-12345".to_owned(),
        };

        #[cfg(target_os = "macos")]
        {
            assert!(
                lower.eq_platform(&upper),
                "eq_platform must be case-insensitive on macOS"
            );
            assert!(
                upper.eq_platform(&lower),
                "eq_platform must be symmetric on macOS"
            );
        }
        #[cfg(not(target_os = "macos"))]
        {
            assert!(
                !lower.eq_platform(&upper),
                "eq_platform must be case-sensitive on non-macOS"
            );
            // Identical strings must still be equal.
            assert!(
                lower.eq_platform(&lower.clone()),
                "eq_platform must return true for identical strings"
            );
        }
    }
}
