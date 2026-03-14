use std::path::Path;

/// Validate a session name: must match `^[a-zA-Z0-9][-a-zA-Z0-9]*$`.
pub fn is_valid_session_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let bytes = name.as_bytes();

    // First char must be alphanumeric
    if !bytes[0].is_ascii_alphanumeric() {
        return false;
    }

    // Rest must be alphanumeric or hyphen
    bytes[1..].iter().all(|&b| b.is_ascii_alphanumeric() || b == b'-')
}

/// Extract session name from a session file path (strip `.md` extension).
pub fn session_name_from_path(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- is_valid_session_name ---

    #[test]
    fn valid_simple_name() {
        assert!(is_valid_session_name("test"));
    }

    #[test]
    fn valid_with_numbers() {
        assert!(is_valid_session_name("test123"));
    }

    #[test]
    fn valid_with_hyphens() {
        assert!(is_valid_session_name("my-session"));
    }

    #[test]
    fn valid_starts_with_number() {
        assert!(is_valid_session_name("1session"));
    }

    #[test]
    fn valid_single_char() {
        assert!(is_valid_session_name("a"));
    }

    #[test]
    fn valid_single_digit() {
        assert!(is_valid_session_name("1"));
    }

    #[test]
    fn valid_mixed_case() {
        assert!(is_valid_session_name("MySession"));
    }

    #[test]
    fn invalid_empty() {
        assert!(!is_valid_session_name(""));
    }

    #[test]
    fn invalid_starts_with_hyphen() {
        assert!(!is_valid_session_name("-session"));
    }

    #[test]
    fn invalid_contains_space() {
        assert!(!is_valid_session_name("my session"));
    }

    #[test]
    fn invalid_contains_underscore() {
        assert!(!is_valid_session_name("my_session"));
    }

    #[test]
    fn invalid_contains_dot() {
        assert!(!is_valid_session_name("my.session"));
    }

    #[test]
    fn invalid_contains_slash() {
        assert!(!is_valid_session_name("my/session"));
    }

    #[test]
    fn invalid_special_chars() {
        assert!(!is_valid_session_name("test@name"));
        assert!(!is_valid_session_name("test!"));
    }

    // --- Property-based tests ---

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn valid_names_always_start_with_alnum(
                first in "[a-zA-Z0-9]",
                rest in "[a-zA-Z0-9-]{0,20}"
            ) {
                let name = format!("{first}{rest}");
                prop_assert!(is_valid_session_name(&name));
            }

            #[test]
            fn names_with_non_alnum_hyphen_are_invalid(
                name in "[a-zA-Z0-9][a-zA-Z0-9-]*[^a-zA-Z0-9-][a-zA-Z0-9-]*"
            ) {
                prop_assert!(!is_valid_session_name(&name));
            }

            #[test]
            fn empty_name_is_never_valid(name in "\\PC{0,0}") {
                let _ = name;
                prop_assert!(!is_valid_session_name(""));
            }
        }
    }

    // --- session_name_from_path ---

    #[test]
    fn name_from_path_basic() {
        let path = Path::new("/home/user/.claude/claw/sessions/my-session.md");
        assert_eq!(session_name_from_path(path), Some("my-session".to_string()));
    }

    #[test]
    fn name_from_path_no_extension() {
        let path = Path::new("/tmp/session");
        assert_eq!(session_name_from_path(path), Some("session".to_string()));
    }
}
