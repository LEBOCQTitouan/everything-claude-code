//! Pure validation helpers for content files.

use std::collections::HashMap;

/// Valid model identifiers for agent frontmatter.
pub const VALID_MODELS: &[&str] = &["haiku", "sonnet", "opus"];

/// Valid effort levels for agent thinking budget configuration.
pub const VALID_EFFORT_LEVELS: &[&str] = &["low", "medium", "high", "max"];

/// Effort-to-tokens mapping for MAX_THINKING_TOKENS.
/// Authoritative source for the effort → token budget lookup.
pub const EFFORT_TOKENS: &[(&str, u32)] = &[
    ("low", 2_048),
    ("medium", 8_192),
    ("high", 16_384),
    ("max", 32_768),
];


/// Canonical language identifiers for pattern frontmatter (AC-001.5).
///
/// 10 named languages plus the special value "all" for universal patterns.
pub const VALID_PATTERN_LANGUAGES: &[&str] = &[
    "rust",
    "go",
    "python",
    "typescript",
    "java",
    "kotlin",
    "csharp",
    "cpp",
    "swift",
    "shell",
    "all",
];

/// Canonical difficulty values for pattern frontmatter (AC-001.6).
pub const VALID_PATTERN_DIFFICULTIES: &[&str] = &["beginner", "intermediate", "advanced"];

/// Required markdown sections that every pattern file must contain (AC-002.5).
pub const REQUIRED_PATTERN_SECTIONS: &[&str] = &[
    "Intent",
    "Problem",
    "Solution",
    "Language Implementations",
    "When to Use",
    "When NOT to Use",
    "Anti-Patterns",
    "Related Patterns",
    "References",
];

/// Known-unsafe code patterns to warn about in pattern file code blocks (AC-002.8).
pub const UNSAFE_CODE_PATTERNS: &[&str] = &[
    "eval(",
    "eval ",
    "exec(",
    "exec ",
    "system(",
    "innerHTML",
    "f\"SELECT",
    "f\"INSERT",
    "f\"UPDATE",
    "f\"DELETE",
    "f'SELECT",
    "f'INSERT",
    "f'UPDATE",
    "f'DELETE",
];

/// Canonical pattern category directory names for validation.
pub const VALID_PATTERN_CATEGORIES: &[&str] = &[
    "creational",
    "architecture",
    "structural",
    "behavioral",
    "concurrency",
    "error-handling",
    "resilience",
    "testing",
    "ddd",
    "api-design",
    "security",
    "observability",
    "cicd",
    "agentic",
    "functional",
    "data-access",
    "idioms",
];

/// Language subdirectories within the idioms category.
pub const IDIOM_SUBCATEGORIES: &[&str] = &["rust", "go", "python", "typescript", "kotlin"];

/// Pattern files exceeding this line count trigger a validation warning.
pub const PATTERN_SIZE_WARNING_LINES: usize = 500;

/// Check if a string matches kebab-case: `^[a-z][a-z0-9]*(-[a-z0-9]+)*$`
pub fn is_kebab_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let bytes = s.as_bytes();
    if !bytes[0].is_ascii_lowercase() {
        return false;
    }
    let mut prev_hyphen = false;
    for &b in &bytes[1..] {
        match b {
            b'-' => {
                if prev_hyphen {
                    return false;
                }
                prev_hyphen = true;
            }
            b'a'..=b'z' | b'0'..=b'9' => {
                prev_hyphen = false;
            }
            _ => return false,
        }
    }
    !prev_hyphen
}

/// Valid hook event types (all 21 Claude Code hook events).
pub const VALID_HOOK_EVENTS: &[&str] = &[
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "PreCompact",
    "PostCompact",
    "SessionStart",
    "SessionEnd",
    "Stop",
    "Notification",
    "SubagentStart",
    "SubagentStop",
    "UserPromptSubmit",
    "InstructionsLoaded",
    "PermissionRequest",
    "TeammateIdle",
    "TaskCompleted",
    "ConfigChange",
    "WorktreeCreate",
    "WorktreeRemove",
    "Elicitation",
    "ElicitationResult",
];

/// Parse a bracket-delimited tool list from frontmatter value string.
///
/// Handles: `["Read", "Write"]`, `[Read, Write]`, `Read` (bare string),
/// `[]` (empty list). Returns parsed tool names with whitespace/quotes trimmed.
pub fn parse_tool_list(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return vec![];
    }
    // Strip outer brackets if present
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
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// Severity for convention lint findings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LintSeverity {
    /// Critical issue (convention violation).
    Error,
    /// Informational warning.
    Warn,
}

/// A single convention lint finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintFinding {
    /// Severity level (Error or Warn).
    pub severity: LintSeverity,
    /// File path where the finding occurred.
    pub file: String,
    /// Description of the lint finding.
    pub message: String,
}

/// Check filename-vs-frontmatter naming consistency for a single file.
///
/// Returns findings (may be empty). `file_stem` is the filename without extension.
/// `frontmatter_name` is the `name` field value from frontmatter (None if missing).
/// `entity_kind` is "agent", "skill", etc. for error messages.
pub fn check_naming_consistency(
    file_stem: &str,
    frontmatter_name: Option<&str>,
    entity_kind: &str,
) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    let label = format!("{entity_kind} '{file_stem}'");

    // Kebab-case check on filename
    if !is_kebab_case(file_stem) {
        findings.push(LintFinding {
            severity: LintSeverity::Error,
            file: file_stem.to_string(),
            message: format!(
                "{label}: filename is not kebab-case (expected pattern: ^[a-z][a-z0-9]*(-[a-z0-9]+)*$)"
            ),
        });
    }

    // Name mismatch check
    match frontmatter_name {
        Some(name) if name.trim().is_empty() => {
            findings.push(LintFinding {
                severity: LintSeverity::Warn,
                file: file_stem.to_string(),
                message: format!("{label}: frontmatter 'name' is empty, skipping name match"),
            });
        }
        Some(name) if name != file_stem => {
            findings.push(LintFinding {
                severity: LintSeverity::Error,
                file: file_stem.to_string(),
                message: format!(
                    "{label}: filename '{file_stem}' differs from frontmatter name '{name}'"
                ),
            });
        }
        None => {
            findings.push(LintFinding {
                severity: LintSeverity::Warn,
                file: file_stem.to_string(),
                message: format!("{label}: missing frontmatter 'name' field, skipping name match"),
            });
        }
        _ => {} // name matches
    }

    findings
}

/// Validate tool names against the provided set of valid tool identifiers.
///
/// `raw_tools` is the raw frontmatter value for tools/allowed-tools.
/// `valid_tools` is the authoritative set of valid tool names (derive from
/// `ToolManifest::tools` in application code, or pass legacy list in tests).
/// Returns findings. Any invalid tool produces an ERROR for the whole file.
pub fn check_tool_values(
    file_stem: &str,
    raw_tools: &str,
    field_name: &str,
    valid_tools: &[&str],
) -> Vec<LintFinding> {
    let tools = parse_tool_list(raw_tools);
    let invalid: Vec<_> = tools
        .iter()
        .filter(|t| !valid_tools.contains(&t.as_str()))
        .collect();

    if invalid.is_empty() {
        return vec![];
    }

    vec![LintFinding {
        severity: LintSeverity::Error,
        file: file_stem.to_string(),
        message: format!(
            "'{file_stem}': invalid {field_name} {:?} — valid tools: {}",
            invalid,
            valid_tools.join(", ")
        ),
    }]
}

/// Extract YAML frontmatter from markdown content into a key-value map.
///
/// Looks for content between `---` delimiters at the start of the file,
/// optionally stripping a BOM prefix.
pub fn extract_frontmatter(content: &str) -> Option<HashMap<String, String>> {
    let clean = content.strip_prefix('\u{FEFF}').unwrap_or(content);
    let rest = clean.strip_prefix("---")?;
    let end = rest.find("\n---")?;
    let frontmatter_str = &rest[..end];

    let mut map = HashMap::new();
    for line in frontmatter_str.lines() {
        if let Some(colon_idx) = line.find(':') {
            let key = line[..colon_idx].trim().to_string();
            let value = line[colon_idx + 1..].trim().to_string();
            if !key.is_empty() {
                map.insert(key, value);
            }
        }
    }

    Some(map)
}

/// Validate a single hook command using the typed model.
///
/// Returns a list of error messages. Empty means the entry is valid.
pub fn check_hook_command(hook: &super::hook_types::HookCommand, label: &str) -> Vec<String> {
    let mut errors = Vec::new();

    match &hook.hook_type {
        Some(t) if !t.is_empty() => {}
        _ => {
            errors.push(format!("{label} missing or invalid 'type' field"));
        }
    }

    // async is always a bool if present (enforced by serde), no validation needed

    if let Some(t) = &hook.timeout {
        match t.as_f64() {
            Some(v) if v >= 0.0 => {}
            _ => {
                errors.push(format!("{label} 'timeout' must be a non-negative number"));
            }
        }
    }

    match &hook.command {
        Some(cmd) if cmd.all_entries_valid() => {}
        Some(super::hook_types::HookCommandValue::Array(_)) => {
            errors.push(format!("{label} invalid 'command' array entries"));
        }
        _ => {
            errors.push(format!("{label} missing or invalid 'command' field"));
        }
    }

    errors
}

/// Validate a single hook entry object (untyped, for backward compatibility).
///
/// Returns a list of error messages. Empty means the entry is valid.
pub fn check_hook_entry(hook: &serde_json::Value, label: &str) -> Vec<String> {
    // Try to deserialize into the typed model
    match serde_json::from_value::<super::hook_types::HookCommand>(hook.clone()) {
        Ok(cmd) => check_hook_command(&cmd, label),
        Err(_) => {
            // Fall back to manual validation for malformed JSON
            let mut errors = Vec::new();

            match hook.get("type").and_then(|v| v.as_str()) {
                Some(t) if !t.is_empty() => {}
                _ => {
                    errors.push(format!("{label} missing or invalid 'type' field"));
                }
            }

            if let Some(a) = hook.get("async")
                && !a.is_boolean()
            {
                errors.push(format!("{label} 'async' must be a boolean"));
            }

            if let Some(t) = hook.get("timeout") {
                match t.as_f64() {
                    Some(v) if v >= 0.0 => {}
                    _ => {
                        errors.push(format!("{label} 'timeout' must be a non-negative number"));
                    }
                }
            }

            match hook.get("command") {
                Some(serde_json::Value::String(s)) if !s.trim().is_empty() => {}
                Some(serde_json::Value::Array(arr)) if !arr.is_empty() => {
                    if !arr
                        .iter()
                        .all(|v| matches!(v, serde_json::Value::String(s) if !s.is_empty()))
                    {
                        errors.push(format!("{label} invalid 'command' array entries"));
                    }
                }
                _ => {
                    errors.push(format!("{label} missing or invalid 'command' field"));
                }
            }

            errors
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::hook_types::{HookCommand, HookCommandValue};

    // --- VALID_EFFORT_LEVELS ---

    #[test]
    fn valid_effort_levels_defined() {
        assert_eq!(VALID_EFFORT_LEVELS, &["low", "medium", "high", "max"]);
    }

    #[test]
    fn effort_tokens_has_entry_for_each_level() {
        for level in VALID_EFFORT_LEVELS {
            assert!(
                EFFORT_TOKENS.iter().any(|(l, _)| l == level),
                "Missing EFFORT_TOKENS entry for '{level}'"
            );
        }
    }

    // --- check_hook_command (typed) ---

    #[test]
    fn typed_valid_hook_command() {
        let cmd = HookCommand {
            hook_type: Some("command".to_string()),
            command: Some(HookCommandValue::Single("echo hello".to_string())),
            r#async: None,
            timeout: None,
        };
        assert!(check_hook_command(&cmd, "test").is_empty());
    }

    #[test]
    fn typed_missing_type() {
        let cmd = HookCommand {
            hook_type: None,
            command: Some(HookCommandValue::Single("echo hello".to_string())),
            r#async: None,
            timeout: None,
        };
        assert!(!check_hook_command(&cmd, "test").is_empty());
    }

    #[test]
    fn typed_missing_command() {
        let cmd = HookCommand {
            hook_type: Some("command".to_string()),
            command: None,
            r#async: None,
            timeout: None,
        };
        assert!(!check_hook_command(&cmd, "test").is_empty());
    }

    #[test]
    fn typed_valid_array_command() {
        let cmd = HookCommand {
            hook_type: Some("command".to_string()),
            command: Some(HookCommandValue::Array(vec![
                "echo".to_string(),
                "hello".to_string(),
            ])),
            r#async: None,
            timeout: None,
        };
        assert!(check_hook_command(&cmd, "test").is_empty());
    }

    // --- extract_frontmatter ---

    #[test]
    fn extracts_frontmatter_fields() {
        let content = "---\nname: test-agent\nmodel: sonnet\ntools: Read, Write\n---\n# Body";
        let fm = extract_frontmatter(content).unwrap();
        assert_eq!(fm.get("name").unwrap(), "test-agent");
        assert_eq!(fm.get("model").unwrap(), "sonnet");
        assert_eq!(fm.get("tools").unwrap(), "Read, Write");
    }

    #[test]
    fn missing_frontmatter_returns_none() {
        assert!(extract_frontmatter("# No frontmatter here").is_none());
    }

    #[test]
    fn bom_stripped() {
        let content = "\u{FEFF}---\nmodel: haiku\n---\n# Body";
        let fm = extract_frontmatter(content).unwrap();
        assert_eq!(fm.get("model").unwrap(), "haiku");
    }

    #[test]
    fn empty_value_preserved() {
        let content = "---\nmodel: \ntools: Read\n---\n";
        let fm = extract_frontmatter(content).unwrap();
        assert_eq!(fm.get("model").unwrap(), "");
    }

    // --- VALID_HOOK_EVENTS ---

    #[test]
    fn all_21_hook_events_accepted() {
        let expected = [
            "PreToolUse",
            "PostToolUse",
            "PostToolUseFailure",
            "PreCompact",
            "PostCompact",
            "SessionStart",
            "SessionEnd",
            "Stop",
            "Notification",
            "SubagentStart",
            "SubagentStop",
            "UserPromptSubmit",
            "InstructionsLoaded",
            "PermissionRequest",
            "TeammateIdle",
            "TaskCompleted",
            "ConfigChange",
            "WorktreeCreate",
            "WorktreeRemove",
            "Elicitation",
            "ElicitationResult",
        ];
        assert_eq!(VALID_HOOK_EVENTS.len(), 21);
        for event in &expected {
            assert!(VALID_HOOK_EVENTS.contains(event), "Missing event: {event}");
        }
    }

    #[test]
    fn invalid_hook_event_not_accepted() {
        assert!(!VALID_HOOK_EVENTS.contains(&"BogusEvent"));
        assert!(!VALID_HOOK_EVENTS.contains(&""));
        assert!(!VALID_HOOK_EVENTS.contains(&"preToolUse"));
    }

    // --- check_hook_entry ---

    #[test]
    fn valid_hook_entry() {
        let hook = serde_json::json!({
            "type": "command",
            "command": "echo hello"
        });
        assert!(check_hook_entry(&hook, "test").is_empty());
    }

    #[test]
    fn hook_missing_type() {
        let hook = serde_json::json!({
            "command": "echo hello"
        });
        assert!(!check_hook_entry(&hook, "test").is_empty());
    }

    #[test]
    fn hook_missing_command() {
        let hook = serde_json::json!({
            "type": "command"
        });
        assert!(!check_hook_entry(&hook, "test").is_empty());
    }

    #[test]
    fn hook_invalid_async() {
        let hook = serde_json::json!({
            "type": "command",
            "command": "echo hello",
            "async": "yes"
        });
        assert!(!check_hook_entry(&hook, "test").is_empty());
    }

    #[test]
    fn hook_valid_array_command() {
        let hook = serde_json::json!({
            "type": "command",
            "command": ["echo", "hello"]
        });
        assert!(check_hook_entry(&hook, "test").is_empty());
    }

    #[test]
    fn hook_invalid_timeout() {
        let hook = serde_json::json!({
            "type": "command",
            "command": "echo hello",
            "timeout": -5
        });
        assert!(!check_hook_entry(&hook, "test").is_empty());
    }

    // --- Property-based tests ---

    // --- is_kebab_case (PC-001) ---

    #[test]
    fn is_kebab_case_accepts_simple() {
        assert!(is_kebab_case("my-agent"));
        assert!(is_kebab_case("a"));
        assert!(is_kebab_case("a1"));
        assert!(is_kebab_case("abc-def-ghi"));
    }

    #[test]
    fn is_kebab_case_rejects_invalid() {
        assert!(!is_kebab_case("MyAgent"));
        assert!(!is_kebab_case("my_agent"));
        assert!(!is_kebab_case("-bad"));
        assert!(!is_kebab_case("bad-"));
        assert!(!is_kebab_case("BAD"));
        assert!(!is_kebab_case(""));
    }

    // --- parse_tool_list (PC-002) ---

    #[test]
    fn parse_tool_list_bracket_quoted() {
        let result = parse_tool_list(r#"["Read", "Write"]"#);
        assert_eq!(result, vec!["Read".to_string(), "Write".to_string()]);
    }

    #[test]
    fn parse_tool_list_bare_string() {
        let result = parse_tool_list("Read");
        assert_eq!(result, vec!["Read".to_string()]);
    }

    #[test]
    fn parse_tool_list_empty_brackets() {
        let result = parse_tool_list("[]");
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn parse_tool_list_whitespace() {
        let result = parse_tool_list("[ Read , Write ]");
        assert_eq!(result, vec!["Read".to_string(), "Write".to_string()]);
    }

    // --- check_naming_consistency (PC-003) ---

    #[test]
    fn check_naming_returns_error_when_stem_differs_from_name() {
        let findings = check_naming_consistency("my-agent", Some("other-agent"), "agent");
        assert!(findings.iter().any(|f| f.severity == LintSeverity::Error));
    }

    #[test]
    fn check_naming_returns_warn_when_name_is_none() {
        let findings = check_naming_consistency("my-agent", None, "agent");
        assert!(findings.iter().any(|f| f.severity == LintSeverity::Warn));
        assert!(!findings.iter().any(|f| f.severity == LintSeverity::Error));
    }

    #[test]
    fn check_naming_returns_empty_when_stem_equals_name() {
        let findings = check_naming_consistency("my-agent", Some("my-agent"), "agent");
        assert!(findings.is_empty());
    }

    #[test]
    fn check_naming_returns_error_for_non_kebab_stem() {
        let findings = check_naming_consistency("MyAgent", Some("MyAgent"), "agent");
        assert!(findings.iter().any(|f| f.severity == LintSeverity::Error));
    }

    // --- check_tool_values (PC-003) ---

    const TOOL_FIXTURES: &[&str] = &["Read", "Write", "Edit", "Bash", "Glob", "Grep"];

    #[test]
    fn check_tool_values_returns_error_for_unknown_tool() {
        let findings =
            check_tool_values("my-agent", r#"["Read", "UnknownTool"]"#, "tools", TOOL_FIXTURES);
        assert!(findings.iter().any(|f| f.severity == LintSeverity::Error));
    }

    #[test]
    fn check_tool_values_returns_empty_for_all_valid_tools() {
        let findings =
            check_tool_values("my-agent", r#"["Read", "Write"]"#, "tools", TOOL_FIXTURES);
        assert!(findings.is_empty());
    }

    #[test]
    fn check_tool_values_returns_empty_for_empty_list() {
        let findings = check_tool_values("my-agent", "[]", "tools", TOOL_FIXTURES);
        assert!(findings.is_empty());
    }

    // --- Pattern domain constants (PC-001..PC-004) ---

    #[test]
    fn valid_pattern_languages() {
        // AC-001.5: 10 language identifiers + "all" = 11 entries
        assert_eq!(VALID_PATTERN_LANGUAGES.len(), 11);
        for lang in &[
            "rust",
            "go",
            "python",
            "typescript",
            "java",
            "kotlin",
            "csharp",
            "cpp",
            "swift",
            "shell",
            "all",
        ] {
            assert!(
                VALID_PATTERN_LANGUAGES.contains(lang),
                "Missing language: {lang}"
            );
        }
    }

    #[test]
    fn valid_pattern_difficulties() {
        // AC-001.6: exactly 3 difficulty values
        assert_eq!(VALID_PATTERN_DIFFICULTIES.len(), 3);
        for diff in &["beginner", "intermediate", "advanced"] {
            assert!(
                VALID_PATTERN_DIFFICULTIES.contains(diff),
                "Missing difficulty: {diff}"
            );
        }
    }

    #[test]
    fn unsafe_code_patterns_non_empty() {
        // AC-002.8: deny-list constant is non-empty
        assert!(!UNSAFE_CODE_PATTERNS.is_empty());
    }

    #[test]
    fn required_pattern_sections() {
        // AC-002.5: exactly 9 required sections
        assert_eq!(REQUIRED_PATTERN_SECTIONS.len(), 9);
        for section in &[
            "Intent",
            "Problem",
            "Solution",
            "Language Implementations",
            "When to Use",
            "When NOT to Use",
            "Anti-Patterns",
            "Related Patterns",
            "References",
        ] {
            assert!(
                REQUIRED_PATTERN_SECTIONS.contains(section),
                "Missing section: {section}"
            );
        }
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn frontmatter_roundtrip_preserves_keys(
                key in "[a-z]{1,10}",
                value in "[a-zA-Z0-9 _-]{0,20}"
            ) {
                let content = format!("---\n{key}: {value}\n---\n# Body");
                let fm = extract_frontmatter(&content).unwrap();
                prop_assert_eq!(fm.get(&key).unwrap().trim(), value.trim());
            }

            #[test]
            fn frontmatter_never_panics(content in "\\PC{0,200}") {
                // Should not panic on any input
                let _ = extract_frontmatter(&content);
            }
        }
    }
}
