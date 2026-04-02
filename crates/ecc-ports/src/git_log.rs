//! Port for reading git log data.
//!
//! Abstracts git log operations so that domain and app layers
//! never construct git CLI arguments directly.

use std::path::Path;

/// Errors from git log operations.
#[derive(Debug, thiserror::Error)]
pub enum GitLogError {
    /// The `git` binary was not found on PATH.
    #[error("git not found on PATH. Install git to use analyze commands.")]
    GitNotFound,
    /// The target directory is not a git repository.
    #[error("not a git repository: {0}")]
    NotARepo(String),
    /// The `--since` value is invalid (unrecognised tag, bad date format).
    #[error("invalid --since value: {0}")]
    InvalidSince(String),
    /// A git command failed with an error message.
    #[error("git command failed: {0}")]
    CommandFailed(String),
}

/// A raw commit record as returned by the port.
#[derive(Debug, Clone)]
pub struct RawCommit {
    /// Full commit hash.
    pub hash: String,
    /// Author name.
    pub author: String,
    /// Commit subject line (first line of message).
    pub message: String,
    /// Files changed in this commit.
    pub files: Vec<String>,
}

/// Port for reading git log data.
///
/// All methods use `--no-merges` by default.
/// The `since` parameter accepts git-compatible values: tags (`v1.0.0`),
/// dates (`2024-01-01`), or relative durations (`90.days.ago`).
pub trait GitLogPort: Send + Sync {
    /// Fetch commits with their changed files.
    fn log_with_files(
        &self,
        repo_dir: &Path,
        since: Option<&str>,
    ) -> Result<Vec<RawCommit>, GitLogError>;

    /// Fetch `(file_path, author)` tuples for bus factor analysis.
    fn log_file_authors(
        &self,
        repo_dir: &Path,
        since: Option<&str>,
    ) -> Result<Vec<(String, String)>, GitLogError>;
}
