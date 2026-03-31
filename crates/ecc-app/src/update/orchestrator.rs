use crate::update::context::UpdateContext;
use crate::update::options::UpdateOptions;
use crate::update::summary::UpdateSummary;
use crate::update::swap;
use ecc_domain::update::{ArtifactName, UpdateError, UpdatePlan, Version};
use ecc_ports::extract::ExtractError;
use ecc_ports::lock::LockError;
use ecc_ports::release::{ChecksumResult, CosignResult};
use std::path::{Path, PathBuf};
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
    let install_dir_for_lock = resolve_install_dir(ctx);

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

    // 6. Check permissions and download + verify artifacts
    let install_dir = resolve_install_dir(ctx);
    check_install_dir_writable(ctx, &install_dir)?;

    let temp_dir = ctx.env.temp_dir().join("ecc-update");
    let tarball_path = temp_dir.join(format!("{}.tar.gz", artifact.as_str()));

    download_and_verify(ctx, &target, &artifact, &temp_dir, &tarball_path, on_progress)?;

    // 7. Extract and swap binaries
    let swapped = extract_and_swap(ctx, &tarball_path, &temp_dir, &install_dir)?;

    // 8. Sync config or rollback on failure
    let files_synced = sync_config_or_rollback(ctx, &swapped)?;

    // 9. Verify new version
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

    // 10. Clean up backup files from swap
    swap::cleanup_backups(ctx.fs, &swapped);

    // 11. Clean up temp directory
    let _ = ctx.fs.remove_dir_all(&temp_dir);

    Ok(UpdateOutcome::Updated(UpdateSummary {
        old_version: current.to_string(),
        new_version: target.to_string(),
        release_notes: release_info.release_notes,
        files_synced,
    }))
}

// ── Helper functions ─────────────────────────────────────────────────────────

/// Resolve the install directory from the current executable path.
fn resolve_install_dir(ctx: &UpdateContext<'_>) -> PathBuf {
    ctx.env
        .current_exe()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("/usr/local/bin"))
}

/// Check that the install directory is writable before starting the download.
fn check_install_dir_writable(
    ctx: &UpdateContext<'_>,
    install_dir: &Path,
) -> Result<(), UpdateError> {
    let probe_path = install_dir.join(".ecc-update-probe");
    match ctx.fs.write(&probe_path, "") {
        Ok(()) => {
            let _ = ctx.fs.remove_file(&probe_path);
            Ok(())
        }
        Err(_) => Err(UpdateError::PermissionDenied {
            path: install_dir.display().to_string(),
            reason: "install directory is not writable".to_string(),
        }),
    }
}

/// Download the tarball and verify its checksum and cosign signature.
fn download_and_verify(
    ctx: &UpdateContext<'_>,
    target: &Version,
    artifact: &ArtifactName,
    temp_dir: &Path,
    tarball_path: &Path,
    on_progress: &dyn Fn(u64, u64),
) -> Result<(), UpdateError> {
    ctx.fs
        .create_dir_all(temp_dir)
        .map_err(|e| UpdateError::NetworkError {
            reason: format!("failed to create temp dir: {e}"),
        })?;

    let download_result = ctx.release_client.download_tarball(
        target.as_str(),
        artifact.as_str(),
        tarball_path,
        on_progress,
    );

    if let Err(e) = download_result {
        let _ = ctx.fs.remove_file(tarball_path);
        let _ = ctx.fs.remove_dir_all(temp_dir);
        if e.to_string().contains("interrupted") {
            return Err(UpdateError::DownloadInterrupted);
        }
        return Err(UpdateError::NetworkError {
            reason: e.to_string(),
        });
    }

    // Verify checksum
    let checksum = ctx
        .release_client
        .verify_checksum(target.as_str(), artifact.as_str(), tarball_path)
        .map_err(|e| UpdateError::NetworkError {
            reason: format!("checksum verification error: {e}"),
        })?;

    if checksum == ChecksumResult::Mismatch {
        let _ = ctx.fs.remove_dir_all(temp_dir);
        return Err(UpdateError::ChecksumMismatch);
    }

    // Download cosign bundle and verify signature
    let bundle_path = PathBuf::from(format!("{}.bundle", tarball_path.display()));
    if let Err(e) = ctx.release_client.download_cosign_bundle(
        target.as_str(),
        artifact.as_str(),
        &bundle_path,
    ) {
        let _ = ctx.fs.remove_dir_all(temp_dir);
        return Err(UpdateError::NetworkError {
            reason: format!("failed to download cosign bundle: {e}"),
        });
    }

    let cosign = ctx
        .release_client
        .verify_cosign(
            target.as_str(),
            artifact.as_str(),
            tarball_path,
            &bundle_path,
        )
        .map_err(|_| UpdateError::CosignUnavailable)?;

    match cosign {
        CosignResult::Verified => {
            ctx.terminal.stdout_write("Cosign signature verified\n");
        }
        CosignResult::NotInstalled => {
            let _ = ctx.fs.remove_dir_all(temp_dir);
            return Err(UpdateError::CosignUnavailable);
        }
        CosignResult::Failed => {
            let _ = ctx.fs.remove_dir_all(temp_dir);
            return Err(UpdateError::SecurityVerificationFailed {
                reason: "cosign signature verification failed".to_string(),
            });
        }
    }

    Ok(())
}

/// Extract the tarball and swap binaries into the install directory.
fn extract_and_swap(
    ctx: &UpdateContext<'_>,
    tarball_path: &Path,
    temp_dir: &Path,
    install_dir: &Path,
) -> Result<Vec<(PathBuf, PathBuf)>, UpdateError> {
    let extraction_dir = temp_dir.join("extracted");
    ctx.fs
        .create_dir_all(&extraction_dir)
        .map_err(|e| UpdateError::SwapFailed {
            reason: format!("failed to create extraction dir: {e}"),
        })?;

    ctx.extractor
        .extract(tarball_path, &extraction_dir)
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

    let swapped = swap::sequential_swap(
        ctx.fs,
        &extraction_dir,
        install_dir,
        &["ecc", "ecc-workflow"],
    )?;

    let _shim_count = swap::update_shims(ctx.fs, &extraction_dir, install_dir)?;

    Ok(swapped)
}

/// Run `ecc install` to sync config. Rollback swapped binaries on failure.
fn sync_config_or_rollback(
    ctx: &UpdateContext<'_>,
    swapped: &[(PathBuf, PathBuf)],
) -> Result<usize, UpdateError> {
    let install_result = ctx.shell.run_command("ecc", &["install"]);
    match install_result {
        Ok(output) if output.success() => Ok(output.stdout.lines().count()),
        Ok(output) => {
            let _ = swap::rollback_swapped(ctx.fs, swapped);
            Err(UpdateError::ConfigSyncFailed {
                reason: format!(
                    "ecc install failed (exit {}): {}. Rolled back. Backup available in install directory.",
                    output.exit_code, output.stderr
                ),
            })
        }
        Err(e) => {
            let _ = swap::rollback_swapped(ctx.fs, swapped);
            Err(UpdateError::ConfigSyncFailed {
                reason: format!(
                    "ecc install failed: {e}. Rolled back. Backup available in install directory."
                ),
            })
        }
    }
}

#[cfg(test)]
#[path = "orchestrator_tests.rs"]
mod tests;
