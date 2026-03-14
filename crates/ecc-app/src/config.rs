//! Config I/O operations — reads/writes config files via FileSystem port.

pub mod clean {
    use ecc_domain::config::clean::{CleanReport, ARTIFACT_DIRS};
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
    fn is_current_ecc_hook(entry: &serde_json::Value) -> bool {
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
    ) -> Result<Option<usize>, String> {
        let content = fs
            .read_to_string(settings_path)
            .map_err(|e| e.to_string())?;
        let mut settings: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| e.to_string())?;

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
            let json =
                serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
            fs.write(settings_path, &format!("{json}\n"))
                .map_err(|e| e.to_string())?;
        }

        Ok(Some(removed_count))
    }

    /// Remove only files listed in the manifest (surgical cleanup).
    /// In `dry_run` mode, records what would be removed without actually removing.
    pub fn clean_from_manifest(
        fs: &dyn FileSystem,
        dir: &Path,
        manifest: &EccManifest,
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

        // Remove rule files (grouped by language/group)
        for (group, files) in &manifest.artifacts.rules {
            for file in files {
                let file_path = dir.join("rules").join(group).join(file);
                let label = format!("rules/{group}/{file}");
                remove_file(fs, &file_path, &label, dry_run, &mut report);
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
                    commands: vec!["plan.md".to_string()],
                    skills: vec!["tdd".to_string()],
                    rules,
                    hook_descriptions: vec![],
                },
            }
        }

        fn build_populated_fs() -> InMemoryFileSystem {
            InMemoryFileSystem::new()
                .with_dir("/claude/agents")
                .with_file("/claude/agents/planner.md", "# Planner")
                .with_dir("/claude/commands")
                .with_file("/claude/commands/plan.md", "# Plan")
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

            let report = clean_from_manifest(&fs, dir, &manifest, false);

            assert!(report.errors.is_empty());
            assert!(!report.removed.is_empty());
            assert!(!fs.exists(&dir.join("agents/planner.md")));
            assert!(!fs.exists(&dir.join("commands/plan.md")));
            assert!(!fs.exists(&dir.join("rules/common/style.md")));
            assert!(!fs.exists(&dir.join("rules/common/security.md")));
            assert!(!fs.exists(&dir.join(".ecc-manifest.json")));
        }

        #[test]
        fn clean_from_manifest_skips_missing() {
            let fs = InMemoryFileSystem::new();
            let manifest = sample_manifest();
            let dir = Path::new("/claude");

            let report = clean_from_manifest(&fs, dir, &manifest, false);

            assert!(report.removed.is_empty());
            assert!(!report.skipped.is_empty());
            assert!(report.errors.is_empty());
            // agents/planner.md, commands/plan.md, skills/tdd, rules/common/style.md,
            // rules/common/security.md, .ecc-manifest.json = 6 skipped
            assert_eq!(report.skipped.len(), 6);
        }

        #[test]
        fn clean_from_manifest_dry_run_does_not_remove() {
            let fs = build_populated_fs();
            let manifest = sample_manifest();
            let dir = Path::new("/claude");

            let report = clean_from_manifest(&fs, dir, &manifest, true);

            assert!(!report.removed.is_empty());
            // Files should still exist
            assert!(fs.exists(&dir.join("agents/planner.md")));
            assert!(fs.exists(&dir.join("commands/plan.md")));
            assert!(fs.exists(&dir.join(".ecc-manifest.json")));
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
            assert!(!fs.exists(&dir.join("commands/plan.md")));
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
            assert!(
                report
                    .removed
                    .iter()
                    .any(|r| r.contains("1 ECC hook(s)"))
            );

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
            let is_legacy = |v: &serde_json::Value| {
                v.get("type").and_then(|t| t.as_str()) == Some("legacy")
            };

            let report = clean_all(&fs, dir, &is_legacy, false);

            assert!(
                report
                    .removed
                    .iter()
                    .any(|r| r.contains("1 ECC hook(s)"))
            );
        }
    }
}

pub mod merge {
    use ecc_domain::config::merge::{contents_differ, FileToReview};
    use ecc_ports::fs::FileSystem;
    use std::path::Path;

    /// Pre-scan a directory to identify files that need review (new or changed).
    /// Returns `(files_to_review, unchanged_filenames)`.
    pub fn pre_scan_directory(
        fs: &dyn FileSystem,
        src_dir: &Path,
        dest_dir: &Path,
        ext: &str,
    ) -> (Vec<FileToReview>, Vec<String>) {
        let mut files_to_review = Vec::new();
        let mut unchanged = Vec::new();

        let entries = match fs.read_dir(src_dir) {
            Ok(e) => e,
            Err(_) => return (files_to_review, unchanged),
        };

        let src_files: Vec<String> = entries
            .iter()
            .filter_map(|p| {
                let name = p.file_name()?.to_string_lossy().into_owned();
                if name.ends_with(ext) {
                    Some(name)
                } else {
                    None
                }
            })
            .collect();

        for filename in src_files {
            let src_path = src_dir.join(&filename);
            let dest_path = dest_dir.join(&filename);

            if !fs.exists(&dest_path) {
                files_to_review.push(FileToReview {
                    filename,
                    src_path,
                    dest_path,
                    is_new: true,
                });
            } else {
                let src_content = fs.read_to_string(&src_path).unwrap_or_default();
                let dest_content = fs.read_to_string(&dest_path).unwrap_or_default();

                if contents_differ(&src_content, &dest_content) {
                    files_to_review.push(FileToReview {
                        filename,
                        src_path,
                        dest_path,
                        is_new: false,
                    });
                } else {
                    unchanged.push(filename);
                }
            }
        }

        (files_to_review, unchanged)
    }

    /// Copy a file from source to destination.
    /// In dry-run mode, the copy is skipped.
    pub fn apply_accept(
        fs: &dyn FileSystem,
        src_path: &Path,
        dest_path: &Path,
        dry_run: bool,
    ) -> Result<(), String> {
        if dry_run {
            return Ok(());
        }

        if let Some(parent) = dest_path.parent() {
            fs.create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {e}"))?;
        }

        fs.copy(src_path, dest_path)
            .map_err(|e| format!("Failed to copy file: {e}"))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ecc_test_support::InMemoryFileSystem;
        use std::path::Path;

        // --- pre_scan_directory ---

        #[test]
        fn pre_scan_directory_new_files() {
            let fs = InMemoryFileSystem::new()
                .with_file("/src/a.md", "content a")
                .with_file("/src/b.md", "content b");

            let (to_review, unchanged) =
                pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
            assert_eq!(to_review.len(), 2);
            assert!(unchanged.is_empty());
            assert!(to_review[0].is_new);
            assert!(to_review[1].is_new);
        }

        #[test]
        fn pre_scan_directory_changed_files() {
            let fs = InMemoryFileSystem::new()
                .with_file("/src/a.md", "new content")
                .with_file("/dest/a.md", "old content");

            let (to_review, unchanged) =
                pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
            assert_eq!(to_review.len(), 1);
            assert!(unchanged.is_empty());
            assert!(!to_review[0].is_new);
        }

        #[test]
        fn pre_scan_directory_unchanged_files() {
            let fs = InMemoryFileSystem::new()
                .with_file("/src/a.md", "same content")
                .with_file("/dest/a.md", "same content");

            let (to_review, unchanged) =
                pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
            assert!(to_review.is_empty());
            assert_eq!(unchanged, vec!["a.md"]);
        }

        #[test]
        fn pre_scan_directory_filters_by_ext() {
            let fs = InMemoryFileSystem::new()
                .with_file("/src/a.md", "content")
                .with_file("/src/b.txt", "content");

            let (to_review, _) =
                pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
            assert_eq!(to_review.len(), 1);
            assert_eq!(to_review[0].filename, "a.md");
        }

        #[test]
        fn pre_scan_directory_mixed() {
            let fs = InMemoryFileSystem::new()
                .with_file("/src/new.md", "brand new")
                .with_file("/src/changed.md", "updated")
                .with_file("/dest/changed.md", "original")
                .with_file("/src/same.md", "same")
                .with_file("/dest/same.md", "same");

            let (to_review, unchanged) =
                pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
            assert_eq!(to_review.len(), 2);
            assert_eq!(unchanged, vec!["same.md"]);
        }

        #[test]
        fn pre_scan_directory_empty_src() {
            let fs = InMemoryFileSystem::new();
            let (to_review, unchanged) =
                pre_scan_directory(&fs, Path::new("/src"), Path::new("/dest"), ".md");
            assert!(to_review.is_empty());
            assert!(unchanged.is_empty());
        }

        // --- apply_accept ---

        #[test]
        fn apply_accept_copies_file() {
            let fs = InMemoryFileSystem::new()
                .with_file("/src/a.md", "content");

            let result = apply_accept(
                &fs,
                Path::new("/src/a.md"),
                Path::new("/dest/a.md"),
                false,
            );
            assert!(result.is_ok());
            assert_eq!(
                fs.read_to_string(Path::new("/dest/a.md")).unwrap(),
                "content"
            );
        }

        #[test]
        fn apply_accept_dry_run_skips() {
            let fs = InMemoryFileSystem::new()
                .with_file("/src/a.md", "content");

            let result = apply_accept(
                &fs,
                Path::new("/src/a.md"),
                Path::new("/dest/a.md"),
                true,
            );
            assert!(result.is_ok());
            assert!(!fs.exists(Path::new("/dest/a.md")));
        }

        #[test]
        fn apply_accept_creates_parent_dirs() {
            let fs = InMemoryFileSystem::new()
                .with_file("/src/a.md", "content");

            let result = apply_accept(
                &fs,
                Path::new("/src/a.md"),
                Path::new("/dest/sub/dir/a.md"),
                false,
            );
            assert!(result.is_ok());
            assert!(fs.exists(Path::new("/dest/sub/dir/a.md")));
        }

        #[test]
        fn apply_accept_error_on_missing_src() {
            let fs = InMemoryFileSystem::new();

            let result = apply_accept(
                &fs,
                Path::new("/nonexistent.md"),
                Path::new("/dest/a.md"),
                false,
            );
            assert!(result.is_err());
        }
    }
}

pub mod manifest {
    use ecc_domain::config::manifest::{EccManifest, MANIFEST_FILENAME};
    use ecc_ports::fs::FileSystem;
    use std::path::Path;

    /// Read an existing manifest from a directory via the FileSystem port.
    /// Returns None if not found or corrupted.
    pub fn read_manifest(fs: &dyn FileSystem, dir: &Path) -> Option<EccManifest> {
        let manifest_path = dir.join(MANIFEST_FILENAME);
        let content = fs.read_to_string(&manifest_path).ok()?;
        let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
        if parsed.get("version").is_none() || parsed.get("artifacts").is_none() {
            return None;
        }
        serde_json::from_value(parsed).ok()
    }

    /// Write a manifest to a directory via the FileSystem port.
    pub fn write_manifest(
        fs: &dyn FileSystem,
        dir: &Path,
        manifest: &EccManifest,
    ) -> Result<(), ecc_ports::fs::FsError> {
        fs.create_dir_all(dir)?;
        let manifest_path = dir.join(MANIFEST_FILENAME);
        let json = serde_json::to_string_pretty(manifest)
            .expect("manifest serialization should not fail");
        fs.write(&manifest_path, &format!("{json}\n"))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ecc_domain::config::manifest::{create_manifest, Artifacts};
        use ecc_test_support::InMemoryFileSystem;
        use std::collections::BTreeMap;
        use std::path::Path;

        fn sample_artifacts() -> Artifacts {
            Artifacts {
                agents: vec!["agent1.md".into(), "agent2.md".into()],
                commands: vec!["cmd1.md".into()],
                skills: vec!["skill1".into()],
                rules: {
                    let mut m = BTreeMap::new();
                    m.insert("common".into(), vec!["rule1.md".into()]);
                    m
                },
                hook_descriptions: vec!["hook1".into()],
            }
        }

        #[test]
        fn read_manifest_not_found() {
            let fs = InMemoryFileSystem::new();
            assert!(read_manifest(&fs, Path::new("/project/.claude")).is_none());
        }

        #[test]
        fn read_manifest_invalid_json() {
            let fs = InMemoryFileSystem::new()
                .with_file("/project/.claude/.ecc-manifest.json", "not json");
            assert!(read_manifest(&fs, Path::new("/project/.claude")).is_none());
        }

        #[test]
        fn read_manifest_missing_version() {
            let fs = InMemoryFileSystem::new().with_file(
                "/project/.claude/.ecc-manifest.json",
                r#"{"artifacts": {}}"#,
            );
            assert!(read_manifest(&fs, Path::new("/project/.claude")).is_none());
        }

        #[test]
        fn write_and_read_manifest_roundtrip() {
            let fs = InMemoryFileSystem::new();
            let dir = Path::new("/project/.claude");
            let original = create_manifest(
                "4.0.0",
                "2026-03-14T00:00:00Z",
                &["rust".into()],
                sample_artifacts(),
            );
            write_manifest(&fs, dir, &original).unwrap();
            let read_back = read_manifest(&fs, dir).unwrap();
            assert_eq!(read_back, original);
        }

        #[test]
        fn json_uses_camel_case() {
            let m = create_manifest("4.0.0", "now", &[], Artifacts::default());
            let json = serde_json::to_string(&m).unwrap();
            assert!(json.contains("installedAt"));
            assert!(json.contains("updatedAt"));
            assert!(json.contains("hookDescriptions"));
            assert!(!json.contains("installed_at"));
        }
    }
}

pub mod gitignore {
    use ecc_domain::config::gitignore::{
        build_gitignore_section, parse_gitignore_patterns, GitignoreEntry, GitignoreResult,
        ECC_GITIGNORE_ENTRIES,
    };
    use ecc_ports::fs::FileSystem;
    use ecc_ports::shell::ShellExecutor;
    use std::path::Path;

    /// Check if a directory is inside a git repository.
    pub fn is_git_repo(shell: &dyn ShellExecutor, dir: &Path) -> bool {
        shell
            .run_command_in_dir("git", &["rev-parse", "--git-dir"], dir)
            .is_ok_and(|out| out.success())
    }

    /// Ensure ECC entries are present in .gitignore.
    /// Creates .gitignore if it doesn't exist (only in git repos).
    pub fn ensure_gitignore_entries(
        fs: &dyn FileSystem,
        shell: &dyn ShellExecutor,
        dir: &Path,
        entries: Option<&[GitignoreEntry]>,
    ) -> GitignoreResult {
        let entries = entries.unwrap_or(ECC_GITIGNORE_ENTRIES);

        if !is_git_repo(shell, dir) {
            return GitignoreResult {
                added: vec![],
                already_present: vec![],
                skipped: true,
            };
        }

        let gitignore_path = dir.join(".gitignore");
        let existing_content = fs
            .read_to_string(&gitignore_path)
            .unwrap_or_default();

        let existing_patterns = parse_gitignore_patterns(&existing_content);
        let mut added = Vec::new();
        let mut already_present = Vec::new();
        let mut to_add = Vec::new();

        for entry in entries {
            if existing_patterns.contains(entry.pattern) {
                already_present.push(entry.pattern.to_string());
            } else {
                to_add.push(entry);
                added.push(entry.pattern.to_string());
            }
        }

        if to_add.is_empty() {
            return GitignoreResult {
                added,
                already_present,
                skipped: false,
            };
        }

        let section = build_gitignore_section(&to_add);
        let new_content = format!("{}\n{}", existing_content.trim_end(), section);
        let _ = fs.write(&gitignore_path, &new_content);

        GitignoreResult {
            added,
            already_present,
            skipped: false,
        }
    }

    /// Find ECC-generated files that are currently tracked by git.
    pub fn find_tracked_ecc_files(
        shell: &dyn ShellExecutor,
        fs: &dyn FileSystem,
        dir: &Path,
    ) -> Vec<String> {
        if !is_git_repo(shell, dir) {
            return vec![];
        }

        let mut tracked = Vec::new();
        for entry in ECC_GITIGNORE_ENTRIES {
            if entry.pattern.ends_with('/') {
                // Directory — check if any files inside are tracked
                let full_path = dir.join(entry.pattern);
                if fs.exists(&full_path)
                    && let Ok(out) =
                        shell.run_command_in_dir("git", &["ls-files", entry.pattern], dir)
                    && out.success()
                    && !out.stdout.trim().is_empty()
                {
                    tracked.push(entry.pattern.to_string());
                }
            } else if let Ok(out) = shell.run_command_in_dir(
                "git",
                &["ls-files", "--error-unmatch", entry.pattern],
                dir,
            )
                && out.success()
            {
                tracked.push(entry.pattern.to_string());
            }
        }
        tracked
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ecc_ports::shell::CommandOutput;
        use ecc_test_support::{InMemoryFileSystem, MockExecutor};

        fn git_success() -> CommandOutput {
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            }
        }

        fn git_executor() -> MockExecutor {
            MockExecutor::new().on("git", git_success())
        }

        // --- is_git_repo ---

        #[test]
        fn is_git_repo_true() {
            let shell = git_executor();
            assert!(is_git_repo(&shell, Path::new("/project")));
        }

        #[test]
        fn is_git_repo_false() {
            let shell = MockExecutor::new().on(
                "git",
                CommandOutput {
                    stdout: String::new(),
                    stderr: "not a git repo".into(),
                    exit_code: 128,
                },
            );
            assert!(!is_git_repo(&shell, Path::new("/project")));
        }

        // --- ensure_gitignore_entries ---

        #[test]
        fn ensure_skips_non_git_repo() {
            let fs = InMemoryFileSystem::new();
            let shell = MockExecutor::new().on(
                "git",
                CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 128,
                },
            );
            let result = ensure_gitignore_entries(&fs, &shell, Path::new("/project"), None);
            assert!(result.skipped);
            assert!(result.added.is_empty());
        }

        #[test]
        fn ensure_adds_all_entries_to_new_gitignore() {
            let fs = InMemoryFileSystem::new();
            let shell = git_executor();
            let result = ensure_gitignore_entries(&fs, &shell, Path::new("/project"), None);
            assert!(!result.skipped);
            assert_eq!(result.added.len(), ECC_GITIGNORE_ENTRIES.len());
            assert!(result.already_present.is_empty());
        }

        #[test]
        fn ensure_detects_already_present() {
            let fs = InMemoryFileSystem::new()
                .with_file("/project/.gitignore", ".claude/settings.local.json\n");
            let shell = git_executor();
            let result = ensure_gitignore_entries(&fs, &shell, Path::new("/project"), None);
            assert_eq!(result.already_present.len(), 1);
            assert_eq!(result.added.len(), ECC_GITIGNORE_ENTRIES.len() - 1);
        }

        #[test]
        fn ensure_all_present_adds_nothing() {
            let content = ECC_GITIGNORE_ENTRIES
                .iter()
                .map(|e| e.pattern)
                .collect::<Vec<_>>()
                .join("\n");
            let fs = InMemoryFileSystem::new().with_file("/project/.gitignore", &content);
            let shell = git_executor();
            let result = ensure_gitignore_entries(&fs, &shell, Path::new("/project"), None);
            assert!(result.added.is_empty());
            assert_eq!(result.already_present.len(), ECC_GITIGNORE_ENTRIES.len());
        }
    }
}
