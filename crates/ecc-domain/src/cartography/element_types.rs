//! Element types for cartography elements.
//!
//! Pure domain types — no I/O.

/// A single element entry parsed from a `docs/cartography/elements/*.md` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementEntry {
    /// Slug derived from the file stem (e.g. `cartographer` for `cartographer.md`).
    pub slug: String,
    /// Raw markdown content.
    pub content: String,
}

/// Infer the element type from a file path.
///
/// Returns `Some(type_str)` when the path belongs to a known element category,
/// or `None` when the path is not an element target.
///
/// Element target directories: `agents/`, `commands/`, `skills/`, `hooks/`, `rules/`, `crates/`.
pub fn infer_element_type_from_path(path: &str) -> Option<&'static str> {
    let prefixes: &[(&str, &str)] = &[
        ("agents/", "agent"),
        ("commands/", "command"),
        ("skills/", "skill"),
        ("hooks/", "hook"),
        ("rules/", "rule"),
        ("crates/", "crate"),
    ];
    for (prefix, kind) in prefixes {
        if path.starts_with(prefix) {
            return Some(kind);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_agent_type() {
        assert_eq!(
            infer_element_type_from_path("agents/cartographer.md"),
            Some("agent")
        );
    }

    #[test]
    fn infer_crate_type() {
        assert_eq!(
            infer_element_type_from_path("crates/ecc-domain/src/lib.rs"),
            Some("crate")
        );
    }

    #[test]
    fn non_element_returns_none() {
        assert_eq!(infer_element_type_from_path("docs/guide.md"), None);
        assert_eq!(infer_element_type_from_path("src/main.rs"), None);
    }
}
