//! Pure slug derivation for cartography filenames.
//!
//! Rules (Decision #11, #13):
//! - Input: first changed file's parent directory name, command name, or crate name
//! - Transform: lowercase, replace non-alphanumeric chars with hyphens,
//!   collapse consecutive hyphens to one, truncate at 60 characters
//!
//! No I/O — pure string transformation.

/// Derive a slug from the given input string.
///
/// Rules:
/// 1. Lowercase all characters
/// 2. Replace any non-alphanumeric character with a hyphen
/// 3. Collapse consecutive hyphens into a single hyphen
/// 4. Truncate to 60 characters maximum
/// 5. Strip leading/trailing hyphens after truncation
pub fn derive_slug(_input: &str) -> String {
    // Stub: returns empty string — tests will fail (RED phase)
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_is_lowercase() {
        assert_eq!(derive_slug("EccDomain"), "eccdomain");
    }

    #[test]
    fn slug_replaces_non_alphanumeric_with_hyphens() {
        assert_eq!(derive_slug("ecc_domain"), "ecc-domain");
        assert_eq!(derive_slug("ecc domain"), "ecc-domain");
        assert_eq!(derive_slug("ecc.domain"), "ecc-domain");
    }

    #[test]
    fn slug_collapses_multiple_hyphens() {
        assert_eq!(derive_slug("ecc--domain"), "ecc-domain");
        assert_eq!(derive_slug("ecc___domain"), "ecc-domain");
        assert_eq!(derive_slug("ecc - domain"), "ecc-domain");
    }

    #[test]
    fn slug_truncates_at_60_chars() {
        let long_input = "a".repeat(80);
        let result = derive_slug(&long_input);
        assert_eq!(result.len(), 60);
        assert_eq!(result, "a".repeat(60));
    }

    #[test]
    fn slug_truncates_and_strips_trailing_hyphens() {
        // 59 'a' chars + non-alphanumeric at position 60 — after truncation strip trailing hyphen
        let input = format!("{}-extra", "a".repeat(59));
        let result = derive_slug(&input);
        assert!(result.len() <= 60);
        assert!(!result.ends_with('-'));
    }

    #[test]
    fn slug_strips_leading_and_trailing_hyphens() {
        assert_eq!(derive_slug("_ecc_"), "ecc");
    }

    #[test]
    fn slug_handles_empty_input() {
        assert_eq!(derive_slug(""), "");
    }

    #[test]
    fn slug_realistic_crate_name() {
        assert_eq!(derive_slug("ecc-domain"), "ecc-domain");
    }

    #[test]
    fn slug_realistic_command_name() {
        assert_eq!(derive_slug("spec-dev"), "spec-dev");
    }
}
