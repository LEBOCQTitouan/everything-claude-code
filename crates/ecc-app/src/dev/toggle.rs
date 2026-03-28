use crate::config::clean::clean_from_manifest;
use crate::config::manifest::read_manifest;
use crate::install::{InstallContext, InstallOptions, InstallSummary, install_global};
use ecc_domain::config::clean::format_clean_report;
use ecc_domain::config::dev_profile::MANAGED_DIRS;
use ecc_domain::config::merge as domain_merge;
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

/// Result of `dev off`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevOffResult {
    pub report_text: String,
    pub removed_count: usize,
    pub error_count: usize,
    pub success: bool,
}

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
///
/// If managed directories are symlinks (Dev profile active), they are removed
/// with `remove_file` instead of `remove_dir_all` to avoid traversing into
/// the ECC repository root (AC-005.10).
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

    // Guard: remove symlinked managed dirs with remove_file before clean,
    // to prevent clean_from_manifest from traversing into ECC_ROOT.
    if !dry_run {
        for dir in MANAGED_DIRS {
            let link = claude_dir.join(dir);
            if fs.is_symlink(&link) {
                let _ = fs.remove_file(&link);
            }
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::config::manifest::{Artifacts, EccManifest};
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

    #[test]
    fn dev_off_removes_symlinks_safely() {
        let fs = InMemoryFileSystem::new()
            .with_symlink("/claude/agents", "/ecc/agents")
            .with_symlink("/claude/commands", "/ecc/commands")
            .with_symlink("/claude/skills", "/ecc/skills")
            .with_symlink("/claude/rules", "/ecc/rules")
            .with_file("/ecc/agents/planner.md", "# Planner")
            .with_file("/claude/.ecc-manifest.json", &manifest_json(&sample_manifest()));
        let terminal = BufferedTerminal::new();

        let _result = dev_off(&fs, &terminal, Path::new("/claude"), false);

        for dir in MANAGED_DIRS {
            let link = Path::new("/claude").join(dir);
            assert!(
                !fs.is_symlink(&link),
                "dev_off must remove symlink: {link:?}"
            );
        }
        assert!(
            fs.exists(Path::new("/ecc/agents/planner.md")),
            "dev_off must not delete ECC repo files"
        );
    }
}
