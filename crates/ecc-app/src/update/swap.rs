use ecc_domain::update::UpdateError;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use std::path::{Path, PathBuf};

/// Perform an atomic binary swap: backup the existing binary, rename the new
/// one into place, and set executable permissions.
///
/// The temp file must be co-located on the same filesystem as `target_path`.
/// Returns the backup path on success.
pub fn atomic_swap(
    fs: &dyn FileSystem,
    new_binary: &Path,
    target_path: &Path,
) -> Result<PathBuf, UpdateError> {
    let backup_path = target_path.with_extension("bak");

    // Backup existing binary (if it exists)
    if fs.exists(target_path) {
        fs.copy(target_path, &backup_path).map_err(|e| UpdateError::SwapFailed {
            reason: format!("backup failed: {e}"),
        })?;
    }

    // Atomic rename
    fs.rename(new_binary, target_path).map_err(|e| {
        // Attempt to restore backup on failure
        let _ = restore_backup(fs, &backup_path, target_path);
        UpdateError::SwapFailed {
            reason: format!("rename failed: {e}"),
        }
    })?;

    // Set executable permissions
    fs.set_permissions(target_path, 0o755).map_err(|e| UpdateError::SwapFailed {
        reason: format!("set_permissions failed: {e}"),
    })?;

    Ok(backup_path)
}

/// Restore a backup binary to its original location.
pub fn restore_backup(
    fs: &dyn FileSystem,
    backup_path: &Path,
    target_path: &Path,
) -> Result<(), UpdateError> {
    if fs.exists(backup_path) {
        fs.rename(backup_path, target_path).map_err(|e| UpdateError::BackupRestoreFailed {
            reason: format!("restore from {} failed: {e}", backup_path.display()),
        })?;
    }
    Ok(())
}

/// Swap both ecc and ecc-workflow binaries sequentially.
/// Returns a vec of (target_path, backup_path) pairs.
pub fn sequential_swap(
    fs: &dyn FileSystem,
    extraction_dir: &Path,
    install_dir: &Path,
    binaries: &[&str],
) -> Result<Vec<(PathBuf, PathBuf)>, UpdateError> {
    let mut swapped = Vec::new();

    for name in binaries {
        let new_path = extraction_dir.join("bin").join(name);
        let target = install_dir.join(name);

        if !fs.exists(&new_path) {
            // Binary not in tarball — skip silently (e.g., ecc-workflow optional)
            continue;
        }

        let backup = atomic_swap(fs, &new_path, &target)?;
        swapped.push((target, backup));
    }

    Ok(swapped)
}

/// Update shell shims from the extraction directory.
pub fn update_shims(
    fs: &dyn FileSystem,
    extraction_dir: &Path,
    install_dir: &Path,
) -> Result<usize, UpdateError> {
    let shims = ["ecc-hook", "ecc-shell-hook.sh"];
    let mut updated = 0;

    for shim in &shims {
        let src = extraction_dir.join("bin").join(shim);
        let dst = install_dir.join(shim);

        if fs.exists(&src) {
            fs.copy(&src, &dst).map_err(|e| UpdateError::SwapFailed {
                reason: format!("shim update failed for {shim}: {e}"),
            })?;
            fs.set_permissions(&dst, 0o755).map_err(|e| UpdateError::SwapFailed {
                reason: format!("shim permissions failed for {shim}: {e}"),
            })?;
            updated += 1;
        }
    }

    Ok(updated)
}

/// Detect a partial update by comparing ecc and ecc-workflow versions.
pub fn detect_partial_update(
    shell: &dyn ShellExecutor,
    install_dir: &Path,
) -> Result<bool, UpdateError> {
    let ecc_path = install_dir.join("ecc");
    let workflow_path = install_dir.join("ecc-workflow");

    let ecc_version = shell
        .run_command(ecc_path.to_str().unwrap_or("ecc"), &["version"])
        .map(|o| o.stdout.trim().to_string())
        .unwrap_or_default();

    let workflow_version = shell
        .run_command(workflow_path.to_str().unwrap_or("ecc-workflow"), &["--version"])
        .map(|o| o.stdout.trim().to_string())
        .unwrap_or_default();

    // If both exist but differ, it's a partial update
    Ok(!ecc_version.is_empty() && !workflow_version.is_empty() && ecc_version != workflow_version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{InMemoryFileSystem, MockExecutor};
    use ecc_ports::shell::CommandOutput;

    fn setup_fs_with_binary(name: &str) -> InMemoryFileSystem {
        let fs = InMemoryFileSystem::new();
        let install_dir = Path::new("/usr/local/bin");
        let _ = fs.create_dir_all(install_dir);
        let _ = fs.write(
            &install_dir.join(name),
            "old-binary-content",
        );
        let extraction_dir = Path::new("/tmp/ecc-update");
        let _ = fs.create_dir_all(&extraction_dir.join("bin"));
        let _ = fs.write(
            &extraction_dir.join("bin").join(name),
            "new-binary-content",
        );
        fs
    }

    #[test]
    fn colocated_temp_file() {
        // The swap function expects the new binary to be in an extraction dir
        // that we control — temp files are co-located by the orchestrator.
        let fs = setup_fs_with_binary("ecc");
        let new_bin = Path::new("/tmp/ecc-update/bin/ecc");
        let target = Path::new("/usr/local/bin/ecc");

        let result = atomic_swap(&fs, new_bin, target);
        assert!(result.is_ok());
        // Backup is co-located with the target
        let backup = result.unwrap();
        assert_eq!(backup.parent(), target.parent());
    }

    #[test]
    fn backup_rename_permissions() {
        let fs = setup_fs_with_binary("ecc");
        let new_bin = Path::new("/tmp/ecc-update/bin/ecc");
        let target = Path::new("/usr/local/bin/ecc");

        let backup_path = atomic_swap(&fs, new_bin, target).unwrap();

        // Backup exists
        assert!(fs.exists(&backup_path));
        // Target has new content
        let content = fs.read_to_string(target).unwrap();
        assert_eq!(content, "new-binary-content");
        // Target is executable
        assert!(fs.is_executable(target));
    }

    #[test]
    fn failure_restores_backup() {
        let fs = InMemoryFileSystem::new();
        let _ = fs.create_dir_all(Path::new("/usr/local/bin"));
        let _ = fs.write(Path::new("/usr/local/bin/ecc"), "old-binary");

        // The new binary doesn't exist — rename will fail
        let result = atomic_swap(
            &fs,
            Path::new("/nonexistent/ecc"),
            Path::new("/usr/local/bin/ecc"),
        );
        assert!(result.is_err());

        // Original binary should be restored from backup
        let target = Path::new("/usr/local/bin/ecc");
        assert!(fs.exists(target), "original binary should be restored after failed swap");
        assert_eq!(fs.read_to_string(target).unwrap(), "old-binary");
    }

    #[test]
    fn sequential_binary_swap() {
        let fs = InMemoryFileSystem::new();
        let install = Path::new("/usr/local/bin");
        let extract = Path::new("/tmp/ecc-update");
        let _ = fs.create_dir_all(install);
        let _ = fs.create_dir_all(&extract.join("bin"));

        // Set up both binaries
        for name in ["ecc", "ecc-workflow"] {
            let _ = fs.write(&install.join(name), "old");
            let _ = fs.write(&extract.join("bin").join(name), "new");
        }

        let result = sequential_swap(&fs, extract, install, &["ecc", "ecc-workflow"]);
        assert!(result.is_ok());
        let swapped = result.unwrap();
        assert_eq!(swapped.len(), 2);

        // Both updated
        assert_eq!(fs.read_to_string(&install.join("ecc")).unwrap(), "new");
        assert_eq!(fs.read_to_string(&install.join("ecc-workflow")).unwrap(), "new");
    }

    #[test]
    fn shims_updated() {
        let fs = InMemoryFileSystem::new();
        let install = Path::new("/usr/local/bin");
        let extract = Path::new("/tmp/ecc-update");
        let _ = fs.create_dir_all(install);
        let _ = fs.create_dir_all(&extract.join("bin"));

        let _ = fs.write(&extract.join("bin/ecc-hook"), "new-hook");
        let _ = fs.write(&extract.join("bin/ecc-shell-hook.sh"), "new-shell-hook");

        let count = update_shims(&fs, extract, install).unwrap();
        assert_eq!(count, 2);
        assert!(fs.is_executable(&install.join("ecc-hook")));
        assert!(fs.is_executable(&install.join("ecc-shell-hook.sh")));
    }

    #[test]
    fn rollback_swapped_restores() {
        // PC-011: rollback_swapped restores all binaries from .bak backups
        let fs = InMemoryFileSystem::new();
        let _ = fs.create_dir_all(Path::new("/usr/local/bin"));

        // Set up two targets and their .bak backups (simulating a swapped state)
        let target1 = PathBuf::from("/usr/local/bin/ecc");
        let backup1 = PathBuf::from("/usr/local/bin/ecc.bak");
        let target2 = PathBuf::from("/usr/local/bin/ecc-workflow");
        let backup2 = PathBuf::from("/usr/local/bin/ecc-workflow.bak");

        // Write backup files (the originals saved before swap)
        let _ = fs.write(&backup1, "original-ecc");
        let _ = fs.write(&backup2, "original-ecc-workflow");
        // Write current targets (the newly swapped binaries)
        let _ = fs.write(&target1, "new-ecc");
        let _ = fs.write(&target2, "new-ecc-workflow");

        let swapped = vec![
            (target1.clone(), backup1.clone()),
            (target2.clone(), backup2.clone()),
        ];

        let result = rollback_swapped(&fs, &swapped);
        assert!(result.is_ok(), "rollback_swapped should succeed: {result:?}");

        // Targets should now be restored to original content
        assert_eq!(fs.read_to_string(&target1).unwrap(), "original-ecc");
        assert_eq!(fs.read_to_string(&target2).unwrap(), "original-ecc-workflow");
        // Backups should be gone (renamed to targets)
        assert!(!fs.exists(&backup1));
        assert!(!fs.exists(&backup2));
    }

    #[test]
    fn detects_partial_update() {
        let shell = MockExecutor::new()
            .on_args("/install/ecc", &["version"], CommandOutput {
                stdout: "4.3.0\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            })
            .on_args("/install/ecc-workflow", &["--version"], CommandOutput {
                stdout: "4.2.0\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            });

        let result = detect_partial_update(&shell, Path::new("/install")).unwrap();
        assert!(result, "should detect version mismatch as partial update");
    }
}
