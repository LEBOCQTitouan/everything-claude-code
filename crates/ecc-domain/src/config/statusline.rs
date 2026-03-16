//! Statusline configuration — pure types and logic for managing the ECC statusline script.
//!
//! Follows the deny_rules pattern: domain holds pure functions,
//! app layer handles I/O (reading/writing files).

/// Filename of the bundled statusline script.
pub const STATUSLINE_SCRIPT_FILENAME: &str = "statusline-command.sh";

/// Placeholder in the script template that gets replaced with the actual ECC version.
pub const STATUSLINE_VERSION_PLACEHOLDER: &str = "__ECC_VERSION__";

/// Result of ensuring statusline configuration in settings.json.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusLineResult {
    /// Statusline was freshly installed (no prior `statusLine` key).
    Installed,
    /// Statusline was updated (was ECC-managed, path changed).
    Updated,
    /// User has a custom statusline — left untouched.
    AlreadyCustom,
}

/// Ensure statusline configuration is present in settings.json.
///
/// Pure function — takes settings value and script path, returns new settings + result.
/// Never mutates the input.
///
/// Logic:
/// - No `statusLine` key → add it, return `Installed`
/// - `statusLine.command` contains `statusline-command.sh` → update path if different, return `Updated`
/// - `statusLine` exists but not our script → return `AlreadyCustom`
pub fn ensure_statusline(
    _settings: &serde_json::Value,
    _script_path: &str,
) -> (serde_json::Value, StatusLineResult) {
    todo!()
}

/// Replace the version placeholder in a script template.
///
/// Returns a new string with all occurrences of `__ECC_VERSION__` replaced by `version`.
pub fn prepare_script(_template: &str, _version: &str) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- ensure_statusline ---

    #[test]
    fn ensure_adds_to_empty_settings() {
        let settings = serde_json::json!({});
        let (result, status) = ensure_statusline(&settings, "/home/user/.claude/statusline-command.sh");
        assert_eq!(status, StatusLineResult::Installed);
        let cmd = result["statusLine"]["command"].as_str().unwrap();
        assert_eq!(cmd, "/home/user/.claude/statusline-command.sh");
    }

    #[test]
    fn ensure_adds_when_no_statusline_key() {
        let settings = serde_json::json!({"permissions": {"deny": []}});
        let (result, status) = ensure_statusline(&settings, "/home/user/.claude/statusline-command.sh");
        assert_eq!(status, StatusLineResult::Installed);
        assert!(result["statusLine"]["command"].is_string());
        // Preserves existing keys
        assert!(result["permissions"]["deny"].is_array());
    }

    #[test]
    fn ensure_does_not_overwrite_custom() {
        let settings = serde_json::json!({
            "statusLine": {"command": "my-custom-statusline.sh"}
        });
        let (result, status) = ensure_statusline(&settings, "/home/user/.claude/statusline-command.sh");
        assert_eq!(status, StatusLineResult::AlreadyCustom);
        // Original command preserved
        assert_eq!(result["statusLine"]["command"].as_str().unwrap(), "my-custom-statusline.sh");
    }

    #[test]
    fn ensure_updates_ecc_managed() {
        let settings = serde_json::json!({
            "statusLine": {"command": "/old/path/.claude/statusline-command.sh"}
        });
        let (result, status) = ensure_statusline(&settings, "/new/path/.claude/statusline-command.sh");
        assert_eq!(status, StatusLineResult::Updated);
        assert_eq!(
            result["statusLine"]["command"].as_str().unwrap(),
            "/new/path/.claude/statusline-command.sh"
        );
    }

    #[test]
    fn ensure_returns_new_value_preserves_other_keys() {
        let settings = serde_json::json!({
            "hooks": {"PreToolUse": []},
            "permissions": {"deny": ["rule1"]}
        });
        let (result, _) = ensure_statusline(&settings, "/path/statusline-command.sh");
        assert!(result["hooks"]["PreToolUse"].is_array());
        assert_eq!(result["permissions"]["deny"][0].as_str().unwrap(), "rule1");
    }

    #[test]
    fn ensure_handles_statusline_without_command_field() {
        // statusLine key exists but no command field — treat as custom
        let settings = serde_json::json!({
            "statusLine": {"enabled": true}
        });
        let (result, status) = ensure_statusline(&settings, "/path/statusline-command.sh");
        assert_eq!(status, StatusLineResult::AlreadyCustom);
        // Original structure preserved
        assert_eq!(result["statusLine"]["enabled"], true);
    }

    // --- prepare_script ---

    #[test]
    fn prepare_script_replaces_placeholder() {
        let template = r#"ECC_VERSION="__ECC_VERSION__""#;
        let result = prepare_script(template, "4.2.0");
        assert_eq!(result, r#"ECC_VERSION="4.2.0""#);
    }

    #[test]
    fn prepare_script_no_placeholder_unchanged() {
        let template = "#!/bin/bash\necho hello";
        let result = prepare_script(template, "1.0.0");
        assert_eq!(result, template);
    }

    #[test]
    fn prepare_script_multiple_occurrences() {
        let template = "__ECC_VERSION__ and __ECC_VERSION__";
        let result = prepare_script(template, "5.0.0");
        assert_eq!(result, "5.0.0 and 5.0.0");
    }
}
