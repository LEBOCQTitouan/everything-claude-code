use super::super::*;
use crate::update::context::UpdateContext;
use crate::update::options::UpdateOptions;
use ecc_ports::env::Architecture;
use ecc_ports::release::{ChecksumResult, CosignResult, ReleaseInfo};
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

fn progress_noop(_: u64, _: u64) {}

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
fn version_not_found() {
    let fs = InMemoryFileSystem::new();
    let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
    let shell = MockExecutor::new();
    let terminal = BufferedTerminal::new();
    let client =
        MockReleaseClient::new().with_error(MockError::NotFound("99.0.0 not found".to_string()));
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
    use super::happy_path_tests::fs_with_extracted_binaries;
    use ecc_ports::shell::CommandOutput;
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
    use ecc_ports::fs::FileSystem;
    assert!(!fs.exists(std::path::Path::new("/tmp/ecc-update")));
}

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
        fn canonicalize(&self, path: &Path) -> Result<std::path::PathBuf, std::io::Error> {
            Ok(path.to_path_buf())
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
