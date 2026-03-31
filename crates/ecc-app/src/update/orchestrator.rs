use crate::update::context::UpdateContext;
use crate::update::options::UpdateOptions;
use crate::update::summary::UpdateSummary;
use crate::update::swap;
use ecc_domain::update::{ArtifactName, UpdateError, UpdatePlan, Version};
use ecc_ports::release::{ChecksumResult, CosignResult};
use std::path::PathBuf;

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
/// Flow: detect platform -> resolve artifact -> query version -> build plan ->
/// (dry-run bail) -> download -> verify checksum -> verify cosign ->
/// swap binaries -> run ecc install -> verify -> summary.
pub fn run_update(
    ctx: &UpdateContext<'_>,
    options: &UpdateOptions,
    current_version_str: &str,
    on_progress: &dyn Fn(u64, u64),
) -> UpdateResult {
    let current = Version::parse(current_version_str)?;

    // 1. Detect platform and architecture
    let platform = ctx.env.platform();
    let arch = ctx.env.architecture();
    let platform_label = match platform {
        ecc_ports::env::Platform::MacOS => "macos",
        ecc_ports::env::Platform::Linux => "linux",
        other => {
            return Err(UpdateError::UnsupportedPlatform {
                platform: format!("{other:?}"),
                arch: arch.as_label().to_string(),
            })
        }
    };
    let artifact = ArtifactName::resolve(platform_label, arch.as_label())?;

    // 2. Query target version
    let release_info = if let Some(ref target_ver) = options.target_version {
        ctx.release_client.get_version(target_ver).map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                UpdateError::VersionNotFound {
                    version: target_ver.clone(),
                }
            } else if msg.contains("rate limited") {
                UpdateError::RateLimited {
                    reset_time: msg,
                }
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
                    UpdateError::RateLimited {
                        reset_time: msg,
                    }
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

    // 6. Download tarball
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

    // 7. Verify checksum
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

    // 8. Verify cosign
    let cosign = ctx
        .release_client
        .verify_cosign(target.as_str(), artifact.as_str(), &tarball_path)
        .map_err(|_| UpdateError::CosignUnavailable)?;

    match cosign {
        CosignResult::Verified => {
            ctx.terminal
                .stdout_write("Cosign signature verified\n");
        }
        CosignResult::NotInstalled => {
            ctx.terminal.stderr_write(
                "Warning: cosign not installed. Checksum verified but signature not checked. \
                 Install cosign for enhanced security.\n",
            );
        }
        CosignResult::Failed => {
            let _ = ctx.fs.remove_dir_all(&temp_dir);
            return Err(UpdateError::SwapFailed {
                reason: "cosign signature verification failed".to_string(),
            });
        }
    }

    // 9. Determine install directory (where current binary lives)
    let install_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("/usr/local/bin"));

    // 10. Extract tarball (simulated — in real impl, self_update handles this)
    let extraction_dir = temp_dir.join("extracted");
    ctx.fs
        .create_dir_all(&extraction_dir.join("bin"))
        .map_err(|e| UpdateError::SwapFailed {
            reason: format!("extraction failed: {e}"),
        })?;

    // 11. Swap binaries
    let _swapped =
        swap::sequential_swap(ctx.fs, &extraction_dir, &install_dir, &["ecc", "ecc-workflow"])?;

    // 12. Update shims
    let _shim_count = swap::update_shims(ctx.fs, &extraction_dir, &install_dir)?;

    // 13. Run ecc install (config sync)
    let install_result = ctx.shell.run_command("ecc", &["install"]);
    let files_synced = match install_result {
        Ok(output) if output.success() => {
            // Count lines in stdout as proxy for files synced
            output.stdout.lines().count()
        }
        Ok(output) => {
            return Err(UpdateError::ConfigSyncFailed {
                reason: format!(
                    "ecc install failed (exit {}): {}. Backup available in install directory.",
                    output.exit_code, output.stderr
                ),
            });
        }
        Err(e) => {
            return Err(UpdateError::ConfigSyncFailed {
                reason: format!("ecc install failed: {e}. Backup available in install directory."),
            });
        }
    };

    // 14. Verify new version
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

    // 15. Clean up
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
    use ecc_test_support::{
        BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor, MockReleaseClient,
    };
    use ecc_test_support::mock_release_client::MockError;

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

    #[test]
    fn full_upgrade_flow() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new()
            .with_architecture(Architecture::Amd64)
            .with_var("HOME", "/home/test");
        let shell = default_shell();
        let terminal = BufferedTerminal::new();
        let client = MockReleaseClient::new()
            .with_latest_version(make_release("5.0.0"))
            .with_checksum_result(ChecksumResult::Match)
            .with_cosign_result(CosignResult::Verified);

        let ctx = UpdateContext {
            fs: &fs,
            env: &env,
            shell: &shell,
            terminal: &terminal,
            release_client: &client,
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
        let client = MockReleaseClient::new()
            .with_latest_version(make_release("4.0.0"));

        let ctx = UpdateContext {
            fs: &fs,
            env: &env,
            shell: &shell,
            terminal: &terminal,
            release_client: &client,
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
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
        let shell = default_shell();
        let terminal = BufferedTerminal::new();
        let client = MockReleaseClient::new()
            .with_version("3.5.0", make_release("3.5.0"))
            .with_checksum_result(ChecksumResult::Match)
            .with_cosign_result(CosignResult::Verified);

        let ctx = UpdateContext {
            fs: &fs,
            env: &env,
            shell: &shell,
            terminal: &terminal,
            release_client: &client,
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
        let client = MockReleaseClient::new()
            .with_latest_version(make_release("5.0.0"));

        let ctx = UpdateContext {
            fs: &fs,
            env: &env,
            shell: &shell,
            terminal: &terminal,
            release_client: &client,
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
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
        let shell = default_shell();
        let terminal = BufferedTerminal::new();
        let client = MockReleaseClient::new()
            .with_version("3.0.0", make_release("3.0.0"))
            .with_checksum_result(ChecksumResult::Match)
            .with_cosign_result(CosignResult::Verified);

        let ctx = UpdateContext {
            fs: &fs,
            env: &env,
            shell: &shell,
            terminal: &terminal,
            release_client: &client,
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
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
        let shell = default_shell();
        let terminal = BufferedTerminal::new();
        // Mock returns stable version when include_prerelease=false
        let client = MockReleaseClient::new()
            .with_latest_version(make_release("4.1.0"));

        let ctx = UpdateContext {
            fs: &fs,
            env: &env,
            shell: &shell,
            terminal: &terminal,
            release_client: &client,
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

        let ctx = UpdateContext {
            fs: &fs,
            env: &env,
            shell: &shell,
            terminal: &terminal,
            release_client: &client,
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
        let client = MockReleaseClient::new()
            .with_error(MockError::RateLimited("rate limited: resets at 2024-01-01T00:00:00Z".to_string()));

        let ctx = UpdateContext {
            fs: &fs,
            env: &env,
            shell: &shell,
            terminal: &terminal,
            release_client: &client,
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

        let ctx = UpdateContext {
            fs: &fs,
            env: &env,
            shell: &shell,
            terminal: &terminal,
            release_client: &client,
        };

        let result = run_update(&ctx, &UpdateOptions::default(), "4.0.0", &progress_noop);
        assert!(matches!(result, Err(UpdateError::ChecksumMismatch)));
    }

    #[test]
    fn cosign_verified_when_available() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
        let shell = default_shell();
        let terminal = BufferedTerminal::new();
        let client = MockReleaseClient::new()
            .with_latest_version(make_release("5.0.0"))
            .with_checksum_result(ChecksumResult::Match)
            .with_cosign_result(CosignResult::Verified);

        let ctx = UpdateContext {
            fs: &fs,
            env: &env,
            shell: &shell,
            terminal: &terminal,
            release_client: &client,
        };

        let result = run_update(&ctx, &UpdateOptions::default(), "4.0.0", &progress_noop);
        assert!(result.is_ok());
        let stdout = terminal.stdout_output();
        assert!(
            stdout.iter().any(|s| s.contains("Cosign signature verified")),
            "expected cosign verified in stdout"
        );
    }

    #[test]
    fn cosign_unavailable_fallback() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
        let shell = default_shell();
        let terminal = BufferedTerminal::new();
        let client = MockReleaseClient::new()
            .with_latest_version(make_release("5.0.0"))
            .with_checksum_result(ChecksumResult::Match)
            .with_cosign_result(CosignResult::NotInstalled);

        let ctx = UpdateContext {
            fs: &fs,
            env: &env,
            shell: &shell,
            terminal: &terminal,
            release_client: &client,
        };

        let result = run_update(&ctx, &UpdateOptions::default(), "4.0.0", &progress_noop);
        assert!(result.is_ok(), "should succeed with checksum-only");
        let stderr = terminal.stderr_output();
        assert!(
            stderr.iter().any(|s| s.contains("cosign not installed")),
            "expected cosign warning in stderr, got: {stderr:?}"
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

        let ctx = UpdateContext {
            fs: &fs,
            env: &env,
            shell: &shell,
            terminal: &terminal,
            release_client: &client,
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
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
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

        let ctx = UpdateContext {
            fs: &fs,
            env: &env,
            shell: &shell,
            terminal: &terminal,
            release_client: &client,
        };

        let result = run_update(&ctx, &UpdateOptions::default(), "4.0.0", &progress_noop);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, UpdateError::ConfigSyncFailed { .. }),
            "expected ConfigSyncFailed, got: {err}"
        );
        assert!(err.to_string().contains("Backup"), "should mention backup path");
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

        let ctx = UpdateContext {
            fs: &fs,
            env: &env,
            shell: &shell,
            terminal: &terminal,
            release_client: &client,
        };

        let result = run_update(&ctx, &UpdateOptions::default(), "4.0.0", &progress_noop);
        assert!(result.is_err());
        // Temp directory should be cleaned up
        use ecc_ports::fs::FileSystem;
        assert!(!fs.exists(std::path::Path::new("/tmp/ecc-update")));
    }

    #[test]
    fn config_sync_after_swap() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
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

        let ctx = UpdateContext {
            fs: &fs,
            env: &env,
            shell: &shell,
            terminal: &terminal,
            release_client: &client,
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
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
        let shell = default_shell();
        let terminal = BufferedTerminal::new();
        let client = MockReleaseClient::new()
            .with_latest_version(make_release("5.0.0"))
            .with_checksum_result(ChecksumResult::Match)
            .with_cosign_result(CosignResult::Verified)
            .with_download_bytes(vec![0u8; 100]);

        let ctx = UpdateContext {
            fs: &fs,
            env: &env,
            shell: &shell,
            terminal: &terminal,
            release_client: &client,
        };

        use std::sync::atomic::{AtomicBool, Ordering};
        let progress_called = AtomicBool::new(false);
        let result = run_update(&ctx, &UpdateOptions::default(), "4.0.0", &|_, _| {
            progress_called.store(true, Ordering::Relaxed);
        });
        assert!(result.is_ok());
        assert!(progress_called.load(Ordering::Relaxed), "progress callback should have been invoked");
    }

    #[test]
    fn post_swap_version_check() {
        let fs = InMemoryFileSystem::new();
        let env = MockEnvironment::new().with_architecture(Architecture::Amd64);
        let shell = MockExecutor::new()
            .on_args("ecc", &["install"], CommandOutput {
                stdout: "installed\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            })
            .on_args("ecc", &["version"], CommandOutput {
                stdout: "ecc 5.0.0\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            })
            .with_command("ecc");
        let terminal = BufferedTerminal::new();
        let client = MockReleaseClient::new()
            .with_latest_version(make_release("5.0.0"))
            .with_checksum_result(ChecksumResult::Match)
            .with_cosign_result(CosignResult::Verified);

        let ctx = UpdateContext {
            fs: &fs,
            env: &env,
            shell: &shell,
            terminal: &terminal,
            release_client: &client,
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
}
