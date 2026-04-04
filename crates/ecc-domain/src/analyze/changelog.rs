//! Changelog generation from conventional commits.
//!
//! Groups commits by type and formats as markdown.

use std::collections::BTreeMap;

use super::commit::{CommitType, ConventionalCommit};

/// Changelog section ordering — features first, then fixes, etc.
const SECTION_ORDER: &[CommitType] = &[
    CommitType::Feat,
    CommitType::Fix,
    CommitType::Refactor,
    CommitType::Perf,
    CommitType::Docs,
    CommitType::Chore,
    CommitType::Ci,
    CommitType::Test,
    CommitType::Style,
    CommitType::Build,
];

/// Group commits by their type.
pub fn group_by_type(commits: &[ConventionalCommit]) -> BTreeMap<String, Vec<&ConventionalCommit>> {
    let mut groups: BTreeMap<String, Vec<&ConventionalCommit>> = BTreeMap::new();
    for commit in commits {
        let key = commit.commit_type.label().to_string();
        groups.entry(key).or_default().push(commit);
    }
    groups
}

/// Format grouped commits into a markdown changelog.
///
/// If `fallback_header` is `Some`, it is included as a note at the top
/// (e.g., "Showing commits from the last 90 days (no semver tags found)").
pub fn format_changelog(commits: &[ConventionalCommit], fallback_header: Option<&str>) -> String {
    if commits.is_empty() {
        return String::from("No commits found in the specified range.\n");
    }

    let mut output = String::new();

    if let Some(header) = fallback_header {
        output.push_str(&format!("> {header}\n\n"));
    }

    // Collect breaking changes
    let breaking: Vec<&ConventionalCommit> = commits.iter().filter(|c| c.breaking).collect();
    if !breaking.is_empty() {
        output.push_str("## BREAKING CHANGES\n\n");
        for commit in &breaking {
            output.push_str(&format_commit_line(commit));
        }
        output.push('\n');
    }

    // Group by type
    let groups = group_by_type(commits);

    // Output in defined order
    for ordered_type in SECTION_ORDER {
        let label = ordered_type.label();
        if let Some(group_commits) = groups.get(label) {
            output.push_str(&format!("## {label}\n\n"));
            for commit in group_commits {
                output.push_str(&format_commit_line(commit));
            }
            output.push('\n');
        }
    }

    // "Other" section for non-conventional / unknown types
    let known_labels: Vec<&str> = SECTION_ORDER.iter().map(|t| t.label()).collect();
    for (label, group_commits) in &groups {
        if !known_labels.contains(&label.as_str()) {
            output.push_str(&format!("## {label}\n\n"));
            for commit in group_commits {
                output.push_str(&format_commit_line(commit));
            }
            output.push('\n');
        }
    }

    output
}

fn format_commit_line(commit: &ConventionalCommit) -> String {
    let hash_short = if commit.hash.len() >= 7 {
        &commit.hash[..7]
    } else {
        &commit.hash
    };

    if let Some(scope) = &commit.scope {
        format!("- **{scope}**: {} ({hash_short})\n", commit.description)
    } else {
        format!("- {} ({hash_short})\n", commit.description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyze::commit::CommitType;

    fn make_commit(
        commit_type: CommitType,
        scope: Option<&str>,
        desc: &str,
        hash: &str,
        breaking: bool,
    ) -> ConventionalCommit {
        ConventionalCommit {
            commit_type,
            scope: scope.map(String::from),
            breaking,
            description: desc.to_string(),
            hash: hash.to_string(),
            author: "alice".to_string(),
        }
    }

    // PC-013: group commits by type
    #[test]
    fn groups_commits_by_type() {
        let commits = vec![
            make_commit(CommitType::Feat, None, "add foo", "abc1234", false),
            make_commit(CommitType::Fix, None, "fix bar", "def5678", false),
            make_commit(CommitType::Feat, None, "add baz", "ghi9012", false),
        ];
        let groups = group_by_type(&commits);
        assert_eq!(groups.get("Features").unwrap().len(), 2);
        assert_eq!(groups.get("Bug Fixes").unwrap().len(), 1);
    }

    // PC-014: format changelog markdown
    #[test]
    fn format_changelog_markdown() {
        let commits = vec![
            make_commit(CommitType::Feat, None, "add foo", "abc1234", false),
            make_commit(CommitType::Fix, None, "fix bar", "def5678", false),
        ];
        let output = format_changelog(&commits, None);
        assert!(output.contains("## Features"));
        assert!(output.contains("## Bug Fixes"));
    }

    // PC-015: empty input
    #[test]
    fn changelog_empty_input() {
        let output = format_changelog(&[], None);
        assert!(output.contains("No commits found"));
    }

    // PC-016: includes hash
    #[test]
    fn changelog_includes_hash() {
        let commits = vec![make_commit(
            CommitType::Feat,
            None,
            "add foo",
            "abc1234def",
            false,
        )];
        let output = format_changelog(&commits, None);
        assert!(output.contains("abc1234"));
    }

    // PC-017: shows scope
    #[test]
    fn changelog_shows_scope() {
        let commits = vec![make_commit(
            CommitType::Feat,
            Some("cli"),
            "add analyze",
            "abc1234",
            false,
        )];
        let output = format_changelog(&commits, None);
        assert!(output.contains("**cli**: add analyze"));
    }

    // PC-018: breaking section
    #[test]
    fn changelog_breaking_section() {
        let commits = vec![make_commit(
            CommitType::Feat,
            None,
            "remove old API",
            "abc1234",
            true,
        )];
        let output = format_changelog(&commits, None);
        assert!(output.contains("## BREAKING CHANGES"));
    }

    // PC-044: Other section for non-conventional
    #[test]
    fn changelog_other_section() {
        let commits = vec![make_commit(
            CommitType::Unknown("wip".to_string()),
            None,
            "work in progress",
            "abc1234",
            false,
        )];
        let output = format_changelog(&commits, None);
        assert!(output.contains("## wip"));
    }

    // PC-045: 90-day fallback header
    #[test]
    fn changelog_fallback_header() {
        let commits = vec![make_commit(
            CommitType::Feat,
            None,
            "add foo",
            "abc1234",
            false,
        )];
        let output = format_changelog(
            &commits,
            Some("Showing commits from the last 90 days (no semver tags found)"),
        );
        assert!(output.contains("last 90 days"));
    }
}
