//! Integration smoke tests for the local `act` CI setup.
//!
//! These tests use real `std::process::Command` and `std::fs` — they are
//! system-level smoke tests, not unit tests behind port abstractions.
//!
//! Guards:
//! - Skipped on GitHub CI (`GITHUB_ACTIONS=true`) with a warning.
//! - Skipped when `act` is not installed locally with a warning.

fn should_skip() -> bool {
    if std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true") {
        eprintln!("WARN: Skipping act integration test on GitHub CI");
        return true;
    }
    if !std::process::Command::new("act")
        .arg("--version")
        .output()
        .map_or(false, |o| o.status.success())
    {
        eprintln!("WARN: act not installed — skipping smoke test");
        return true;
    }
    false
}

/// Find the project root by walking up from the manifest dir.
fn project_root() -> std::path::PathBuf {
    let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // ecc-app is at crates/ecc-app — go up two levels to workspace root
    dir.pop(); // crates/
    dir.pop(); // workspace root
    dir
}

#[test]
fn act_version_smoke() {
    if should_skip() {
        return;
    }
    let output = std::process::Command::new("act")
        .arg("--version")
        .output()
        .expect("failed to run act --version");
    assert!(output.status.success(), "act --version exited non-zero");
}

#[test]
fn act_list_jobs_smoke() {
    if should_skip() {
        return;
    }
    let root = project_root();
    let output = std::process::Command::new("act")
        .arg("-l")
        .current_dir(&root)
        .output()
        .expect("failed to run act -l");
    assert!(output.status.success(), "act -l exited non-zero");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty(), "act -l produced no output");
}

#[test]
fn config_files_exist() {
    if std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true") {
        eprintln!("WARN: Skipping config_files_exist on GitHub CI");
        return;
    }
    let root = project_root();
    assert!(
        root.join(".actrc").exists(),
        ".actrc not found at project root"
    );
    assert!(
        root.join(".secrets.example").exists(),
        ".secrets.example not found at project root"
    );
}
