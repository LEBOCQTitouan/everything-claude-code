//! Self-identity resolver — determines the current Claude Code session's
//! worktree name by inspecting `CLAUDE_PROJECT_DIR` and reading the `.git`
//! file to extract the `gitdir:` pointer.
//!
//! # Security (SEC-002)
//!
//! The gitdir path embedded in the `.git` file is canonicalized via
//! `fs.canonicalize` before parsing the worktree name. This prevents
//! symlink-based mis-attribution attacks where a crafted symlink could cause
//! the resolver to return the wrong worktree name.

use ecc_domain::worktree::WorktreeName;
use ecc_ports::fs::FileSystem;
use std::path::{Path, PathBuf};

/// Resolve the current session's [`WorktreeName`] from `CLAUDE_PROJECT_DIR`.
///
/// # Algorithm
///
/// 1. If `claude_project_dir` is `None` → return `None` (env var missing).
/// 2. Canonicalize the path via `fs.canonicalize`. If that fails → return `None`.
/// 3. Inspect `<canonical>/.git`:
///    - If `.git` is a **directory** → this is the main repo → return `None` (AC-003.5).
///    - If `.git` is a **file** → read its content; parse `gitdir: <abs-path>`.
/// 4. Canonicalize the gitdir path (SEC-002). If that fails → return `None`.
/// 5. Expect the canonical gitdir to end with `.git/worktrees/<name>`. Parse `<name>`.
/// 6. Return `WorktreeName::parse(&name)` wrapped in `Some`, or `None` on failure.
///
/// Whitespace and CRLF in the `gitdir:` line are tolerated (PC-076).
///
/// # Arguments
///
/// - `claude_project_dir`: value of `CLAUDE_PROJECT_DIR` env var, passed in by
///   the caller so this function stays I/O-free except for the `fs` port.
/// - `fs`: filesystem port — used for canonicalize, is_dir, and read_to_string.
pub fn current_worktree(
    claude_project_dir: Option<&Path>,
    fs: &dyn FileSystem,
) -> Option<WorktreeName> {
    let project_dir = claude_project_dir?;

    // Step 2: canonicalize the project dir to defend against symlink attacks.
    let canonical = fs.canonicalize(project_dir).ok()?;

    // Step 3: inspect .git.
    let dot_git = canonical.join(".git");
    if fs.is_dir(&dot_git) {
        // Main repo — not a worktree.
        return None;
    }

    // .git must be a regular file containing "gitdir: <path>".
    let content = fs.read_to_string(&dot_git).ok()?;
    let gitdir_path = parse_gitdir_line(&content)?;

    // Step 4: canonicalize the gitdir path (SEC-002).
    let canonical_gitdir = fs.canonicalize(&gitdir_path).ok()?;

    // Step 5: expect the path to end with `.git/worktrees/<name>`.
    extract_worktree_name(&canonical_gitdir)
}

/// Parse the `gitdir: <path>` line from a `.git` file's content.
/// Tolerates leading/trailing whitespace and CRLF (PC-076).
fn parse_gitdir_line(content: &str) -> Option<PathBuf> {
    for line in content.lines() {
        let line = line.trim_end_matches('\r').trim();
        if let Some(rest) = line.strip_prefix("gitdir:") {
            let path_str = rest.trim();
            if !path_str.is_empty() {
                return Some(PathBuf::from(path_str));
            }
        }
    }
    None
}

/// Extract the worktree name from a canonical gitdir path.
///
/// Expected form: `.../.git/worktrees/<name>`
fn extract_worktree_name(canonical_gitdir: &Path) -> Option<WorktreeName> {
    // The path must end with .git/worktrees/<name>.
    // Components from the end: <name>, worktrees, .git
    let mut components = canonical_gitdir.components().rev();
    let name = components.next()?.as_os_str().to_str()?;
    let worktrees = components.next()?.as_os_str().to_str()?;
    let dot_git = components.next()?.as_os_str().to_str()?;

    if worktrees != "worktrees" || dot_git != ".git" {
        return None;
    }

    // Verify it's a valid session worktree name (has the expected parse structure).
    WorktreeName::parse(name)?;
    // Construct the WorktreeName value object.
    WorktreeName::new(name).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;
    use std::path::PathBuf;

    // ── PC-036: reads_claude_project_dir ────────────────────────────────────

    /// AC-003.1: current_worktree() returns Some(WorktreeName) when in a worktree.
    #[test]
    fn reads_claude_project_dir() {
        // Set up a fake worktree: /worktrees/ecc-session-20260101-000000-feat-12345
        // whose .git file points to /repo/.git/worktrees/ecc-session-20260101-000000-feat-12345
        let wt_name = "ecc-session-20260101-000000-feat-12345";
        let wt_path = format!("/worktrees/{wt_name}");
        let gitdir_path = format!("/repo/.git/worktrees/{wt_name}");
        let dot_git_content = format!("gitdir: {gitdir_path}\n");

        let fs = InMemoryFileSystem::new()
            .with_dir(&wt_path)
            .with_file(&format!("{wt_path}/.git"), &dot_git_content)
            // Register the gitdir path so canonicalize succeeds.
            .with_dir(&gitdir_path);

        let project_dir = PathBuf::from(&wt_path);
        let result = current_worktree(Some(&project_dir), &fs);

        assert!(
            result.is_some(),
            "current_worktree must return Some when in a worktree with valid .git file, got None"
        );
        let wt = result.unwrap();
        assert_eq!(
            wt.as_str(),
            wt_name,
            "returned WorktreeName must match the directory name"
        );
    }

    // ── PC-040: returns_none_for_main_repo ───────────────────────────────────

    /// AC-003.5: current_worktree() returns None when .git is a directory (main repo).
    #[test]
    fn returns_none_for_main_repo() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/repo")
            .with_dir("/repo/.git"); // .git is a directory → main repo

        let result = current_worktree(Some(Path::new("/repo")), &fs);

        assert!(
            result.is_none(),
            "current_worktree must return None for main repo (when .git is a directory)"
        );
    }

    // ── PC-040b: canonicalizes_gitdir_path_against_symlinks (SEC-002) ────────

    /// SEC-002: current_worktree() canonicalizes the gitdir path.
    /// When canonicalize fails (path not registered in in-memory FS), return None.
    #[test]
    fn canonicalizes_gitdir_path_against_symlinks() {
        let wt_name = "ecc-session-20260101-000000-sec-99999";
        let wt_path = format!("/worktrees/{wt_name}");
        // The gitdir path is NOT registered in the FS → canonicalize will fail → None.
        let unregistered_gitdir = "/nonexistent/.git/worktrees/ecc-session-20260101-000000-sec-99999";
        let dot_git_content = format!("gitdir: {unregistered_gitdir}\n");

        let fs = InMemoryFileSystem::new()
            .with_dir(&wt_path)
            .with_file(&format!("{wt_path}/.git"), &dot_git_content);
        // gitdir path is NOT registered → canonicalize returns NotFound → return None

        let result = current_worktree(Some(&PathBuf::from(&wt_path)), &fs);

        assert!(
            result.is_none(),
            "current_worktree must return None when gitdir canonicalize fails (SEC-002)"
        );
    }

    // ── PC-040b supplemental: gitdir_parser_strips_whitespace (PC-076) ───────

    /// PC-076: Parser tolerates trailing whitespace and CRLF in gitdir line.
    #[test]
    fn gitdir_parser_strips_whitespace() {
        let wt_name = "ecc-session-20260101-000000-ws-11111";
        let gitdir_path = format!("/repo/.git/worktrees/{wt_name}");
        // CRLF line ending and trailing space after path
        let dot_git_content = format!("gitdir: {gitdir_path}  \r\n");
        let wt_path = "/worktrees/ws-wt";

        let fs = InMemoryFileSystem::new()
            .with_dir(wt_path)
            .with_file(&format!("{wt_path}/.git"), &dot_git_content)
            .with_dir(&gitdir_path);

        let result = current_worktree(Some(Path::new(wt_path)), &fs);

        assert!(
            result.is_some(),
            "parser must tolerate CRLF and trailing whitespace, got None"
        );
    }

    // ── PC-036 supplemental: missing env var → None ──────────────────────────

    #[test]
    fn returns_none_when_no_project_dir() {
        let fs = InMemoryFileSystem::new();
        let result = current_worktree(None, &fs);
        assert!(result.is_none(), "None project_dir must return None");
    }
}
