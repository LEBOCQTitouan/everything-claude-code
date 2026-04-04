use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Manifest filename constant.
pub const MANIFEST_FILENAME: &str = ".ecc-manifest.json";

/// ECC installation manifest — tracks version, languages, and installed artifacts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EccManifest {
    pub version: String,
    pub installed_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub languages: Vec<String>,
    pub artifacts: Artifacts,
}

/// Artifact lists tracked by the manifest.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Artifacts {
    #[serde(default)]
    pub agents: Vec<String>,
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub rules: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub hook_descriptions: Vec<String>,
}

/// Diff between two file lists — files added, updated (in both), and removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestDiff {
    pub added: Vec<String>,
    pub updated: Vec<String>,
    pub removed: Vec<String>,
}

/// Create a fresh manifest for a new installation.
/// `now` is the ISO 8601 timestamp to use.
pub fn create_manifest(
    version: &str,
    now: &str,
    languages: &[String],
    artifacts: Artifacts,
) -> EccManifest {
    EccManifest {
        version: version.to_string(),
        installed_at: now.to_string(),
        updated_at: now.to_string(),
        languages: languages.to_vec(),
        artifacts,
    }
}

/// Update an existing manifest with new data (returns new object, does not mutate).
/// `now` is the ISO 8601 timestamp to use.
pub fn update_manifest(
    existing: &EccManifest,
    version: &str,
    now: &str,
    languages: &[String],
    artifacts: Artifacts,
) -> EccManifest {
    // Merge languages: union of existing + new, preserving order
    let mut merged_languages = existing.languages.clone();
    for lang in languages {
        if !merged_languages.contains(lang) {
            merged_languages.push(lang.clone());
        }
    }

    EccManifest {
        version: version.to_string(),
        installed_at: existing.installed_at.clone(),
        updated_at: now.to_string(),
        languages: merged_languages,
        artifacts,
    }
}

/// Check if a specific artifact is managed by ECC.
pub fn is_ecc_managed(manifest: Option<&EccManifest>, artifact_type: &str, filename: &str) -> bool {
    let manifest = match manifest {
        Some(m) => m,
        None => return false,
    };

    let list = match artifact_type {
        "agents" => &manifest.artifacts.agents,
        "commands" => &manifest.artifacts.commands,
        "skills" => &manifest.artifacts.skills,
        _ => return false,
    };

    list.iter().any(|f| f == filename)
}

/// Check if a rule file is managed by ECC.
pub fn is_ecc_managed_rule(manifest: Option<&EccManifest>, group: &str, filename: &str) -> bool {
    let manifest = match manifest {
        Some(m) => m,
        None => return false,
    };

    match manifest.artifacts.rules.get(group) {
        Some(rules) => rules.iter().any(|f| f == filename),
        None => false,
    }
}

/// Diff two lists of filenames to compute what changed.
pub fn diff_file_list(existing: &[String], incoming: &[String]) -> ManifestDiff {
    use std::collections::HashSet;
    let existing_set: HashSet<&str> = existing.iter().map(String::as_str).collect();
    let incoming_set: HashSet<&str> = incoming.iter().map(String::as_str).collect();

    ManifestDiff {
        added: incoming
            .iter()
            .filter(|f| !existing_set.contains(f.as_str()))
            .cloned()
            .collect(),
        updated: incoming
            .iter()
            .filter(|f| existing_set.contains(f.as_str()))
            .cloned()
            .collect(),
        removed: existing
            .iter()
            .filter(|f| !incoming_set.contains(f.as_str()))
            .cloned()
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // --- create_manifest ---

    #[test]
    fn create_manifest_sets_timestamps() {
        let m = create_manifest("4.0.0", "2026-03-14T00:00:00Z", &[], Artifacts::default());
        assert_eq!(m.version, "4.0.0");
        assert_eq!(m.installed_at, "2026-03-14T00:00:00Z");
        assert_eq!(m.updated_at, "2026-03-14T00:00:00Z");
    }

    #[test]
    fn create_manifest_copies_languages() {
        let langs = vec!["typescript".into(), "rust".into()];
        let m = create_manifest("4.0.0", "now", &langs, Artifacts::default());
        assert_eq!(m.languages, langs);
    }

    #[test]
    fn create_manifest_copies_artifacts() {
        let arts = sample_artifacts();
        let m = create_manifest("4.0.0", "now", &[], arts.clone());
        assert_eq!(m.artifacts, arts);
    }

    // --- update_manifest ---

    #[test]
    fn update_manifest_preserves_installed_at() {
        let original = create_manifest("3.0.0", "2025-01-01T00:00:00Z", &[], Artifacts::default());
        let updated = update_manifest(
            &original,
            "4.0.0",
            "2026-03-14T00:00:00Z",
            &[],
            Artifacts::default(),
        );
        assert_eq!(updated.installed_at, "2025-01-01T00:00:00Z");
        assert_eq!(updated.updated_at, "2026-03-14T00:00:00Z");
    }

    #[test]
    fn update_manifest_merges_languages() {
        let original =
            create_manifest("3.0.0", "now", &["typescript".into()], Artifacts::default());
        let updated = update_manifest(
            &original,
            "4.0.0",
            "now",
            &["rust".into(), "typescript".into()],
            Artifacts::default(),
        );
        assert_eq!(updated.languages, vec!["typescript", "rust"]);
    }

    #[test]
    fn update_manifest_replaces_artifacts() {
        let original = create_manifest("3.0.0", "now", &[], sample_artifacts());
        let new_arts = Artifacts {
            agents: vec!["new-agent.md".into()],
            ..Artifacts::default()
        };
        let updated = update_manifest(&original, "4.0.0", "now", &[], new_arts.clone());
        assert_eq!(updated.artifacts, new_arts);
    }

    // --- is_ecc_managed ---

    #[test]
    fn is_ecc_managed_none_manifest() {
        assert!(!is_ecc_managed(None, "agents", "agent1.md"));
    }

    #[test]
    fn is_ecc_managed_found() {
        let m = create_manifest("4.0.0", "now", &[], sample_artifacts());
        assert!(is_ecc_managed(Some(&m), "agents", "agent1.md"));
    }

    #[test]
    fn is_ecc_managed_not_found() {
        let m = create_manifest("4.0.0", "now", &[], sample_artifacts());
        assert!(!is_ecc_managed(Some(&m), "agents", "nonexistent.md"));
    }

    #[test]
    fn is_ecc_managed_invalid_type() {
        let m = create_manifest("4.0.0", "now", &[], sample_artifacts());
        assert!(!is_ecc_managed(Some(&m), "invalid", "agent1.md"));
    }

    // --- is_ecc_managed_rule ---

    #[test]
    fn is_ecc_managed_rule_found() {
        let m = create_manifest("4.0.0", "now", &[], sample_artifacts());
        assert!(is_ecc_managed_rule(Some(&m), "common", "rule1.md"));
    }

    #[test]
    fn is_ecc_managed_rule_wrong_group() {
        let m = create_manifest("4.0.0", "now", &[], sample_artifacts());
        assert!(!is_ecc_managed_rule(Some(&m), "typescript", "rule1.md"));
    }

    #[test]
    fn is_ecc_managed_rule_none_manifest() {
        assert!(!is_ecc_managed_rule(None, "common", "rule1.md"));
    }

    // --- diff_file_list ---

    #[test]
    fn diff_file_list_all_new() {
        let diff = diff_file_list(&[], &["a.md".into(), "b.md".into()]);
        assert_eq!(diff.added, vec!["a.md", "b.md"]);
        assert!(diff.updated.is_empty());
        assert!(diff.removed.is_empty());
    }

    #[test]
    fn diff_file_list_all_removed() {
        let diff = diff_file_list(&["a.md".into(), "b.md".into()], &[]);
        assert!(diff.added.is_empty());
        assert!(diff.updated.is_empty());
        assert_eq!(diff.removed, vec!["a.md", "b.md"]);
    }

    #[test]
    fn diff_file_list_mixed() {
        let existing = vec!["a.md".into(), "b.md".into(), "c.md".into()];
        let incoming = vec!["b.md".into(), "c.md".into(), "d.md".into()];
        let diff = diff_file_list(&existing, &incoming);
        assert_eq!(diff.added, vec!["d.md"]);
        assert_eq!(diff.updated, vec!["b.md", "c.md"]);
        assert_eq!(diff.removed, vec!["a.md"]);
    }

    #[test]
    fn diff_file_list_identical() {
        let list = vec!["a.md".into(), "b.md".into()];
        let diff = diff_file_list(&list, &list);
        assert!(diff.added.is_empty());
        assert_eq!(diff.updated, vec!["a.md", "b.md"]);
        assert!(diff.removed.is_empty());
    }

    // --- patterns field ---

    #[test]
    fn patterns_field_defaults_empty() {
        // Simulate an old manifest JSON that lacks the "patterns" field
        let json = r#"{
            "version": "3.0.0",
            "installedAt": "2025-01-01T00:00:00Z",
            "updatedAt": "2025-01-01T00:00:00Z",
            "artifacts": {
                "agents": [],
                "commands": [],
                "skills": [],
                "rules": {},
                "hookDescriptions": []
            }
        }"#;
        let manifest: EccManifest = serde_json::from_str(json).expect("should deserialize");
        assert!(
            manifest.artifacts.patterns.is_empty(),
            "patterns should default to empty vec when missing from JSON"
        );
    }

    #[test]
    fn is_ecc_managed_patterns() {
        let artifacts = Artifacts {
            patterns: vec!["creational".into(), "behavioral".into()],
            ..Artifacts::default()
        };
        let m = create_manifest("4.0.0", "now", &[], artifacts);
        assert!(is_ecc_managed(Some(&m), "patterns", "creational"));
        assert!(!is_ecc_managed(Some(&m), "patterns", "nonexistent"));
    }
}
