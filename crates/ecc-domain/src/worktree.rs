/// Value object for worktree names used in session isolation.
/// Format: `ecc-session-{YYYYMMDD-HHMMSS}-{slug}-{pid}`
///
/// Slug is sanitized: lowercase, `[a-z0-9-]` only, max 40 chars.
/// Names validated against allowlist: `[a-zA-Z0-9/_-]` only, max 255 bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeName {
    name: String,
}

/// Components extracted from a parsed worktree name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedWorktreeName {
    pub timestamp: String,
    pub slug: String,
    pub pid: u32,
}

/// Errors that can occur when creating or validating a `WorktreeName`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorktreeNameError {
    TooLong(usize),
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let parsed =
            WorktreeName::parse("worktree-ecc-session-20260404-150000-my-feature-12345");
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
        assert!(WorktreeName::parse(
            "worktree-worktree-ecc-session-20260404-150000-my-feature-12345"
        )
        .is_none());
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
            WorktreeName::parse("worktree-ecc-session-20260404-150000-my-feature-12345")
                .unwrap();
        assert_eq!(unprefixed.timestamp, prefixed.timestamp);
        assert_eq!(unprefixed.slug, prefixed.slug);
        assert_eq!(unprefixed.pid, prefixed.pid);
    }
}
