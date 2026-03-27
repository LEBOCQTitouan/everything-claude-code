//! POSIX `flock(2)` adapter for the [`FileLock`] port trait.
//!
//! Crash-safe: the kernel releases flock when the process exits or the
//! file descriptor is closed. [`LockGuard`]'s `Drop` impl closes the fd
//! immediately.

use ecc_ports::lock::{FileLock, LockError, LockGuard};
use std::fs::{self, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Production file lock using POSIX `flock(2)`.
///
/// Each lock name maps to a zero-byte file in `.claude/workflow/.locks/`.
/// Advisory locking only — does not prevent non-cooperative processes.
pub struct FlockLock;

impl FlockLock {
    fn lock_dir(repo_root: &Path) -> PathBuf {
        repo_root.join(".claude/workflow/.locks")
    }

    fn ensure_lock_dir(repo_root: &Path) -> Result<(), LockError> {
        let dir = Self::lock_dir(repo_root);
        fs::create_dir_all(&dir).map_err(|e| LockError::DirCreation {
            path: dir,
            message: e.to_string(),
        })
    }

    fn lock_path(repo_root: &Path, name: &str) -> PathBuf {
        Self::lock_dir(repo_root).join(format!("{name}.lock"))
    }

    fn do_acquire(
        repo_root: &Path,
        name: &str,
        timeout: Option<Duration>,
    ) -> Result<LockGuard, LockError> {
        Self::ensure_lock_dir(repo_root)?;
        let lock_path = Self::lock_path(repo_root, name);

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|e| LockError::AcquireFailed {
                path: lock_path.clone(),
                message: e.to_string(),
            })?;

        let raw_fd = file.as_raw_fd();

        match timeout {
            None => {
                // Blocking acquire (LOCK_EX)
                // SAFETY: raw_fd is valid, obtained from an open File.
                let ret = unsafe { libc::flock(raw_fd, libc::LOCK_EX) };
                if ret != 0 {
                    return Err(LockError::AcquireFailed {
                        path: lock_path,
                        message: std::io::Error::last_os_error().to_string(),
                    });
                }
            }
            Some(dur) => {
                // Non-blocking acquire with retry loop
                let deadline = Instant::now() + dur;
                loop {
                    // SAFETY: raw_fd is valid, obtained from an open File.
                    let ret =
                        unsafe { libc::flock(raw_fd, libc::LOCK_EX | libc::LOCK_NB) };
                    if ret == 0 {
                        break; // Lock acquired
                    }
                    let err = std::io::Error::last_os_error();
                    if err.kind() != std::io::ErrorKind::WouldBlock {
                        return Err(LockError::AcquireFailed {
                            path: lock_path,
                            message: err.to_string(),
                        });
                    }
                    if Instant::now() >= deadline {
                        return Err(LockError::Timeout(dur));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        }

        // Leak the File to keep the fd open. The LockGuard's drop will close it.
        std::mem::forget(file);

        Ok(LockGuard::new(lock_path, i64::from(raw_fd), |fd| {
            // SAFETY: fd was obtained from a valid File in do_acquire.
            unsafe {
                libc::flock(fd as i32, libc::LOCK_UN);
                libc::close(fd as i32);
            }
        }))
    }
}

impl FileLock for FlockLock {
    fn acquire(&self, repo_root: &Path, name: &str) -> Result<LockGuard, LockError> {
        Self::do_acquire(repo_root, name, None)
    }

    fn acquire_with_timeout(
        &self,
        repo_root: &Path,
        name: &str,
        timeout: Duration,
    ) -> Result<LockGuard, LockError> {
        Self::do_acquire(repo_root, name, Some(timeout))
    }
}

/// Resolve the main repo root from inside a worktree or subdirectory.
///
/// Uses `git rev-parse --git-common-dir` which returns the path to the
/// shared `.git` directory. The repo root is the parent of `.git`.
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
                // .git dir → repo root is its parent
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
    use ecc_ports::lock::FileLock;
    use tempfile::TempDir;

    #[test]
    fn acquire_creates_lock_dir() {
        let tmp = TempDir::new().unwrap();
        let lock = FlockLock;
        let guard = lock.acquire(tmp.path(), "test-dir").unwrap();
        assert!(FlockLock::lock_dir(tmp.path()).exists());
        drop(guard);
    }

    #[test]
    fn acquire_creates_lock_file() {
        let tmp = TempDir::new().unwrap();
        let lock = FlockLock;
        let guard = lock.acquire(tmp.path(), "test-file").unwrap();
        assert!(FlockLock::lock_path(tmp.path(), "test-file").exists());
        drop(guard);
    }

    #[test]
    fn acquire_and_release_round_trip() {
        let tmp = TempDir::new().unwrap();
        let lock = FlockLock;

        let guard = lock.acquire(tmp.path(), "round-trip").unwrap();
        assert_eq!(
            guard.lock_path,
            FlockLock::lock_path(tmp.path(), "round-trip")
        );
        drop(guard); // releases via RAII
    }

    #[test]
    fn acquire_with_timeout_succeeds_when_free() {
        let tmp = TempDir::new().unwrap();
        let lock = FlockLock;

        let guard = lock
            .acquire_with_timeout(tmp.path(), "timeout-free", Duration::from_secs(1))
            .unwrap();
        drop(guard);
    }

    #[test]
    fn guard_drop_releases_lock() {
        let tmp = TempDir::new().unwrap();
        let lock = FlockLock;

        // Acquire and drop
        let guard = lock.acquire(tmp.path(), "drop-test").unwrap();
        drop(guard);

        // Re-acquire should succeed immediately (lock was released)
        let guard2 = lock.acquire(tmp.path(), "drop-test").unwrap();
        drop(guard2);
    }

    #[test]
    fn resolve_repo_root_returns_project_dir_outside_git() {
        let tmp = TempDir::new().unwrap();
        let result = resolve_repo_root(tmp.path());
        assert_eq!(result, tmp.path().to_path_buf());
    }

    #[test]
    fn resolve_repo_root_returns_actual_repo_root() {
        // This test runs inside the ECC repo itself
        let cwd = std::env::current_dir().unwrap();
        let result = resolve_repo_root(&cwd);
        // Should resolve to the workspace root (contains Cargo.toml)
        assert!(result.join("Cargo.toml").exists());
    }
}
