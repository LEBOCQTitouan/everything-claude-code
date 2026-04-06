//! Project stack detection for install-time rule filtering.
//!
//! Detects which languages and frameworks are present in the project by
//! checking for sentinel marker files using the existing domain detection rules.

use ecc_domain::config::applies_to::DetectedStack;
use ecc_domain::detection::framework::FRAMEWORK_RULES;
use ecc_domain::detection::language::LANGUAGE_RULES;
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// Detect the project stack by scanning for sentinel marker files.
///
/// Scans `project_dir` and one level of immediate subdirectories for marker
/// files from `LANGUAGE_RULES` and `FRAMEWORK_RULES`. Populates `DetectedStack`
/// with deduplicated, sorted lists.
///
/// If detection fails (e.g. permission error), returns an empty `DetectedStack`
/// so callers can fail-open.
pub fn detect_project_stack(fs: &dyn FileSystem, project_dir: &Path) -> DetectedStack {
    let mut languages: Vec<String> = Vec::new();
    let mut frameworks: Vec<String> = Vec::new();
    let mut sentinel_files: Vec<String> = Vec::new();

    // Collect all directories to scan: project root + immediate subdirs (1 level)
    let scan_dirs = collect_scan_dirs(fs, project_dir);

    // Collect sentinel files found at project root (for `files:` condition matching)
    if let Ok(entries) = fs.read_dir(project_dir) {
        for entry in &entries {
            if fs.is_file(entry)
                && let Some(name) = entry.file_name()
            {
                sentinel_files.push(name.to_string_lossy().into_owned());
            }
        }
    }

    // Detect languages
    for rule in LANGUAGE_RULES {
        'marker_loop: for marker in rule.markers {
            for dir in &scan_dirs {
                if fs.exists(&dir.join(marker)) {
                    if !languages.contains(&rule.lang_type.to_string()) {
                        languages.push(rule.lang_type.to_string());
                    }
                    break 'marker_loop;
                }
            }
        }
    }

    // Detect frameworks (marker-based only — package_keys require file content inspection)
    for rule in FRAMEWORK_RULES {
        'marker_loop: for marker in rule.markers {
            for dir in &scan_dirs {
                if fs.exists(&dir.join(marker)) {
                    if !frameworks.contains(&rule.framework.to_string()) {
                        frameworks.push(rule.framework.to_string());
                    }
                    break 'marker_loop;
                }
            }
        }
    }

    languages.sort();
    frameworks.sort();
    sentinel_files.sort();

    DetectedStack {
        languages,
        frameworks,
        files: sentinel_files,
    }
}

/// Collect directories to scan: project root + immediate subdirectories.
fn collect_scan_dirs(fs: &dyn FileSystem, project_dir: &Path) -> Vec<std::path::PathBuf> {
    let mut dirs = vec![project_dir.to_path_buf()];

    if let Ok(entries) = fs.read_dir(project_dir) {
        for entry in entries {
            if fs.is_dir(&entry) {
                dirs.push(entry);
            }
        }
    }

    dirs
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;

    #[test]
    fn detect_rust_project() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/project")
            .with_file("/project/Cargo.toml", "[package]");

        let stack = detect_project_stack(&fs, Path::new("/project"));
        assert!(stack.languages.contains(&"rust".to_string()));
    }

    #[test]
    fn detect_python_project() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/project")
            .with_file("/project/requirements.txt", "django");

        let stack = detect_project_stack(&fs, Path::new("/project"));
        assert!(stack.languages.contains(&"python".to_string()));
    }

    #[test]
    fn detect_python_django_by_manage_py() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/project")
            .with_file("/project/requirements.txt", "django")
            .with_file("/project/manage.py", "#!/usr/bin/env python");

        let stack = detect_project_stack(&fs, Path::new("/project"));
        assert!(stack.languages.contains(&"python".to_string()));
        assert!(stack.frameworks.contains(&"django".to_string()));
    }

    #[test]
    fn detect_monorepo_rust_and_typescript() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/project")
            .with_dir("/project/backend")
            .with_dir("/project/frontend")
            .with_file("/project/backend/Cargo.toml", "[package]")
            .with_file("/project/frontend/tsconfig.json", "{}");

        let stack = detect_project_stack(&fs, Path::new("/project"));
        assert!(stack.languages.contains(&"rust".to_string()));
        assert!(stack.languages.contains(&"typescript".to_string()));
    }

    #[test]
    fn detect_empty_project_returns_empty_stack() {
        let fs = InMemoryFileSystem::new().with_dir("/project");
        let stack = detect_project_stack(&fs, Path::new("/project"));
        assert!(stack.languages.is_empty());
        assert!(stack.frameworks.is_empty());
    }

    #[test]
    fn detect_nonexistent_dir_returns_empty_stack() {
        let fs = InMemoryFileSystem::new();
        let stack = detect_project_stack(&fs, Path::new("/nonexistent"));
        assert!(stack.languages.is_empty());
        assert!(stack.frameworks.is_empty());
    }

    #[test]
    fn detect_collects_sentinel_files_from_root() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/project")
            .with_file("/project/manage.py", "");

        let stack = detect_project_stack(&fs, Path::new("/project"));
        assert!(stack.files.contains(&"manage.py".to_string()));
    }

    #[test]
    fn detect_deduplicates_languages() {
        // Both Cargo.toml and Cargo.lock trigger rust — should only appear once
        let fs = InMemoryFileSystem::new()
            .with_dir("/project")
            .with_file("/project/Cargo.toml", "[package]")
            .with_file("/project/Cargo.lock", "");

        let stack = detect_project_stack(&fs, Path::new("/project"));
        let rust_count = stack.languages.iter().filter(|l| *l == "rust").count();
        assert_eq!(rust_count, 1);
    }

    #[test]
    fn detect_marker_beyond_one_level_not_detected() {
        // Nested 2 levels deep — should NOT detect
        let fs = InMemoryFileSystem::new()
            .with_dir("/project")
            .with_dir("/project/a")
            .with_dir("/project/a/b")
            .with_file("/project/a/b/Cargo.toml", "[package]");

        let stack = detect_project_stack(&fs, Path::new("/project"));
        assert!(!stack.languages.contains(&"rust".to_string()));
    }

    #[test]
    fn detect_results_are_sorted() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/project")
            .with_file("/project/requirements.txt", "")
            .with_file("/project/Cargo.toml", "");

        let stack = detect_project_stack(&fs, Path::new("/project"));
        let sorted = {
            let mut v = stack.languages.clone();
            v.sort();
            v
        };
        assert_eq!(stack.languages, sorted);
    }
}
