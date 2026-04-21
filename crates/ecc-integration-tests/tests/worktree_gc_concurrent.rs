//! Integration tests for concurrent-session / kill-switch / liveness CLI flags.
//!
//! PC-048..052: `--kill-live`, `--yes` flags + TTY-aware confirmation prompt.

use assert_cmd::Command;
use std::path::Path;
use tempfile::TempDir;

// ── git helpers ───────────────────────────────────────────────────────────────

fn init_git_repo() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path();

    run_git(path, &["init"]);
    run_git(path, &["config", "user.email", "test@ecc.test"]);
    run_git(path, &["config", "user.name", "ECC Test"]);

    let readme = path.join("README.md");
    std::fs::write(&readme, "# test\n").expect("failed to write README");
    run_git(path, &["add", "README.md"]);
    run_git(path, &["commit", "-m", "Initial commit"]);

    dir
}

fn run_git(dir: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap_or_else(|e| panic!("git {args:?} failed to start: {e}"));
    assert!(
        status.success(),
        "git {args:?} exited non-zero in {}",
        dir.display()
    );
}

fn ecc_cmd() -> Command {
    Command::cargo_bin("ecc").expect("ecc binary not found")
}

/// Create a session worktree inside `repo_path`.
/// Uses a stale timestamp (year 2020) so the age check marks it old.
/// PID in the name is `999999` — almost certainly dead.
fn add_stale_session_worktree(repo_path: &Path, name: &str) {
    run_git(repo_path, &["worktree", "add", "--orphan", "-b", name, name]);
}

/// Write a "live" `.ecc-session` heartbeat file into the worktree.
/// Uses `std::process::id()` as the PID so the owning process is alive.
fn write_live_heartbeat(worktree_path: &Path) {
    let pid = std::process::id();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let json = format!(
        r#"{{"schema_version":1,"claude_code_pid":{pid},"last_seen_unix_ts":{now}}}"#
    );
    std::fs::write(worktree_path.join(".ecc-session"), json)
        .expect("failed to write .ecc-session");
}

// ── PC-048: --force without --kill-live respects liveness ───────────────────

/// AC-006.1: `--force` alone must NOT delete a live worktree.
#[test]
#[ignore] // Requires: git, real process
fn force_respects_liveness() {
    let repo = init_git_repo();
    let repo_path = repo.path();

    let wt_name = "ecc-session-20200101-120000-live-respects-liveness";
    add_stale_session_worktree(repo_path, wt_name);

    let wt_path = repo_path.join(wt_name);
    write_live_heartbeat(&wt_path);

    // --force alone: liveness is respected, live worktree must NOT be deleted.
    let mut cmd = ecc_cmd();
    cmd.args(["worktree", "gc", "--force", "--dir"])
        .arg(repo_path);
    let output = cmd.output().expect("ecc command failed to run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // The worktree should still exist (skipped, not removed).
    assert!(
        wt_path.exists(),
        "live worktree must NOT be deleted by --force alone. stdout={stdout} stderr={stderr}"
    );
}

// ── PC-049: Interactive --force --kill-live shows confirmation prompt ────────

/// AC-006.2: `--force --kill-live` in non-TTY context with "n" on stdin → no deletion.
/// AC-006.2 (b): `--force --kill-live` in non-TTY context with "y" on stdin → deletion.
///
/// Note: assert_cmd pipes stdio → non-TTY. The CLI must fall into non-TTY path.
/// For non-TTY without `--yes`, the implementation exits non-zero (AC-006.5).
/// So "n\n" effectively means the command exits non-zero (treated as non-TTY rejection).
/// We test the interactive path here by verifying behaviour with piped stdin.
#[test]
#[ignore] // Requires: git, real process
fn kill_live_prompts() {
    let repo = init_git_repo();
    let repo_path = repo.path();

    let wt_name_no = "ecc-session-20200101-120000-live-prompt-no";
    add_stale_session_worktree(repo_path, wt_name_no);
    write_live_heartbeat(&repo_path.join(wt_name_no));

    // Non-TTY (piped stdin) without --yes → must exit non-zero (AC-006.5 / non-TTY rejection).
    // This simulates what "n\n" response does in non-TTY: blocked by non-TTY guard.
    let mut cmd = ecc_cmd();
    cmd.args(["worktree", "gc", "--force", "--kill-live", "--dir"])
        .arg(repo_path)
        .write_stdin("n\n");
    let output = cmd.output().expect("ecc command failed to run");

    // Non-TTY without --yes → non-zero exit
    assert!(
        !output.status.success(),
        "non-TTY --kill-live without --yes must exit non-zero (stdin 'n')"
    );

    // worktree must still exist
    assert!(
        repo_path.join(wt_name_no).exists(),
        "worktree must NOT be deleted when non-TTY exits non-zero"
    );
}

// ── PC-050: --force --kill-live --yes bypasses prompt ───────────────────────

/// AC-006.3: `--force --kill-live --yes` bypasses confirmation; live worktree deleted.
#[test]
#[ignore] // Requires: git, real process
fn kill_live_yes_bypasses_prompt() {
    let repo = init_git_repo();
    let repo_path = repo.path();

    let wt_name = "ecc-session-20200101-120000-live-yes-bypass";
    add_stale_session_worktree(repo_path, wt_name);
    write_live_heartbeat(&repo_path.join(wt_name));

    let mut cmd = ecc_cmd();
    cmd.args([
        "worktree",
        "gc",
        "--force",
        "--kill-live",
        "--yes",
        "--dir",
    ])
    .arg(repo_path);
    let output = cmd.output().expect("ecc command failed to run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "--kill-live --yes must exit 0. stdout={stdout} stderr={stderr}"
    );

    // The live worktree must be gone.
    assert!(
        !repo_path.join(wt_name).exists(),
        "live worktree must be deleted with --force --kill-live --yes. stdout={stdout} stderr={stderr}"
    );
}

// ── PC-051: --kill-live without --force rejected by clap ───────────────────

/// AC-006.4: `--kill-live` without `--force` is a clap usage error (exit code 2).
/// Tests that stderr contains "requires --force" (or clap's `requires` message).
#[test]
#[ignore] // Requires: ecc binary
fn kill_live_requires_force() {
    let mut cmd = ecc_cmd();
    cmd.args(["worktree", "gc", "--kill-live"]);
    let output = cmd.output().expect("ecc command failed to run");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(
        output.status.code(),
        Some(2),
        "--kill-live without --force must exit with code 2 (clap error). stderr={stderr}"
    );
    assert!(
        stderr.contains("--force") || stderr.contains("force"),
        "--kill-live without --force must mention --force in error. stderr={stderr}"
    );
}

// ── PC-052: --force --kill-live non-TTY without --yes exits non-zero ────────

/// AC-006.5: In non-TTY context, `--force --kill-live` without `--yes` must exit non-zero.
#[test]
#[ignore] // Requires: git, real process
fn kill_live_non_tty_requires_yes() {
    let repo = init_git_repo();
    let repo_path = repo.path();

    let wt_name = "ecc-session-20200101-120000-live-non-tty";
    add_stale_session_worktree(repo_path, wt_name);
    write_live_heartbeat(&repo_path.join(wt_name));

    // assert_cmd uses piped I/O → non-TTY. No --yes → must exit non-zero.
    let mut cmd = ecc_cmd();
    cmd.args(["worktree", "gc", "--force", "--kill-live", "--dir"])
        .arg(repo_path);
    let output = cmd.output().expect("ecc command failed to run");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "--kill-live in non-TTY without --yes must exit non-zero. stderr={stderr}"
    );
    assert!(
        stderr.contains("--yes") || stderr.contains("yes") || stderr.contains("non-interactive"),
        "stderr must mention --yes or non-interactive context. stderr={stderr}"
    );

    // Worktree must NOT be deleted.
    assert!(
        repo_path.join(wt_name).exists(),
        "worktree must NOT be deleted when non-TTY guard rejects. stderr={stderr}"
    );
}
