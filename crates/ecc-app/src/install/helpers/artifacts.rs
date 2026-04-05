//! Artifact collection helpers for install operations.

use ecc_domain::config::manifest::Artifacts;
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// Collect rule group directories from ecc_root/rules/, filtered by language.
pub(in crate::install) fn collect_rule_groups(
    fs: &dyn FileSystem,
    ecc_root: &Path,
    languages: &[String],
) -> Vec<String> {
    let rules_dir = ecc_root.join("rules");
    let entries = match fs.read_dir(&rules_dir) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("Cannot read rules directory: {}", e);
            return vec!["common".to_string()];
        }
    };

    let mut groups: Vec<String> = entries
        .iter()
        .filter(|p| fs.is_dir(p))
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .filter(|name| {
            name == "common" || languages.is_empty() || languages.iter().any(|l| l == name)
        })
        .collect();

    groups.sort();
    groups
}

/// Scan claude_dir for currently installed artifacts.
pub(in crate::install) fn collect_installed_artifacts(
    fs: &dyn FileSystem,
    claude_dir: &Path,
) -> Artifacts {
    let agents = list_files_with_ext(fs, &claude_dir.join("agents"), ".md");
    let commands = list_files_with_ext(fs, &claude_dir.join("commands"), ".md");
    let skills = list_dirs(fs, &claude_dir.join("skills"));
    let rules = collect_rules_map(fs, &claude_dir.join("rules"));
    let patterns = list_dirs(fs, &claude_dir.join("patterns"));

    let teams = list_files_with_ext(fs, &claude_dir.join("teams"), ".md");

    Artifacts {
        agents,
        commands,
        skills,
        rules,
        hook_descriptions: vec![],
        patterns,
        teams,
    }
}

fn list_files_with_ext(fs: &dyn FileSystem, dir: &Path, ext: &str) -> Vec<String> {
    let entries = match fs.read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("Cannot list files in {}: {}", dir.display(), e);
            return vec![];
        }
    };
    let mut files: Vec<String> = entries
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
    files.sort();
    files
}

fn list_dirs(fs: &dyn FileSystem, dir: &Path) -> Vec<String> {
    let entries = match fs.read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("Cannot list dirs in {}: {}", dir.display(), e);
            return vec![];
        }
    };
    let mut dirs: Vec<String> = entries
        .iter()
        .filter(|p| fs.is_dir(p))
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    dirs.sort();
    dirs
}

fn collect_rules_map(
    fs: &dyn FileSystem,
    rules_dir: &Path,
) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut map = std::collections::BTreeMap::new();
    let groups = list_dirs(fs, rules_dir);
    for group in groups {
        let files = list_files_with_ext(fs, &rules_dir.join(&group), ".md");
        if !files.is_empty() {
            map.insert(group, files);
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;
    use std::path::Path;

    #[test]
    fn collect_rule_groups_with_languages() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/rules/common")
            .with_dir("/ecc/rules/typescript")
            .with_dir("/ecc/rules/python");

        let groups = collect_rule_groups(&fs, Path::new("/ecc"), &["typescript".to_string()]);
        assert!(groups.contains(&"common".to_string()));
        assert!(groups.contains(&"typescript".to_string()));
        assert!(!groups.contains(&"python".to_string()));
    }

    #[test]
    fn collect_rule_groups_empty_languages() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/rules/common")
            .with_dir("/ecc/rules/typescript");

        let groups = collect_rule_groups(&fs, Path::new("/ecc"), &[]);
        assert!(groups.contains(&"common".to_string()));
        assert!(groups.contains(&"typescript".to_string()));
    }

    #[test]
    fn collect_rule_groups_missing_rules_dir_returns_empty() {
        let fs = InMemoryFileSystem::new().with_dir("/nowhere");
        let groups = collect_rule_groups(&fs, Path::new("/nonexistent"), &[]);
        assert!(groups.is_empty());
    }

    #[test]
    fn collect_artifacts_includes_patterns() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/claude/patterns/testing")
            .with_dir("/claude/patterns/security");

        let artifacts = collect_installed_artifacts(&fs, Path::new("/claude"));
        assert!(
            artifacts.patterns.contains(&"testing".to_string()),
            "expected 'testing' in patterns, got: {:?}",
            artifacts.patterns
        );
        assert!(
            artifacts.patterns.contains(&"security".to_string()),
            "expected 'security' in patterns, got: {:?}",
            artifacts.patterns
        );
    }

    #[test]
    fn collect_artifacts_includes_teams() {
        let fs = InMemoryFileSystem::new()
            .with_file("/claude/teams/implement-team.md", "team content")
            .with_file("/claude/teams/audit-team.md", "team content");

        let artifacts = collect_installed_artifacts(&fs, Path::new("/claude"));
        assert!(
            artifacts.teams.contains(&"implement-team.md".to_string()),
            "expected 'implement-team.md' in teams, got: {:?}",
            artifacts.teams
        );
        assert!(
            artifacts.teams.contains(&"audit-team.md".to_string()),
            "expected 'audit-team.md' in teams, got: {:?}",
            artifacts.teams
        );
    }
}
