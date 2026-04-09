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
    let fs = super::happy_path_tests::fs_with_extracted_binaries();
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
    assert!(
        !lock.is_held("ecc-update"),
        "lock should be released after successful update"
    );

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
    assert!(
        !fs.exists(std::path::Path::new("/usr/local/bin/ecc")),
        "no partial state: ecc should not exist in install dir after corrupt archive"
    );
}

/// PC-022: Orchestrator on ConfigSyncFailed invokes rollback_swapped, message contains "rolled back"
#[test]
fn config_sync_triggers_rollback() {
    use ecc_ports::shell::CommandOutput;
    let fs = super::happy_path_tests::fs_with_extracted_binaries();
    let env = MockEnvironment::new()
        .with_architecture(Architecture::Amd64)
        .with_current_exe("/usr/local/bin/ecc");
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
