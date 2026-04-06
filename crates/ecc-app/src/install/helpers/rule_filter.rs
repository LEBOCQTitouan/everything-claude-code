//! Rule filtering by stack applicability.
//!
//! Reads rule files, parses their `applies-to` frontmatter, and classifies
//! each rule as included or skipped based on the detected project stack.

use ecc_domain::config::applies_to::{DetectedStack, evaluate_applicability, parse_applies_to};
use ecc_domain::config::validate::extract_frontmatter;
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// Result of filtering a set of rule files against the detected stack.
#[derive(Debug, Clone, Default)]
pub struct RuleFilterResult {
    /// Rule file paths that should be installed.
    pub included: Vec<std::path::PathBuf>,
    /// Rule file paths that were skipped (stack mismatch).
    pub skipped: Vec<std::path::PathBuf>,
}

/// Filter rules from the given rule groups by stack applicability.
///
/// For each `.md` file within each group directory in `rules_dir`, reads
/// the file's frontmatter and evaluates `applies-to` against the detected
/// stack.
///
/// Rules without `applies-to` are always included (backwards compatible).
/// Unreadable files are skipped with a warning (fail-open).
pub fn filter_rules_by_stack(
    fs: &dyn FileSystem,
    rules_dir: &Path,
    groups: &[String],
    stack: &DetectedStack,
) -> RuleFilterResult {
    let mut result = RuleFilterResult::default();

    for group in groups {
        let group_dir = rules_dir.join(group);
        let entries = match fs.read_dir(&group_dir) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(
                    "Cannot read rule group directory {}: {}",
                    group_dir.display(),
                    e
                );
                continue;
            }
        };

        let mut md_files: Vec<_> = entries
            .into_iter()
            .filter(|p| p.to_string_lossy().ends_with(".md"))
            .collect();
        md_files.sort();

        for file_path in md_files {
            let content = match fs.read_to_string(&file_path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("Cannot read rule file {}: {}", file_path.display(), e);
                    // Fail-open: treat unreadable files as included
                    result.included.push(file_path);
                    continue;
                }
            };

            let applies_to = extract_frontmatter(&content)
                .as_ref()
                .and_then(parse_applies_to);

            if evaluate_applicability(&applies_to, stack) {
                result.included.push(file_path);
            } else {
                result.skipped.push(file_path);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;

    fn rust_stack() -> DetectedStack {
        DetectedStack {
            languages: vec!["rust".to_string()],
            frameworks: vec![],
            files: vec![],
        }
    }

    fn empty_stack() -> DetectedStack {
        DetectedStack::default()
    }

    #[test]
    fn filter_keeps_universal_rules_no_applies_to() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/rules/common")
            .with_file(
                "/ecc/rules/common/coding-style.md",
                "---\nname: coding-style\n---\n# Rule",
            );

        let groups = vec!["common".to_string()];
        let result = filter_rules_by_stack(&fs, Path::new("/ecc/rules"), &groups, &rust_stack());

        assert_eq!(result.included.len(), 1);
        assert_eq!(result.skipped.len(), 0);
    }

    #[test]
    fn filter_keeps_matching_language_rule() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/rules/rust")
            .with_file(
                "/ecc/rules/rust/patterns.md",
                "---\nname: patterns\napplies-to: { languages: [rust] }\n---\n# Rust patterns",
            );

        let groups = vec!["rust".to_string()];
        let result = filter_rules_by_stack(&fs, Path::new("/ecc/rules"), &groups, &rust_stack());

        assert_eq!(result.included.len(), 1);
        assert_eq!(result.skipped.len(), 0);
    }

    #[test]
    fn filter_skips_non_matching_language_rule() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/rules/python")
            .with_file(
                "/ecc/rules/python/patterns.md",
                "---\nname: patterns\napplies-to: { languages: [python] }\n---\n# Python patterns",
            );

        let groups = vec!["python".to_string()];
        let result = filter_rules_by_stack(&fs, Path::new("/ecc/rules"), &groups, &rust_stack());

        assert_eq!(result.included.len(), 0);
        assert_eq!(result.skipped.len(), 1);
    }

    #[test]
    fn filter_empty_stack_keeps_all_rules_universal_only() {
        // A rule WITH applies-to on empty stack should be skipped
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/rules/rust")
            .with_file(
                "/ecc/rules/rust/patterns.md",
                "---\napplies-to: { languages: [rust] }\n---\n# Rust",
            )
            .with_dir("/ecc/rules/common")
            .with_file(
                "/ecc/rules/common/coding-style.md",
                "---\nname: coding-style\n---\n# Common",
            );

        let groups = vec!["rust".to_string(), "common".to_string()];
        let result = filter_rules_by_stack(&fs, Path::new("/ecc/rules"), &groups, &empty_stack());

        // common (no applies-to) included, rust (applies-to rust but stack empty) skipped
        assert_eq!(result.included.len(), 1);
        assert_eq!(result.skipped.len(), 1);
    }

    #[test]
    fn filter_empty_applies_to_braces_included_always() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/rules/common")
            .with_file(
                "/ecc/rules/common/universal.md",
                "---\napplies-to: {}\n---\n# Universal",
            );

        let groups = vec!["common".to_string()];
        let result = filter_rules_by_stack(&fs, Path::new("/ecc/rules"), &groups, &empty_stack());

        assert_eq!(result.included.len(), 1);
        assert_eq!(result.skipped.len(), 0);
    }

    #[test]
    fn filter_missing_group_dir_does_not_crash() {
        let fs = InMemoryFileSystem::new().with_dir("/ecc/rules");
        let groups = vec!["nonexistent".to_string()];
        let result = filter_rules_by_stack(&fs, Path::new("/ecc/rules"), &groups, &rust_stack());
        assert_eq!(result.included.len(), 0);
        assert_eq!(result.skipped.len(), 0);
    }

    #[test]
    fn filter_mixed_group_some_match_some_not() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/ecc/rules/mixed")
            .with_file(
                "/ecc/rules/mixed/rust-rule.md",
                "---\napplies-to: { languages: [rust] }\n---\n# Rust",
            )
            .with_file(
                "/ecc/rules/mixed/python-rule.md",
                "---\napplies-to: { languages: [python] }\n---\n# Python",
            )
            .with_file(
                "/ecc/rules/mixed/common-rule.md",
                "---\nname: common\n---\n# Common",
            );

        let groups = vec!["mixed".to_string()];
        let result = filter_rules_by_stack(&fs, Path::new("/ecc/rules"), &groups, &rust_stack());

        // rust-rule + common-rule included, python-rule skipped
        assert_eq!(result.included.len(), 2);
        assert_eq!(result.skipped.len(), 1);
    }
}
