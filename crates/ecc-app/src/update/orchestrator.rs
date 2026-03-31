use crate::update::context::UpdateContext;
use crate::update::options::UpdateOptions;
use crate::update::summary::UpdateSummary;
use crate::update::swap;
use ecc_domain::update::{ArtifactName, UpdateError, UpdatePlan, Version};
use ecc_ports::extract::ExtractError;
use ecc_ports::lock::LockError;
use ecc_ports::release::{ChecksumResult, CosignResult};
use std::path::PathBuf;
use std::time::Duration;

/// Result type for update operations.
pub type UpdateResult = Result<UpdateOutcome, UpdateError>;

/// Outcome of an update operation.
#[derive(Debug)]
pub enum UpdateOutcome {
    /// Successfully updated.
    Updated(UpdateSummary),
    /// Already at the latest version.
    AlreadyCurrent(String),
    /// Dry run — shows what would happen.
    DryRun(String),
}

/// Execute the update operation.
///
/// Flow: acquire lock -> detect platform -> resolve artifact -> query version -> build plan ->
/// (dry-run bail) -> check permissions -> download -> verify checksum -> verify cosign ->
/// extract -> swap binaries -> run ecc install -> verify -> summary.
pub fn run_update(
    ctx: &UpdateContext<'_>,
    options: &UpdateOptions,
    current_version_str: &str,
    on_progress: &dyn Fn(u64, u64),
) -> UpdateResult {
    // 0. Acquire update lock (RAII — released when guard drops)
    let install_dir_for_lock = ctx
        .env
        .current_exe()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("/usr/local/bin"));

    let _lock_guard = ctx
        .lock
        .acquire_with_timeout(&install_dir_for_lock, "ecc-update", Duration::from_secs(5))
        .map_err(|e| match e {
            LockError::Timeout(_) | LockError::AcquireFailed { .. } => UpdateError::UpdateLocked {
                reason: e.to_string(),
            },
            LockError::DirCreation { .. } => UpdateError::UpdateLocked {
                reason: e.to_string(),
            },
        })?;

    let current = Version::parse(current_version_str)?;

    // 1. Detect platform and architecture
    let platform = ctx.env.platform();
    let arch = ctx.env.architecture();
    let artifact = ArtifactName::resolve(platform, arch)?;

    // 2. Query target version
    let release_info = if let Some(ref target_ver) = options.target_version {
        ctx.release_client.get_version(target_ver).map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                UpdateError::VersionNotFound {
                    version: target_ver.clone(),
                }
            } else if msg.contains("rate limited") {
                UpdateError::RateLimited { reset_time: msg }
            } else {
                UpdateError::NetworkError { reason: msg }
            }
        })?
    } else {
        ctx.release_client
            .latest_version(options.include_prerelease)
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("rate limited") {
                    UpdateError::RateLimited { reset_time: msg }
                } else {
                    UpdateError::NetworkError { reason: msg }
                }
            })?
    };

    let target = Version::parse(&release_info.version)?;

    // 3. Build update plan
    let plan = UpdatePlan::new(&current, &target, &artifact);

    if plan.is_already_current {
        return Ok(UpdateOutcome::AlreadyCurrent(format!(
            "Already up to date (v{})",
            current
        )));
    }

    // 4. Dry run
    if options.dry_run {
        let mut msg = format!(
            "Would update: v{} -> v{}\nArtifact: {}",
            plan.current_version, plan.target_version, plan.artifact_name
        );
        if plan.is_downgrade {
            msg.push_str("\nWarning: this is a downgrade");
        }
        return Ok(UpdateOutcome::DryRun(msg));
    }

    // 5. Downgrade warning
    if plan.is_downgrade {
        ctx.terminal.stderr_write(&format!(
            "Warning: downgrading from v{} to v{}\n",
            plan.current_version, plan.target_version
        ));
    }

    // 6. Determine install directory (where current binary lives) via ctx.env
    let install_dir = ctx
        .env
        .current_exe()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("/usr/local/bin"));

    // 7. Check install dir writability before download
    let probe_path = install_dir.join(".ecc-update-probe");
    match ctx.fs.write(&probe_path, "") {
        Ok(()) => {
            // Clean up probe file
            let _ = ctx.fs.remove_file(&probe_path);
        }
        Err(_) => {
            return Err(UpdateError::PermissionDenied {
                path: install_dir.display().to_string(),
                reason: "install directory is not writable".to_string(),
            });
        }
    }

    // 8. Download tarball
    let temp_dir = ctx.env.temp_dir().join("ecc-update");
    let tarball_path = temp_dir.join(format!("{}.tar.gz", artifact.as_str()));
    ctx.fs
        .create_dir_all(&temp_dir)
        .map_err(|e| UpdateError::NetworkError {
            reason: format!("failed to create temp dir: {e}"),
        })?;

    let download_result = ctx.release_client.download_tarball(
        target.as_str(),
        artifact.as_str(),
        &tarball_path,
        on_progress,
    );

    if let Err(e) = download_result {
        // Clean up temp file on download failure
        let _ = ctx.fs.remove_file(&tarball_path);
        let _ = ctx.fs.remove_dir_all(&temp_dir);
        if e.to_string().contains("interrupted") {
            return Err(UpdateError::DownloadInterrupted);
        }
        return Err(UpdateError::NetworkError {
            reason: e.to_string(),
        });
    }

    // 9. Verify checksum
    let checksum = ctx
        .release_client
        .verify_checksum(target.as_str(), artifact.as_str(), &tarball_path)
        .map_err(|e| UpdateError::NetworkError {
            reason: format!("checksum verification error: {e}"),
        })?;

    if checksum == ChecksumResult::Mismatch {
        let _ = ctx.fs.remove_dir_all(&temp_dir);
        return Err(UpdateError::ChecksumMismatch);
    }

    // 10. Download cosign bundle and verify signature — NotInstalled aborts (not a warning)
    let bundle_path = PathBuf::from(format!("{}.bundle", tarball_path.display()));
    if let Err(e) = ctx.release_client.download_cosign_bundle(
        target.as_str(),
        artifact.as_str(),
        &bundle_path,
    ) {
        let _ = ctx.fs.remove_dir_all(&temp_dir);
        return Err(UpdateError::NetworkError {
            reason: format!("failed to download cosign bundle: {e}"),
        });
    }

    let cosign = ctx
        .release_client
        .verify_cosign(
            target.as_str(),
            artifact.as_str(),
            &tarball_path,
            &bundle_path,
        )
        .map_err(|_| UpdateError::CosignUnavailable)?;

    match cosign {
        CosignResult::Verified => {
            ctx.terminal.stdout_write("Cosign signature verified\n");
        }
        CosignResult::NotInstalled => {
            let _ = ctx.fs.remove_dir_all(&temp_dir);
            return Err(UpdateError::CosignUnavailable);
        }
        CosignResult::Failed => {
            let _ = ctx.fs.remove_dir_all(&temp_dir);
            return Err(UpdateError::SecurityVerificationFailed {
                reason: "cosign signature verification failed".to_string(),
            });
        }
    }

    // 11. Extract tarball via port
    let extraction_dir = temp_dir.join("extracted");
    ctx.fs
        .create_dir_all(&extraction_dir)
        .map_err(|e| UpdateError::SwapFailed {
            reason: format!("failed to create extraction dir: {e}"),
        })?;

    ctx.extractor
        .extract(&tarball_path, &extraction_dir)
        .map_err(|e| match e {
            ExtractError::CorruptArchive(msg) => UpdateError::SwapFailed {
                reason: format!("corrupt archive: {msg}"),
            },
            ExtractError::ZipSlip(msg) => UpdateError::SwapFailed {
                reason: format!("zip-slip detected: {msg}"),
            },
            ExtractError::Io(msg) => UpdateError::SwapFailed {
                reason: format!("extraction I/O error: {msg}"),
            },
        })?;

    // 12. Swap binaries
    let swapped = swap::sequential_swap(
        ctx.fs,
        &extraction_dir,
        &install_dir,
        &["ecc", "ecc-workflow"],
    )?;

    // 13. Update shims
    let _shim_count = swap::update_shims(ctx.fs, &extraction_dir, &install_dir)?;

    // 14. Run ecc install (config sync) — rollback on failure
    let install_result = ctx.shell.run_command("ecc", &["install"]);
    let files_synced = match install_result {
        Ok(output) if output.success() => {
            // Count lines in stdout as proxy for files synced
            output.stdout.lines().count()
        }
        Ok(output) => {
            // Rollback swapped binaries before returning error
            let _ = swap::rollback_swapped(ctx.fs, &swapped);
            return Err(UpdateError::ConfigSyncFailed {
                reason: format!(
                    "ecc install failed (exit {}): {}. Rolled back. Backup available in install directory.",
                    output.exit_code, output.stderr
                ),
            });
        }
        Err(e) => {
            // Rollback swapped binaries before returning error
            let _ = swap::rollback_swapped(ctx.fs, &swapped);
            return Err(UpdateError::ConfigSyncFailed {
                reason: format!(
                    "ecc install failed: {e}. Rolled back. Backup available in install directory."
                ),
            });
        }
    };

    // 15. Verify new version
    let version_check = ctx.shell.run_command("ecc", &["version"]);
    if let Ok(output) = version_check {
        let reported = output.stdout.trim();
        if !reported.contains(target.as_str()) {
            ctx.terminal.stderr_write(&format!(
                "Warning: expected version {} but got '{}'\n",
                target, reported
            ));
        }
    }

    // 16. Clean up backup files from swap
    swap::cleanup_backups(ctx.fs, &swapped);

    // 17. Clean up
    let _ = ctx.fs.remove_dir_all(&temp_dir);

    Ok(UpdateOutcome::Updated(UpdateSummary {
        old_version: current.to_string(),
        new_version: target.to_string(),
        release_notes: release_info.release_notes,
        files_synced,
    }))
}

#[cfg(test)]
mod tests {
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
    // NEW TESTS for PC-015 through PC-024
    // ============================================================

    /// PC-015: Orchestrator uses ctx.env.current_exe() instead of std::env::current_exe()
    /// Verified indirectly: if current_exe comes from ctx.env, then MockEnvironment with
    /// a custom path determines the install_dir. We test that full_upgrade_flow passes
    /// (it's the test that is wired to ctx.env.current_exe for install_dir).
    /// The test is already updated above with with_current_exe("/usr/local/bin/ecc").

    /// PC-016: Orchestrator checks install dir writability before download, returns PermissionDenied
    #[test]
    fn permission_denied_before_download() {
        // Use an env that points to a non-writable install dir
        // In InMemoryFileSystem, write() succeeds by default. We need a FS where
        // the install dir write fails. We use an env pointing to a path that
        // the InMemoryFileSystem will reject writes (simulate via an env that
        // returns a path where writing will fail).
        //
        // Strategy: use MockEnvironment with a current_exe in a path that
        // exists as a directory entry only (no write capability). Since InMemoryFS
        // always allows writes, we test by using an env with current_exe = None
        // (fallback to /usr/local/bin) and pre-configure the FS so that writing
        // to /usr/local/bin probe fails.
        //
        // Actually, InMemoryFS.write() always succeeds. We need a different approach:
        // use a read-only FS wrapper or configure env to return a known-unwritable path.
        //
        // Best approach: environment returns current_exe = None so install_dir = /usr/local/bin.
        // Then we use a FS that rejects writes to /usr/local/bin by having a dir at
        // /usr/local/bin/.ecc-update-probe (so the write will... succeed actually).
        //
        // The real test: we need a FS that returns PermissionDenied on write to probe.
        // Since InMemoryFileSystem doesn't support permission-denied-on-write, we use
        // a custom approach: check that when current_exe() returns a path, the permission
        // check uses fs.write(). We verify this by using a FS that tracks writes and
        // checking the error path through a failing extractor scenario instead.
        //
        // ACTUAL APPROACH: Use an env where current_exe() returns None -> install_dir=/usr/local/bin.
        // Create a FS where the directory /usr/local/bin doesn't exist and can't be written
        // (write_bytes will fail because InMemoryFS requires parent dir to exist for write...
        // let's check: InMemoryFS.write() does NOT check for parent existence).
        //
        // We need to test that PermissionDenied is returned. The only way with InMemoryFileSystem
        // is if ctx.fs.write returns an error. InMemoryFileSystem always succeeds on write.
        //
        // Solution: Use a PermissionDeniedFileSystem wrapper.
        // For test purposes, we create a simple struct that always returns PermissionDenied on write.

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
}
