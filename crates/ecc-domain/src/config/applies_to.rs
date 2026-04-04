//! AppliesTo frontmatter type — conditional rule loading for stack-based filtering.
//!
//! Rules without `applies-to` are universally applicable (backwards compatible).
//! Rules with `applies-to` are only installed when the detected stack matches.

use std::collections::HashMap;

/// Conditions for when a rule should be installed.
///
/// All conditions are combined with OR semantics: if ANY language matches,
/// OR ANY framework matches, OR ANY file matches, the rule applies.
/// An empty `AppliesTo` (all vecs empty) means universally applicable.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppliesTo {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub files: Vec<String>,
}

impl AppliesTo {
    /// Returns true when all condition lists are empty (universal rule).
    pub fn is_empty(&self) -> bool {
        self.languages.is_empty() && self.frameworks.is_empty() && self.files.is_empty()
    }
}

/// Detected project stack from filesystem marker inspection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DetectedStack {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    /// Sentinel files found at project root (for `files:` condition matching).
    pub files: Vec<String>,
}

/// Parse `applies-to` value from a frontmatter HashMap.
///
/// Returns `None` when the `applies-to` key is absent (universal rule, backwards compat).
/// Returns `Some(AppliesTo::default())` for empty or malformed values (fail-open).
///
/// Input format: `{ languages: [rust, python], frameworks: [django] }`
/// Values are normalised to lowercase. Quoted values are stripped of quotes.
pub fn parse_applies_to(frontmatter: &HashMap<String, String>) -> Option<AppliesTo> {
    let raw = frontmatter.get("applies-to")?;
    Some(parse_applies_to_value(raw))
}

/// Parse the raw string value of the `applies-to` frontmatter field.
fn parse_applies_to_value(raw: &str) -> AppliesTo {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return AppliesTo::default();
    }

    // Strip outer braces: `{ ... }` → `...`
    let inner = if trimmed.starts_with('{') && trimmed.ends_with('}') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        // Malformed: not a brace-enclosed map — fail-open
        return AppliesTo::default();
    };

    let mut applies_to = AppliesTo::default();

    // Split on commas that are NOT inside square brackets
    for segment in split_outside_brackets(inner) {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        if let Some(colon_idx) = segment.find(':') {
            let key = segment[..colon_idx].trim();
            let value_str = segment[colon_idx + 1..].trim();
            let values = parse_bracket_list(value_str);
            match key {
                "languages" => applies_to.languages = values,
                "frameworks" => applies_to.frameworks = values,
                "files" => applies_to.files = values,
                _ => {} // Unknown keys are ignored (fail-open)
            }
        }
    }

    applies_to
}

/// Split a string on commas that are not inside square brackets `[...]`.
fn split_outside_brackets(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0_usize;
    let mut start = 0;

    for (i, ch) in s.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                result.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    result.push(&s[start..]);
    result
}

/// Parse a bracket-delimited list of values.
///
/// Handles: `[rust, python]`, `["manage.py"]`, `[rust]`, bare `rust`.
/// Values are normalised to lowercase and have quotes stripped.
fn parse_bracket_list(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    let inner = if trimmed.starts_with('[') && trimmed.ends_with(']') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };
    if inner.trim().is_empty() {
        return vec![];
    }
    inner
        .split(',')
        .map(|s| {
            s.trim()
                .trim_matches('"')
                .trim_matches('\'')
                .trim()
                .to_lowercase()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// Valid language identifiers (derived from `LANGUAGE_RULES`).
pub const VALID_LANGUAGES: &[&str] = &[
    "python",
    "typescript",
    "javascript",
    "golang",
    "rust",
    "ruby",
    "java",
    "csharp",
    "swift",
    "kotlin",
    "elixir",
    "php",
    "cpp",
    "shell",
    "yaml",
    "json",
    "perl",
];

/// Valid framework identifiers (derived from `FRAMEWORK_RULES`).
pub const VALID_FRAMEWORKS: &[&str] = &[
    "django",
    "fastapi",
    "flask",
    "nextjs",
    "react",
    "vue",
    "angular",
    "svelte",
    "express",
    "nestjs",
    "remix",
    "astro",
    "nuxt",
    "electron",
    "rails",
    "gin",
    "echo",
    "actix",
    "axum",
    "spring",
    "laravel",
    "symfony",
    "phoenix",
    "ktor",
    "aspnetcore",
];

/// Validate an `AppliesTo` value and return warning messages for unknown values.
///
/// Returns empty vec when all values are valid. Validation warnings do NOT
/// cause rule installation to fail — they are informational only.
pub fn validate_applies_to(applies_to: &AppliesTo) -> Vec<String> {
    let mut warnings = Vec::new();

    for lang in &applies_to.languages {
        if !VALID_LANGUAGES.contains(&lang.as_str()) {
            warnings.push(format!(
                "applies-to: unknown language '{}'; valid values: {}",
                lang,
                VALID_LANGUAGES.join(", ")
            ));
        }
    }

    for fw in &applies_to.frameworks {
        if !VALID_FRAMEWORKS.contains(&fw.as_str()) {
            warnings.push(format!(
                "applies-to: unknown framework '{}'; valid values: {}",
                fw,
                VALID_FRAMEWORKS.join(", ")
            ));
        }
    }

    warnings
}

/// Evaluate whether a rule applies to the detected project stack.
///
/// Returns `true` when:
/// - `applies_to` is `None` (no field = universal)
/// - `applies_to` is `Some` with all empty vecs (empty conditions = universal)
/// - ANY language in `applies_to.languages` is present in `stack.languages`
/// - ANY framework in `applies_to.frameworks` is present in `stack.frameworks`
/// - ANY file in `applies_to.files` is present in `stack.files`
///
/// Otherwise returns `false`.
pub fn evaluate_applicability(applies_to: &Option<AppliesTo>, stack: &DetectedStack) -> bool {
    let at = match applies_to {
        None => return true,
        Some(at) => at,
    };

    if at.is_empty() {
        return true;
    }

    let lang_match = at
        .languages
        .iter()
        .any(|l| stack.languages.contains(l));
    let fw_match = at
        .frameworks
        .iter()
        .any(|f| stack.frameworks.contains(f));
    let file_match = at.files.iter().any(|f| stack.files.contains(f));

    lang_match || fw_match || file_match
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------------------
    // parse_applies_to tests
    // ---------------------------------------------------------------------------

    fn fm(key: &str, value: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert(key.to_string(), value.to_string());
        map
    }

    #[test]
    fn parse_languages_single() {
        let map = fm("applies-to", "{ languages: [rust] }");
        let result = parse_applies_to(&map).unwrap();
        assert_eq!(result.languages, vec!["rust".to_string()]);
        assert!(result.frameworks.is_empty());
        assert!(result.files.is_empty());
    }

    #[test]
    fn parse_languages_multiple() {
        let map = fm("applies-to", "{ languages: [rust, python] }");
        let result = parse_applies_to(&map).unwrap();
        assert_eq!(result.languages, vec!["rust".to_string(), "python".to_string()]);
    }

    #[test]
    fn parse_frameworks_single() {
        let map = fm("applies-to", "{ frameworks: [django] }");
        let result = parse_applies_to(&map).unwrap();
        assert_eq!(result.frameworks, vec!["django".to_string()]);
        assert!(result.languages.is_empty());
    }

    #[test]
    fn parse_files_single() {
        let map = fm("applies-to", r#"{ files: ["manage.py"] }"#);
        let result = parse_applies_to(&map).unwrap();
        assert_eq!(result.files, vec!["manage.py".to_string()]);
    }

    #[test]
    fn parse_files_without_quotes() {
        let map = fm("applies-to", "{ files: [manage.py] }");
        let result = parse_applies_to(&map).unwrap();
        assert_eq!(result.files, vec!["manage.py".to_string()]);
    }

    #[test]
    fn parse_multiple_conditions() {
        let map = fm("applies-to", "{ languages: [python], frameworks: [django] }");
        let result = parse_applies_to(&map).unwrap();
        assert_eq!(result.languages, vec!["python".to_string()]);
        assert_eq!(result.frameworks, vec!["django".to_string()]);
    }

    #[test]
    fn parse_no_applies_to_key_returns_none() {
        let map: HashMap<String, String> = HashMap::new();
        assert_eq!(parse_applies_to(&map), None);
    }

    #[test]
    fn parse_empty_value_returns_default() {
        let map = fm("applies-to", "");
        let result = parse_applies_to(&map).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_empty_braces_returns_default() {
        let map = fm("applies-to", "{}");
        let result = parse_applies_to(&map).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_malformed_no_braces_returns_default() {
        let map = fm("applies-to", "languages: [rust]");
        let result = parse_applies_to(&map).unwrap();
        // No outer braces → fail-open → empty
        assert!(result.is_empty());
    }

    #[test]
    fn parse_values_normalized_to_lowercase() {
        let map = fm("applies-to", "{ languages: [Rust, PYTHON] }");
        let result = parse_applies_to(&map).unwrap();
        assert_eq!(result.languages, vec!["rust".to_string(), "python".to_string()]);
    }

    #[test]
    fn parse_whitespace_variants() {
        let map = fm("applies-to", "{  languages:  [ rust ,  python ]  }");
        let result = parse_applies_to(&map).unwrap();
        assert_eq!(result.languages, vec!["rust".to_string(), "python".to_string()]);
    }

    #[test]
    fn is_empty_on_default() {
        let at = AppliesTo::default();
        assert!(at.is_empty());
    }

    #[test]
    fn is_empty_on_populated() {
        let at = AppliesTo {
            languages: vec!["rust".to_string()],
            frameworks: vec![],
            files: vec![],
        };
        assert!(!at.is_empty());
    }

    // ---------------------------------------------------------------------------
    // evaluate_applicability tests
    // ---------------------------------------------------------------------------

    fn stack_with_lang(lang: &str) -> DetectedStack {
        DetectedStack {
            languages: vec![lang.to_string()],
            frameworks: vec![],
            files: vec![],
        }
    }

    fn stack_with_framework(fw: &str) -> DetectedStack {
        DetectedStack {
            languages: vec![],
            frameworks: vec![fw.to_string()],
            files: vec![],
        }
    }

    #[test]
    fn evaluate_none_applies_to_is_universal() {
        let stack = stack_with_lang("rust");
        assert!(evaluate_applicability(&None, &stack));
    }

    #[test]
    fn evaluate_none_applies_to_is_universal_with_empty_stack() {
        let stack = DetectedStack::default();
        assert!(evaluate_applicability(&None, &stack));
    }

    #[test]
    fn evaluate_empty_applies_to_is_universal() {
        let at = Some(AppliesTo::default());
        let stack = stack_with_lang("rust");
        assert!(evaluate_applicability(&at, &stack));
    }

    #[test]
    fn evaluate_language_match_returns_true() {
        let at = Some(AppliesTo {
            languages: vec!["rust".to_string()],
            frameworks: vec![],
            files: vec![],
        });
        let stack = stack_with_lang("rust");
        assert!(evaluate_applicability(&at, &stack));
    }

    #[test]
    fn evaluate_language_no_match_returns_false() {
        let at = Some(AppliesTo {
            languages: vec!["python".to_string()],
            frameworks: vec![],
            files: vec![],
        });
        let stack = stack_with_lang("rust");
        assert!(!evaluate_applicability(&at, &stack));
    }

    #[test]
    fn evaluate_framework_match_returns_true() {
        let at = Some(AppliesTo {
            languages: vec![],
            frameworks: vec!["actix".to_string()],
            files: vec![],
        });
        let stack = stack_with_framework("actix");
        assert!(evaluate_applicability(&at, &stack));
    }

    #[test]
    fn evaluate_file_match_returns_true() {
        let at = Some(AppliesTo {
            languages: vec![],
            frameworks: vec![],
            files: vec!["manage.py".to_string()],
        });
        let stack = DetectedStack {
            languages: vec![],
            frameworks: vec![],
            files: vec!["manage.py".to_string()],
        };
        assert!(evaluate_applicability(&at, &stack));
    }

    #[test]
    fn evaluate_or_semantics_any_match_wins() {
        // languages: [python], frameworks: [actix] — stack has actix only
        let at = Some(AppliesTo {
            languages: vec!["python".to_string()],
            frameworks: vec!["actix".to_string()],
            files: vec![],
        });
        let stack = stack_with_framework("actix");
        assert!(evaluate_applicability(&at, &stack));
    }

    #[test]
    fn evaluate_no_match_returns_false() {
        let at = Some(AppliesTo {
            languages: vec!["python".to_string()],
            frameworks: vec!["django".to_string()],
            files: vec!["manage.py".to_string()],
        });
        let stack = stack_with_lang("rust");
        assert!(!evaluate_applicability(&at, &stack));
    }

    #[test]
    fn evaluate_empty_stack_with_conditions_returns_false() {
        let at = Some(AppliesTo {
            languages: vec!["rust".to_string()],
            frameworks: vec![],
            files: vec![],
        });
        let stack = DetectedStack::default();
        assert!(!evaluate_applicability(&at, &stack));
    }

    // ---------------------------------------------------------------------------
    // validate_applies_to tests
    // ---------------------------------------------------------------------------

    #[test]
    fn validate_known_language_returns_empty() {
        let at = AppliesTo {
            languages: vec!["rust".to_string()],
            frameworks: vec![],
            files: vec![],
        };
        assert!(validate_applies_to(&at).is_empty());
    }

    #[test]
    fn validate_unknown_language_returns_warning() {
        let at = AppliesTo {
            languages: vec!["bogus".to_string()],
            frameworks: vec![],
            files: vec![],
        };
        let warnings = validate_applies_to(&at);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("bogus"));
    }

    #[test]
    fn validate_known_framework_returns_empty() {
        let at = AppliesTo {
            languages: vec![],
            frameworks: vec!["django".to_string()],
            files: vec![],
        };
        assert!(validate_applies_to(&at).is_empty());
    }

    #[test]
    fn validate_unknown_framework_returns_warning() {
        let at = AppliesTo {
            languages: vec![],
            frameworks: vec!["bogus-fw".to_string()],
            files: vec![],
        };
        let warnings = validate_applies_to(&at);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("bogus-fw"));
    }

    #[test]
    fn validate_files_condition_no_warning() {
        // File conditions are not validated (any filename is valid)
        let at = AppliesTo {
            languages: vec![],
            frameworks: vec![],
            files: vec!["some-marker.json".to_string()],
        };
        assert!(validate_applies_to(&at).is_empty());
    }

    #[test]
    fn validate_empty_applies_to_returns_empty() {
        let at = AppliesTo::default();
        assert!(validate_applies_to(&at).is_empty());
    }
}
