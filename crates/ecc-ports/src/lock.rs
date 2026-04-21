use std::path::{Path, PathBuf};
use std::time::Duration;

/// Port for file-based locking.
///
/// Production: POSIX `flock(2)` via `FlockLock` in `ecc-infra`.
/// Tests: in-memory mutex map via `InMemoryLock` in `ecc-test-support`.
///
/// Lock files live in `.claude/workflow/.locks/` under the repo root.
/// The caller is responsible for resolving the repo root (especially from
/// inside a worktree) before calling `acquire`.
///
/// # Pattern
///
/// Port \[Hexagonal Architecture\]
pub trait FileLock: Send + Sync {
    /// Acquire an exclusive lock, blocking until available.
    ///
    /// Creates `.claude/workflow/.locks/{name}.lock` under `repo_root`.
    /// Returns a [`LockGuard`] that releases the lock when dropped (RAII).
    fn acquire(&self, repo_root: &Path, name: &str) -> Result<LockGuard, LockError>;

    /// Acquire an exclusive lock with a timeout.
    ///
    /// Returns [`LockError::Timeout`] if the lock is not available within `timeout`.
    fn acquire_with_timeout(
        &self,
        repo_root: &Path,
        name: &str,
        timeout: Duration,
    ) -> Result<LockGuard, LockError>;
}

/// RAII guard that releases a file lock when dropped.
///
/// The lock is released either by dropping this guard or when the process exits
/// (POSIX flock kernel semantics). Do not leak this value — always let it drop
/// or store it in a scope that ends predictably.
pub struct LockGuard {
    /// Path to the lock file (for diagnostics).
    pub lock_path: PathBuf,
    fd: i64,
    release_fn: Option<Box<dyn FnOnce(i64) + Send>>,
}

impl LockGuard {
    /// Create a new `LockGuard`. Called by `FileLock` implementations.
    pub fn new(lock_path: PathBuf, fd: i64, release_fn: impl FnOnce(i64) + Send + 'static) -> Self {
        Self {
            lock_path,
            fd,
            release_fn: Some(Box::new(release_fn)),
        }
    }

    /// Create a `LockGuard` with no release action (for test doubles).
    pub fn sentinel(lock_path: PathBuf) -> Self {
        Self {
            lock_path,
            fd: -1,
            release_fn: None,
        }
    }
}

impl std::fmt::Debug for LockGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LockGuard")
            .field("lock_path", &self.lock_path)
            .field("fd", &self.fd)
            .finish()
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        if let Some(release) = self.release_fn.take() {
            release(self.fd);
        }
    }
}

/// Errors that can occur during lock operations.
#[derive(Debug, thiserror::Error)]
pub enum LockError {
    /// Failed to create the `.locks/` directory.
    #[error("failed to create lock directory {path}: {message}")]
    DirCreation {
        /// Path to the locks directory that could not be created.
        path: PathBuf,
        /// Human-readable error description.
        message: String,
    },

    /// Failed to acquire the lock (I/O or permission error).
    #[error("failed to acquire lock on {path}: {message}")]
    AcquireFailed {
        /// Path to the lock file that could not be acquired.
        path: PathBuf,
        /// Human-readable error description.
        message: String,
    },

    /// Lock acquisition timed out.
    #[error("lock acquisition timed out after {0:?}")]
    Timeout(Duration),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_error_display_dir_creation() {
        let err = LockError::DirCreation {
            path: PathBuf::from("/tmp/.locks"),
            message: "permission denied".to_string(),
        };
        assert!(err.to_string().contains("lock directory"));
        assert!(err.to_string().contains("/tmp/.locks"));
    }

    #[test]
    fn lock_error_display_acquire_failed() {
        let err = LockError::AcquireFailed {
            path: PathBuf::from("/tmp/.locks/test.lock"),
            message: "resource busy".to_string(),
        };
        assert!(err.to_string().contains("acquire lock"));
    }

    #[test]
    fn lock_error_display_timeout() {
        let err = LockError::Timeout(Duration::from_secs(60));
        assert!(err.to_string().contains("timed out"));
        assert!(err.to_string().contains("60"));
    }

    #[test]
    fn lock_guard_debug_format() {
        let guard = LockGuard::sentinel(PathBuf::from("/tmp/test.lock"));
        let debug = format!("{guard:?}");
        assert!(debug.contains("test.lock"));
    }

    #[test]
    fn lock_guard_drop_calls_release_fn() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        {
            let _guard = LockGuard::new(PathBuf::from("/tmp/test.lock"), 42, move |fd| {
                assert_eq!(fd, 42);
                released_clone.store(true, Ordering::SeqCst);
            });
            assert!(!released.load(Ordering::SeqCst));
        } // guard dropped here

        assert!(released.load(Ordering::SeqCst));
    }
}
