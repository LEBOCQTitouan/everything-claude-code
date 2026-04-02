/// Lexical path normalization — pure, no I/O.
///
/// Rules:
/// - `.` components are skipped
/// - `..` pops the last normal component; at root level `..` is preserved
/// - Leading `/` is preserved for absolute paths
/// - Empty input returns empty string
pub fn normalize_path(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }

    let absolute = path.starts_with('/');
    let mut components: Vec<&str> = Vec::new();

    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                // Pop if top is a normal component; preserve leading `..` otherwise
                if components.last().is_some_and(|c| *c != "..") {
                    components.pop();
                } else {
                    components.push("..");
                }
            }
            segment => components.push(segment),
        }
    }

    if absolute {
        format!("/{}", components.join("/"))
    } else if components.is_empty() {
        String::new()
    } else {
        components.join("/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // PC-001: strips `..` components
    #[test]
    fn strips_dotdot_components() {
        assert_eq!(
            normalize_path("docs/specs/../../src/evil.rs"),
            "src/evil.rs"
        );
    }

    // PC-001: leading `..` is preserved (at root, cannot go above)
    #[test]
    fn preserves_leading_dotdot() {
        assert_eq!(normalize_path("../outside"), "../outside");
    }

    // PC-002: strips `.` components
    #[test]
    fn strips_dot_components() {
        assert_eq!(normalize_path("./docs/specs/foo.md"), "docs/specs/foo.md");
    }

    // PC-003: preserves absolute paths
    #[test]
    fn preserves_absolute_paths() {
        assert_eq!(normalize_path("/absolute/path"), "/absolute/path");
    }

    // PC-004: handles complex traversal
    #[test]
    fn handles_complex_traversal() {
        assert_eq!(normalize_path("a/b/../c/./d"), "a/c/d");
    }

    // Edge cases
    #[test]
    fn empty_string_returns_empty() {
        assert_eq!(normalize_path(""), "");
    }

    #[test]
    fn single_dot_returns_empty() {
        assert_eq!(normalize_path("."), "");
    }
}
