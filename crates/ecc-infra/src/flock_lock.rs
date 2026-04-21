//! POSIX `flock(2)` adapter for the [`FileLock`] port trait.
//!
//! Delegates raw flock operations to the `ecc-flock` crate, converting
//! [`ecc_flock::FlockGuard`] into the [`LockGuard`] type required by the port.
//!
//! Crash-safe: the kernel releases flock when the process exits or the
//! file descriptor is closed. [`LockGuard`]'s `Drop` impl closes the fd
//! immediately.

use ecc_ports::lock::{FileLock, LockError, LockGuard};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Production file lock using POSIX `flock(2)`.
///
/// Each lock name maps to a zero-byte file in `.claude/workflow/.locks/`.
/// Advisory locking only — does not prevent non-cooperative processes.
///
/// # Pattern
///
/// Adapter \[Hexagonal Architecture\] — implements `ecc_ports::lock::FileLock`
pub struct FlockLock;

impl FlockLock {
    fn convert_error(err: ecc_flock::FlockError) -> LockError {
        match err {
            ecc_flock::FlockError::DirCreation { path, message } => {
                LockError::DirCreation { path, message }
            }
            ecc_flock::FlockError::AcquireFailed { path, message } => {
                LockError::AcquireFailed { path, message }
            }
            ecc_flock::FlockError::Timeout(dur) => LockError::Timeout(dur),
        }
    }

    fn do_acquire(
        repo_root: &Path,
        name: &str,
        timeout: Option<Duration>,
    ) -> Result<LockGuard, LockError> {
        let flock_guard = match timeout {
            None => ecc_flock::acquire(repo_root, name).map_err(Self::convert_error)?,
            Some(dur) => ecc_flock::acquire_with_timeout(repo_root, name, dur)
                .map_err(Self::convert_error)?,
        };

        let (lock_path, raw_fd) = flock_guard.into_raw_fd();

        Ok(LockGuard::new(lock_path, i64::from(raw_fd), |fd| {
            // SAFETY: fd was obtained from a valid File in ecc_flock::do_acquire.
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
/// Delegates to [`ecc_flock::resolve_repo_root`].
pub fn resolve_repo_root(project_dir: &Path) -> PathBuf {
    ecc_flock::resolve_repo_root(project_dir)
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
        assert!(ecc_flock::lock_dir(tmp.path()).exists());
        drop(guard);
    }

    #[test]
    fn acquire_creates_lock_file() {
        let tmp = TempDir::new().unwrap();
        let lock = FlockLock;
        let guard = lock.acquire(tmp.path(), "test-file").unwrap();
        let expected = ecc_flock::lock_dir(tmp.path()).join("test-file.lock");
        assert!(expected.exists());
        drop(guard);
    }

    #[test]
    fn acquire_and_release_round_trip() {
        let tmp = TempDir::new().unwrap();
        let lock = FlockLock;

        let guard = lock.acquire(tmp.path(), "round-trip").unwrap();
        let expected = ecc_flock::lock_dir(tmp.path()).join("round-trip.lock");
        assert_eq!(guard.lock_path, expected);
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
