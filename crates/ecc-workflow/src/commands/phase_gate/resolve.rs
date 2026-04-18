//! Worktree state directory resolution for the phase-gate hook.

/// Maximum bytes to read from a `.git` file when detecting worktree gitdir.
pub(super) const GIT_FILE_MAX_BYTES: usize = 4096;

/// Maximum parent directory traversal depth for worktree detection.
pub(super) const WORKTREE_DEPTH_LIMIT: usize = 50;

/// Derive the worktree-scoped state directory from a gated file path.
///
/// When Claude Code's hook subprocess sets `CLAUDE_PROJECT_DIR` to the main repo
/// root, `resolve_state_dir()` reads the wrong `.state-dir` anchor. This function
/// bypasses that by walking the gated file path's parents to find the worktree's
/// `.git` file, then reading its `gitdir:` line to find the correct git-dir.
///
/// Returns `Some(state_dir)` if the file is inside a worktree checkout.
/// Returns `None` if the file is in a main repo, not absolute, or detection fails.
pub(super) fn resolve_worktree_state_dir(file_path: &str) -> Option<std::path::PathBuf> {
    let path = std::path::Path::new(file_path);
    if !path.is_absolute() {
        return None;
    }

    let mut current = path.parent()?;
    for _ in 0..WORKTREE_DEPTH_LIMIT {
        let git_entry = current.join(".git");
        if git_entry.exists() {
            if git_entry.is_file() {
                // Worktree: .git is a file containing "gitdir: <path>"
                use std::io::Read;
                let file = std::fs::File::open(&git_entry).ok()?;
                let mut content = String::new();
                file.take(GIT_FILE_MAX_BYTES as u64 + 1)
                    .read_to_string(&mut content)
                    .ok()?;
                if content.len() > GIT_FILE_MAX_BYTES {
                    return None;
                }
                let gitdir_line = content.lines().find(|l| l.starts_with("gitdir:"))?;
                let raw_path = gitdir_line.strip_prefix("gitdir:")?.trim();
                let gitdir = if std::path::Path::new(raw_path).is_absolute() {
                    std::path::PathBuf::from(raw_path)
                } else {
                    current.join(raw_path)
                };
                return Some(gitdir.join("ecc-workflow"));
            }
            // .git is a directory — main repo checkout, not a worktree
            return None;
        }
        current = current.parent()?;
    }
    None
}
