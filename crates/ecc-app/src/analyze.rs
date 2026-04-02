//! Use cases for deterministic git analytics.
//!
//! Each function takes a `&dyn GitLogPort` + config parameters,
//! calls the port, then delegates to domain logic.

use std::path::Path;

use ecc_domain::analyze::bus_factor::{self, BusFactor};
use ecc_domain::analyze::changelog;
use ecc_domain::analyze::commit::{self, ConventionalCommit};
use ecc_domain::analyze::coupling::{self, CouplingPair};
use ecc_domain::analyze::error::AnalyzeError;
use ecc_domain::analyze::hotspot::{self, Hotspot};
use ecc_ports::git_log::{GitLogError, GitLogPort};

/// Errors from analyze use cases.
#[derive(Debug, thiserror::Error)]
pub enum AnalyzeAppError {
    /// Error from git log operations.
    #[error(transparent)]
    GitLog(#[from] GitLogError),
    /// Error from domain logic.
    #[error(transparent)]
    Domain(#[from] AnalyzeError),
}

/// Generate a markdown changelog from conventional commits.
pub fn generate_changelog(
    port: &dyn GitLogPort,
    repo: &Path,
    since: Option<&str>,
) -> Result<String, AnalyzeAppError> {
    let raw_commits = port.log_with_files(repo, since)?;

    let conventional: Vec<ConventionalCommit> = raw_commits
        .iter()
        .filter_map(|rc| commit::parse_conventional_commit(&rc.hash, &rc.author, &rc.message))
        .collect();

    // Also include non-conventional commits as "Other"
    let all_commits: Vec<ConventionalCommit> = raw_commits
        .iter()
        .map(|rc| {
            commit::parse_conventional_commit(&rc.hash, &rc.author, &rc.message).unwrap_or(
                ConventionalCommit {
                    commit_type: commit::CommitType::Unknown("Other".to_string()),
                    scope: None,
                    breaking: false,
                    description: rc.message.clone(),
                    hash: rc.hash.clone(),
                    author: rc.author.clone(),
                },
            )
        })
        .collect();

    // Determine if we should show fallback header
    let fallback_header = if since.is_none() {
        Some("Showing commits from the last 90 days (no --since specified)")
    } else {
        None
    };

    // Use all commits (including non-conventional as "Other")
    let _ = conventional; // conventional was for filtering, but we want all
    Ok(changelog::format_changelog(&all_commits, fallback_header))
}

/// Compute file change hotspots.
pub fn compute_hotspots(
    port: &dyn GitLogPort,
    repo: &Path,
    since: Option<&str>,
    top_n: usize,
    max_files_per_commit: usize,
) -> Result<Vec<Hotspot>, AnalyzeAppError> {
    let raw_commits = port.log_with_files(repo, since)?;
    let commit_files: Vec<Vec<String>> = raw_commits.into_iter().map(|rc| rc.files).collect();
    Ok(hotspot::compute_hotspots(
        &commit_files,
        top_n,
        max_files_per_commit,
    )?)
}

/// Compute co-change coupling pairs.
pub fn compute_coupling(
    port: &dyn GitLogPort,
    repo: &Path,
    since: Option<&str>,
    threshold: f64,
    min_commits: u32,
    max_files_per_commit: usize,
) -> Result<Vec<CouplingPair>, AnalyzeAppError> {
    let raw_commits = port.log_with_files(repo, since)?;
    let commit_files: Vec<Vec<String>> = raw_commits.into_iter().map(|rc| rc.files).collect();
    Ok(coupling::compute_coupling(
        &commit_files,
        threshold,
        min_commits,
        max_files_per_commit,
    ))
}

/// Compute bus factor per file.
pub fn compute_bus_factor(
    port: &dyn GitLogPort,
    repo: &Path,
    since: Option<&str>,
    top_n: usize,
) -> Result<Vec<BusFactor>, AnalyzeAppError> {
    let file_authors = port.log_file_authors(repo, since)?;
    Ok(bus_factor::compute_bus_factor(&file_authors, top_n)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::git_log::RawCommit;
    use std::path::PathBuf;

    /// Simple mock GitLogPort for testing use cases.
    struct MockGitLog {
        commits: Vec<RawCommit>,
        file_authors: Vec<(String, String)>,
    }

    impl GitLogPort for MockGitLog {
        fn log_with_files(
            &self,
            _repo_dir: &Path,
            _since: Option<&str>,
        ) -> Result<Vec<RawCommit>, GitLogError> {
            Ok(self.commits.clone())
        }

        fn log_file_authors(
            &self,
            _repo_dir: &Path,
            _since: Option<&str>,
        ) -> Result<Vec<(String, String)>, GitLogError> {
            Ok(self.file_authors.clone())
        }
    }

    fn mock_with_commits(commits: Vec<RawCommit>) -> MockGitLog {
        MockGitLog {
            commits,
            file_authors: vec![],
        }
    }

    fn mock_with_authors(file_authors: Vec<(String, String)>) -> MockGitLog {
        MockGitLog {
            commits: vec![],
            file_authors,
        }
    }

    // PC-019: changelog use case
    #[test]
    fn changelog_use_case() {
        let mock = mock_with_commits(vec![
            RawCommit {
                hash: "abc1234".into(),
                author: "alice".into(),
                message: "feat(cli): add analyze".into(),
                files: vec!["a.rs".into()],
            },
            RawCommit {
                hash: "def5678".into(),
                author: "bob".into(),
                message: "fix: handle empty input".into(),
                files: vec!["b.rs".into()],
            },
        ]);
        let result = generate_changelog(&mock, &PathBuf::from("."), Some("v1.0.0")).unwrap();
        assert!(result.contains("## Features"));
        assert!(result.contains("## Bug Fixes"));
    }

    // PC-024: hotspots use case
    #[test]
    fn hotspots_use_case() {
        let mock = mock_with_commits(vec![
            RawCommit {
                hash: "abc".into(),
                author: "alice".into(),
                message: "feat: x".into(),
                files: vec!["a.rs".into(), "b.rs".into()],
            },
            RawCommit {
                hash: "def".into(),
                author: "bob".into(),
                message: "fix: y".into(),
                files: vec!["a.rs".into()],
            },
        ]);
        let result = compute_hotspots(&mock, &PathBuf::from("."), None, 10, 20).unwrap();
        assert_eq!(result[0].path, "a.rs");
        assert_eq!(result[0].change_count, 2);
    }

    // PC-032: coupling use case
    #[test]
    fn coupling_use_case() {
        let mock = mock_with_commits(vec![
            RawCommit {
                hash: "abc".into(),
                author: "alice".into(),
                message: "feat: x".into(),
                files: vec!["a.rs".into(), "b.rs".into()],
            },
            RawCommit {
                hash: "def".into(),
                author: "bob".into(),
                message: "fix: y".into(),
                files: vec!["a.rs".into(), "b.rs".into()],
            },
        ]);
        let result = compute_coupling(&mock, &PathBuf::from("."), None, 0.5, 1, 20).unwrap();
        assert!(!result.is_empty());
        assert!((result[0].coupling_ratio - 1.0).abs() < f64::EPSILON);
    }

    // PC-037: bus_factor use case
    #[test]
    fn bus_factor_use_case() {
        let mock = mock_with_authors(vec![
            ("a.rs".into(), "alice".into()),
            ("a.rs".into(), "alice".into()),
            ("b.rs".into(), "alice".into()),
            ("b.rs".into(), "bob".into()),
        ]);
        let result = compute_bus_factor(&mock, &PathBuf::from("."), None, 10).unwrap();
        assert_eq!(result[0].path, "a.rs");
        assert_eq!(result[0].unique_authors, 1);
        assert!(result[0].is_risk);
    }

    // PC-043: empty log returns empty
    #[test]
    fn empty_log_returns_empty() {
        let mock = mock_with_commits(vec![]);
        let result = compute_hotspots(&mock, &PathBuf::from("."), None, 10, 20).unwrap();
        assert!(result.is_empty());
    }
}
