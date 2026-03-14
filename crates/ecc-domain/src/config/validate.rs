//! Pure validation helpers for content files.

use std::collections::HashMap;

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

/// Validate a single hook entry object.
///
/// Returns `true` if there are errors, `false` if the entry is valid.
/// Errors are printed to stderr.
pub fn validate_hook_entry(hook: &serde_json::Value, label: &str) -> bool {
    let mut has_errors = false;

    match hook.get("type").and_then(|v| v.as_str()) {
        Some(t) if !t.is_empty() => {}
        _ => {
            eprintln!("ERROR: {} missing or invalid 'type' field", label);
            has_errors = true;
        }
    }

    if let Some(a) = hook.get("async")
        && !a.is_boolean()
    {
        eprintln!("ERROR: {} 'async' must be a boolean", label);
        has_errors = true;
    }

    if let Some(t) = hook.get("timeout") {
        match t.as_f64() {
            Some(v) if v >= 0.0 => {}
            _ => {
                eprintln!("ERROR: {} 'timeout' must be a non-negative number", label);
                has_errors = true;
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
                eprintln!("ERROR: {} invalid 'command' array entries", label);
                has_errors = true;
            }
        }
        _ => {
            eprintln!("ERROR: {} missing or invalid 'command' field", label);
            has_errors = true;
        }
    }

    has_errors
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // --- validate_hook_entry ---

    #[test]
    fn valid_hook_entry() {
        let hook = serde_json::json!({
            "type": "command",
            "command": "echo hello"
        });
        assert!(!validate_hook_entry(&hook, "test"));
    }

    #[test]
    fn hook_missing_type() {
        let hook = serde_json::json!({
            "command": "echo hello"
        });
        assert!(validate_hook_entry(&hook, "test"));
    }

    #[test]
    fn hook_missing_command() {
        let hook = serde_json::json!({
            "type": "command"
        });
        assert!(validate_hook_entry(&hook, "test"));
    }

    #[test]
    fn hook_invalid_async() {
        let hook = serde_json::json!({
            "type": "command",
            "command": "echo hello",
            "async": "yes"
        });
        assert!(validate_hook_entry(&hook, "test"));
    }

    #[test]
    fn hook_valid_array_command() {
        let hook = serde_json::json!({
            "type": "command",
            "command": ["echo", "hello"]
        });
        assert!(!validate_hook_entry(&hook, "test"));
    }

    #[test]
    fn hook_invalid_timeout() {
        let hook = serde_json::json!({
            "type": "command",
            "command": "echo hello",
            "timeout": -5
        });
        assert!(validate_hook_entry(&hook, "test"));
    }
}
