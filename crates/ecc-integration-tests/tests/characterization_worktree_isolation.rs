//! Characterization test for worktree state isolation (PC-035).
//!
//! Creates a git worktree and verifies that workflow state is independent
//! between the main repo and the worktree.
//!
//! This test is #[ignore]'d by default because it requires git and creates
//! temporary worktrees. It will be un-ignored after the worktree-scoped
//! state implementation is complete.

use assert_cmd::Command;
use std::path::Path;
use tempfile::TempDir;

fn wf_cmd(project_dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("ecc-workflow").expect("ecc-workflow binary not found");
    cmd.env("CLAUDE_PROJECT_DIR", project_dir);
    cmd.env("ECC_WORKFLOW_BYPASS", "0");
    cmd
}

fn read_phase(project_dir: &Path) -> Option<String> {
    let path = project_dir.join(".claude/workflow/state.json");
    let content = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    v["phase"].as_str().map(|s| s.to_owned())
}

/// PC-035: Worktree isolation — main and worktree have independent state.
///
/// This test currently documents that the ecc-workflow binary stores state
/// at `.claude/workflow/state.json` relative to CLAUDE_PROJECT_DIR. When
/// CLAUDE_PROJECT_DIR points to different directories, state is naturally
/// independent.
#[test]
#[ignore = "Requires git; un-ignore after worktree-scoped state is implemented"]
fn worktree_state_isolation() {
    let tmp = TempDir::new().unwrap();
    let main_dir = tmp.path().join("main");
    std::fs::create_dir_all(&main_dir).unwrap();

    // Initialize a git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&main_dir)
        .output()
        .expect("git init failed");

    // Create initial commit (needed for worktree)
    std::fs::write(main_dir.join("README.md"), "# Test").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&main_dir)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "init", "--allow-empty"])
        .current_dir(&main_dir)
        .output()
        .unwrap();

    // Create a worktree
    let wt_dir = tmp.path().join("worktree");
    std::process::Command::new("git")
        .args(["worktree", "add", wt_dir.to_str().unwrap(), "-b", "feature"])
        .current_dir(&main_dir)
        .output()
        .expect("git worktree add failed");

    // Init workflow in main
    wf_cmd(&main_dir)
        .args(["init", "dev", "main-feature"])
        .assert()
        .success();

    // Init workflow in worktree
    wf_cmd(&wt_dir)
        .args(["init", "fix", "wt-feature"])
        .assert()
        .success();

    // Both should have plan phase
    assert_eq!(read_phase(&main_dir), Some("plan".to_owned()));
    assert_eq!(read_phase(&wt_dir), Some("plan".to_owned()));

    // Transition main to solution — worktree should stay at plan
    wf_cmd(&main_dir)
        .args(["transition", "solution"])
        .assert()
        .success();

    assert_eq!(
        read_phase(&main_dir),
        Some("solution".to_owned()),
        "main should be at solution"
    );
    assert_eq!(
        read_phase(&wt_dir),
        Some("plan".to_owned()),
        "worktree should still be at plan (independent state)"
    );

    // Cleanup worktree
    std::process::Command::new("git")
        .args(["worktree", "remove", wt_dir.to_str().unwrap(), "--force"])
        .current_dir(&main_dir)
        .output()
        .ok();
}
