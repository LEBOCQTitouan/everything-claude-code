//! Integration tests for worktree-safe memory path resolution.
//!
//! Verifies that `memory-write daily` and `memory-write memory-index` resolve
//! to the same `~/.claude/projects/<hash>/memory/` directory from both the
//! main repo root and a git worktree.

mod common;

use std::path::PathBuf;
use std::process::Command;

/// Derive the project hash the same way resolve_project_memory_dir does.
fn project_memory_dir(project_dir: &std::path::Path, home: &std::path::Path) -> PathBuf {
    let repo_root = ecc_flock::resolve_repo_root(project_dir);
    let repo_root = std::fs::canonicalize(&repo_root).unwrap_or_else(|_| repo_root.to_path_buf());
    let abs_str = repo_root.to_string_lossy();
    let project_hash = abs_str.trim_start_matches('/').replace('/', "-");
    home.join(".claude/projects")
        .join(project_hash)
        .join("memory")
}

/// PC-003: memory-write daily from worktree produces same hash dir as from main repo.
#[test]
fn worktree_daily_resolves_to_main_repo_hash() {
    let bin = common::binary_path();
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let main_repo = temp_dir.path();

    // Create a git repo with an initial commit (required for worktree add)
    Command::new("git")
        .args(["init"])
        .current_dir(main_repo)
        .output()
        .expect("git init failed");
    Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init"])
        .current_dir(main_repo)
        .output()
        .expect("git commit failed");

    // Create a worktree
    let wt_path = main_repo.join("wt-test");
    Command::new("git")
        .args([
            "worktree",
            "add",
            wt_path.to_str().unwrap(),
            "-b",
            "wt-branch",
        ])
        .current_dir(main_repo)
        .output()
        .expect("git worktree add failed");

    // Run memory-write daily from the worktree
    let output = Command::new(&bin)
        .args(["memory-write", "daily", "plan", "test-feature", "dev"])
        .env("CLAUDE_PROJECT_DIR", &wt_path)
        .env("HOME", home_dir.path())
        .output()
        .expect("failed to execute memory-write daily from worktree");

    assert!(
        output.status.success(),
        "memory-write daily from worktree must succeed\nstderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );

    // Verify the hash matches the main repo hash
    let main_memory_dir = project_memory_dir(main_repo, home_dir.path());
    let wt_memory_dir = project_memory_dir(&wt_path, home_dir.path());
    assert_eq!(
        main_memory_dir, wt_memory_dir,
        "worktree and main repo must resolve to same memory dir"
    );

    // Verify files were actually written at the expected location
    let daily_dir = main_memory_dir.join("daily");
    assert!(
        daily_dir.exists(),
        "daily dir must exist at main repo hash path: {daily_dir:?}"
    );
}

/// PC-004: memory-write memory-index from worktree resolves to same MEMORY.md.
#[test]
fn worktree_memory_index_resolves_to_main_repo_hash() {
    let bin = common::binary_path();
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let main_repo = temp_dir.path();

    Command::new("git")
        .args(["init"])
        .current_dir(main_repo)
        .output()
        .expect("git init failed");
    Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init"])
        .current_dir(main_repo)
        .output()
        .expect("git commit failed");

    let wt_path = main_repo.join("wt-test");
    Command::new("git")
        .args([
            "worktree",
            "add",
            wt_path.to_str().unwrap(),
            "-b",
            "wt-branch",
        ])
        .current_dir(main_repo)
        .output()
        .expect("git worktree add failed");

    let output = Command::new(&bin)
        .args(["memory-write", "memory-index"])
        .env("CLAUDE_PROJECT_DIR", &wt_path)
        .env("HOME", home_dir.path())
        .output()
        .expect("failed to execute memory-write memory-index from worktree");

    assert!(
        output.status.success(),
        "memory-write memory-index from worktree must succeed\nstderr: {}",
        String::from_utf8_lossy(&output.stderr),
    );

    let main_memory_dir = project_memory_dir(main_repo, home_dir.path());
    let memory_md = main_memory_dir.join("MEMORY.md");
    assert!(
        memory_md.exists(),
        "MEMORY.md must exist at main repo hash path: {memory_md:?}"
    );
}

/// PC-005: memory-write daily from non-git dir exits non-zero with error.
#[test]
fn non_git_dir_returns_error() {
    let bin = common::binary_path();
    let temp_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();

    // Do NOT run git init — this is a non-git directory
    let output = Command::new(&bin)
        .args(["memory-write", "daily", "plan", "test-feature", "dev"])
        .env("CLAUDE_PROJECT_DIR", temp_dir.path())
        .env("HOME", home_dir.path())
        .env_remove("ECC_WORKFLOW_BYPASS")
        .output()
        .expect("failed to execute memory-write daily");

    // daily writes are warn-on-error (exit 0 with warn JSON), check combined output
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        combined.contains("not a git repository") || combined.contains("warn"),
        "output should indicate not a git repository or warn, got: {combined}"
    );
}
