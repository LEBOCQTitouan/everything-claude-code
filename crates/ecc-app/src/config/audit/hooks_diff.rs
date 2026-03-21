use ecc_domain::config::audit::{
    ArtifactAudit, HookDiffEntry, HooksDiff, exists_in_settings_typed, exists_in_source_typed,
    is_ecc_managed_hook_typed,
};
use ecc_domain::config::hook_types::HooksMap;
use ecc_ports::fs::FileSystem;
use std::path::Path;

use super::read_json_safe;

/// Read and deserialize hooks from a settings.json file into a typed HooksMap.
fn read_hooks_from_settings(fs: &dyn FileSystem, settings_path: &Path) -> HooksMap {
    read_json_safe(fs, settings_path)
        .ok()
        .flatten()
        .and_then(|s| s.get("hooks").cloned())
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// Read and deserialize hooks from a hooks.json source file into a typed HooksMap.
fn read_hooks_from_source(fs: &dyn FileSystem, hooks_json_path: &Path) -> HooksMap {
    read_json_safe(fs, hooks_json_path)
        .ok()
        .flatten()
        .and_then(|s| s.get("hooks").cloned())
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// Compare hooks in settings.json against the source hooks.json.
pub fn diff_hooks(fs: &dyn FileSystem, settings_path: &Path, hooks_json_path: &Path) -> HooksDiff {
    let settings_hooks = read_hooks_from_settings(fs, settings_path);
    let source_hooks = read_hooks_from_source(fs, hooks_json_path);

    let mut stale = Vec::new();
    let mut matching = Vec::new();
    let mut user_hooks = Vec::new();

    for (event, entries) in &settings_hooks {
        for entry in entries {
            // Serialize back to Value for HookDiffEntry (still uses Value in struct)
            let entry_value = serde_json::to_value(entry).unwrap_or_default();

            if is_ecc_managed_hook_typed(entry, &source_hooks) {
                if exists_in_source_typed(event, entry, &source_hooks) {
                    matching.push(HookDiffEntry {
                        event: event.clone(),
                        entry: entry_value,
                    });
                } else {
                    stale.push(HookDiffEntry {
                        event: event.clone(),
                        entry: entry_value,
                    });
                }
            } else {
                user_hooks.push(HookDiffEntry {
                    event: event.clone(),
                    entry: entry_value,
                });
            }
        }
    }

    let mut missing = Vec::new();
    for (event, entries) in &source_hooks {
        for entry in entries {
            if !exists_in_settings_typed(event, entry, &settings_hooks) {
                let entry_value = serde_json::to_value(entry).unwrap_or_default();
                missing.push(HookDiffEntry {
                    event: event.clone(),
                    entry: entry_value,
                });
            }
        }
    }

    HooksDiff {
        stale,
        missing,
        matching,
        user_hooks,
    }
}

/// Compare files in a source directory against an installed directory.
pub fn audit_artifact_dir(
    fs: &dyn FileSystem,
    src_dir: &Path,
    dest_dir: &Path,
    ext: &str,
) -> ArtifactAudit {
    let mut matching = Vec::new();
    let mut outdated = Vec::new();
    let mut missing = Vec::new();

    if !fs.exists(src_dir) {
        return ArtifactAudit {
            matching,
            outdated,
            missing,
        };
    }

    let entries = match fs.read_dir(src_dir) {
        Ok(e) => e,
        Err(_) => {
            return ArtifactAudit {
                matching,
                outdated,
                missing,
            };
        }
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
            missing.push(filename);
        } else {
            let src_content = fs.read_to_string(&src_path).unwrap_or_default();
            let dest_content = fs.read_to_string(&dest_path).unwrap_or_default();
            if src_content.trim() != dest_content.trim() {
                outdated.push(filename);
            } else {
                matching.push(filename);
            }
        }
    }

    ArtifactAudit {
        matching,
        outdated,
        missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::audit::run_all_checks;
    use ecc_test_support::InMemoryFileSystem;
    use std::path::Path;

    // --- audit_artifact_dir ---

    #[test]
    fn audit_artifact_dir_matching_files() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/a.md", "content a")
            .with_file("/dest/a.md", "content a");

        let result = audit_artifact_dir(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert_eq!(result.matching, vec!["a.md"]);
        assert!(result.outdated.is_empty());
        assert!(result.missing.is_empty());
    }

    #[test]
    fn audit_artifact_dir_outdated_files() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/a.md", "new content")
            .with_file("/dest/a.md", "old content");

        let result = audit_artifact_dir(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert!(result.matching.is_empty());
        assert_eq!(result.outdated, vec!["a.md"]);
    }

    #[test]
    fn audit_artifact_dir_missing_files() {
        let fs = InMemoryFileSystem::new().with_file("/src/a.md", "content");

        let result = audit_artifact_dir(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert!(result.matching.is_empty());
        assert_eq!(result.missing, vec!["a.md"]);
    }

    #[test]
    fn audit_artifact_dir_no_src() {
        let fs = InMemoryFileSystem::new();
        let result = audit_artifact_dir(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert!(result.matching.is_empty());
        assert!(result.outdated.is_empty());
        assert!(result.missing.is_empty());
    }

    #[test]
    fn audit_artifact_dir_filters_by_ext() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/a.md", "content")
            .with_file("/src/b.txt", "content");

        let result = audit_artifact_dir(&fs, Path::new("/src"), Path::new("/dest"), ".md");
        assert_eq!(result.missing, vec!["a.md"]);
    }

    // --- diff_hooks ---

    #[test]
    fn diff_hooks_matching() {
        let settings = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [{"command": "ecc-hook format"}]}
                ]
            }
        });
        let source = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [{"command": "ecc-hook format"}]}
                ]
            }
        });
        let fs = InMemoryFileSystem::new()
            .with_file("/settings.json", &settings.to_string())
            .with_file("/hooks.json", &source.to_string());

        let diff = diff_hooks(&fs, Path::new("/settings.json"), Path::new("/hooks.json"));
        assert_eq!(diff.matching.len(), 1);
        assert!(diff.stale.is_empty());
        assert!(diff.missing.is_empty());
    }

    #[test]
    fn diff_hooks_missing_in_settings() {
        let settings = serde_json::json!({"hooks": {}});
        let source = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [{"command": "ecc-hook format"}]}
                ]
            }
        });
        let fs = InMemoryFileSystem::new()
            .with_file("/settings.json", &settings.to_string())
            .with_file("/hooks.json", &source.to_string());

        let diff = diff_hooks(&fs, Path::new("/settings.json"), Path::new("/hooks.json"));
        assert_eq!(diff.missing.len(), 1);
        assert!(diff.matching.is_empty());
    }

    #[test]
    fn diff_hooks_stale_in_settings() {
        let settings = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [{"command": "ecc-hook old-format"}]}
                ]
            }
        });
        let source = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [{"command": "ecc-hook new-format"}]}
                ]
            }
        });
        let fs = InMemoryFileSystem::new()
            .with_file("/settings.json", &settings.to_string())
            .with_file("/hooks.json", &source.to_string());

        let diff = diff_hooks(&fs, Path::new("/settings.json"), Path::new("/hooks.json"));
        assert_eq!(diff.stale.len(), 1);
        assert_eq!(diff.missing.len(), 1);
    }

    #[test]
    fn diff_hooks_user_hooks_preserved() {
        let settings = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"hooks": [{"command": "my-custom-hook"}]}
                ]
            }
        });
        let source = serde_json::json!({"hooks": {}});
        let fs = InMemoryFileSystem::new()
            .with_file("/settings.json", &settings.to_string())
            .with_file("/hooks.json", &source.to_string());

        let diff = diff_hooks(&fs, Path::new("/settings.json"), Path::new("/hooks.json"));
        assert_eq!(diff.user_hooks.len(), 1);
    }

    // --- run_all_checks integration ---

    #[test]
    fn run_all_checks_clean_setup() {
        let deny_array: Vec<serde_json::Value> = ecc_domain::config::deny_rules::ECC_DENY_RULES
            .iter()
            .map(|r| serde_json::Value::String(r.to_string()))
            .collect();
        let settings = serde_json::json!({
            "permissions": { "deny": deny_array },
            "hooks": {}
        });
        let gitignore_content: String = ecc_domain::config::gitignore::ECC_GITIGNORE_ENTRIES
            .iter()
            .map(|e| e.pattern)
            .collect::<Vec<_>>()
            .join("\n");

        let fs = InMemoryFileSystem::new()
            .with_file("/claude/settings.json", &settings.to_string())
            .with_file("/claude/CLAUDE.md", "# Global\nShort file\n")
            .with_file("/project/.gitignore", &gitignore_content)
            .with_file("/project/CLAUDE.md", "# Project\nSmall file\n");

        let report = run_all_checks(
            &fs,
            Path::new("/claude"),
            Path::new("/project"),
            Path::new("/ecc"),
        );

        assert_eq!(report.checks.len(), 8);
        assert!(report.score >= 90);
        assert_eq!(report.grade, "A");
    }

    #[test]
    fn run_all_checks_empty_setup() {
        let fs = InMemoryFileSystem::new();

        let report = run_all_checks(
            &fs,
            Path::new("/claude"),
            Path::new("/project"),
            Path::new("/ecc"),
        );

        assert_eq!(report.checks.len(), 8);
        let total_findings: usize = report.checks.iter().map(|c| c.findings.len()).sum();
        assert!(total_findings >= 2);
        assert!(report.score < 90);
    }
}
