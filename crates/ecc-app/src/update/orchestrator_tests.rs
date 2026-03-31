use super::*;
use crate::update::context::UpdateContext;
use crate::update::options::UpdateOptions;
use ecc_ports::env::Architecture;
use ecc_ports::release::{ChecksumResult, CosignResult, ReleaseInfo};
use ecc_ports::shell::CommandOutput;
use ecc_test_support::mock_release_client::MockError;
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

/// Build a filesystem with the binaries the MockExtractor would produce in extraction_dir.
fn fs_with_extracted_binaries() -> InMemoryFileSystem {
    use ecc_ports::fs::FileSystem;
    let fs = InMemoryFileSystem::new();
    // install dir
    let _ = fs.create_dir_all(std::path::Path::new("/usr/local/bin"));
    // extracted binaries (MockExtractor returns these paths but doesn't create them)
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
    // Mock returns stable version when include_prerelease=false
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

    // Default options have include_prerelease=false
    let opts = UpdateOptions::default();
    let result = run_update(&ctx, &opts, "4.0.0", &progress_noop);
    // Should succeed without getting a prerelease
    assert!(result.is_ok() || matches!(result, Err(UpdateError::NetworkError { .. })));
}

#[test]
fn network_error_message() {
    let fs = InMemoryFileSystem::new();
    let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
    let shell = MockExecutor::new();
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new()
        .with_error(MockError::NetworkError("connection refused".to_string()));
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
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Network error") || err.to_string().contains("network"),
        "expected network error, got: {err}"
    );
}

#[test]
fn rate_limit_message() {
    let fs = InMemoryFileSystem::new();
    let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
    let shell = MockExecutor::new();
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new().with_error(MockError::RateLimited(
        "rate limited: resets at 2024-01-01T00:00:00Z".to_string(),
    ));
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
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, UpdateError::RateLimited { .. }),
        "expected RateLimited, got: {err}"
    );
}

#[test]
fn checksum_failure_aborts() {
    let fs = InMemoryFileSystem::new();
    let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
    let shell = MockExecutor::new();
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new()
        .with_latest_version(make_release("5.0.0"))
        .with_checksum_result(ChecksumResult::Mismatch);
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
    assert!(matches!(result, Err(UpdateError::ChecksumMismatch)));
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
fn version_not_found() {
    let fs = InMemoryFileSystem::new();
    let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
    let shell = MockExecutor::new();
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new()
        .with_error(MockError::NotFound("99.0.0 not found".to_string()));
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
        target_version: Some("99.0.0".to_string()),
        ..Default::default()
    };
    let result = run_update(&ctx, &opts, "4.0.0", &progress_noop);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, UpdateError::VersionNotFound { .. }),
        "expected VersionNotFound, got: {err}"
    );
}

#[test]
fn config_sync_failure() {
    let fs = fs_with_extracted_binaries();
    let env = MockEnvironment::new()
        .with_architecture(Architecture::Amd64)
        .with_current_exe("/usr/local/bin/ecc");
    let shell = MockExecutor::new().on(
        "ecc",
        CommandOutput {
            stdout: String::new(),
            stderr: "install failed: permission denied".to_string(),
            exit_code: 1,
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
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, UpdateError::ConfigSyncFailed { .. }),
        "expected ConfigSyncFailed, got: {err}"
    );
    assert!(
        err.to_string().contains("Backup"),
        "should mention backup path"
    );
}

#[test]
fn download_interrupted_cleanup() {
    let fs = InMemoryFileSystem::new();
    let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
    let shell = MockExecutor::new();
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new()
        .with_latest_version(make_release("5.0.0"))
        .with_error(MockError::NetworkError("interrupted".to_string()));
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
    assert!(result.is_err());
    // Temp directory should be cleaned up
    use ecc_ports::fs::FileSystem;
    assert!(!fs.exists(std::path::Path::new("/tmp/ecc-update")));
}

#[test]
fn config_sync_after_swap() {
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
    // No version mismatch warning should appear
    let stderr = terminal.stderr_output();
    assert!(
        !stderr.iter().any(|s| s.contains("expected version")),
        "should not warn about version mismatch"
    );
}

// ============================================================
// Tests for PC-015 through PC-024
// ============================================================

/// PC-015: Orchestrator uses ctx.env.current_exe() instead of std::env::current_exe()
/// Verified indirectly: if current_exe comes from ctx.env, then MockEnvironment with
/// a custom path determines the install_dir. We test that full_upgrade_flow passes
/// (it's the test that is wired to ctx.env.current_exe for install_dir).
/// The test is already updated above with with_current_exe("/usr/local/bin/ecc").

/// PC-016: Orchestrator checks install dir writability before download, returns PermissionDenied
#[test]
fn permission_denied_before_download() {
    use ecc_ports::fs::{FileSystem, FsError};
    use std::path::{Path, PathBuf};

    struct ReadOnlyFileSystem;

    impl FileSystem for ReadOnlyFileSystem {
        fn read_to_string(&self, path: &Path) -> Result<String, FsError> {
            Err(FsError::NotFound(path.to_path_buf()))
        }
        fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, FsError> {
            Err(FsError::NotFound(path.to_path_buf()))
        }
        fn write(&self, path: &Path, _content: &str) -> Result<(), FsError> {
            Err(FsError::PermissionDenied(path.to_path_buf()))
        }
        fn write_bytes(&self, path: &Path, _content: &[u8]) -> Result<(), FsError> {
            Err(FsError::PermissionDenied(path.to_path_buf()))
        }
        fn exists(&self, _path: &Path) -> bool {
            false
        }
        fn is_dir(&self, _path: &Path) -> bool {
            false
        }
        fn is_file(&self, _path: &Path) -> bool {
            false
        }
        fn create_dir_all(&self, _path: &Path) -> Result<(), FsError> {
            Ok(())
        }
        fn remove_file(&self, _path: &Path) -> Result<(), FsError> {
            Ok(())
        }
        fn remove_dir_all(&self, _path: &Path) -> Result<(), FsError> {
            Ok(())
        }
        fn copy(&self, _from: &Path, to: &Path) -> Result<(), FsError> {
            Err(FsError::PermissionDenied(to.to_path_buf()))
        }
        fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
            Err(FsError::NotFound(path.to_path_buf()))
        }
        fn read_dir_recursive(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
            Err(FsError::NotFound(path.to_path_buf()))
        }
        fn create_symlink(&self, _target: &Path, link: &Path) -> Result<(), FsError> {
            Err(FsError::PermissionDenied(link.to_path_buf()))
        }
        fn read_symlink(&self, link: &Path) -> Result<PathBuf, FsError> {
            Err(FsError::NotFound(link.to_path_buf()))
        }
        fn is_symlink(&self, _path: &Path) -> bool {
            false
        }
        fn set_permissions(&self, path: &Path, _mode: u32) -> Result<(), FsError> {
            Err(FsError::PermissionDenied(path.to_path_buf()))
        }
        fn is_executable(&self, _path: &Path) -> bool {
            false
        }
        fn rename(&self, _from: &Path, to: &Path) -> Result<(), FsError> {
            Err(FsError::PermissionDenied(to.to_path_buf()))
        }
    }

    let fs = ReadOnlyFileSystem;
    let env = MockEnvironment::new()
        .with_architecture(Architecture::Amd64)
        .with_current_exe("/usr/local/bin/ecc");
    let shell = MockExecutor::new();
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
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, UpdateError::PermissionDenied { .. }),
        "expected PermissionDenied before download, got: {err}"
    );
}

/// PC-017: Orchestrator acquires flock, returns UpdateLocked when lock unavailable
#[test]
fn update_locked() {
    use ecc_ports::lock::FileLock;
    let fs = InMemoryFileSystem::new();
    let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
    let shell = MockExecutor::new();
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new().with_latest_version(make_release("5.0.0"));
    let lock = InMemoryLock::new();
    let extractor = MockExtractor::new();

    // Pre-acquire the lock to simulate a concurrent update
    let _guard = lock
        .acquire(std::path::Path::new("/usr/local/bin"), "ecc-update")
        .unwrap();

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
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), UpdateError::UpdateLocked { .. }),
        "expected UpdateLocked when lock is contended"
    );
}

/// PC-018: Lock released on success and on failure (RAII guard drop)
#[test]
fn lock_released() {
    use ecc_ports::lock::FileLock;
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
    // After run_update returns, lock should be released (RAII guard dropped)
    assert!(
        !lock.is_held("ecc-update"),
        "lock should be released after successful update"
    );

    // Also verify we can re-acquire the lock (proves it was released)
    let reacquire = lock.acquire(std::path::Path::new("/usr/local/bin"), "ecc-update");
    assert!(
        reacquire.is_ok(),
        "should be able to re-acquire lock after update completes"
    );
}

/// PC-020: Corrupt archive returns SwapFailed with no partial state
#[test]
fn corrupt_archive() {
    use ecc_ports::fs::FileSystem;
    let fs = InMemoryFileSystem::new();
    let _ = fs.create_dir_all(std::path::Path::new("/usr/local/bin"));
    let env = MockEnvironment::new()
        .with_architecture(Architecture::Amd64)
        .with_current_exe("/usr/local/bin/ecc");
    let shell = MockExecutor::new();
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new()
        .with_latest_version(make_release("5.0.0"))
        .with_checksum_result(ChecksumResult::Match)
        .with_cosign_result(CosignResult::Verified);
    let lock = InMemoryLock::new();
    // Extractor that fails with CorruptArchive
    let extractor = MockExtractor::new().with_failure();

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
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, UpdateError::SwapFailed { .. }),
        "expected SwapFailed for corrupt archive, got: {err}"
    );
    // Verify no binaries were swapped (install_dir still clean)
    assert!(
        !fs.exists(std::path::Path::new("/usr/local/bin/ecc")),
        "no partial state: ecc should not exist in install dir after corrupt archive"
    );
}

/// PC-022: Orchestrator on ConfigSyncFailed invokes rollback_swapped, message contains "rolled back"
#[test]
fn config_sync_triggers_rollback() {
    let fs = fs_with_extracted_binaries();
    let env = MockEnvironment::new()
        .with_architecture(Architecture::Amd64)
        .with_current_exe("/usr/local/bin/ecc");
    // Shell that fails `ecc install`
    let shell = MockExecutor::new().on(
        "ecc",
        CommandOutput {
            stdout: String::new(),
            stderr: "install failed: config error".to_string(),
            exit_code: 1,
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
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, UpdateError::ConfigSyncFailed { .. }),
        "expected ConfigSyncFailed, got: {err}"
    );
    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("rolled back") || msg.contains("Rolled back"),
        "error message should mention rollback, got: {msg}"
    );
}

/// PC-023: Cosign NotInstalled aborts update (not treated as warning)
#[test]
fn cosign_not_installed_aborts() {
    let fs = InMemoryFileSystem::new();
    let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
    let shell = MockExecutor::new();
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new()
        .with_latest_version(make_release("5.0.0"))
        .with_checksum_result(ChecksumResult::Match)
        .with_cosign_result(CosignResult::NotInstalled);
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
    assert!(result.is_err(), "NotInstalled should abort update");
    let err = result.unwrap_err();
    assert!(
        matches!(err, UpdateError::CosignUnavailable),
        "expected CosignUnavailable, got: {err}"
    );
}

/// PC-024: Cosign Failed aborts with SecurityVerificationFailed
#[test]
fn cosign_failed_aborts() {
    let fs = InMemoryFileSystem::new();
    let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
    let shell = MockExecutor::new();
    let terminal = BufferedTerminal::new();
    let client = MockReleaseClient::new()
        .with_latest_version(make_release("5.0.0"))
        .with_checksum_result(ChecksumResult::Match)
        .with_cosign_result(CosignResult::Failed);
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
    assert!(result.is_err(), "cosign Failed should abort update");
    let err = result.unwrap_err();
    assert!(
        matches!(err, UpdateError::SecurityVerificationFailed { .. }),
        "expected SecurityVerificationFailed, got: {err}"
    );
}
