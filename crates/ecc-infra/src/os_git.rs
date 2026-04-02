//! Production adapter for [`ecc_ports::git::GitInfo`].
//!
//! Uses `git rev-parse` to query repository information.

use ecc_ports::git::{GitError, GitInfo};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Git info adapter that shells out to the `git` binary.
pub struct OsGitInfo;

impl GitInfo for OsGitInfo {
    fn git_dir(&self, working_dir: &Path) -> Result<PathBuf, GitError> {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--git-dir")
            .current_dir(working_dir)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("not a git repository") {
                return Err(GitError::NotARepo);
            }
            return Err(GitError::CommandFailed(stderr.into_owned()));
        }

        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        let path = PathBuf::from(&path_str);

        // git rev-parse --git-dir returns a relative path when inside the repo.
        // Make it absolute relative to working_dir.
        if path.is_relative() {
            Ok(working_dir.join(path))
        } else {
            Ok(path)
        }
    }

}
