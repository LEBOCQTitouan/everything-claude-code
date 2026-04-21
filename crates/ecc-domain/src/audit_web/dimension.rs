//! Audit dimension domain types and query template sanitization.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// A single audit dimension — standard or custom — used in Phase 2 scanning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditDimension {
    /// Name of the dimension (e.g., "Security").
    pub name: String,
    /// Query template with optional `{project}` placeholder.
    pub query_template: String,
    /// Whether this dimension is active.
    pub enabled: bool,
    /// True if this is a user-defined custom dimension.
    pub is_custom: bool,
}

/// Errors from dimension validation.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum DimensionError {
    /// Query template contains shell metacharacters or other unsafe characters.
    #[error(
        "query template contains disallowed characters. \
         Only alphanumeric, spaces, hyphens, underscores, dots, \
         forward slashes, and {{placeholder}} tokens are allowed."
    )]
    InvalidQueryTemplate,
}

/// Regex pattern for allowed query template characters.
const SAFE_TEMPLATE_PATTERN: &str = r"^[a-zA-Z0-9 _./{}\-]+$";

static RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(SAFE_TEMPLATE_PATTERN).expect("BUG: invalid SAFE_TEMPLATE_PATTERN regex")
});

/// Validate that a query template contains only safe characters.
///
/// Allowed: `[a-zA-Z0-9 -_./{}]`
/// Rejected: shell metacharacters (`;`, `|`, `$`, backtick, `<`, `>`, `&`, `!`, `(`, `)`, `'`, `"`)
pub fn validate_query_template(template: &str) -> Result<(), DimensionError> {
    if RE.is_match(template) {
        Ok(())
    } else {
        Err(DimensionError::InvalidQueryTemplate)
    }
}

/// Return the 8 standard audit dimensions.
pub fn standard_dimensions() -> Vec<AuditDimension> {
    vec![
        AuditDimension {
            name: "Techniques".to_owned(),
            query_template: "{project} software engineering techniques best practices".to_owned(),
            enabled: true,
            is_custom: false,
        },
        AuditDimension {
            name: "Tools".to_owned(),
            query_template: "{project} developer tools CLI utilities".to_owned(),
            enabled: true,
            is_custom: false,
        },
        AuditDimension {
            name: "Platforms".to_owned(),
            query_template: "{project} cloud platform infrastructure".to_owned(),
            enabled: true,
            is_custom: false,
        },
        AuditDimension {
            name: "Languages & Frameworks".to_owned(),
            query_template: "{project} programming languages frameworks libraries".to_owned(),
            enabled: true,
            is_custom: false,
        },
        AuditDimension {
            name: "Security".to_owned(),
            query_template: "{project} security vulnerabilities advisories".to_owned(),
            enabled: true,
            is_custom: false,
        },
        AuditDimension {
            name: "Testing".to_owned(),
            query_template: "{project} testing strategies quality assurance".to_owned(),
            enabled: true,
            is_custom: false,
        },
        AuditDimension {
            name: "Observability".to_owned(),
            query_template: "{project} observability monitoring tracing logging".to_owned(),
            enabled: true,
            is_custom: false,
        },
        AuditDimension {
            name: "Feature Opportunities".to_owned(),
            query_template: "{project} emerging features trends roadmap".to_owned(),
            enabled: true,
            is_custom: false,
        },
    ]
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
            assert!(!dim.name.is_empty(), "dimension name must not be empty");
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
            assert!(
                !dim.is_custom,
                "standard dimensions must have is_custom=false"
            );
            assert!(
                dim.enabled,
                "standard dimensions must be enabled by default"
            );
        }
    }
}
