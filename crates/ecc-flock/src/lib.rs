//! Shared POSIX `flock(2)` utility for ECC crates.
//!
//! Provides a low-level [`FlockGuard`] RAII handle and helper functions
//! for creating and acquiring advisory file locks.

use std::path::{Path, PathBuf};
use std::time::Duration;

/// Error variants for flock operations.
#[derive(Debug)]
pub enum FlockError {
    DirCreation { path: PathBuf, message: String },
    AcquireFailed { path: PathBuf, message: String },
    Timeout(Duration),
}

impl std::fmt::Display for FlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DirCreation { path, message } => {
                write!(
                    f,
                    "failed to create lock dir {}: {}",
                    path.display(),
                    message
                )
            }
            Self::AcquireFailed { path, message } => {
                write!(f, "failed to acquire lock {}: {}", path.display(), message)
            }
            Self::Timeout(dur) => write!(f, "lock acquire timed out after {:?}", dur),
        }
    }
}

impl std::error::Error for FlockError {}

/// RAII guard that holds an open file descriptor for a POSIX flock.
///
/// The lock is released and the file descriptor is closed when this guard
/// is dropped.
pub struct FlockGuard {
    lock_path: PathBuf,
    fd: i32,
}

impl Drop for FlockGuard {
    fn drop(&mut self) {
        if self.fd >= 0 {
            // SAFETY: fd was obtained from a valid open File in acquire/acquire_with_timeout.
            unsafe {
                libc::flock(self.fd, libc::LOCK_UN);
                libc::close(self.fd);
            }
        }
    }
}

impl FlockGuard {
    /// Returns the path to the lock file.
    pub fn lock_path(&self) -> &Path {
        &self.lock_path
    }

    /// Consumes the guard and returns the raw file descriptor without releasing the lock.
    ///
    /// The caller takes responsibility for calling `libc::flock(fd, LOCK_UN)` and
    /// `libc::close(fd)` when the lock should be released.
    pub fn into_raw_fd(mut self) -> (PathBuf, i32) {
        let fd = self.fd;
        let path = self.lock_path.clone();
        // Prevent double-close: set fd to -1 before forgetting the guard.
        self.fd = -1;
        std::mem::forget(self);
        (path, fd)
    }
}

/// Returns the `.locks` directory path for a given repo root.
pub fn lock_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(".claude/workflow/.locks")
}

/// Creates the `.locks` directory if it does not already exist.
pub fn ensure_lock_dir(repo_root: &Path) -> Result<(), FlockError> {
    let dir = lock_dir(repo_root);
    std::fs::create_dir_all(&dir).map_err(|e| FlockError::DirCreation {
        path: dir,
        message: e.to_string(),
    })
}

/// Acquires an exclusive POSIX flock on `<repo_root>/.claude/workflow/.locks/<name>.lock`.
///
/// Blocks indefinitely until the lock can be acquired.
pub fn acquire(repo_root: &Path, name: &str) -> Result<FlockGuard, FlockError> {
    do_acquire(repo_root, name, None)
}

/// Acquires an exclusive POSIX flock, retrying until `timeout` elapses.
pub fn acquire_with_timeout(
    repo_root: &Path,
    name: &str,
    timeout: Duration,
) -> Result<FlockGuard, FlockError> {
    do_acquire(repo_root, name, Some(timeout))
}

fn do_acquire(
    repo_root: &Path,
    name: &str,
    timeout: Option<Duration>,
) -> Result<FlockGuard, FlockError> {
    use std::fs::OpenOptions;
    use std::os::unix::io::AsRawFd;
    use std::time::Instant;

    ensure_lock_dir(repo_root)?;
    let lock_path = lock_dir(repo_root).join(format!("{name}.lock"));

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(|e| FlockError::AcquireFailed {
            path: lock_path.clone(),
            message: e.to_string(),
        })?;

    let raw_fd = file.as_raw_fd();

    match timeout {
        None => {
            // SAFETY: raw_fd is valid, obtained from an open File.
            let ret = unsafe { libc::flock(raw_fd, libc::LOCK_EX) };
            if ret != 0 {
                return Err(FlockError::AcquireFailed {
                    path: lock_path,
                    message: std::io::Error::last_os_error().to_string(),
                });
            }
        }
        Some(dur) => {
            let deadline = Instant::now() + dur;
            loop {
                // SAFETY: raw_fd is valid, obtained from an open File.
                let ret = unsafe { libc::flock(raw_fd, libc::LOCK_EX | libc::LOCK_NB) };
                if ret == 0 {
                    break;
                }
                let err = std::io::Error::last_os_error();
                if err.kind() != std::io::ErrorKind::WouldBlock {
                    return Err(FlockError::AcquireFailed {
                        path: lock_path,
                        message: err.to_string(),
                    });
                }
                if Instant::now() >= deadline {
                    return Err(FlockError::Timeout(dur));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }

    // Leak the File to keep the fd open. The FlockGuard's drop will close it.
    std::mem::forget(file);

    Ok(FlockGuard {
        lock_path,
        fd: raw_fd,
    })
}

/// Resolves the main repo root from inside a worktree or subdirectory.
///
/// Uses `git rev-parse --git-common-dir` to find the shared `.git` directory.
/// Falls back to `project_dir` if git is unavailable or not in a repo.
pub fn resolve_repo_root(project_dir: &Path) -> PathBuf {
    std::process::Command::new("git")
        .args(["rev-parse", "--git-common-dir"])
        .current_dir(project_dir)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let git_common = String::from_utf8(o.stdout).ok()?;
                let git_common = git_common.trim();
                let git_path = Path::new(git_common);
                let abs_git = if git_path.is_absolute() {
                    git_path.to_path_buf()
                } else {
                    project_dir.join(git_common)
                };
                abs_git.parent().map(|p| p.to_path_buf())
            } else {
                None
            }
        })
        .unwrap_or_else(|| project_dir.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn guard_drop_releases() {
        let tmp = TempDir::new().unwrap();
        let guard = acquire(tmp.path(), "drop-test").unwrap();
        drop(guard);
        // Re-acquire should succeed immediately (lock was released)
        let guard2 = acquire(tmp.path(), "drop-test").unwrap();
        drop(guard2);
    }

    #[test]
    fn acquire_creates_lock() {
        let tmp = TempDir::new().unwrap();
        let guard = acquire(tmp.path(), "test-create").unwrap();
        // .locks/ dir exists
        assert!(lock_dir(tmp.path()).exists());
        // .lock file exists
        let lock_file = lock_dir(tmp.path()).join("test-create.lock");
        assert!(lock_file.exists());
        drop(guard);
    }

    #[test]
    fn acquire_with_timeout_succeeds_when_free() {
        let tmp = TempDir::new().unwrap();
        let guard =
            acquire_with_timeout(tmp.path(), "timeout-free", Duration::from_secs(1)).unwrap();
        drop(guard);
    }

    #[test]
    fn resolve_repo_root_outside_git() {
        let tmp = TempDir::new().unwrap();
        let result = resolve_repo_root(tmp.path());
        assert_eq!(result, tmp.path().to_path_buf());
    }
}
