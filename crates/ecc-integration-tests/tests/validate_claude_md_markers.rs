//! Integration tests for `ecc validate claude-md markers` CLI surface.
//!
//! Covers: PC-023..026, PC-041

use assert_cmd::Command;

fn ecc_cmd() -> Command {
    Command::cargo_bin("ecc").expect("ecc binary not found")
}

/// Create a tempdir with CLAUDE.md content.
fn make_temp_repo(claude_md_content: &str) -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().expect("tempdir");
    std::fs::write(dir.path().join("CLAUDE.md"), claude_md_content).expect("write CLAUDE.md");
    dir
}

/// Create a tempdir with CLAUDE.md content and backlog files in docs/backlog/.
fn make_temp_repo_with_backlog(
    claude_md_content: &str,
    backlog_files: &[(&str, &str)],
) -> tempfile::TempDir {
    let dir = make_temp_repo(claude_md_content);
    let backlog_dir = dir.path().join("docs").join("backlog");
    std::fs::create_dir_all(&backlog_dir).expect("create backlog dir");
    for (name, content) in backlog_files {
        std::fs::write(backlog_dir.join(name), content).expect("write backlog file");
    }
    dir
}

// PC-023: Legacy --counts flag emits deprecation warning on stderr.
#[test]
fn counts_flag_deprecation_warning() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let output = ecc_cmd()
        .arg("validate")
        .arg("--ecc-root")
        .arg(dir.path())
        .args(["claude-md", "--counts"])
        .output()
        .expect("failed to spawn ecc");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "DEPRECATED: use 'ecc validate claude-md counts' (subcommand form); --counts will be removed in the next minor release."
        ),
        "Expected deprecation warning on stderr, got: {stderr}"
    );
}

// PC-024: markers --strict happy path exits 0; missing-BL fail shows three
//         substrings simultaneously: path, :line:, and BL-NNN.
#[test]
fn markers_strict_happy_and_fail_message_composition() {
    // Happy path: BL-001 has a corresponding backlog file.
    let happy_dir = make_temp_repo_with_backlog(
        "TEMPORARY (BL-001): some note\n",
        &[("BL-001-foo.md", "# BL-001")],
    );
    let happy_output = ecc_cmd()
        .arg("validate")
        .arg("--ecc-root")
        .arg(happy_dir.path())
        .args(["claude-md", "markers", "--strict"])
        .output()
        .expect("failed to spawn ecc");
    assert!(
        happy_output.status.success(),
        "Expected exit 0 for valid markers, got: {:?}",
        happy_output.status.code()
    );

    // Fail path: BL-999 is referenced but no backlog file exists.
    let fail_dir = make_temp_repo("TEMPORARY (BL-999): missing entry\n");
    let fail_output = ecc_cmd()
        .arg("validate")
        .arg("--ecc-root")
        .arg(fail_dir.path())
        .args(["claude-md", "markers", "--strict"])
        .output()
        .expect("failed to spawn ecc");

    let exit_code = fail_output.status.code().unwrap_or(-1);
    assert_eq!(exit_code, 1, "Expected exit 1 for missing marker backlog file");

    let stderr = String::from_utf8_lossy(&fail_output.stderr);
    // Check: file path present (either "CLAUDE.md" substring or the tempdir path)
    let tempdir_path_str = fail_dir.path().to_string_lossy();
    assert!(
        stderr.contains("CLAUDE.md") || stderr.contains(tempdir_path_str.as_ref()),
        "Expected file path in stderr, got: {stderr}"
    );
    // Check: :line_number: format
    assert!(
        stderr.contains(":1:") || (stderr.contains(':') && stderr.contains("BL-999")),
        "Expected :line: format in stderr, got: {stderr}"
    );
    // Check: BL-999 mentioned
    assert!(stderr.contains("BL-999"), "Expected BL-999 in stderr, got: {stderr}");
}

// PC-025: counts subcommand rejects --strict (clap usage error, exit non-zero).
#[test]
fn strict_scoped_to_markers() {
    let output = ecc_cmd()
        .args(["validate", "claude-md", "counts", "--strict"])
        .output()
        .expect("failed to spawn ecc");

    let exit_code = output.status.code().unwrap_or(-1);
    assert_ne!(exit_code, 0, "Expected non-zero exit for --strict on counts subcommand");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let has_unexpected = stderr.contains("unexpected") || stderr.contains("unrecognized");
    assert!(
        has_unexpected && stderr.contains("--strict"),
        "Expected clap error mentioning --strict, got: {stderr}"
    );
}

// PC-026: ECC_CLAUDE_MD_MARKERS_DISABLED=1 short-circuits markers check and
//         emits the expected notice on stderr.
#[test]
fn kill_switch_env_subprocess() {
    let output = ecc_cmd()
        .env("ECC_CLAUDE_MD_MARKERS_DISABLED", "1")
        .args(["validate", "claude-md", "markers"])
        .output()
        .expect("failed to spawn ecc");

    assert!(
        output.status.success(),
        "Expected exit 0 with kill-switch enabled, got: {:?}",
        output.status.code()
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("markers: disabled via ECC_CLAUDE_MD_MARKERS_DISABLED"),
        "Expected kill-switch notice on stderr, got: {stderr}"
    );
}

// PC-041: Clap compatibility smoke — none of the listed invocations return
//         exit code 2 (clap argparse error).
#[test]
fn clap_surface_smoke() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    std::fs::write(dir.path().join("CLAUDE.md"), "").expect("write CLAUDE.md");

    // Each entry is (args_after_validate_ecc_root_dir)
    let invocations: &[&[&str]] = &[
        &["claude-md", "counts"],
        &["claude-md", "--counts"],
        &["claude-md", "markers"],
        &["claude-md", "markers", "--strict"],
        &["claude-md", "markers", "--audit-report"],
        &["claude-md", "markers", "--strict", "--audit-report"],
        &["claude-md", "all"],
        &["claude-md", "all", "--strict"],
    ];

    for args in invocations {
        let output = ecc_cmd()
            .arg("validate")
            .arg("--ecc-root")
            .arg(dir.path())
            .args(*args)
            .output()
            .unwrap_or_else(|e| panic!("failed to spawn ecc for args {args:?}: {e}"));

        let exit_code = output.status.code().unwrap_or(-1);
        assert_ne!(
            exit_code, 2,
            "Clap parse error (exit 2) for args {:?}; stderr: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
