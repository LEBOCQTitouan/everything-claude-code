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
    settings: &serde_json::Value,
    script_path: &str,
) -> (serde_json::Value, StatusLineResult) {
    let mut new_settings = settings.clone();

    match settings.get("statusLine") {
        None => {
            // No statusLine key — install ours
            new_settings["statusLine"] = serde_json::json!({
                "command": script_path
            });
            (new_settings, StatusLineResult::Installed)
        }
        Some(status_line) => {
            match status_line.get("command").and_then(|c| c.as_str()) {
                Some(cmd) if cmd.contains(STATUSLINE_SCRIPT_FILENAME) => {
                    // ECC-managed — update path if different
                    new_settings["statusLine"]["command"] =
                        serde_json::Value::String(script_path.to_string());
                    (new_settings, StatusLineResult::Updated)
                }
                _ => {
                    // Custom or no command field — leave untouched
                    (new_settings, StatusLineResult::AlreadyCustom)
                }
            }
        }
    }
}

/// Fields displayable in the statusline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatuslineField {
    /// The current model name.
    Model,
    /// A visual progress bar of context window usage.
    ContextBar,
    /// The cost of tokens used in the session.
    Cost,
    /// How long the session has run.
    Duration,
    /// Number of lines changed in the session.
    LinesChanged,
    /// The current git branch.
    GitBranch,
    /// The 5-hour rate limit status.
    RateLimitFiveHour,
    /// The 7-day rate limit status.
    RateLimitSevenDay,
    /// Detailed token usage counts.
    TokenCounts,
    /// The ECC version string.
    EccVersion,
    /// The current worktree name.
    Worktree,
    /// The vim mode indicator.
    VimMode,
}

/// Color thresholds for the context window progress bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextThresholds {
    /// Percentage at which the bar turns yellow.
    pub yellow_pct: u32,
    /// Percentage at which the bar turns red.
    pub red_pct: u32,
}

/// Configuration for the statusline display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatuslineConfig {
    /// TTL in seconds for cached statusline data.
    pub cache_ttl_secs: u32,
    /// Color thresholds for the context progress bar.
    pub context_thresholds: ContextThresholds,
    /// Ordered list of fields to display in the statusline.
    pub field_order: Vec<StatuslineField>,
}

impl Default for StatuslineConfig {
    fn default() -> Self {
        Self {
            cache_ttl_secs: 5,
            context_thresholds: ContextThresholds {
                yellow_pct: 60,
                red_pct: 80,
            },
            field_order: vec![
                StatuslineField::Model,
                StatuslineField::ContextBar,
                StatuslineField::RateLimitFiveHour,
                StatuslineField::RateLimitSevenDay,
                StatuslineField::GitBranch,
                StatuslineField::Worktree,
                StatuslineField::TokenCounts,
                StatuslineField::LinesChanged,
                StatuslineField::Duration,
                StatuslineField::Cost,
                StatuslineField::EccVersion,
            ],
        }
    }
}

/// Replace the version placeholder in a script template.
///
/// Returns a new string with all occurrences of `__ECC_VERSION__` replaced by `version`.
pub fn prepare_script(template: &str, version: &str) -> String {
    template.replace(STATUSLINE_VERSION_PLACEHOLDER, version)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- ensure_statusline ---

    #[test]
    fn ensure_adds_to_empty_settings() {
        let settings = serde_json::json!({});
        let (result, status) =
            ensure_statusline(&settings, "/home/user/.claude/statusline-command.sh");
        assert_eq!(status, StatusLineResult::Installed);
        let cmd = result["statusLine"]["command"].as_str().unwrap();
        assert_eq!(cmd, "/home/user/.claude/statusline-command.sh");
    }

    #[test]
    fn ensure_adds_when_no_statusline_key() {
        let settings = serde_json::json!({"permissions": {"deny": []}});
        let (result, status) =
            ensure_statusline(&settings, "/home/user/.claude/statusline-command.sh");
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
        let (result, status) =
            ensure_statusline(&settings, "/home/user/.claude/statusline-command.sh");
        assert_eq!(status, StatusLineResult::AlreadyCustom);
        // Original command preserved
        assert_eq!(
            result["statusLine"]["command"].as_str().unwrap(),
            "my-custom-statusline.sh"
        );
    }

    #[test]
    fn ensure_updates_ecc_managed() {
        let settings = serde_json::json!({
            "statusLine": {"command": "/old/path/.claude/statusline-command.sh"}
        });
        let (result, status) =
            ensure_statusline(&settings, "/new/path/.claude/statusline-command.sh");
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

    // --- StatuslineConfig, ContextThresholds, StatuslineField ---

    #[test]
    fn statusline_config_default_construction() {
        let config = StatuslineConfig::default();
        // Must not panic and must return a valid config
        assert!(config.cache_ttl_secs > 0);
        assert!(!config.field_order.is_empty());
    }

    #[test]
    fn statusline_field_variants() {
        // Verify all 12 variants exist by constructing each one
        let variants = [
            StatuslineField::Model,
            StatuslineField::ContextBar,
            StatuslineField::Cost,
            StatuslineField::Duration,
            StatuslineField::LinesChanged,
            StatuslineField::GitBranch,
            StatuslineField::RateLimitFiveHour,
            StatuslineField::RateLimitSevenDay,
            StatuslineField::TokenCounts,
            StatuslineField::EccVersion,
            StatuslineField::Worktree,
            StatuslineField::VimMode,
        ];
        assert_eq!(variants.len(), 12);
    }

    #[test]
    fn statusline_config_default_values() {
        let config = StatuslineConfig::default();
        assert_eq!(config.cache_ttl_secs, 5);
        assert_eq!(config.context_thresholds.yellow_pct, 60);
        assert_eq!(config.context_thresholds.red_pct, 80);
    }

    #[test]
    fn statusline_config_derives() {
        let a = StatuslineConfig::default();
        let b = a.clone();
        assert_eq!(a, b);
        // Debug: must not panic
        let _ = format!("{:?}", a);
    }

    #[test]
    fn field_order_contains_worktree_after_git_branch() {
        let config = StatuslineConfig::default();
        let order = &config.field_order;
        let git_branch_idx = order
            .iter()
            .position(|f| *f == StatuslineField::GitBranch)
            .expect("GitBranch must be in field_order");
        let worktree_idx = order
            .iter()
            .position(|f| *f == StatuslineField::Worktree)
            .expect("Worktree must be in field_order");
        assert_eq!(
            worktree_idx,
            git_branch_idx + 1,
            "Worktree must appear immediately after GitBranch in field_order"
        );
    }

    #[test]
    fn field_order_matches_script_rendering_priority() {
        let config = StatuslineConfig::default();
        let expected = vec![
            StatuslineField::Model,
            StatuslineField::ContextBar,
            StatuslineField::RateLimitFiveHour,
            StatuslineField::RateLimitSevenDay,
            StatuslineField::GitBranch,
            StatuslineField::Worktree,
            StatuslineField::TokenCounts,
            StatuslineField::LinesChanged,
            StatuslineField::Duration,
            StatuslineField::Cost,
            StatuslineField::EccVersion,
        ];
        assert_eq!(
            config.field_order, expected,
            "field_order must match script rendering priority (11 entries, VimMode excluded)"
        );
    }
}
