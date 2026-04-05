use super::error::ConfigAppError;
use ecc_domain::config::clean::{ARTIFACT_DIRS, CleanReport};
use ecc_domain::config::manifest::{EccManifest, MANIFEST_FILENAME};
use ecc_ports::fs::FileSystem;
use std::path::Path;

fn remove_file(
    fs: &dyn FileSystem,
    path: &Path,
    label: &str,
    dry_run: bool,
    report: &mut CleanReport,
) {
    if !fs.exists(path) {
        report.skipped.push(label.to_string());
        return;
    }
    if dry_run {
        report.removed.push(label.to_string());
        return;
    }
    match fs.remove_file(path) {
        Ok(()) => report.removed.push(label.to_string()),
        Err(e) => report.errors.push(format!("{label}: {e}")),
    }
}

fn remove_directory(
    fs: &dyn FileSystem,
    path: &Path,
    label: &str,
    dry_run: bool,
    report: &mut CleanReport,
) {
    if !fs.exists(path) {
        report.skipped.push(label.to_string());
        return;
    }
    if dry_run {
        report.removed.push(label.to_string());
        return;
    }
    match fs.remove_dir_all(path) {
        Ok(()) => report.removed.push(label.to_string()),
        Err(e) => report.errors.push(format!("{label}: {e}")),
    }
}

/// Returns true if a hook entry's `hooks[].command` starts with "ecc-hook " or "ecc-shell-hook ".
pub(crate) fn is_current_ecc_hook(entry: &serde_json::Value) -> bool {
    let Some(hooks) = entry.get("hooks").and_then(|h| h.as_array()) else {
        return false;
    };

    hooks.iter().any(|hook| {
        let Some(cmd) = hook.get("command").and_then(|c| c.as_str()) else {
            return false;
        };
        cmd.starts_with("ecc-hook ") || cmd.starts_with("ecc-shell-hook ")
    })
}

/// Remove ECC hooks from settings.json, preserving user-added hooks.
/// Returns `Ok(Some(count))` if hooks were removed, `Ok(None)` if no changes, `Err` on failure.
fn remove_ecc_hooks(
    fs: &dyn FileSystem,
    settings_path: &Path,
    is_legacy_hook: &dyn Fn(&serde_json::Value) -> bool,
    dry_run: bool,
) -> Result<Option<usize>, ConfigAppError> {
    let content = fs
        .read_to_string(settings_path)
        .map_err(|e| ConfigAppError::ReadFile {
            path: settings_path.display().to_string(),
            reason: e.to_string(),
        })?;
    let mut settings: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| ConfigAppError::ParseSettings {
            reason: e.to_string(),
        })?;

    let Some(hooks_obj) = settings.get_mut("hooks").and_then(|h| h.as_object_mut()) else {
        return Ok(None);
    };

    let hooks_before: usize = hooks_obj
        .values()
        .filter_map(|v| v.as_array())
        .map(|a| a.len())
        .sum();

    for entries in hooks_obj.values_mut() {
        let Some(arr) = entries.as_array_mut() else {
            continue;
        };
        arr.retain(|entry| !is_legacy_hook(entry) && !is_current_ecc_hook(entry));
    }

    let hooks_after: usize = hooks_obj
        .values()
        .filter_map(|v| v.as_array())
        .map(|a| a.len())
        .sum();

    let removed_count = hooks_before - hooks_after;
    if removed_count == 0 {
        return Ok(None);
    }

    if !dry_run {
        let json = serde_json::to_string_pretty(&settings).map_err(|e| {
            ConfigAppError::SerializeSettings {
                reason: e.to_string(),
            }
        })?;
        fs.write(settings_path, &format!("{json}\n")).map_err(|e| {
            ConfigAppError::WriteSettings {
                reason: e.to_string(),
            }
        })?;
    }

    Ok(Some(removed_count))
}

/// Remove only files listed in the manifest (surgical cleanup).
/// In `dry_run` mode, records what would be removed without actually removing.
/// `is_legacy_hook` identifies legacy ECC hooks to remove from settings.json.
pub fn clean_from_manifest(
    fs: &dyn FileSystem,
    dir: &Path,
    manifest: &EccManifest,
    is_legacy_hook: &dyn Fn(&serde_json::Value) -> bool,
    dry_run: bool,
) -> CleanReport {
    let mut report = CleanReport::new();

    // Remove agent files
    for agent in &manifest.artifacts.agents {
        let file_path = dir.join("agents").join(agent);
        let label = format!("agents/{agent}");
        remove_file(fs, &file_path, &label, dry_run, &mut report);
    }

    // Remove command files
    for command in &manifest.artifacts.commands {
        let file_path = dir.join("commands").join(command);
        let label = format!("commands/{command}");
        remove_file(fs, &file_path, &label, dry_run, &mut report);
    }

    // Remove skill directories
    for skill in &manifest.artifacts.skills {
        let dir_path = dir.join("skills").join(skill);
        let label = format!("skills/{skill}");
        remove_directory(fs, &dir_path, &label, dry_run, &mut report);
    }

    // Remove team files
    for team in &manifest.artifacts.teams {
        let file_path = dir.join("teams").join(team);
        let label = format!("teams/{team}");
        remove_file(fs, &file_path, &label, dry_run, &mut report);
    }

    // Remove rule files (grouped by language/group)
    for (group, files) in &manifest.artifacts.rules {
        for file in files {
            let file_path = dir.join("rules").join(group).join(file);
            let label = format!("rules/{group}/{file}");
            remove_file(fs, &file_path, &label, dry_run, &mut report);
        }
    }

    // Remove ECC hooks from settings.json
    let settings_path = dir.join("settings.json");
    if fs.exists(&settings_path) {
        match remove_ecc_hooks(fs, &settings_path, is_legacy_hook, dry_run) {
            Ok(Some(count)) => {
                report
                    .removed
                    .push(format!("settings.json ({count} ECC hook(s))"));
            }
            Ok(None) => {}
            Err(msg) => {
                report.errors.push(format!("settings.json: {msg}"));
            }
        }
    }

    // Remove manifest itself
    let manifest_path = dir.join(MANIFEST_FILENAME);
    remove_file(fs, &manifest_path, MANIFEST_FILENAME, dry_run, &mut report);

    report
}

/// Remove entire ECC directories and clean hooks from settings.json (nuclear option).
/// `is_legacy_hook` is a predicate that identifies legacy ECC hooks to remove.
pub fn clean_all(
    fs: &dyn FileSystem,
    dir: &Path,
    is_legacy_hook: &dyn Fn(&serde_json::Value) -> bool,
    dry_run: bool,
) -> CleanReport {
    let mut report = CleanReport::new();

    // Remove entire artifact directories
    for artifact_dir in ARTIFACT_DIRS {
        let dir_path = dir.join(artifact_dir);
        remove_directory(fs, &dir_path, artifact_dir, dry_run, &mut report);
    }

    // Remove ECC hooks from settings.json
    let settings_path = dir.join("settings.json");
    if fs.exists(&settings_path) {
        match remove_ecc_hooks(fs, &settings_path, is_legacy_hook, dry_run) {
            Ok(Some(count)) => {
                report
                    .removed
                    .push(format!("settings.json ({count} ECC hook(s))"));
            }
            Ok(None) => {}
            Err(msg) => {
                report.errors.push(format!("settings.json: {msg}"));
            }
        }
    }

    // Remove manifest
    let manifest_path = dir.join(MANIFEST_FILENAME);
    remove_file(fs, &manifest_path, MANIFEST_FILENAME, dry_run, &mut report);

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::config::manifest::Artifacts;
    use ecc_test_support::InMemoryFileSystem;
    use std::collections::BTreeMap;
    use std::path::Path;

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
                hook_descriptions: vec![],
                patterns: vec![],
                teams: vec![],
            },
        }
    }

    fn build_populated_fs() -> InMemoryFileSystem {
        InMemoryFileSystem::new()
            .with_dir("/claude/agents")
            .with_file("/claude/agents/planner.md", "# Planner")
            .with_dir("/claude/commands")
            .with_file("/claude/commands/spec.md", "# Spec")
            .with_dir("/claude/skills")
            .with_dir("/claude/skills/tdd")
            .with_file("/claude/skills/tdd/SKILL.md", "# TDD")
            .with_dir("/claude/rules")
            .with_dir("/claude/rules/common")
            .with_file("/claude/rules/common/style.md", "# Style")
            .with_file("/claude/rules/common/security.md", "# Security")
            .with_file("/claude/.ecc-manifest.json", "{}")
    }

    // --- clean_from_manifest ---

    #[test]
    fn clean_from_manifest_removes_listed_files() {
        let fs = build_populated_fs();
        let manifest = sample_manifest();
        let dir = Path::new("/claude");

        let no_legacy = |_: &serde_json::Value| false;
        let report = clean_from_manifest(&fs, dir, &manifest, &no_legacy, false);

        assert!(report.errors.is_empty());
        assert!(!report.removed.is_empty());
        assert!(!fs.exists(&dir.join("agents/planner.md")));
        assert!(!fs.exists(&dir.join("commands/spec.md")));
        assert!(!fs.exists(&dir.join("rules/common/style.md")));
        assert!(!fs.exists(&dir.join("rules/common/security.md")));
        assert!(!fs.exists(&dir.join(".ecc-manifest.json")));
    }

    #[test]
    fn clean_from_manifest_skips_missing() {
        let fs = InMemoryFileSystem::new();
        let manifest = sample_manifest();
        let dir = Path::new("/claude");

        let no_legacy = |_: &serde_json::Value| false;
        let report = clean_from_manifest(&fs, dir, &manifest, &no_legacy, false);

        assert!(report.removed.is_empty());
        assert!(!report.skipped.is_empty());
        assert!(report.errors.is_empty());
        // agents/planner.md, commands/spec.md, skills/tdd, rules/common/style.md,
        // rules/common/security.md, .ecc-manifest.json = 6 skipped
        assert_eq!(report.skipped.len(), 6);
    }

    #[test]
    fn clean_from_manifest_dry_run_does_not_remove() {
        let fs = build_populated_fs();
        let manifest = sample_manifest();
        let dir = Path::new("/claude");

        let no_legacy = |_: &serde_json::Value| false;
        let report = clean_from_manifest(&fs, dir, &manifest, &no_legacy, true);

        assert!(!report.removed.is_empty());
        // Files should still exist
        assert!(fs.exists(&dir.join("agents/planner.md")));
        assert!(fs.exists(&dir.join("commands/spec.md")));
        assert!(fs.exists(&dir.join(".ecc-manifest.json")));
    }

    #[test]
    fn clean_from_manifest_removes_ecc_hooks() {
        let settings = r#"{
            "hooks": {
                "PreToolUse": [
                    {"description": "ECC format", "hooks": [{"command": "ecc-hook format"}]},
                    {"description": "User hook", "hooks": [{"command": "my-custom-hook"}]}
                ]
            }
        }"#;
        let fs = build_populated_fs().with_file("/claude/settings.json", settings);
        let manifest = sample_manifest();
        let dir = Path::new("/claude");
        let no_legacy = |_: &serde_json::Value| false;

        let report = clean_from_manifest(&fs, dir, &manifest, &no_legacy, false);

        assert!(report.removed.iter().any(|r| r.contains("1 ECC hook(s)")));

        // Verify user hook preserved
        let updated = fs
            .read_to_string(Path::new("/claude/settings.json"))
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&updated).unwrap();
        let pre_hooks = parsed["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pre_hooks.len(), 1);
        assert_eq!(pre_hooks[0]["description"], "User hook");
    }

    // --- clean_all ---

    #[test]
    fn clean_all_removes_directories() {
        let fs = build_populated_fs();
        let dir = Path::new("/claude");
        let no_legacy = |_: &serde_json::Value| false;

        let report = clean_all(&fs, dir, &no_legacy, false);

        assert!(report.errors.is_empty());
        assert!(!fs.exists(&dir.join("agents/planner.md")));
        assert!(!fs.exists(&dir.join("commands/spec.md")));
        assert!(!fs.exists(&dir.join("skills/tdd/SKILL.md")));
    }

    #[test]
    fn clean_all_cleans_hooks() {
        let settings = r#"{
            "hooks": {
                "PreToolUse": [
                    {"description": "ECC format", "hooks": [{"command": "ecc-hook format"}]},
                    {"description": "User hook", "hooks": [{"command": "my-custom-hook"}]}
                ]
            },
            "other": "preserved"
        }"#;
        let fs = InMemoryFileSystem::new()
            .with_file("/claude/settings.json", settings)
            .with_file("/claude/.ecc-manifest.json", "{}");
        let dir = Path::new("/claude");
        let no_legacy = |_: &serde_json::Value| false;

        let report = clean_all(&fs, dir, &no_legacy, false);

        // Should have removed 1 ECC hook
        assert!(report.removed.iter().any(|r| r.contains("1 ECC hook(s)")));

        // Verify settings.json was rewritten with user hook preserved
        let updated = fs
            .read_to_string(Path::new("/claude/settings.json"))
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&updated).unwrap();
        let pre_hooks = parsed["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pre_hooks.len(), 1);
        assert_eq!(pre_hooks[0]["description"], "User hook");
        // Other fields preserved
        assert_eq!(parsed["other"], "preserved");
    }

    #[test]
    fn clean_all_with_legacy_hook_predicate() {
        let settings = r#"{
            "hooks": {
                "PreToolUse": [
                    {"description": "legacy", "type": "legacy"},
                    {"description": "keep", "hooks": [{"command": "safe"}]}
                ]
            }
        }"#;
        let fs = InMemoryFileSystem::new()
            .with_file("/claude/settings.json", settings)
            .with_file("/claude/.ecc-manifest.json", "{}");
        let dir = Path::new("/claude");
        let is_legacy =
            |v: &serde_json::Value| v.get("type").and_then(|t| t.as_str()) == Some("legacy");

        let report = clean_all(&fs, dir, &is_legacy, false);

        assert!(report.removed.iter().any(|r| r.contains("1 ECC hook(s)")));
    }
}
