use super::super::*;
use crate::update::context::UpdateContext;
use crate::update::options::UpdateOptions;
use ecc_ports::env::Architecture;
use ecc_ports::release::{ChecksumResult, CosignResult, ReleaseInfo};
use ecc_test_support::{
    BufferedTerminal, InMemoryFileSystem, InMemoryLock, MockEnvironment, MockExecutor,
    MockExtractor, MockReleaseClient,
};

fn make_release(version: &str) -> ReleaseInfo {
    ReleaseInfo {
        version: version.to_string(),
        release_notes: "- Test release".to_string(),
    }
}

fn default_shell() -> MockExecutor {
    use ecc_ports::shell::CommandOutput;
    MockExecutor::new()
        .on(
            "ecc",
            CommandOutput {
                stdout: "Installed 42 files\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        )
        .with_command("ecc")
}

fn progress_noop(_: u64, _: u64) {}

pub(super) fn fs_with_extracted_binaries() -> InMemoryFileSystem {
    use ecc_ports::fs::FileSystem;
    let fs = InMemoryFileSystem::new();
    let _ = fs.create_dir_all(std::path::Path::new("/usr/local/bin"));
    let _ = fs.create_dir_all(std::path::Path::new("/tmp/ecc-update/extracted/bin"));
    let _ = fs.write(
        std::path::Path::new("/tmp/ecc-update/extracted/bin/ecc"),
        "new-ecc",
    );
    let _ = fs.write(
        std::path::Path::new("/tmp/ecc-update/extracted/bin/ecc-workflow"),
        "new-ecc-workflow",
    );
    fs
}

#[test]
fn full_upgrade_flow() {
    let fs = fs_with_extracted_binaries();
    let env = MockEnvironment::new()
        .with_architecture(Architecture::Amd64)
        .with_var("HOME", "/home/test")
        .with_current_exe("/usr/local/bin/ecc");
    let shell = default_shell();
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new()
        .with_latest_version(make_release("5.0.0"))
        .with_checksum_result(ChecksumResult::Match)
        .with_cosign_result(CosignResult::Verified);
    let lock = InMemoryLock::new();
    let extractor = MockExtractor::new();

    let ctx = UpdateContext {
        fs: &fs,
        env: &env,
        shell: &shell,
        terminal: &terminal,
        release_client: &client,
        lock: &lock,
        extractor: &extractor,
    };

    let result = run_update(&ctx, &UpdateOptions::default(), "4.0.0", &progress_noop);
    assert!(result.is_ok(), "expected success, got: {result:?}");
    match result.unwrap() {
        UpdateOutcome::Updated(summary) => {
            assert_eq!(summary.old_version, "4.0.0");
            assert_eq!(summary.new_version, "5.0.0");
        }
        other => panic!("expected Updated, got: {other:?}"),
    }
}

#[test]
fn already_up_to_date() {
    let fs = InMemoryFileSystem::new();
    let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
    let shell = default_shell();
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new().with_latest_version(make_release("4.0.0"));
    let lock = InMemoryLock::new();
    let extractor = MockExtractor::new();

    let ctx = UpdateContext {
        fs: &fs,
        env: &env,
        shell: &shell,
        terminal: &terminal,
        release_client: &client,
        lock: &lock,
        extractor: &extractor,
    };

    let result = run_update(&ctx, &UpdateOptions::default(), "4.0.0", &progress_noop);
    match result.unwrap() {
        UpdateOutcome::AlreadyCurrent(msg) => {
            assert!(msg.contains("4.0.0"));
        }
        other => panic!("expected AlreadyCurrent, got: {other:?}"),
    }
}

#[test]
fn specific_version() {
    let fs = fs_with_extracted_binaries();
    let env = MockEnvironment::new()
        .with_architecture(Architecture::Amd64)
        .with_current_exe("/usr/local/bin/ecc");
    let shell = default_shell();
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new()
        .with_version("3.5.0", make_release("3.5.0"))
        .with_checksum_result(ChecksumResult::Match)
        .with_cosign_result(CosignResult::Verified);
    let lock = InMemoryLock::new();
    let extractor = MockExtractor::new();

    let ctx = UpdateContext {
        fs: &fs,
        env: &env,
        shell: &shell,
        terminal: &terminal,
        release_client: &client,
        lock: &lock,
        extractor: &extractor,
    };

    let opts = UpdateOptions {
        target_version: Some("3.5.0".to_string()),
        ..Default::default()
    };
    let result = run_update(&ctx, &opts, "4.0.0", &progress_noop);
    assert!(result.is_ok());
}

#[test]
fn dry_run_no_writes() {
    let fs = InMemoryFileSystem::new();
    let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
    let shell = MockExecutor::new();
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new().with_latest_version(make_release("5.0.0"));
    let lock = InMemoryLock::new();
    let extractor = MockExtractor::new();

    let ctx = UpdateContext {
        fs: &fs,
        env: &env,
        shell: &shell,
        terminal: &terminal,
        release_client: &client,
        lock: &lock,
        extractor: &extractor,
    };

    let opts = UpdateOptions {
        dry_run: true,
        ..Default::default()
    };
    let result = run_update(&ctx, &opts, "4.0.0", &progress_noop);
    match result.unwrap() {
        UpdateOutcome::DryRun(msg) => {
            assert!(msg.contains("4.0.0"));
            assert!(msg.contains("5.0.0"));
        }
        other => panic!("expected DryRun, got: {other:?}"),
    }
}

#[test]
fn downgrade_warning() {
    let fs = fs_with_extracted_binaries();
    let env = MockEnvironment::new()
        .with_architecture(Architecture::Amd64)
        .with_current_exe("/usr/local/bin/ecc");
    let shell = default_shell();
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new()
        .with_version("3.0.0", make_release("3.0.0"))
        .with_checksum_result(ChecksumResult::Match)
        .with_cosign_result(CosignResult::Verified);
    let lock = InMemoryLock::new();
    let extractor = MockExtractor::new();

    let ctx = UpdateContext {
        fs: &fs,
        env: &env,
        shell: &shell,
        terminal: &terminal,
        release_client: &client,
        lock: &lock,
        extractor: &extractor,
    };

    let opts = UpdateOptions {
        target_version: Some("3.0.0".to_string()),
        ..Default::default()
    };
    let result = run_update(&ctx, &opts, "4.0.0", &progress_noop);
    assert!(result.is_ok());
    let stderr = terminal.stderr_output();
    assert!(
        stderr.iter().any(|s| s.contains("downgrad")),
        "expected downgrade warning in stderr, got: {stderr:?}"
    );
}

#[test]
fn skips_prerelease_by_default() {
    let fs = fs_with_extracted_binaries();
    let env = MockEnvironment::new()
        .with_architecture(Architecture::Amd64)
        .with_current_exe("/usr/local/bin/ecc");
    let shell = default_shell();
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new().with_latest_version(make_release("4.1.0"));
    let lock = InMemoryLock::new();
    let extractor = MockExtractor::new();

    let ctx = UpdateContext {
        fs: &fs,
        env: &env,
        shell: &shell,
        terminal: &terminal,
        release_client: &client,
        lock: &lock,
        extractor: &extractor,
    };

    let opts = UpdateOptions::default();
    let result = run_update(&ctx, &opts, "4.0.0", &progress_noop);
    assert!(result.is_ok() || matches!(result, Err(UpdateError::NetworkError { .. })));
}

#[test]
fn cosign_verified_when_available() {
    let fs = fs_with_extracted_binaries();
    let env = MockEnvironment::new()
        .with_architecture(Architecture::Amd64)
        .with_current_exe("/usr/local/bin/ecc");
    let shell = default_shell();
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new()
        .with_latest_version(make_release("5.0.0"))
        .with_checksum_result(ChecksumResult::Match)
        .with_cosign_result(CosignResult::Verified);
    let lock = InMemoryLock::new();
    let extractor = MockExtractor::new();

    let ctx = UpdateContext {
        fs: &fs,
        env: &env,
        shell: &shell,
        terminal: &terminal,
        release_client: &client,
        lock: &lock,
        extractor: &extractor,
    };

    let result = run_update(&ctx, &UpdateOptions::default(), "4.0.0", &progress_noop);
    assert!(result.is_ok());
    let stdout = terminal.stdout_output();
    assert!(
        stdout
            .iter()
            .any(|s| s.contains("Cosign signature verified")),
        "expected cosign verified in stdout"
    );
}

#[test]
fn config_sync_after_swap() {
    use ecc_ports::shell::CommandOutput;
    let fs = fs_with_extracted_binaries();
    let env = MockEnvironment::new()
        .with_architecture(Architecture::Amd64)
        .with_current_exe("/usr/local/bin/ecc");
    let shell = MockExecutor::new().on(
        "ecc",
        CommandOutput {
            stdout: "file1\nfile2\nfile3\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
        },
    );
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new()
        .with_latest_version(make_release("5.0.0"))
        .with_checksum_result(ChecksumResult::Match)
        .with_cosign_result(CosignResult::Verified);
    let lock = InMemoryLock::new();
    let extractor = MockExtractor::new();

    let ctx = UpdateContext {
        fs: &fs,
        env: &env,
        shell: &shell,
        terminal: &terminal,
        release_client: &client,
        lock: &lock,
        extractor: &extractor,
    };

    let result = run_update(&ctx, &UpdateOptions::default(), "4.0.0", &progress_noop);
    match result.unwrap() {
        UpdateOutcome::Updated(summary) => {
            assert!(summary.files_synced > 0, "should report files synced");
        }
        other => panic!("expected Updated, got: {other:?}"),
    }
}

#[test]
fn progress_callback() {
    let fs = fs_with_extracted_binaries();
    let env = MockEnvironment::new()
        .with_architecture(Architecture::Amd64)
        .with_current_exe("/usr/local/bin/ecc");
    let shell = default_shell();
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new()
        .with_latest_version(make_release("5.0.0"))
        .with_checksum_result(ChecksumResult::Match)
        .with_cosign_result(CosignResult::Verified)
        .with_download_bytes(vec![0u8; 100]);
    let lock = InMemoryLock::new();
    let extractor = MockExtractor::new();

    let ctx = UpdateContext {
        fs: &fs,
        env: &env,
        shell: &shell,
        terminal: &terminal,
        release_client: &client,
        lock: &lock,
        extractor: &extractor,
    };

    use std::sync::atomic::{AtomicBool, Ordering};
    let progress_called = AtomicBool::new(false);
    let result = run_update(&ctx, &UpdateOptions::default(), "4.0.0", &|_, _| {
        progress_called.store(true, Ordering::Relaxed);
    });
    assert!(result.is_ok());
    assert!(
        progress_called.load(Ordering::Relaxed),
        "progress callback should have been invoked"
    );
}

#[test]
fn post_swap_version_check() {
    use ecc_ports::shell::CommandOutput;
    let fs = fs_with_extracted_binaries();
    let env = MockEnvironment::new()
        .with_architecture(Architecture::Amd64)
        .with_current_exe("/usr/local/bin/ecc");
    let shell = MockExecutor::new()
        .on_args(
            "ecc",
            &["install"],
            CommandOutput {
                stdout: "installed\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        )
        .on_args(
            "ecc",
            &["version"],
            CommandOutput {
                stdout: "ecc 5.0.0\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        )
        .with_command("ecc");
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new()
        .with_latest_version(make_release("5.0.0"))
        .with_checksum_result(ChecksumResult::Match)
        .with_cosign_result(CosignResult::Verified);
    let lock = InMemoryLock::new();
    let extractor = MockExtractor::new();

    let ctx = UpdateContext {
        fs: &fs,
        env: &env,
        shell: &shell,
        terminal: &terminal,
        release_client: &client,
        lock: &lock,
        extractor: &extractor,
    };

    let result = run_update(&ctx, &UpdateOptions::default(), "4.0.0", &progress_noop);
    assert!(result.is_ok());
    let stderr = terminal.stderr_output();
    assert!(
        !stderr.iter().any(|s| s.contains("expected version")),
        "should not warn about version mismatch"
    );
}
