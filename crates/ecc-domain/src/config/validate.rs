//! Pure validation helpers for content files.

use std::collections::HashMap;

/// Valid model identifiers for agent frontmatter.
pub const VALID_MODELS: &[&str] = &["haiku", "sonnet", "opus"];

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
