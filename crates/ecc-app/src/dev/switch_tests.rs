use super::*;
use ecc_domain::config::dev_profile::MANAGED_DIRS;
use ecc_test_support::{BufferedTerminal, InMemoryFileSystem};
use std::path::Path;

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

#[test]
fn dev_switch_default_restores_copies() {
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

    for dir in MANAGED_DIRS {
        let link = Path::new("/claude").join(dir);
        assert!(
            !fs.is_symlink(&link),
            "symlink should be removed for Default profile: {link:?}"
        );
    }
}

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
    for dir in MANAGED_DIRS {
        let link = Path::new("/claude").join(dir);
        assert!(
            !fs.is_symlink(&link),
            "dry_run must not create symlinks: {link:?}"
        );
    }
    let output = terminal.stdout_output().join("");
    assert!(
        !output.is_empty(),
        "dry_run should print planned operations"
    );
}

#[test]
fn dev_switch_rollback_on_error() {
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
    let first_link = Path::new("/claude").join(first_dir);
    assert!(
        !fs.is_symlink(&first_link),
        "rollback must remove the already-created symlink: {first_link:?}"
    );
}

#[test]
fn dev_switch_validates_targets_within_ecc_root() {
    let fs = InMemoryFileSystem::new();
    let terminal = BufferedTerminal::new();

    let result = dev_switch(
        &fs,
        &terminal,
        Path::new("relative/path"),
        Path::new("/claude"),
        ecc_domain::config::dev_profile::DevProfile::Dev,
        false,
    );

    assert!(
        matches!(result, Err(DevError::RelativePath(_))),
        "expected RelativePath error, got: {result:?}"
    );
}

#[test]
fn dev_switch_dev_target_must_exist() {
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

#[test]
fn dev_switch_manifest_preservation() {
    use ecc_domain::config::manifest::{Artifacts, EccManifest};
    use std::collections::BTreeMap;

    let mut rules = BTreeMap::new();
    rules.insert("common".to_string(), vec!["style.md".to_string()]);
    let m = EccManifest {
        version: "4.0.0".to_string(),
        installed_at: "2026-03-14T00:00:00Z".to_string(),
        updated_at: "2026-03-14T00:00:00Z".to_string(),
        languages: vec![],
        artifacts: Artifacts {
            agents: vec![],
            commands: vec![],
            skills: vec![],
            rules,
            hook_descriptions: vec![],
        },
    };
    let manifest_content = serde_json::to_string_pretty(&m).unwrap();
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

    use ecc_ports::fs::FileSystem;
    let after = fs
        .read_to_string(Path::new("/claude/.ecc-manifest.json"))
        .expect("manifest must still exist");
    assert_eq!(
        after, manifest_content,
        "dev_switch Dev must NOT modify the manifest"
    );
}

#[test]
fn dev_switch_handles_dangling_symlinks() {
    let fs = InMemoryFileSystem::new()
        .with_symlink("/claude/agents", "/old/agents")
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

    assert!(
        result.is_ok(),
        "should handle dangling symlinks: {result:?}"
    );
    let target = fs.read_symlink(Path::new("/claude/agents")).unwrap();
    assert_eq!(target, Path::new("/ecc/agents"));
}

#[test]
fn dev_switch_removes_existing_dirs() {
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

#[test]
fn dev_switch_error_returns_failure() {
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
        result.is_err(),
        "should return Err when targets are missing"
    );
}
