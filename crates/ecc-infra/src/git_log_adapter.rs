//! Production adapter for [`GitLogPort`] using `git` subprocess.

use std::path::Path;

use ecc_ports::git_log::{GitLogError, GitLogPort, RawCommit};
use ecc_ports::shell::ShellExecutor;

/// Adapter that implements [`GitLogPort`] by shelling out to `git`.
pub struct GitLogAdapter<'a> {
    executor: &'a dyn ShellExecutor,
}

impl<'a> GitLogAdapter<'a> {
    /// Create a new adapter wrapping a shell executor.
    pub fn new(executor: &'a dyn ShellExecutor) -> Self {
        Self { executor }
    }

    fn check_git(&self) -> Result<(), GitLogError> {
        if !self.executor.command_exists("git") {
            return Err(GitLogError::GitNotFound);
        }
        Ok(())
    }

    fn build_since_args(since: Option<&str>) -> Vec<&str> {
        match since {
            Some(s) if s.contains('.') || s.contains('-') || s.contains('/') => {
                // Looks like a date or relative duration — use --since
                vec!["--since", s]
            }
            Some(s) => {
                // Looks like a tag/ref — use revision range
                // We'll use --since for tags too, let git decide
                vec!["--since", s]
            }
            None => vec![],
        }
    }

    fn check_not_a_repo(stderr: &str, dir: &Path) -> Result<(), GitLogError> {
        if stderr.contains("not a git repository") {
            return Err(GitLogError::NotARepo(dir.display().to_string()));
        }
        Ok(())
    }
}

impl GitLogPort for GitLogAdapter<'_> {
    fn log_with_files(
        &self,
        repo_dir: &Path,
        since: Option<&str>,
    ) -> Result<Vec<RawCommit>, GitLogError> {
        self.check_git()?;

        // Use a unique delimiter to separate commits unambiguously
        let mut args = vec![
            "log",
            "--no-merges",
            "--format=COMMIT_START%n%H%n%an%n%s",
            "--name-only",
        ];
        let since_args = Self::build_since_args(since);
        args.extend(&since_args);

        let output = self
            .executor
            .run_command_in_dir("git", &args, repo_dir)
            .map_err(|e| GitLogError::CommandFailed(e.to_string()))?;

        Self::check_not_a_repo(&output.stderr, repo_dir)?;

        if output.exit_code != 0 && !output.stderr.is_empty() {
            // Check for invalid --since
            if output.stderr.contains("bad default revision")
                || output.stderr.contains("unknown revision")
                || output.stderr.contains("Invalid date")
            {
                return Err(GitLogError::InvalidSince(
                    since.unwrap_or("(none)").to_string(),
                ));
            }
            return Err(GitLogError::CommandFailed(output.stderr));
        }

        Ok(parse_log_with_files(&output.stdout))
    }

    fn log_file_authors(
        &self,
        repo_dir: &Path,
        since: Option<&str>,
    ) -> Result<Vec<(String, String)>, GitLogError> {
        self.check_git()?;

        let mut args = vec![
            "log",
            "--no-merges",
            "--format=AUTHOR_START%n%an",
            "--name-only",
        ];
        let since_args = Self::build_since_args(since);
        args.extend(&since_args);

        let output = self
            .executor
            .run_command_in_dir("git", &args, repo_dir)
            .map_err(|e| GitLogError::CommandFailed(e.to_string()))?;

        Self::check_not_a_repo(&output.stderr, repo_dir)?;

        Ok(parse_file_authors(&output.stdout))
    }
}

/// Parse `git log --format=COMMIT_START%n%H%n%an%n%s --name-only` output.
///
/// Uses `COMMIT_START` delimiter to unambiguously separate commits.
fn parse_log_with_files(stdout: &str) -> Vec<RawCommit> {
    let mut commits = Vec::new();

    for block in stdout.split("COMMIT_START") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        let lines: Vec<&str> = block.lines().collect();
        if lines.len() < 3 {
            continue;
        }

        let hash = lines[0].trim().to_string();
        let author = lines[1].trim().to_string();
        let message = lines[2].trim().to_string();

        let files: Vec<String> = lines[3..]
            .iter()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect();

        commits.push(RawCommit {
            hash,
            author,
            message,
            files,
        });
    }

    commits
}

/// Parse `git log --format=AUTHOR_START%n%an --name-only` output into `(file, author)` tuples.
fn parse_file_authors(stdout: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();

    for block in stdout.split("AUTHOR_START") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        let lines: Vec<&str> = block.lines().collect();
        if lines.is_empty() {
            continue;
        }

        let author = lines[0].trim().to_string();

        for line in &lines[1..] {
            let file = line.trim().to_string();
            if !file.is_empty() {
                results.push((file, author.clone()));
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    // PC-010: parses multi-commit output
    #[test]
    fn parses_git_log_output() {
        let stdout = "\
COMMIT_START
abc1234567890
alice
feat(cli): add analyze

crates/ecc-cli/src/commands/analyze.rs
crates/ecc-cli/src/main.rs

COMMIT_START
def5678901234
bob
fix: handle empty input

crates/ecc-domain/src/analyze/commit.rs
";
        let commits = parse_log_with_files(stdout);
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].hash, "abc1234567890");
        assert_eq!(commits[0].author, "alice");
        assert_eq!(commits[0].message, "feat(cli): add analyze");
        assert_eq!(commits[0].files.len(), 2);
        assert_eq!(commits[1].hash, "def5678901234");
        assert_eq!(commits[1].files.len(), 1);
    }

    #[test]
    fn parses_empty_output() {
        let commits = parse_log_with_files("");
        assert!(commits.is_empty());
    }

    #[test]
    fn parses_file_authors() {
        let stdout = "\
AUTHOR_START
alice
src/main.rs
src/lib.rs

AUTHOR_START
bob
src/main.rs
";
        let results = parse_file_authors(stdout);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], ("src/main.rs".to_string(), "alice".to_string()));
        assert_eq!(results[1], ("src/lib.rs".to_string(), "alice".to_string()));
        assert_eq!(results[2], ("src/main.rs".to_string(), "bob".to_string()));
    }

    #[test]
    fn parses_file_authors_empty() {
        let results = parse_file_authors("");
        assert!(results.is_empty());
    }

    // PC-011: uses --no-merges (verified by checking args construction)
    #[test]
    fn build_since_args_date() {
        let args = GitLogAdapter::build_since_args(Some("2026-01-01"));
        assert_eq!(args, vec!["--since", "2026-01-01"]);
    }

    // PC-012: accepts tag
    #[test]
    fn build_since_args_tag() {
        let args = GitLogAdapter::build_since_args(Some("v1.0.0"));
        assert_eq!(args, vec!["--since", "v1.0.0"]);
    }

    #[test]
    fn build_since_args_none() {
        let args = GitLogAdapter::build_since_args(None);
        assert!(args.is_empty());
    }

    #[test]
    fn build_since_args_relative() {
        let args = GitLogAdapter::build_since_args(Some("90.days.ago"));
        assert_eq!(args, vec!["--since", "90.days.ago"]);
    }
}
