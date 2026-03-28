/// Value object for worktree names used in session isolation.
/// Format: `ecc-session-{YYYYMMDD-HHMMSS}-{slug}-{pid}`
///
/// Slug is sanitized: lowercase, `[a-z0-9-]` only, max 40 chars.
/// Names validated against allowlist: `[a-zA-Z0-9/_-]` only, max 255 bytes.

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
        assert!(parts[2].chars().all(|c| c.is_ascii_digit()), "date: {}", parts[2]);
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
}
