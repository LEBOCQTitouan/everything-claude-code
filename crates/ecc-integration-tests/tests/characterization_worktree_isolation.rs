//! Characterization test for worktree state isolation (PC-029, PC-030, PC-031).
//!
//! Creates a git worktree and verifies that workflow state is independent
//! between the main repo and the worktree, stored under the git directory.

use assert_cmd::Command;
use std::path::Path;
use tempfile::TempDir;

fn wf_cmd(project_dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("ecc-workflow").expect("ecc-workflow binary not found");
    cmd.env("CLAUDE_PROJECT_DIR", project_dir);
    cmd
}

/// Get the git-dir for a given project directory.
fn git_dir(project_dir: &Path) -> std::path::PathBuf {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(project_dir)
        .output()
        .expect("git rev-parse --git-dir failed");
    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let path = std::path::PathBuf::from(&path_str);
    if path.is_relative() {
        project_dir.join(path)
    } else {
        path
    }
}

/// Read the workflow phase from the worktree-scoped state directory:
/// `<git-dir>/ecc-workflow/state.json`
fn read_phase(project_dir: &Path) -> Option<String> {
    let gd = git_dir(project_dir);
    let path = gd.join("ecc-workflow/state.json");
    let content = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    v["phase"].as_str().map(|s| s.to_owned())
}

/// PC-029, PC-030, PC-031: Worktree isolation — state stored in git-dir,
/// and each worktree has an independent phase after divergent transitions.
#[test]
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

    // PC-029: state must NOT be at .claude/workflow (old location)
    assert!(
        !wt_dir.join(".claude/workflow/state.json").exists(),
        "state must NOT be at .claude/workflow/state.json in worktree"
    );

    // PC-030: state must be at <git-dir>/ecc-workflow/state.json
    let wt_git_dir = git_dir(&wt_dir);
    let wt_state_file = wt_git_dir.join("ecc-workflow/state.json");
    assert!(
        wt_state_file.exists(),
        "state must be at <git-dir>/ecc-workflow/state.json, git_dir={:?}",
        wt_git_dir
    );

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
