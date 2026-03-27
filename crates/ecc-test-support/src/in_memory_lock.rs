//! In-memory test double for the [`FileLock`] port trait.
//!
//! Does NOT block on contention — returns [`LockError::AcquireFailed`] instead.
//! This is the correct behavior for single-threaded unit tests.

use ecc_ports::lock::{FileLock, LockError, LockGuard};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// In-memory file lock for deterministic unit testing.
///
/// Tracks which lock names are currently held. Acquire fails immediately
/// if the lock is already held (no blocking, no timeout wait).
#[derive(Debug, Clone)]
pub struct InMemoryLock {
    held: Arc<Mutex<HashSet<String>>>,
}

impl InMemoryLock {
    pub fn new() -> Self {
        Self {
            held: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Query whether a lock name is currently held (test assertion helper).
    pub fn is_held(&self, name: &str) -> bool {
        self.held.lock().unwrap().contains(name)
    }
}

impl Default for InMemoryLock {
    fn default() -> Self {
        Self::new()
    }
}

impl FileLock for InMemoryLock {
    fn acquire(&self, _repo_root: &Path, name: &str) -> Result<LockGuard, LockError> {
        let mut held = self.held.lock().unwrap();
        if held.contains(name) {
            return Err(LockError::AcquireFailed {
                path: PathBuf::from(format!(".locks/{name}.lock")),
                message: "lock already held (in-memory)".to_string(),
            });
        }
        held.insert(name.to_string());

        let held_clone = self.held.clone();
        let name_owned = name.to_string();

        Ok(LockGuard::new(
            PathBuf::from(format!(".locks/{name}.lock")),
            -1,
            move |_fd| {
                let mut held = held_clone.lock().unwrap();
                held.remove(&name_owned);
            },
        ))
    }

    fn acquire_with_timeout(
        &self,
        repo_root: &Path,
        name: &str,
        _timeout: Duration,
    ) -> Result<LockGuard, LockError> {
        // In-memory double ignores timeout — same as blocking acquire
        self.acquire(repo_root, name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_and_release_round_trip() {
        let lock = InMemoryLock::new();
        let guard = lock.acquire(Path::new("/tmp"), "test").unwrap();
        assert!(lock.is_held("test"));
        drop(guard);
        assert!(!lock.is_held("test"));
    }

    #[test]
    fn double_acquire_fails() {
        let lock = InMemoryLock::new();
        let _guard = lock.acquire(Path::new("/tmp"), "test").unwrap();
        let result = lock.acquire(Path::new("/tmp"), "test");
        assert!(result.is_err());
    }

    #[test]
    fn release_then_reacquire() {
        let lock = InMemoryLock::new();
        let guard = lock.acquire(Path::new("/tmp"), "test").unwrap();
        drop(guard);
        let guard2 = lock.acquire(Path::new("/tmp"), "test");
        assert!(guard2.is_ok());
    }

    #[test]
    fn different_names_independent() {
        let lock = InMemoryLock::new();
        let _g1 = lock.acquire(Path::new("/tmp"), "alpha").unwrap();
        let _g2 = lock.acquire(Path::new("/tmp"), "beta").unwrap();
        assert!(lock.is_held("alpha"));
        assert!(lock.is_held("beta"));
    }

    #[test]
    fn is_held_reflects_state() {
        let lock = InMemoryLock::new();
        assert!(!lock.is_held("test"));
        let guard = lock.acquire(Path::new("/tmp"), "test").unwrap();
        assert!(lock.is_held("test"));
        drop(guard);
        assert!(!lock.is_held("test"));
    }
}
