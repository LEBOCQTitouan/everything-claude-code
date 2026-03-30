//! Settings helpers — deny rules and statusline management.

use ecc_domain::config::deny_rules;
use ecc_domain::config::statusline::{self, StatusLineResult};
use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// Ensure deny rules are present in settings.json.
/// Returns `(added, existing)` if settings were updated, `None` on error.
pub(in crate::install) fn ensure_deny_rules_in_settings(
    fs: &dyn FileSystem,
    settings_path: &Path,
    dry_run: bool,
) -> Option<(usize, usize)> {
    let content = fs
        .read_to_string(settings_path)
        .unwrap_or_else(|_| "{}".to_string());
    let mut settings: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(
                "Malformed settings.json at {}: {}",
                settings_path.display(),
                e
            );
            return None;
        }
    };

    let existing_deny: Vec<String> = settings
        .get("permissions")
        .and_then(|p| p.get("deny"))
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let (merged, result) = deny_rules::ensure_deny_rules(&existing_deny);

    if result.added > 0 && !dry_run {
        let permissions = settings
            .as_object_mut()?
            .entry("permissions")
            .or_insert_with(|| serde_json::json!({}));
        permissions.as_object_mut()?.insert(
            "deny".to_string(),
            serde_json::Value::Array(merged.into_iter().map(serde_json::Value::String).collect()),
        );

        let json = match serde_json::to_string_pretty(&settings) {
            Ok(j) => j,
            Err(e) => {
                tracing::warn!("Failed to serialize settings: {}", e);
                return None;
            }
        };
        if let Err(e) = fs.write(settings_path, &format!("{json}\n")) {
            tracing::warn!("Failed to write settings.json: {}", e);
            return None;
        }
    }

    Some((result.added, result.existing))
}

/// Ensure the ECC statusline script is installed and settings.json references it.
///
/// Flow:
/// 1. Read bundled script from `ecc_root/statusline/statusline-command.sh`
/// 2. Embed version via `prepare_script()`
/// 3. Write prepared script to `claude_dir/statusline-command.sh`
/// 4. Read settings.json (or `"{}"` if missing)
/// 5. Call `ensure_statusline()` with absolute script path
/// 6. Write updated settings if needed
///
/// Returns `None` on error (missing source script, malformed settings).
pub(in crate::install) fn ensure_statusline_in_settings(
    fs: &dyn FileSystem,
    env: &dyn Environment,
    settings_path: &Path,
    ecc_root: &Path,
    version: &str,
    dry_run: bool,
) -> Option<StatusLineResult> {
    // Read bundled script template
    let source_script = ecc_root
        .join("statusline")
        .join(statusline::STATUSLINE_SCRIPT_FILENAME);
    let template = match fs.read_to_string(&source_script) {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!(
                "Cannot read statusline script at {}: {}",
                source_script.display(),
                e
            );
            return None;
        }
    };

    // Embed version
    let prepared = statusline::prepare_script(&template, version);

    // Compute target paths
    let home = env.home_dir()?;
    let target_script = home
        .join(".claude")
        .join(statusline::STATUSLINE_SCRIPT_FILENAME);
    let absolute_script_path = target_script.to_string_lossy().to_string();

    if !dry_run {
        // Write prepared script
        if let Err(e) = fs.write(&target_script, &prepared) {
            tracing::warn!("Failed to write statusline script: {}", e);
            return None;
        }
        // Set executable permissions
        if let Err(e) = fs.set_permissions(&target_script, 0o755) {
            tracing::warn!("Failed to set statusline script permissions: {}", e);
        }
    }

    // Read settings
    let content = fs
        .read_to_string(settings_path)
        .unwrap_or_else(|_| "{}".to_string());
    let settings: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(
                "Malformed settings.json at {}: {}",
                settings_path.display(),
                e
            );
            return None;
        }
    };

    // Apply domain logic
    let (new_settings, result) = statusline::ensure_statusline(&settings, &absolute_script_path);

    // Write back if changed
    if matches!(
        result,
        StatusLineResult::Installed | StatusLineResult::Updated
    ) && !dry_run
    {
        let json = match serde_json::to_string_pretty(&new_settings) {
            Ok(j) => j,
            Err(e) => {
                tracing::warn!("Failed to serialize settings: {}", e);
                return None;
            }
        };
        if let Err(e) = fs.write(settings_path, &format!("{json}\n")) {
            tracing::warn!("Failed to write settings.json: {}", e);
            return None;
        }
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{InMemoryFileSystem, MockEnvironment};
    use std::path::Path;

    fn statusline_source_fs() -> InMemoryFileSystem {
        InMemoryFileSystem::new()
            .with_dir("/ecc/statusline")
            .with_file(
                "/ecc/statusline/statusline-command.sh",
                POWERUSER_STATUSLINE_TEMPLATE,
            )
            .with_dir("/home/user/.claude")
    }

    /// Realistic power-user statusline template with cost/token/jq/rate_limits markers.
    const POWERUSER_STATUSLINE_TEMPLATE: &str = concat!(
        "#!/usr/bin/env bash\n",
        "set -uo pipefail\n",
        "ECC_VERSION=\"__ECC_VERSION__\"\n",
        "command -v jq >/dev/null 2>&1 || { echo \"ECC\"; exit 0; }\n",
        "INPUT=$(cat)\n",
        "eval \"$(echo \"$INPUT\" | jq -r '\n",
        "  @sh \"COST_USD=\\(.cost.total_cost_usd // 0)\",\n",
        "  @sh \"DISPLAY_NAME=\\(.model.display_name // \\\"\\\")\",\n",
        "  @sh \"USED_PCT=\\(.context_window.used_percentage // 0)\",\n",
        "  @sh \"RL_5H=\\(.rate_limits.five_hour.used_percentage // \\\"\\\")\"\n",
        "')\"\n",
        "echo \"$DISPLAY_NAME $COST_USD $ECC_VERSION\"\n",
    );

    fn statusline_env() -> MockEnvironment {
        MockEnvironment::new()
            .with_var("NO_COLOR", "1")
            .with_home("/home/user")
    }

    #[test]
    fn statusline_installs_script_and_updates_settings() {
        let fs = statusline_source_fs();
        let env = statusline_env();

        let result = ensure_statusline_in_settings(
            &fs,
            &env,
            Path::new("/home/user/.claude/settings.json"),
            Path::new("/ecc"),
            "4.2.0",
            false,
        );

        assert_eq!(result, Some(StatusLineResult::Installed));

        let script = fs
            .read_to_string(Path::new("/home/user/.claude/statusline-command.sh"))
            .unwrap();
        assert!(script.contains("4.2.0"));
        assert!(!script.contains("__ECC_VERSION__"));

        let settings_str = fs
            .read_to_string(Path::new("/home/user/.claude/settings.json"))
            .unwrap();
        let settings: serde_json::Value = serde_json::from_str(&settings_str).unwrap();
        assert!(
            settings["statusLine"]["command"]
                .as_str()
                .unwrap()
                .contains("statusline-command.sh")
        );
    }

    #[test]
    fn statusline_dry_run_no_writes() {
        let fs = statusline_source_fs();
        let env = statusline_env();

        let result = ensure_statusline_in_settings(
            &fs,
            &env,
            Path::new("/home/user/.claude/settings.json"),
            Path::new("/ecc"),
            "4.2.0",
            true,
        );

        assert_eq!(result, Some(StatusLineResult::Installed));
        assert!(!fs.exists(Path::new("/home/user/.claude/statusline-command.sh")));
        assert!(!fs.exists(Path::new("/home/user/.claude/settings.json")));
    }

    #[test]
    fn statusline_does_not_overwrite_custom() {
        let fs = statusline_source_fs().with_file(
            "/home/user/.claude/settings.json",
            &serde_json::json!({"statusLine": {"command": "my-custom-script.sh"}}).to_string(),
        );
        let env = statusline_env();

        let result = ensure_statusline_in_settings(
            &fs,
            &env,
            Path::new("/home/user/.claude/settings.json"),
            Path::new("/ecc"),
            "4.2.0",
            false,
        );

        assert_eq!(result, Some(StatusLineResult::AlreadyCustom));

        let settings_str = fs
            .read_to_string(Path::new("/home/user/.claude/settings.json"))
            .unwrap();
        let settings: serde_json::Value = serde_json::from_str(&settings_str).unwrap();
        assert_eq!(
            settings["statusLine"]["command"].as_str().unwrap(),
            "my-custom-script.sh"
        );
    }

    #[test]
    fn statusline_updates_existing_ecc_statusline() {
        let fs = statusline_source_fs().with_file(
            "/home/user/.claude/settings.json",
            &serde_json::json!({"statusLine": {"command": "/old/path/statusline-command.sh"}})
                .to_string(),
        );
        let env = statusline_env();

        let result = ensure_statusline_in_settings(
            &fs,
            &env,
            Path::new("/home/user/.claude/settings.json"),
            Path::new("/ecc"),
            "4.2.0",
            false,
        );

        assert_eq!(result, Some(StatusLineResult::Updated));

        let settings_str = fs
            .read_to_string(Path::new("/home/user/.claude/settings.json"))
            .unwrap();
        let settings: serde_json::Value = serde_json::from_str(&settings_str).unwrap();
        assert_eq!(
            settings["statusLine"]["command"].as_str().unwrap(),
            "/home/user/.claude/statusline-command.sh"
        );
    }

    #[test]
    fn statusline_handles_missing_source_script() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc")
            .with_dir("/home/user/.claude");
        let env = statusline_env();

        let result = ensure_statusline_in_settings(
            &fs,
            &env,
            Path::new("/home/user/.claude/settings.json"),
            Path::new("/ecc"),
            "4.2.0",
            false,
        );

        assert_eq!(result, None);
    }

    #[test]
    fn statusline_handles_malformed_settings() {
        let fs =
            statusline_source_fs().with_file("/home/user/.claude/settings.json", "not json{{{");
        let env = statusline_env();

        let result = ensure_statusline_in_settings(
            &fs,
            &env,
            Path::new("/home/user/.claude/settings.json"),
            Path::new("/ecc"),
            "4.2.0",
            false,
        );

        assert_eq!(result, None);
    }

    /// PC-038 — deployed script must contain power-user markers from the real template.
    #[test]
    fn install_deploys_poweruser_statusline() {
        let fs = statusline_source_fs();
        let env = statusline_env();

        ensure_statusline_in_settings(
            &fs,
            &env,
            Path::new("/home/user/.claude/settings.json"),
            Path::new("/ecc"),
            "1.0.0",
            false,
        );

        let deployed = fs
            .read_to_string(Path::new("/home/user/.claude/statusline-command.sh"))
            .expect("deployed script must exist");

        assert!(
            deployed.contains("total_cost_usd"),
            "deployed script must contain power-user marker 'total_cost_usd'"
        );
        assert!(deployed.contains("jq"), "deployed script must depend on jq");
        assert!(
            deployed.starts_with("#!/usr/bin/env bash"),
            "deployed script must have a valid shebang"
        );
    }

    #[test]
    fn statusline_script_has_executable_permissions() {
        let fs = statusline_source_fs();
        let env = statusline_env();

        ensure_statusline_in_settings(
            &fs,
            &env,
            Path::new("/home/user/.claude/settings.json"),
            Path::new("/ecc"),
            "4.2.0",
            false,
        );

        let perms = fs.get_permissions(Path::new("/home/user/.claude/statusline-command.sh"));
        assert_eq!(perms, Some(0o755), "statusline script must be executable");
    }

    #[test]
    fn statusline_embeds_version_in_script() {
        let fs = statusline_source_fs();
        let env = statusline_env();

        ensure_statusline_in_settings(
            &fs,
            &env,
            Path::new("/home/user/.claude/settings.json"),
            Path::new("/ecc"),
            "99.0.0-beta",
            false,
        );

        let script = fs
            .read_to_string(Path::new("/home/user/.claude/statusline-command.sh"))
            .unwrap();
        assert!(script.contains("99.0.0-beta"));
    }
}
