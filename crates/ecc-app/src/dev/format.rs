use super::status::{DevProfileStatus, DevStatus};
use crate::config::clean::is_current_ecc_hook;
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// Count ECC hooks present in settings.json.
pub fn count_ecc_hooks_in_settings(fs: &dyn FileSystem, claude_dir: &Path) -> usize {
    let settings_path = claude_dir.join("settings.json");
    let content = match fs.read_to_string(&settings_path) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    let settings: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return 0,
    };
    let Some(hooks_obj) = settings.get("hooks").and_then(|h| h.as_object()) else {
        return 0;
    };

    hooks_obj
        .values()
        .filter_map(|v| v.as_array())
        .flat_map(|arr| arr.iter())
        .filter(|entry| is_current_ecc_hook(entry))
        .count()
}

/// Format a `DevStatus` for display.
pub fn format_status(status: &DevStatus, colored: bool) -> String {
    use ecc_domain::ansi;

    if !status.active {
        return format!(
            "{}\n\nECC is not installed. Run {} to activate.\n",
            ansi::bold("ECC Status: inactive", colored),
            ansi::cyan("ecc dev on", colored),
        );
    }

    let version = status.version.as_deref().unwrap_or("unknown");
    let installed_at = status.installed_at.as_deref().unwrap_or("unknown");

    let profile_label = match status.profile {
        DevProfileStatus::Dev => "Dev (symlinked)",
        DevProfileStatus::Default => "Default (copied)",
        DevProfileStatus::Inactive => "Inactive",
        DevProfileStatus::Mixed => "Mixed",
    };

    format!(
        "{}\n\n\
         Version:    {version}\n\
         Installed:  {installed_at}\n\
         Profile:    {profile_label}\n\
         Agents:     {}\n\
         Commands:   {}\n\
         Skills:     {}\n\
         Rules:      {}\n\
         Hooks:      {}\n",
        ansi::bold("ECC Status: active", colored),
        status.agents,
        status.commands,
        status.skills,
        status.rules,
        status.hooks,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;

    #[test]
    fn format_status_inactive() {
        let status = DevStatus {
            active: false,
            version: None,
            agents: 0,
            commands: 0,
            skills: 0,
            rules: 0,
            hooks: 0,
            installed_at: None,
            profile: DevProfileStatus::Inactive,
        };
        let output = format_status(&status, false);
        assert!(output.contains("inactive"));
        assert!(output.contains("ecc dev on"));
    }

    #[test]
    fn format_status_active() {
        let status = DevStatus {
            active: true,
            version: Some("4.0.0".to_string()),
            agents: 10,
            commands: 5,
            skills: 3,
            rules: 8,
            hooks: 12,
            installed_at: Some("2026-03-14T00:00:00Z".to_string()),
            profile: DevProfileStatus::Default,
        };
        let output = format_status(&status, false);
        assert!(output.contains("active"));
        assert!(output.contains("4.0.0"));
        assert!(output.contains("10"));
        assert!(output.contains("12"));
    }

    #[test]
    fn format_status_includes_profile() {
        let status = DevStatus {
            active: true,
            version: Some("4.0.0".to_string()),
            agents: 1,
            commands: 1,
            skills: 1,
            rules: 2,
            hooks: 1,
            installed_at: Some("2026-03-23T00:00:00Z".to_string()),
            profile: DevProfileStatus::Dev,
        };

        let output = format_status(&status, false);

        assert!(output.contains("Profile:"), "output must contain 'Profile:' line");
        assert!(output.contains("Dev"), "output must display the Dev profile name");
    }

    #[test]
    fn count_ecc_hooks_no_settings() {
        let fs = InMemoryFileSystem::new();
        assert_eq!(count_ecc_hooks_in_settings(&fs, Path::new("/claude")), 0);
    }

    #[test]
    fn count_ecc_hooks_counts_only_ecc() {
        let settings = r#"{
            "hooks": {
                "PreToolUse": [
                    {"description": "ECC", "hooks": [{"command": "ecc-hook format"}]},
                    {"description": "user", "hooks": [{"command": "my-hook"}]}
                ],
                "PostToolUse": [
                    {"description": "ECC2", "hooks": [{"command": "ecc-shell-hook check"}]}
                ]
            }
        }"#;
        let fs = InMemoryFileSystem::new().with_file("/claude/settings.json", settings);
        assert_eq!(count_ecc_hooks_in_settings(&fs, Path::new("/claude")), 2);
    }
}
