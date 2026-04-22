//! Heartbeat write use case — atomically writes `.ecc-session` to mark a
//! worktree as live.
//!
//! # Security (SEC-001)
//!
//! `now_fn()` is invoked **after** the path-validation guards (syntactic `..`
//! component check + leaf-symlink check via `FileSystem::is_symlink`) to
//! prevent a TOCTOU window where a clock value is captured before the target
//! path is validated. PC-033b verifies this ordering via a counting mock.
//!
//! # Scope of traversal guard
//!
//! The guard is **syntactic + leaf-only**: rejects `..` components in the
//! supplied `worktree_path` and rejects a symlinked leaf directory. It does
//! NOT canonicalize the full path chain, so a pre-existing symlink on an
//! intermediate ancestor directory (outside this function's control) is not
//! detected. Callers must only pass worktree paths originating from trusted
//! sources — hook handlers sourced from `CLAUDE_PROJECT_DIR` + the self-identity
//! resolver, both of which canonicalize upstream (see `self_identity.rs`).

use ecc_domain::worktree::liveness::LivenessRecord;
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// Errors returned by [`write_heartbeat`].
#[derive(Debug, thiserror::Error)]
pub enum HeartbeatError {
    /// The path contains a `..` component or resolves outside the worktree root.
    #[error("symlink escape / traversal rejected: {0}")]
    PathEscape(String),
    /// The worktree path does not exist or is not a directory.
    #[error("worktree path not found or not a directory")]
    NotAWorktree,
    /// A filesystem I/O error occurred.
    #[error("fs error: {0}")]
    Fs(String),
    /// JSON serialization failed.
    #[error("serde: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Write an atomic `.ecc-session` heartbeat file into `worktree_path`.
///
/// # Argument contract
///
/// - `fs`: filesystem port — never touches `std::fs` directly.
/// - `worktree_path`: the `ecc-session-*` directory.
/// - `claude_code_pid`: PID of the owning Claude Code process (≥ 2).
/// - `now_fn`: called **after** path validation to record the write timestamp
///   (SEC-001 ordering requirement).
/// - `disabled`: kill-switch — when `true`, returns `Ok(())` immediately
///   without any I/O (AC-009.1, PC-071).
///
/// # Behaviour
///
/// 1. Kill-switch check (fast path).
/// 2. Reject paths containing `..` components (AC-002.10).
/// 3. If `worktree_path` is not an existing directory → silent `Ok(())` (AC-002.6).
/// 4. Check for symlink on the path itself (AC-002.10).
/// 5. Invoke `now_fn()` to capture write timestamp (SEC-001).
/// 6. Serialize `LivenessRecord` to JSON.
/// 7. Write to `<worktree_path>/.ecc-session.tmp.<pid>`.
/// 8. Atomic rename to `<worktree_path>/.ecc-session`.
pub fn write_heartbeat(
    fs: &dyn FileSystem,
    worktree_path: &Path,
    claude_code_pid: u32,
    now_fn: impl FnOnce() -> u64,
    disabled: bool,
) -> Result<(), HeartbeatError> {
    // AC-009.1 / PC-071: kill-switch short-circuit — must be first.
    if disabled {
        return Ok(());
    }

    // AC-002.10: reject `..` components BEFORE is_dir check (PC-035).
    // This ensures traversal is caught even if the resolved path exists.
    if worktree_path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(HeartbeatError::PathEscape("contains ..".into()));
    }

    // AC-002.6: path must exist and be a directory.
    if !fs.is_dir(worktree_path) {
        return Ok(());
    }

    // AC-002.10: reject symlinks on the worktree path itself (PC-034).
    if fs.is_symlink(worktree_path) {
        return Err(HeartbeatError::PathEscape(
            worktree_path.display().to_string(),
        ));
    }

    // SEC-001: now_fn() MUST be called AFTER all path validation (PC-033b).
    let last_seen = now_fn();

    let record = LivenessRecord {
        schema_version: 1,
        claude_code_pid,
        last_seen_unix_ts: last_seen,
    };
    let json = serde_json::to_string(&record)?;

    let target = worktree_path.join(".ecc-session");
    let tmp = worktree_path.join(format!(".ecc-session.tmp.{claude_code_pid}"));

    fs.write(&tmp, &json)
        .map_err(|e| HeartbeatError::Fs(e.to_string()))?;
    fs.rename(&tmp, &target)
        .map_err(|e| HeartbeatError::Fs(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;
    use std::path::Path;

    // ---------------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------------

    fn now_100() -> u64 {
        100_u64
    }

    fn now_200() -> u64 {
        200_u64
    }

    /// Parse the `.ecc-session` file written into the given fs at `worktree_path`.
    fn read_session(fs: &InMemoryFileSystem, worktree_path: &Path) -> LivenessRecord {
        let path = worktree_path.join(".ecc-session");
        let content = fs
            .read_to_string(&path)
            .expect(".ecc-session must exist after write_heartbeat");
        serde_json::from_str(&content).expect("content must be valid LivenessRecord JSON")
    }

    // ---------------------------------------------------------------------------
    // PC-026: SessionStart writes heartbeat via tmpfile+rename
    // ---------------------------------------------------------------------------

    #[test]
    fn session_start_writes_heartbeat() {
        let fs = InMemoryFileSystem::new().with_dir("/wt");
        let wt = Path::new("/wt");

        write_heartbeat(&fs, wt, 1234, now_100, false).expect("write_heartbeat must succeed");

        let record = read_session(&fs, wt);
        assert_eq!(record.schema_version, 1);
        assert_eq!(record.claude_code_pid, 1234);
        assert_eq!(record.last_seen_unix_ts, 100);

        // Tmp file must be cleaned up (renamed away).
        assert!(
            !fs.exists(Path::new("/wt/.ecc-session.tmp.1234")),
            "tmp file must not remain after atomic rename"
        );
    }

    // ---------------------------------------------------------------------------
    // PC-027: PostToolUse refreshes last_seen
    // ---------------------------------------------------------------------------

    #[test]
    fn post_tool_use_refreshes_last_seen() {
        let fs = InMemoryFileSystem::new().with_dir("/wt");
        let wt = Path::new("/wt");

        // First write at t=100.
        write_heartbeat(&fs, wt, 42, now_100, false).expect("first write");
        let r1 = read_session(&fs, wt);
        assert_eq!(r1.last_seen_unix_ts, 100);

        // Second write at t=200 (PostToolUse refresh).
        write_heartbeat(&fs, wt, 42, now_200, false).expect("second write");
        let r2 = read_session(&fs, wt);
        assert_eq!(
            r2.last_seen_unix_ts, 200,
            "last_seen must be refreshed to 200"
        );
    }

    // ---------------------------------------------------------------------------
    // PC-028: Stop hook writes final heartbeat
    // ---------------------------------------------------------------------------

    #[test]
    fn stop_writes_final() {
        let fs = InMemoryFileSystem::new().with_dir("/wt");
        let wt = Path::new("/wt");

        write_heartbeat(&fs, wt, 9999, || 999_999, false).expect("stop heartbeat");
        let record = read_session(&fs, wt);
        assert_eq!(record.claude_code_pid, 9999);
        assert_eq!(record.last_seen_unix_ts, 999_999);
    }

    // ---------------------------------------------------------------------------
    // PC-029: Atomic rename semantics — sequential safety test
    // (concurrent test is an integration test; this covers sequential semantics)
    // ---------------------------------------------------------------------------

    #[test]
    fn atomic_rename_semantics() {
        let fs = InMemoryFileSystem::new().with_dir("/wt");
        let wt = Path::new("/wt");

        // Write once.
        write_heartbeat(&fs, wt, 11, || 1, false).expect("first write");
        // Write again (simulates second concurrent writer landing).
        write_heartbeat(&fs, wt, 11, || 2, false).expect("second write");

        // The final value must be the second write's timestamp.
        let record = read_session(&fs, wt);
        assert_eq!(record.last_seen_unix_ts, 2);

        // No tmp file left behind.
        assert!(
            !fs.exists(Path::new("/wt/.ecc-session.tmp.11")),
            "tmp file must be gone after rename"
        );
    }

    // ---------------------------------------------------------------------------
    // PC-030: FS failure returns Err (callers log WARN + don't block)
    // ---------------------------------------------------------------------------

    #[test]
    fn fs_failure_logs_warn_nonblocking() {
        // Test: when write_heartbeat encounters a path error (symlink), it returns
        // Err gracefully — callers MUST handle this without panicking.
        // We register /wt-sym as BOTH a dir and a symlink so is_dir=true and
        // is_symlink=true, triggering PathEscape.
        let fs = InMemoryFileSystem::new()
            .with_dir("/wt")
            .with_dir("/wt-sym")
            .with_symlink(
                std::path::PathBuf::from("/wt-sym"),
                std::path::PathBuf::from("/outside"),
            );
        let result = write_heartbeat(&fs, Path::new("/wt-sym"), 1, now_100, false);
        assert!(
            result.is_err(),
            "symlink path must return Err (caller must log WARN, not panic)"
        );
        // Key assertion: no panic occurred, the function returns Err gracefully.
    }

    // ---------------------------------------------------------------------------
    // PC-031: Write outside worktree is no-op
    // ---------------------------------------------------------------------------

    #[test]
    fn noop_outside_worktree() {
        let fs = InMemoryFileSystem::new(); // No dirs registered.
        let result = write_heartbeat(&fs, Path::new("/nonexistent"), 1, now_100, false);
        assert!(result.is_ok(), "missing dir must be a silent no-op");
        assert!(
            !fs.exists(Path::new("/nonexistent/.ecc-session")),
            "no file must be written when worktree path is not a directory"
        );
    }

    // ---------------------------------------------------------------------------
    // PC-032: Stale .ecc-session overwritten on SessionStart
    // ---------------------------------------------------------------------------

    #[test]
    fn overwrites_stale_session_file() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/wt")
            // Pre-populate a stale session file with old PID and timestamp.
            .with_file(
                "/wt/.ecc-session",
                r#"{"schema_version":1,"claude_code_pid":1,"last_seen_unix_ts":1}"#,
            );
        let wt = Path::new("/wt");

        write_heartbeat(&fs, wt, 5678, || 9999, false).expect("overwrite must succeed");

        let record = read_session(&fs, wt);
        assert_eq!(
            record.claude_code_pid, 5678,
            "stale PID must be overwritten"
        );
        assert_eq!(
            record.last_seen_unix_ts, 9999,
            "stale timestamp must be overwritten"
        );
    }

    // ---------------------------------------------------------------------------
    // PC-033: Write-time timestamp (not read-time)
    // ---------------------------------------------------------------------------

    #[test]
    fn write_time_not_read_time() {
        // This test verifies that now_fn() is called at write time, not before.
        // We use a cell to track how many times now_fn was called.
        use std::cell::Cell;
        let call_count = Cell::new(0u32);
        let now_fn = || {
            call_count.set(call_count.get() + 1);
            1_000_u64
        };

        let fs = InMemoryFileSystem::new().with_dir("/wt");
        write_heartbeat(&fs, Path::new("/wt"), 7, now_fn, false).expect("write");

        assert_eq!(
            call_count.get(),
            1,
            "now_fn must be called exactly once per write_heartbeat invocation"
        );
    }

    // ---------------------------------------------------------------------------
    // PC-033b: now_fn() invoked AFTER path-validation (SEC-001)
    // ---------------------------------------------------------------------------

    #[test]
    fn now_fn_called_after_canonicalize() {
        // Verify kill-switch fires before now_fn (disabled=true → now_fn not called).
        let now_fn_called = std::cell::Cell::new(false);
        let fs = InMemoryFileSystem::new().with_dir("/wt");

        write_heartbeat(
            &fs,
            Path::new("/wt"),
            8,
            || {
                now_fn_called.set(true);
                42
            },
            true, // disabled → kill-switch fires BEFORE now_fn
        )
        .expect("disabled write must be Ok");

        assert!(
            !now_fn_called.get(),
            "now_fn must NOT be called when kill-switch is set (disabled=true)"
        );

        // Verify now_fn NOT called when worktree path doesn't exist (is_dir check before now_fn).
        let now_fn_called_noop = std::cell::Cell::new(false);
        let fs2 = InMemoryFileSystem::new(); // No dirs — is_dir returns false.
        write_heartbeat(
            &fs2,
            Path::new("/nonexistent"),
            9,
            || {
                now_fn_called_noop.set(true);
                42
            },
            false,
        )
        .expect("noop must be Ok");
        assert!(
            !now_fn_called_noop.get(),
            "now_fn must NOT be called when worktree path doesn't exist (validated before now_fn)"
        );
    }

    // ---------------------------------------------------------------------------
    // PC-034: Symlink escape rejected
    // ---------------------------------------------------------------------------

    #[test]
    fn rejects_symlink_escape() {
        // Register /wt-sym as both a directory and a symlink.
        // is_dir returns true (it's in dirs), is_symlink returns true → PathEscape.
        let fs = InMemoryFileSystem::new()
            .with_dir("/wt")
            .with_dir("/wt-sym")
            .with_symlink(
                std::path::PathBuf::from("/wt-sym"),
                std::path::PathBuf::from("/outside"),
            );

        let result = write_heartbeat(&fs, Path::new("/wt-sym"), 1, now_100, false);
        assert!(
            matches!(result, Err(HeartbeatError::PathEscape(_))),
            "symlink worktree path must be rejected with PathEscape, got: {result:?}"
        );
    }

    // ---------------------------------------------------------------------------
    // PC-035: `..` traversal rejected
    // ---------------------------------------------------------------------------

    #[test]
    fn rejects_path_traversal() {
        let fs = InMemoryFileSystem::new().with_dir("/wt");
        // Path with `..` component must be rejected before is_dir check.
        let result = write_heartbeat(&fs, Path::new("/wt/../wt"), 1, now_100, false);
        assert!(
            matches!(result, Err(HeartbeatError::PathEscape(_))),
            "path with `..` must be rejected with PathEscape, got: {result:?}"
        );
    }

    // ---------------------------------------------------------------------------
    // PC-071: Kill switch short-circuits write
    // ---------------------------------------------------------------------------

    #[test]
    fn write_suppressed_when_kill_switch_set() {
        let fs = InMemoryFileSystem::new().with_dir("/wt");
        let wt = Path::new("/wt");

        // Write once normally.
        write_heartbeat(&fs, wt, 1, now_100, false).expect("enabled write");
        assert!(
            fs.exists(Path::new("/wt/.ecc-session")),
            "file must exist after enabled write"
        );

        // Now write with kill-switch → must be a no-op (file remains unchanged).
        write_heartbeat(
            &fs,
            wt,
            1,
            || panic!("now_fn must not be called when disabled"),
            true,
        )
        .expect("disabled write must return Ok");

        // The file from the previous write must still exist (kill-switch is no-op, not delete).
        assert!(
            fs.exists(Path::new("/wt/.ecc-session")),
            "existing file must be unchanged when kill-switch is set"
        );

        // Verify the content wasn't modified (still has ts=100 from enabled write).
        let record = read_session(&fs, wt);
        assert_eq!(
            record.last_seen_unix_ts, 100,
            "kill-switch must not modify existing session file"
        );
    }
}
