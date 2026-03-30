//! Audit dimension domain types and query template sanitization.

use serde::{Deserialize, Serialize};

/// A single audit dimension — standard or custom — used in Phase 2 scanning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditDimension {
    pub name: String,
    pub query_template: String,
    pub enabled: bool,
    pub is_custom: bool,
}

/// Errors from dimension validation.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DimensionError {
    #[error(
        "query template contains disallowed characters. \
         Only alphanumeric, spaces, hyphens, underscores, dots, \
         forward slashes, and {{placeholder}} tokens are allowed."
    )]
    InvalidQueryTemplate,
}

/// Validate that a query template contains only safe characters.
///
/// Allowed: `[a-zA-Z0-9 -_./{}]`
/// Rejected: shell metacharacters (`;`, `|`, `$`, backtick, `<`, `>`, `&`, `!`, `(`, `)`, `'`, `"`)
pub fn validate_query_template(template: &str) -> Result<(), DimensionError> {
    todo!("validate_query_template not implemented")
}

/// Return the 8 standard audit dimensions.
pub fn standard_dimensions() -> Vec<AuditDimension> {
    todo!("standard_dimensions not implemented")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_query_template() {
        let result = validate_query_template("rust {project} performance");
        assert!(result.is_ok(), "expected Ok, got {result:?}");
    }

    #[test]
    fn rejects_shell_metacharacters() {
        let metacharacters = [";", "|", "$", "`", "<", ">", "&", "!", "(", ")", "'", "\""];
        for ch in metacharacters {
            let template = format!("safe text {ch} more");
            let result = validate_query_template(&template);
            assert!(
                result.is_err(),
                "expected error for template containing '{ch}', got Ok"
            );
        }
    }

    #[test]
    fn allows_safe_chars() {
        let safe_templates = [
            "alphanumeric123",
            "with spaces",
            "with-hyphens",
            "with_underscores",
            "with.dots",
            "with/slashes",
            "{placeholder} token",
            "mixed ABC-123_test.query/{placeholder}",
        ];
        for template in safe_templates {
            let result = validate_query_template(template);
            assert!(
                result.is_ok(),
                "expected Ok for template '{template}', got {result:?}"
            );
        }
    }

    #[test]
    fn standard_dimensions() {
        let dims = super::standard_dimensions();
        assert_eq!(dims.len(), 8, "expected exactly 8 standard dimensions");
        for dim in &dims {
            assert!(
                !dim.name.is_empty(),
                "dimension name must not be empty"
            );
            assert!(
                !dim.query_template.is_empty(),
                "dimension '{}' must have a query template",
                dim.name
            );
            let validation = validate_query_template(&dim.query_template);
            assert!(
                validation.is_ok(),
                "standard dimension '{}' has invalid template '{}': {validation:?}",
                dim.name,
                dim.query_template
            );
            assert!(!dim.is_custom, "standard dimensions must have is_custom=false");
            assert!(dim.enabled, "standard dimensions must be enabled by default");
        }
    }
}
