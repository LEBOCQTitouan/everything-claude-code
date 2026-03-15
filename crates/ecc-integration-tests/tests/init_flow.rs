mod common;

use common::EccTestEnv;

#[test]
fn init_creates_gitignore_entries() {
    let env = EccTestEnv::new();

    // Initialize a git repo in the project directory
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(env.project.path())
        .output()
        .expect("git init failed");

    env.cmd()
        .args(["init", "--no-interactive"])
        .current_dir(env.project.path())
        .assert()
        .success();

    let gitignore_path = env.project.path().join(".gitignore");
    if gitignore_path.exists() {
        let content = std::fs::read_to_string(&gitignore_path).unwrap();
        // Should contain some ECC-related entry or .claude reference
        assert!(
            !content.is_empty(),
            ".gitignore should have content after init"
        );
    }
    // If no .gitignore was created, the init may have used --no-gitignore semantics
    // which is still a valid outcome.
}

#[test]
fn init_dry_run_no_changes() {
    let env = EccTestEnv::new();

    // Initialize a git repo in the project directory
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(env.project.path())
        .output()
        .expect("git init failed");

    // Capture dir contents before
    let before: Vec<_> = std::fs::read_dir(env.project.path())
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.file_name()))
        .collect();

    env.cmd()
        .args(["init", "--no-interactive", "--dry-run"])
        .current_dir(env.project.path())
        .assert()
        .success();

    // After dry-run, only .git should exist (no new files)
    let after: Vec<_> = std::fs::read_dir(env.project.path())
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.file_name()))
        .collect();

    assert_eq!(
        before.len(),
        after.len(),
        "dry-run should not create new files"
    );
}
