use crate::config::manifest::read_manifest;
use ecc_domain::config::dev_profile::MANAGED_DIRS;
use ecc_domain::config::manifest::EccManifest;
use ecc_ports::fs::FileSystem;
use std::path::Path;

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
pub fn detect_profile(fs: &dyn FileSystem, claude_dir: &Path) -> DevProfileStatus {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::config::manifest::Artifacts;
    use ecc_test_support::InMemoryFileSystem;
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
                patterns: vec![],
                teams: vec![],
            },
        }
    }

    fn manifest_json(m: &EccManifest) -> String {
        serde_json::to_string_pretty(m).unwrap()
    }

    #[test]
    fn dev_status_symlinked_profile() {
        let m = sample_manifest();
        let fs = InMemoryFileSystem::new()
            .with_file("/claude/.ecc-manifest.json", &manifest_json(&m))
            .with_symlink("/claude/agents", "/ecc/agents")
            .with_symlink("/claude/commands", "/ecc/commands")
            .with_symlink("/claude/skills", "/ecc/skills")
            .with_symlink("/claude/rules", "/ecc/rules")
            .with_symlink("/claude/teams", "/ecc/teams");

        let status = dev_status(&fs, Path::new("/claude"));

        assert_eq!(status.profile, DevProfileStatus::Dev);
    }

    #[test]
    fn dev_status_copied_profile() {
        let m = sample_manifest();
        let fs = InMemoryFileSystem::new()
            .with_file("/claude/.ecc-manifest.json", &manifest_json(&m))
            .with_dir("/claude/agents")
            .with_dir("/claude/commands")
            .with_dir("/claude/skills")
            .with_dir("/claude/rules")
            .with_dir("/claude/teams");

        let status = dev_status(&fs, Path::new("/claude"));

        assert_eq!(status.profile, DevProfileStatus::Default);
    }

    #[test]
    fn dev_status_inactive_no_errors() {
        let fs = InMemoryFileSystem::new();

        let status = dev_status(&fs, Path::new("/claude"));

        assert!(!status.active);
        assert_eq!(status.profile, DevProfileStatus::Inactive);
    }

    #[test]
    fn dev_status_mixed_state() {
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
        let m = sample_manifest();

        let fs_dev = InMemoryFileSystem::new()
            .with_file("/claude/.ecc-manifest.json", &manifest_json(&m))
            .with_symlink("/claude/agents", "/ecc/agents")
            .with_symlink("/claude/commands", "/ecc/commands")
            .with_symlink("/claude/skills", "/ecc/skills")
            .with_symlink("/claude/rules", "/ecc/rules")
            .with_symlink("/claude/teams", "/ecc/teams");
        assert_eq!(
            dev_status(&fs_dev, Path::new("/claude")).profile,
            DevProfileStatus::Dev
        );

        let fs_default = InMemoryFileSystem::new()
            .with_file("/claude/.ecc-manifest.json", &manifest_json(&m))
            .with_dir("/claude/agents")
            .with_dir("/claude/commands")
            .with_dir("/claude/skills")
            .with_dir("/claude/rules")
            .with_dir("/claude/teams");
        assert_eq!(
            dev_status(&fs_default, Path::new("/claude")).profile,
            DevProfileStatus::Default
        );

        let fs_inactive = InMemoryFileSystem::new();
        assert_eq!(
            dev_status(&fs_inactive, Path::new("/claude")).profile,
            DevProfileStatus::Inactive
        );
    }

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
}
