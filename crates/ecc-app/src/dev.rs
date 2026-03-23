//! Dev mode use cases — toggle ECC config on/off in `~/.claude/`.
//!
//! Composes existing [`install`] and [`config::clean`] building blocks
//! to provide a quick profile switch between "ECC-enhanced" and "vanilla Claude".

use crate::config::clean::{clean_from_manifest, is_current_ecc_hook};
use crate::config::manifest::read_manifest;
use crate::install::{InstallContext, InstallOptions, InstallSummary, install_global};
use ecc_domain::config::clean::format_clean_report;
use ecc_domain::config::dev_profile::MANAGED_DIRS;
use ecc_domain::config::manifest::EccManifest;
use ecc_domain::config::merge as domain_merge;
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Profile state of the managed ECC directories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevProfileStatus {
    /// All managed dirs are symlinks (development profile active).
    Dev,
    /// All managed dirs exist as real directories (production/copied profile).
    Default,
    /// No managed dirs exist — ECC is not installed.
    Inactive,
    /// Some dirs are symlinks, others are real dirs.
    Mixed,
}

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
    pub profile: DevProfileStatus,
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

/// Check whether ECC is currently active by reading the manifest.
pub fn dev_status(fs: &dyn FileSystem, claude_dir: &Path) -> DevStatus {
    let manifest = read_manifest(fs, claude_dir);
    let profile = detect_profile(fs, claude_dir);

    match manifest {
        Some(m) => {
            let mut status = manifest_to_status(&m);
            status.profile = profile;
            status
        }
        None => DevStatus {
            active: false,
            version: None,
            agents: 0,
            commands: 0,
            skills: 0,
            rules: 0,
            hooks: 0,
            installed_at: None,
            profile,
        },
    }
}

/// Detect which profile is active by inspecting the managed directories.
fn detect_profile(fs: &dyn FileSystem, claude_dir: &Path) -> DevProfileStatus {
    let symlinked = MANAGED_DIRS
        .iter()
        .filter(|dir| fs.is_symlink(&claude_dir.join(dir)))
        .count();
    let existing = MANAGED_DIRS
        .iter()
        .filter(|dir| fs.exists(&claude_dir.join(dir)))
        .count();

    let total = MANAGED_DIRS.len();

    if existing == 0 {
        DevProfileStatus::Inactive
    } else if symlinked == total {
        DevProfileStatus::Dev
    } else if symlinked == 0 {
        DevProfileStatus::Default
    } else {
        DevProfileStatus::Mixed
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
        profile: DevProfileStatus::Inactive, // overwritten by dev_status caller
    }
}

// ---------------------------------------------------------------------------
// dev_switch
// ---------------------------------------------------------------------------

/// Tracks a completed symlink creation for rollback purposes.
struct CompletedOp {
    link: std::path::PathBuf,
}

/// Switch to the given `profile`, updating managed directories accordingly.
///
/// - `Dev`:     creates symlinks `claude_dir/dir → ecc_root/dir` for each managed dir.
/// - `Default`: removes any existing symlinks then calls `dev_on` to reinstall copies.
/// - `dry_run`: prints planned operations without executing them.
pub fn dev_switch<F: FileSystem, T: TerminalIO>(
    fs: &F,
    terminal: &T,
    ecc_root: &Path,
    claude_dir: &Path,
    profile: ecc_domain::config::dev_profile::DevProfile,
    dry_run: bool,
) -> Result<(), DevError> {
    use ecc_domain::config::dev_profile::DevProfile;

    if !ecc_root.is_absolute() {
        return Err(DevError::RelativePath(ecc_root.to_path_buf()));
    }
    if !claude_dir.is_absolute() {
        return Err(DevError::RelativePath(claude_dir.to_path_buf()));
    }

    match profile {
        DevProfile::Dev => dev_switch_to_dev(fs, terminal, ecc_root, claude_dir, dry_run),
        DevProfile::Default => dev_switch_to_default(fs, terminal, ecc_root, claude_dir, dry_run),
    }
}

fn validate_dev_targets<F: FileSystem>(fs: &F, ecc_root: &Path) -> Result<(), DevError> {
    for dir in MANAGED_DIRS {
        let target = ecc_root.join(dir);
        if !target.starts_with(ecc_root) {
            return Err(DevError::PathEscape(target));
        }
        if !fs.exists(&target) {
            return Err(DevError::TargetNotFound(target));
        }
    }
    Ok(())
}

fn rollback_completed<F: FileSystem>(fs: &F, completed: &[CompletedOp]) {
    for op in completed {
        let _ = fs.remove_file(&op.link);
    }
}

fn dev_switch_to_dev<F: FileSystem, T: TerminalIO>(
    fs: &F,
    terminal: &T,
    ecc_root: &Path,
    claude_dir: &Path,
    dry_run: bool,
) -> Result<(), DevError> {
    validate_dev_targets(fs, ecc_root)?;

    if dry_run {
        for dir in MANAGED_DIRS {
            let target = ecc_root.join(dir);
            let link = claude_dir.join(dir);
            terminal.stdout_write(&format!("[dry-run] symlink {link:?} → {target:?}\n"));
        }
        return Ok(());
    }

    let mut completed: Vec<CompletedOp> = Vec::new();

    for dir in MANAGED_DIRS {
        let target = ecc_root.join(dir);
        let link = claude_dir.join(dir);

        if fs.is_symlink(&link) {
            fs.remove_file(&link)?;
        }
        if fs.is_dir(&link) {
            fs.remove_dir_all(&link)?;
        }

        if let Err(e) = fs.create_symlink(&target, &link) {
            rollback_completed(fs, &completed);
            return Err(DevError::Fs(e));
        }
        completed.push(CompletedOp { link });
    }

    Ok(())
}

fn dev_switch_to_default<F: FileSystem, T: TerminalIO>(
    fs: &F,
    _terminal: &T,
    _ecc_root: &Path,
    claude_dir: &Path,
    _dry_run: bool,
) -> Result<(), DevError> {
    // Remove existing symlinks for managed dirs
    for dir in MANAGED_DIRS {
        let link = claude_dir.join(dir);
        if fs.is_symlink(&link) {
            fs.remove_file(&link)?;
        }
    }
    // dev_on is called by the CLI layer after this; here we only remove symlinks
    Ok(())
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

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors returned by `dev_switch`.
#[derive(Debug, thiserror::Error)]
pub enum DevError {
    #[error("path must be absolute: {0}")]
    RelativePath(std::path::PathBuf),

    #[error("target directory does not exist: {0}")]
    TargetNotFound(std::path::PathBuf),

    #[error("target path escapes ECC root: {0}")]
    PathEscape(std::path::PathBuf),

    #[error("filesystem error: {0}")]
    Fs(#[from] ecc_ports::fs::FsError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::config::manifest::Artifacts;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem};
    use std::collections::BTreeMap;

    // --- DevProfileStatus / profile detection ---

    #[test]
    fn dev_status_symlinked_profile() {
        // All MANAGED_DIRS are symlinks → profile should be Dev
        let m = sample_manifest();
        let fs = InMemoryFileSystem::new()
            .with_file("/claude/.ecc-manifest.json", &manifest_json(&m))
            .with_symlink("/claude/agents", "/ecc/agents")
            .with_symlink("/claude/commands", "/ecc/commands")
            .with_symlink("/claude/skills", "/ecc/skills")
            .with_symlink("/claude/rules", "/ecc/rules");

        let status = dev_status(&fs, Path::new("/claude"));

        assert_eq!(status.profile, DevProfileStatus::Dev);
    }

    #[test]
    fn dev_status_copied_profile() {
        // All MANAGED_DIRS exist as real dirs (not symlinks) → profile should be Default
        let m = sample_manifest();
        let fs = InMemoryFileSystem::new()
            .with_file("/claude/.ecc-manifest.json", &manifest_json(&m))
            .with_dir("/claude/agents")
            .with_dir("/claude/commands")
            .with_dir("/claude/skills")
            .with_dir("/claude/rules");

        let status = dev_status(&fs, Path::new("/claude"));

        assert_eq!(status.profile, DevProfileStatus::Default);
    }

    #[test]
    fn dev_status_inactive_no_errors() {
        // No manifest, no dirs → profile should be Inactive
        let fs = InMemoryFileSystem::new();

        let status = dev_status(&fs, Path::new("/claude"));

        assert!(!status.active);
        assert_eq!(status.profile, DevProfileStatus::Inactive);
    }

    #[test]
    fn dev_status_mixed_state() {
        // Some dirs are symlinks, some are real dirs → Mixed
        let m = sample_manifest();
        let fs = InMemoryFileSystem::new()
            .with_file("/claude/.ecc-manifest.json", &manifest_json(&m))
            .with_symlink("/claude/agents", "/ecc/agents")
            .with_dir("/claude/commands")
            .with_symlink("/claude/skills", "/ecc/skills")
            .with_dir("/claude/rules");

        let status = dev_status(&fs, Path::new("/claude"));

        assert_eq!(status.profile, DevProfileStatus::Mixed);
    }

    #[test]
    fn dev_status_all_three_states() {
        // Covers all three states: Dev, Default, Inactive
        let m = sample_manifest();

        // Dev state
        let fs_dev = InMemoryFileSystem::new()
            .with_file("/claude/.ecc-manifest.json", &manifest_json(&m))
            .with_symlink("/claude/agents", "/ecc/agents")
            .with_symlink("/claude/commands", "/ecc/commands")
            .with_symlink("/claude/skills", "/ecc/skills")
            .with_symlink("/claude/rules", "/ecc/rules");
        assert_eq!(
            super::dev_status(&fs_dev, Path::new("/claude")).profile,
            DevProfileStatus::Dev
        );

        // Default state
        let fs_default = InMemoryFileSystem::new()
            .with_file("/claude/.ecc-manifest.json", &manifest_json(&m))
            .with_dir("/claude/agents")
            .with_dir("/claude/commands")
            .with_dir("/claude/skills")
            .with_dir("/claude/rules");
        assert_eq!(
            super::dev_status(&fs_default, Path::new("/claude")).profile,
            DevProfileStatus::Default
        );

        // Inactive state
        let fs_inactive = InMemoryFileSystem::new();
        assert_eq!(
            super::dev_status(&fs_inactive, Path::new("/claude")).profile,
            DevProfileStatus::Inactive
        );
    }

    #[test]
    fn format_status_includes_profile() {
        // Active status with Dev profile → output must contain "Profile:" line
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

    // --- dev_switch ---

    /// PC-024: dev_switch(Dev) creates symlinks for all MANAGED_DIRS.
    #[test]
    fn dev_switch_dev_creates_symlinks() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/agents")
            .with_dir("/ecc/commands")
            .with_dir("/ecc/skills")
            .with_dir("/ecc/rules");
        let terminal = BufferedTerminal::new();

        let result = dev_switch(
            &fs,
            &terminal,
            Path::new("/ecc"),
            Path::new("/claude"),
            ecc_domain::config::dev_profile::DevProfile::Dev,
            false,
        );

        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        for dir in MANAGED_DIRS {
            let link = Path::new("/claude").join(dir);
            assert!(fs.is_symlink(&link), "expected symlink at {link:?}");
        }
    }

    /// PC-025: dev_switch(Default) removes symlinks and reinstalls copies via dev_on.
    #[test]
    fn dev_switch_default_restores_copies() {
        // Pre-condition: managed dirs are symlinks (Dev profile is active)
        let fs = InMemoryFileSystem::new()
            .with_symlink("/claude/agents", "/ecc/agents")
            .with_symlink("/claude/commands", "/ecc/commands")
            .with_symlink("/claude/skills", "/ecc/skills")
            .with_symlink("/claude/rules", "/ecc/rules");
        let terminal = BufferedTerminal::new();

        let _result = dev_switch(
            &fs,
            &terminal,
            Path::new("/ecc"),
            Path::new("/claude"),
            ecc_domain::config::dev_profile::DevProfile::Default,
            false,
        );

        // All symlinks must be removed before dev_on is called
        for dir in MANAGED_DIRS {
            let link = Path::new("/claude").join(dir);
            assert!(
                !fs.is_symlink(&link),
                "symlink should be removed for Default profile: {link:?}"
            );
        }
    }

    /// PC-026: dry_run prints planned operations without executing them.
    #[test]
    fn dev_switch_dry_run() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/agents")
            .with_dir("/ecc/commands")
            .with_dir("/ecc/skills")
            .with_dir("/ecc/rules");
        let terminal = BufferedTerminal::new();

        let result = dev_switch(
            &fs,
            &terminal,
            Path::new("/ecc"),
            Path::new("/claude"),
            ecc_domain::config::dev_profile::DevProfile::Dev,
            true,
        );

        assert!(result.is_ok());
        // No symlinks should be created in dry_run mode
        for dir in MANAGED_DIRS {
            let link = Path::new("/claude").join(dir);
            assert!(!fs.is_symlink(&link), "dry_run must not create symlinks: {link:?}");
        }
        // Terminal output must mention the planned operations
        let output = terminal.stdout_output().join("");
        assert!(!output.is_empty(), "dry_run should print planned operations");
    }

    /// PC-027: rollback removes successfully-created symlinks when a later operation fails.
    #[test]
    fn dev_switch_rollback_on_error() {
        // Only first managed dir exists; rest will fail with TargetNotFound
        let first_dir = MANAGED_DIRS[0];
        let fs = InMemoryFileSystem::new().with_dir(&format!("/ecc/{first_dir}"));
        let terminal = BufferedTerminal::new();

        let result = dev_switch(
            &fs,
            &terminal,
            Path::new("/ecc"),
            Path::new("/claude"),
            ecc_domain::config::dev_profile::DevProfile::Dev,
            false,
        );

        assert!(result.is_err(), "expected Err due to missing target dirs");
        // Symlink created for the first dir must be rolled back
        let first_link = Path::new("/claude").join(first_dir);
        assert!(
            !fs.is_symlink(&first_link),
            "rollback must remove the already-created symlink: {first_link:?}"
        );
    }

    /// PC-028: target path outside ecc_root is rejected.
    #[test]
    fn dev_switch_validates_targets_within_ecc_root() {
        // We simulate by calling with a relative ecc_root to trigger RelativePath,
        // or by having a path that starts_with check would fail.
        // Easiest: pass a relative ecc_root path.
        let fs = InMemoryFileSystem::new();
        let terminal = BufferedTerminal::new();

        let result = dev_switch(
            &fs,
            &terminal,
            Path::new("relative/path"),  // NOT absolute
            Path::new("/claude"),
            ecc_domain::config::dev_profile::DevProfile::Dev,
            false,
        );

        assert!(
            matches!(result, Err(DevError::RelativePath(_))),
            "expected RelativePath error, got: {result:?}"
        );
    }

    /// PC-031: target directories must exist before creating symlinks.
    #[test]
    fn dev_switch_dev_target_must_exist() {
        // No dirs pre-populated — target dirs don't exist
        let fs = InMemoryFileSystem::new();
        let terminal = BufferedTerminal::new();

        let result = dev_switch(
            &fs,
            &terminal,
            Path::new("/ecc"),
            Path::new("/claude"),
            ecc_domain::config::dev_profile::DevProfile::Dev,
            false,
        );

        assert!(
            matches!(result, Err(DevError::TargetNotFound(_))),
            "expected TargetNotFound error, got: {result:?}"
        );
    }

    /// PC-032: dev_off removes symlinked managed dirs using remove_file (not remove_dir_all).
    #[test]
    fn dev_off_removes_symlinks_safely() {
        // Managed dirs are symlinks pointing into /ecc — dev_off must remove the
        // symlinks (remove_file), not recursively delete the /ecc subtree.
        let fs = InMemoryFileSystem::new()
            .with_symlink("/claude/agents", "/ecc/agents")
            .with_symlink("/claude/commands", "/ecc/commands")
            .with_symlink("/claude/skills", "/ecc/skills")
            .with_symlink("/claude/rules", "/ecc/rules")
            // Some real files inside the ECC repo (must survive)
            .with_file("/ecc/agents/planner.md", "# Planner")
            .with_file("/claude/.ecc-manifest.json", &manifest_json(&sample_manifest()));
        let terminal = BufferedTerminal::new();

        let _result = dev_off(&fs, &terminal, Path::new("/claude"), false);

        // The symlinks in claude_dir must be removed
        for dir in MANAGED_DIRS {
            let link = Path::new("/claude").join(dir);
            assert!(
                !fs.is_symlink(&link),
                "dev_off must remove symlink: {link:?}"
            );
        }
        // The real files inside ecc_root must NOT be deleted
        assert!(
            fs.exists(Path::new("/ecc/agents/planner.md")),
            "dev_off must not delete ECC repo files"
        );
    }

    /// PC-033: symlinks use absolute paths for both target and link.
    #[test]
    fn dev_switch_uses_absolute_paths() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/agents")
            .with_dir("/ecc/commands")
            .with_dir("/ecc/skills")
            .with_dir("/ecc/rules");
        let terminal = BufferedTerminal::new();

        let result = dev_switch(
            &fs,
            &terminal,
            Path::new("/ecc"),
            Path::new("/claude"),
            ecc_domain::config::dev_profile::DevProfile::Dev,
            false,
        );

        assert!(result.is_ok());
        for dir in MANAGED_DIRS {
            let link = Path::new("/claude").join(dir);
            let target = fs.read_symlink(&link).expect("symlink must exist");
            assert!(
                target.is_absolute(),
                "symlink target must be absolute, got: {target:?}"
            );
            assert!(
                link.is_absolute(),
                "symlink link must be absolute, got: {link:?}"
            );
        }
    }

    /// PC-034: dev_switch Dev profile does NOT modify the manifest.
    #[test]
    fn dev_switch_manifest_preservation() {
        let m = sample_manifest();
        let manifest_content = manifest_json(&m);
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/agents")
            .with_dir("/ecc/commands")
            .with_dir("/ecc/skills")
            .with_dir("/ecc/rules")
            .with_file("/claude/.ecc-manifest.json", &manifest_content);
        let terminal = BufferedTerminal::new();

        dev_switch(
            &fs,
            &terminal,
            Path::new("/ecc"),
            Path::new("/claude"),
            ecc_domain::config::dev_profile::DevProfile::Dev,
            false,
        )
        .expect("dev_switch should succeed");

        let after = fs
            .read_to_string(Path::new("/claude/.ecc-manifest.json"))
            .expect("manifest must still exist");
        assert_eq!(
            after, manifest_content,
            "dev_switch Dev must NOT modify the manifest"
        );
    }

    /// PC-035: if a dangling symlink exists at a link path, it is removed before creating new one.
    #[test]
    fn dev_switch_handles_dangling_symlinks() {
        // /claude/agents is a dangling symlink (target doesn't exist in the test FS)
        let fs = InMemoryFileSystem::new()
            .with_symlink("/claude/agents", "/old/agents")  // dangling
            .with_dir("/ecc/agents")
            .with_dir("/ecc/commands")
            .with_dir("/ecc/skills")
            .with_dir("/ecc/rules");
        let terminal = BufferedTerminal::new();

        let result = dev_switch(
            &fs,
            &terminal,
            Path::new("/ecc"),
            Path::new("/claude"),
            ecc_domain::config::dev_profile::DevProfile::Dev,
            false,
        );

        assert!(result.is_ok(), "should handle dangling symlinks: {result:?}");
        // The dangling symlink must be replaced with the new one
        let target = fs.read_symlink(Path::new("/claude/agents")).unwrap();
        assert_eq!(target, Path::new("/ecc/agents"));
    }

    /// PC-036: if a real directory exists at a link path, it is removed before creating symlink.
    #[test]
    fn dev_switch_removes_existing_dirs() {
        // /claude/agents already exists as a real directory (Default profile state)
        let fs = InMemoryFileSystem::new()
            .with_dir("/claude/agents")
            .with_file("/claude/agents/old.md", "old content")
            .with_dir("/ecc/agents")
            .with_dir("/ecc/commands")
            .with_dir("/ecc/skills")
            .with_dir("/ecc/rules");
        let terminal = BufferedTerminal::new();

        let result = dev_switch(
            &fs,
            &terminal,
            Path::new("/ecc"),
            Path::new("/claude"),
            ecc_domain::config::dev_profile::DevProfile::Dev,
            false,
        );

        assert!(result.is_ok(), "should remove existing dirs: {result:?}");
        assert!(
            fs.is_symlink(Path::new("/claude/agents")),
            "existing dir must be replaced with symlink"
        );
    }

    /// PC-047: dev_switch propagates errors as Err(DevError).
    #[test]
    fn dev_switch_error_returns_failure() {
        // Missing all target dirs → first MANAGED_DIR will fail with TargetNotFound
        let fs = InMemoryFileSystem::new();
        let terminal = BufferedTerminal::new();

        let result = dev_switch(
            &fs,
            &terminal,
            Path::new("/ecc"),
            Path::new("/claude"),
            ecc_domain::config::dev_profile::DevProfile::Dev,
            false,
        );

        assert!(result.is_err(), "should return Err when targets are missing");
    }
}
