//! Dev mode use cases — toggle ECC config on/off in `~/.claude/`.
//!
//! Composes existing [`install`] and [`config::clean`] building blocks
//! to provide a quick profile switch between "ECC-enhanced" and "vanilla Claude".

use crate::config::clean::{clean_from_manifest, is_current_ecc_hook};
use crate::config::manifest::read_manifest;
use crate::install::{InstallContext, InstallOptions, InstallSummary, install_global};
use ecc_domain::config::clean::format_clean_report;
use ecc_domain::config::manifest::EccManifest;
use ecc_domain::config::merge as domain_merge;
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Result of `dev off`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevOffResult {
    pub report_text: String,
    pub removed_count: usize,
    pub error_count: usize,
    pub success: bool,
}

/// Status snapshot of the current ECC installation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevStatus {
    pub active: bool,
    pub version: Option<String>,
    pub agents: usize,
    pub commands: usize,
    pub skills: usize,
    pub rules: usize,
    pub hooks: usize,
    pub installed_at: Option<String>,
}

// ---------------------------------------------------------------------------
// Use cases
// ---------------------------------------------------------------------------

/// Activate ECC config by running a clean + force reinstall.
pub fn dev_on(
    ctx: &InstallContext,
    ecc_root: &Path,
    claude_dir: &Path,
    version: &str,
    now: &str,
    dry_run: bool,
) -> InstallSummary {
    let options = InstallOptions {
        force: true,
        interactive: false,
        clean: true,
        dry_run,
        ..InstallOptions::default()
    };
    install_global(ctx, ecc_root, claude_dir, version, now, &options)
}

/// Deactivate ECC config by removing manifest-tracked artifacts.
pub fn dev_off(
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    claude_dir: &Path,
    dry_run: bool,
) -> DevOffResult {
    let manifest = read_manifest(fs, claude_dir);

    let Some(manifest) = manifest else {
        let msg = "No ECC manifest found — ECC does not appear to be active.\n";
        terminal.stdout_write(msg);
        return DevOffResult {
            report_text: msg.to_string(),
            removed_count: 0,
            error_count: 0,
            success: true,
        };
    };

    let report = clean_from_manifest(
        fs,
        claude_dir,
        &manifest,
        &|entry| domain_merge::is_legacy_ecc_hook(entry),
        dry_run,
    );

    let report_text = format_clean_report(&report, dry_run);
    let removed_count = report.removed.len();
    let error_count = report.errors.len();

    terminal.stdout_write(&format!("{report_text}\n"));

    DevOffResult {
        report_text,
        removed_count,
        error_count,
        success: report.errors.is_empty(),
    }
}

/// Check whether ECC is currently active by reading the manifest.
pub fn dev_status(fs: &dyn FileSystem, claude_dir: &Path) -> DevStatus {
    let manifest = read_manifest(fs, claude_dir);

    match manifest {
        Some(m) => manifest_to_status(&m),
        None => DevStatus {
            active: false,
            version: None,
            agents: 0,
            commands: 0,
            skills: 0,
            rules: 0,
            hooks: 0,
            installed_at: None,
        },
    }
}

fn manifest_to_status(m: &EccManifest) -> DevStatus {
    let rules: usize = m.artifacts.rules.values().map(|v| v.len()).sum();

    DevStatus {
        active: true,
        version: Some(m.version.clone()),
        agents: m.artifacts.agents.len(),
        commands: m.artifacts.commands.len(),
        skills: m.artifacts.skills.len(),
        rules,
        hooks: m.artifacts.hook_descriptions.len(),
        installed_at: Some(m.installed_at.clone()),
    }
}

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

    format!(
        "{}\n\n\
         Version:    {version}\n\
         Installed:  {installed_at}\n\
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
    use ecc_domain::config::manifest::Artifacts;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem};
    use std::collections::BTreeMap;

    fn sample_manifest() -> EccManifest {
        let mut rules = BTreeMap::new();
        rules.insert(
            "common".to_string(),
            vec!["style.md".to_string(), "security.md".to_string()],
        );

        EccManifest {
            version: "4.0.0".to_string(),
            installed_at: "2026-03-14T00:00:00Z".to_string(),
            updated_at: "2026-03-14T00:00:00Z".to_string(),
            languages: vec!["rust".to_string()],
            artifacts: Artifacts {
                agents: vec!["planner.md".to_string()],
                commands: vec!["spec.md".to_string()],
                skills: vec!["tdd".to_string()],
                rules,
                hook_descriptions: vec!["phase-gate".to_string()],
            },
        }
    }

    fn manifest_json(m: &EccManifest) -> String {
        serde_json::to_string_pretty(m).unwrap()
    }

    // --- dev_status ---

    #[test]
    fn dev_status_inactive_when_no_manifest() {
        let fs = InMemoryFileSystem::new();
        let status = dev_status(&fs, Path::new("/claude"));
        assert!(!status.active);
        assert!(status.version.is_none());
        assert_eq!(status.agents, 0);
    }

    #[test]
    fn dev_status_active_when_manifest_exists() {
        let m = sample_manifest();
        let fs =
            InMemoryFileSystem::new().with_file("/claude/.ecc-manifest.json", &manifest_json(&m));

        let status = dev_status(&fs, Path::new("/claude"));

        assert!(status.active);
        assert_eq!(status.version.as_deref(), Some("4.0.0"));
        assert_eq!(status.agents, 1);
        assert_eq!(status.commands, 1);
        assert_eq!(status.skills, 1);
        assert_eq!(status.rules, 2);
        assert_eq!(status.hooks, 1);
        assert_eq!(status.installed_at.as_deref(), Some("2026-03-14T00:00:00Z"));
    }

    // --- dev_off ---

    #[test]
    fn dev_off_no_manifest_reports_inactive() {
        let fs = InMemoryFileSystem::new();
        let terminal = BufferedTerminal::new();

        let result = dev_off(&fs, &terminal, Path::new("/claude"), false);

        assert!(result.success);
        assert_eq!(result.removed_count, 0);
        let output = terminal.stdout_output().join("");
        assert!(output.contains("not appear"));
    }

    #[test]
    fn dev_off_removes_manifest_tracked_files() {
        let m = sample_manifest();
        let fs = InMemoryFileSystem::new()
            .with_file("/claude/.ecc-manifest.json", &manifest_json(&m))
            .with_file("/claude/agents/planner.md", "# Planner")
            .with_dir("/claude/commands")
            .with_file("/claude/commands/plan.md", "# Plan")
            .with_dir("/claude/skills/tdd")
            .with_file("/claude/skills/tdd/SKILL.md", "# TDD")
            .with_dir("/claude/rules/common")
            .with_file("/claude/rules/common/style.md", "# Style")
            .with_file("/claude/rules/common/security.md", "# Security");
        let terminal = BufferedTerminal::new();

        let result = dev_off(&fs, &terminal, Path::new("/claude"), false);

        assert!(result.success);
        assert!(result.removed_count > 0);
        assert!(!fs.exists(Path::new("/claude/.ecc-manifest.json")));
        assert!(!fs.exists(Path::new("/claude/agents/planner.md")));
    }

    #[test]
    fn dev_off_preserves_user_hooks_in_settings() {
        let m = sample_manifest();
        let settings = r#"{
            "hooks": {
                "PreToolUse": [
                    {"description": "ECC hook", "hooks": [{"command": "ecc-hook format"}]},
                    {"description": "My hook", "hooks": [{"command": "my-custom-hook"}]}
                ]
            }
        }"#;
        let fs = InMemoryFileSystem::new()
            .with_file("/claude/.ecc-manifest.json", &manifest_json(&m))
            .with_file("/claude/settings.json", settings);
        let terminal = BufferedTerminal::new();

        let result = dev_off(&fs, &terminal, Path::new("/claude"), false);

        assert!(result.success);
        let updated = fs
            .read_to_string(Path::new("/claude/settings.json"))
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&updated).unwrap();
        let pre_hooks = parsed["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pre_hooks.len(), 1);
        assert_eq!(pre_hooks[0]["description"], "My hook");
    }

    #[test]
    fn dev_off_dry_run_does_not_remove() {
        let m = sample_manifest();
        let fs = InMemoryFileSystem::new()
            .with_file("/claude/.ecc-manifest.json", &manifest_json(&m))
            .with_file("/claude/agents/planner.md", "# Planner");
        let terminal = BufferedTerminal::new();

        let result = dev_off(&fs, &terminal, Path::new("/claude"), true);

        assert!(result.success);
        assert!(result.removed_count > 0);
        assert!(fs.exists(Path::new("/claude/agents/planner.md")));
        assert!(fs.exists(Path::new("/claude/.ecc-manifest.json")));
    }

    // --- format_status ---

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
        };
        let output = format_status(&status, false);
        assert!(output.contains("active"));
        assert!(output.contains("4.0.0"));
        assert!(output.contains("10"));
        assert!(output.contains("12"));
    }

    // --- count_ecc_hooks_in_settings ---

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
