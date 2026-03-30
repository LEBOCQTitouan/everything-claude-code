use super::DevError;
use ecc_domain::config::dev_profile::MANAGED_DIRS;
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

/// Tracks a completed symlink creation for rollback purposes.
struct CompletedOp {
    link: std::path::PathBuf,
}

/// Switch to the given `profile`, updating managed directories accordingly.
///
/// - `Dev`:     creates symlinks `claude_dir/dir → ecc_root/dir` for each managed dir.
/// - `Default`: removes any existing symlinks then calls `dev_on` to reinstall copies.
/// - `dry_run`: prints planned operations without executing them.
pub fn dev_switch<F: FileSystem, T: TerminalIO>(
    fs: &F,
    terminal: &T,
    ecc_root: &Path,
    claude_dir: &Path,
    profile: ecc_domain::config::dev_profile::DevProfile,
    dry_run: bool,
) -> Result<(), DevError> {
    use ecc_domain::config::dev_profile::DevProfile;

    if !ecc_root.is_absolute() {
        return Err(DevError::RelativePath(ecc_root.to_path_buf()));
    }
    if !claude_dir.is_absolute() {
        return Err(DevError::RelativePath(claude_dir.to_path_buf()));
    }

    match profile {
        DevProfile::Dev => dev_switch_to_dev(fs, terminal, ecc_root, claude_dir, dry_run),
        DevProfile::Default => dev_switch_to_default(fs, terminal, ecc_root, claude_dir, dry_run),
    }
}

fn validate_dev_targets<F: FileSystem>(fs: &F, ecc_root: &Path) -> Result<(), DevError> {
    for dir in MANAGED_DIRS {
        let target = ecc_root.join(dir);
        if !target.starts_with(ecc_root) {
            return Err(DevError::PathEscape(target));
        }
        if !fs.exists(&target) {
            return Err(DevError::TargetNotFound(target));
        }
    }
    Ok(())
}

fn rollback_completed<F: FileSystem>(fs: &F, completed: &[CompletedOp]) {
    for op in completed {
        if let Err(e) = fs.remove_file(&op.link) {
            tracing::error!(
                "dev_switch rollback: failed to remove {}: {e}",
                op.link.display()
            );
        }
    }
}

fn dev_switch_to_dev<F: FileSystem, T: TerminalIO>(
    fs: &F,
    terminal: &T,
    ecc_root: &Path,
    claude_dir: &Path,
    dry_run: bool,
) -> Result<(), DevError> {
    validate_dev_targets(fs, ecc_root)?;

    if dry_run {
        for dir in MANAGED_DIRS {
            let target = ecc_root.join(dir);
            let link = claude_dir.join(dir);
            terminal.stdout_write(&format!("[dry-run] symlink {link:?} → {target:?}\n"));
        }
        return Ok(());
    }

    let mut completed: Vec<CompletedOp> = Vec::new();

    for dir in MANAGED_DIRS {
        let target = ecc_root.join(dir);
        let link = claude_dir.join(dir);

        if fs.is_symlink(&link) {
            fs.remove_file(&link)?;
        }
        if fs.is_dir(&link) {
            fs.remove_dir_all(&link)?;
        }

        if let Err(e) = fs.create_symlink(&target, &link) {
            rollback_completed(fs, &completed);
            return Err(DevError::Fs(e));
        }
        completed.push(CompletedOp { link });
    }

    Ok(())
}

fn dev_switch_to_default<F: FileSystem, T: TerminalIO>(
    fs: &F,
    terminal: &T,
    _ecc_root: &Path,
    claude_dir: &Path,
    dry_run: bool,
) -> Result<(), DevError> {
    // Remove existing symlinks for managed dirs
    for dir in MANAGED_DIRS {
        let link = claude_dir.join(dir);
        if fs.is_symlink(&link) {
            if dry_run {
                terminal.stdout_write(&format!("Would remove symlink: {}\n", link.display()));
            } else {
                fs.remove_file(&link)?;
            }
        }
    }
    // Note: the CLI layer calls dev_on after this to reinstall copied files
    Ok(())
}

#[cfg(test)]
#[path = "switch_tests.rs"]
mod tests;
