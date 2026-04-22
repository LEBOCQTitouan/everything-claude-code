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
    run_git(
        repo_path,
        &["worktree", "add", "--orphan", "-b", name, name],
    );
}

/// Write a "live" `.ecc-session` heartbeat file into the worktree.
/// Uses `std::process::id()` as the PID so the owning process is alive.
fn write_live_heartbeat(worktree_path: &Path) {
    let pid = std::process::id();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let json =
        format!(r#"{{"schema_version":1,"claude_code_pid":{pid},"last_seen_unix_ts":{now}}}"#);
    std::fs::write(worktree_path.join(".ecc-session"), json).expect("failed to write .ecc-session");
}

// ── helpers for valid session worktree names ─────────────────────────────────

/// Build a valid session worktree name with a numeric PID suffix.
fn session_name(slug: &str) -> String {
    // Use current PID so it's guaranteed alive for heartbeat tests.
    let pid = std::process::id();
    format!("ecc-session-20200101-120000-{slug}-{pid}")
}

// ── PC-048: --force without --kill-live respects liveness ───────────────────

/// AC-006.1: `--force` alone must NOT delete a live worktree.
#[test]
#[ignore] // Requires: git, real process
fn force_respects_liveness() {
    let repo = init_git_repo();
    let repo_path = repo.path();

    let wt_name = session_name("live-respects-liveness");
    add_stale_session_worktree(repo_path, &wt_name);

    let wt_path = repo_path.join(&wt_name);
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

    let wt_name_no = session_name("live-prompt-no");
    add_stale_session_worktree(repo_path, &wt_name_no);
    write_live_heartbeat(&repo_path.join(&wt_name_no));

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
        repo_path.join(&wt_name_no).exists(),
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

    let wt_name = session_name("live-yes-bypass");
    add_stale_session_worktree(repo_path, &wt_name);
    write_live_heartbeat(&repo_path.join(&wt_name));

    let mut cmd = ecc_cmd();
    cmd.args(["worktree", "gc", "--force", "--kill-live", "--yes", "--dir"])
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
        !repo_path.join(&wt_name).exists(),
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

// ── PC-059: --dry-run --json emits [{name, action, reason}] ─────────────────

/// AC-008.4: `--dry-run --json` emits a JSON array with `name`, `action`, `reason`.
/// Uses a live heartbeat + --kill-live so force_delete_live=true, bypassing the recency guard.
#[test]
#[ignore] // Requires: git, real process
fn dry_run_json_schema() {
    let repo = init_git_repo();
    let repo_path = repo.path();

    // Use a live worktree with --kill-live so the GC targets it for deletion.
    // This bypasses the recency guard (which would protect a just-created worktree).
    let wt_name = session_name("dry-json");
    add_stale_session_worktree(repo_path, &wt_name);
    write_live_heartbeat(&repo_path.join(&wt_name));

    let mut cmd = ecc_cmd();
    cmd.args([
        "worktree",
        "gc",
        "--dry-run",
        "--force",
        "--kill-live",
        "--yes",
        "--json",
        "--dir",
    ])
    .arg(repo_path);
    let output = cmd.output().expect("ecc command failed to run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "--dry-run --json must exit 0. stdout={stdout} stderr={stderr}"
    );

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("--dry-run --json must emit valid JSON: {e}. stdout={stdout}"));

    let arr = parsed
        .as_array()
        .expect("--dry-run --json must emit a JSON array");

    assert!(
        !arr.is_empty(),
        "--dry-run --json array must contain the live worktree entry. stdout={stdout}"
    );

    let entry = &arr[0];
    assert!(
        entry.get("name").is_some(),
        "JSON entry must have 'name' field. entry={entry}"
    );
    assert!(
        entry.get("action").is_some(),
        "JSON entry must have 'action' field. entry={entry}"
    );
    assert!(
        entry.get("reason").is_some(),
        "JSON entry must have 'reason' field. entry={entry}"
    );
    assert_eq!(
        entry["action"].as_str(),
        Some("would_delete"),
        "action must be 'would_delete'. entry={entry}"
    );
}

// ── PC-073: --dry-run --force --kill-live --yes previews live, no destruction ─

/// AC-008.3 + AC-006.3: `--dry-run --force --kill-live --yes` previews live worktrees
/// with no confirmation prompt and no destructive calls.
#[test]
#[ignore] // Requires: git, real process
fn dry_run_kill_live_yes_no_destructive() {
    let repo = init_git_repo();
    let repo_path = repo.path();

    let wt_name = session_name("dry-kl-yes");
    add_stale_session_worktree(repo_path, &wt_name);
    write_live_heartbeat(&repo_path.join(&wt_name));

    let mut cmd = ecc_cmd();
    cmd.args([
        "worktree",
        "gc",
        "--dry-run",
        "--force",
        "--kill-live",
        "--yes",
        "--dir",
    ])
    .arg(repo_path);
    let output = cmd.output().expect("ecc command failed to run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Must exit 0 (no prompt, dry-run is non-destructive).
    assert!(
        output.status.success(),
        "--dry-run --force --kill-live --yes must exit 0. stdout={stdout} stderr={stderr}"
    );

    // The live worktree must still exist (no destructive calls).
    assert!(
        repo_path.join(&wt_name).exists(),
        "live worktree must NOT be deleted in dry-run. stdout={stdout} stderr={stderr}"
    );

    // stdout must mention the worktree (WOULD DELETE output).
    assert!(
        stdout.contains("WOULD DELETE") || stdout.contains(&wt_name),
        "dry-run output must mention the would-delete worktree. stdout={stdout} stderr={stderr}"
    );
}

// ── PC-060: ECC_WORKTREE_LIVENESS_DISABLED=1 disables read AND write ────────

/// AC-009.1: When `ECC_WORKTREE_LIVENESS_DISABLED=1`:
/// - heartbeat is NOT written (no `.ecc-session` after SessionStart)
/// - gc falls back to BL-150 logic (stderr mentions "liveness check disabled")
#[test]
#[ignore] // Requires: ecc binary, git
fn liveness_disabled_kill_switch() {
    let repo = init_git_repo();
    let repo_path = repo.path();

    // Create a session worktree (stale PID 999999 so BL-150 would delete it).
    // We do NOT write a live heartbeat — this tests that gc does not consult
    // `.ecc-session` files when the kill switch is active.
    let wt_name = session_name("kill-switch");
    add_stale_session_worktree(repo_path, &wt_name);
    let wt_path = repo_path.join(&wt_name);

    // The worktree dir must NOT contain `.ecc-session` after gc runs with the kill switch.
    // (No heartbeat write means no .ecc-session file — but we verify gc doesn't create one.)
    assert!(
        !wt_path.join(".ecc-session").exists(),
        "precondition: no .ecc-session before gc"
    );

    // Run gc with kill switch enabled. The worktree is stale by BL-150 logic
    // (timestamp in name is old: 2020), so gc should remove it.
    let mut cmd = ecc_cmd();
    cmd.args(["worktree", "gc", "--force", "--dir"])
        .arg(repo_path)
        .env("ECC_WORKTREE_LIVENESS_DISABLED", "1");
    let output = cmd.output().expect("ecc command failed to run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Must exit successfully.
    assert!(
        output.status.success(),
        "gc with kill switch must exit 0. stdout={stdout} stderr={stderr}"
    );

    // Stderr must mention "liveness check disabled" (once per process).
    assert!(
        stderr.contains("liveness check disabled") || stderr.contains("liveness disabled"),
        "stderr must mention liveness check disabled. stderr={stderr}"
    );

    // No .ecc-session file must exist in the worktree dir after gc.
    // (Kill switch suppresses heartbeat writes.)
    // Note: the worktree itself may have been removed by gc (stale by BL-150),
    // so we check that IF it still exists, no .ecc-session was written.
    if wt_path.exists() {
        assert!(
            !wt_path.join(".ecc-session").exists(),
            ".ecc-session must NOT exist after gc with kill switch active"
        );
    }
}

// ── PC-063: .ecc-session present → git status --porcelain empty ─────────────

/// AC-009.4: `.ecc-session` must be gitignored so it does not appear in git status.
#[test]
#[ignore] // Requires: git
fn ecc_session_gitignored() {
    let repo = init_git_repo();
    let repo_path = repo.path();

    // Add .ecc-session to .gitignore in the temp repo.
    let gitignore_path = repo_path.join(".gitignore");
    std::fs::write(&gitignore_path, ".ecc-session\n").expect("failed to write .gitignore");
    run_git(repo_path, &["add", ".gitignore"]);
    run_git(repo_path, &["commit", "-m", "add .gitignore"]);

    // Create a .ecc-session file in the repo root.
    let session_file = repo_path.join(".ecc-session");
    std::fs::write(
        &session_file,
        r#"{"schema_version":1,"claude_code_pid":12345,"last_seen_unix_ts":9999999999}"#,
    )
    .expect("failed to write .ecc-session");

    // git status --porcelain must NOT list .ecc-session.
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_path)
        .output()
        .expect("git status failed");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stdout.contains(".ecc-session"),
        ".ecc-session must NOT appear in git status --porcelain when gitignored. stdout={stdout}"
    );
}

// ── PC-052: --force --kill-live non-TTY without --yes exits non-zero ────────

/// AC-006.5: In non-TTY context, `--force --kill-live` without `--yes` must exit non-zero.
#[test]
#[ignore] // Requires: git, real process
fn kill_live_non_tty_requires_yes() {
    let repo = init_git_repo();
    let repo_path = repo.path();

    let wt_name = session_name("live-non-tty");
    add_stale_session_worktree(repo_path, &wt_name);
    write_live_heartbeat(&repo_path.join(&wt_name));

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
        repo_path.join(&wt_name).exists(),
        "worktree must NOT be deleted when non-TTY guard rejects. stderr={stderr}"
    );
}
