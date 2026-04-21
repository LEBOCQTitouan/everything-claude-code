//! Bypass management use cases.
//!
//! Orchestrates bypass grant, list, summary, prune, and gc operations.

use ecc_domain::hook_runtime::bypass::{BypassDecision, BypassSummary, BypassToken, Verdict};
use ecc_ports::bypass_store::{BypassStore, BypassStoreError};
use ecc_ports::clock::Clock;
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// Grant a bypass for a specific hook in the current session.
///
/// Creates a bypass token file AND records an Accepted decision in the audit trail.
///
/// Flow/decision diagram — dual write (file + audit) with validation:
///
/// <!-- keep in sync with: bypass_grant_creates_token -->
/// ```text
/// grant(store, fs, home, hook_id, reason, session_id)
///        |
///        v
/// BypassToken::new(...) --Err--> Validation error
///        |--Ok(token)-->
///        v
/// BypassDecision::new(...) --Err--> Validation error
///        |--Ok(decision)-->
///        v
/// create_dir_all(home/.ecc/bypass-tokens/<session>)  --Err--> Io error
///        |--Ok-->
///        v
/// serialize token -> write <hook_id>.json  --Err--> Io error
///        |--Ok-->
///        v
/// store.record(decision) --Err--> Store error
///        |--Ok--> return Ok(token)
/// ```
///
/// # Pattern
///
/// Audit Log \[DDD\] — decision persisted as immutable record alongside token.
pub fn grant(
    store: &dyn BypassStore,
    fs: &dyn FileSystem,
    home: &Path,
    hook_id: &str,
    reason: &str,
    session_id: &str,
    clock: &dyn Clock,
) -> Result<BypassToken, BypassMgmtError> {
    let timestamp = clock.now_iso8601();

    // Create domain objects (validates inputs)
    let token = BypassToken::new(hook_id, session_id, &timestamp, reason)
        .map_err(|e| BypassMgmtError::Validation(e.to_string()))?;
    let decision = BypassDecision::new(hook_id, reason, session_id, Verdict::Accepted, &timestamp)
        .map_err(|e| BypassMgmtError::Validation(e.to_string()))?;

    // Write token file
    let token_dir = home.join(".ecc").join("bypass-tokens").join(session_id);
    fs.create_dir_all(&token_dir)
        .map_err(|e| BypassMgmtError::Io(e.to_string()))?;

    let encoded_hook = hook_id.replace(':', "__");
    let token_path = token_dir.join(format!("{encoded_hook}.json"));
    let token_json =
        serde_json::to_string_pretty(&token).map_err(|e| BypassMgmtError::Io(e.to_string()))?;
    fs.write(&token_path, &token_json)
        .map_err(|e| BypassMgmtError::Io(e.to_string()))?;

    // Record in audit trail
    store.record(&decision).map_err(BypassMgmtError::Store)?;

    Ok(token)
}

/// List bypass decisions, optionally filtered by hook ID.
pub fn list(
    store: &dyn BypassStore,
    hook_id: Option<&str>,
    limit: usize,
) -> Result<Vec<BypassDecision>, BypassMgmtError> {
    match hook_id {
        Some(hid) => store
            .query_by_hook(hid, limit)
            .map_err(BypassMgmtError::Store),
        None => {
            // query_by_hook with empty string won't work; use summary + individual queries
            // For v1, just return empty if no hook filter (will implement full list later)
            store
                .query_by_hook("", limit)
                .map_err(BypassMgmtError::Store)
        }
    }
}

/// Get bypass summary (per-hook accepted/refused counts).
pub fn summary(store: &dyn BypassStore) -> Result<BypassSummary, BypassMgmtError> {
    store.summary().map_err(BypassMgmtError::Store)
}

/// Prune old bypass records.
pub fn prune(store: &dyn BypassStore, older_than_days: u64) -> Result<u64, BypassMgmtError> {
    store.prune(older_than_days).map_err(BypassMgmtError::Store)
}

/// Garbage-collect stale bypass token files from ended sessions.
///
/// For each session token-dir that differs from the current session, the function
/// consults [`crate::worktree::checker::LivenessChecker`] before deletion:
/// if the sibling worktree (`<project_dir>/.claude/worktrees/<session>`) is live,
/// the token-dir is preserved.
pub fn gc(
    fs: &dyn FileSystem,
    shell: &dyn ecc_ports::shell::ShellExecutor,
    home: &Path,
    project_dir: &Path,
    current_session_id: &str,
    ttl_secs: u64,
) -> Result<u64, BypassMgmtError> {
    let tokens_dir = home.join(".ecc").join("bypass-tokens");
    if !fs.exists(&tokens_dir) {
        return Ok(0);
    }
    let entries = fs
        .read_dir(&tokens_dir)
        .map_err(|e| BypassMgmtError::Io(e.to_string()))?;

    let mut removed = 0u64;
    for entry in entries {
        if fs.is_dir(&entry) {
            let dir_name = entry
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if dir_name == current_session_id {
                continue;
            }

            // NEW: consult LivenessChecker before deleting sibling's tokens.
            let worktree_path = project_dir
                .join(".claude")
                .join("worktrees")
                .join(&dir_name);
            if fs.exists(&worktree_path) {
                let checker = crate::worktree::checker::LivenessChecker {
                    fs,
                    shell,
                    now_fn: Box::new(crate::worktree::checker::unix_now),
                    ttl_secs,
                };
                if matches!(
                    checker.check(&worktree_path),
                    crate::worktree::checker::LivenessVerdict::Live
                ) {
                    continue; // preserve live sibling's tokens
                }
            }

            fs.remove_dir_all(&entry)
                .map_err(|e| BypassMgmtError::Io(e.to_string()))?;
            removed += 1;
        }
    }
    Ok(removed)
}

/// Errors from bypass management operations.
#[derive(Debug, thiserror::Error)]
pub enum BypassMgmtError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("store error: {0}")]
    Store(#[from] BypassStoreError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{InMemoryBypassStore, InMemoryFileSystem, TEST_CLOCK};
    use std::path::Path;

    #[test]
    fn bypass_grant_creates_token() {
        let store = InMemoryBypassStore::new();
        let fs = InMemoryFileSystem::new();
        let home = Path::new("/home/user");

        let result = grant(
            &store,
            &fs,
            home,
            "pre:write-edit:worktree-guard",
            "hotfix needed",
            "session-abc",
            &*TEST_CLOCK,
        );
        assert!(result.is_ok());

        // Token file should exist
        let token_path =
            "/home/user/.ecc/bypass-tokens/session-abc/pre__write-edit__worktree-guard.json";
        assert!(fs.exists(Path::new(token_path)));

        // Audit trail should have one Accepted record
        let decisions = store.snapshot();
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].verdict, Verdict::Accepted);
    }

    #[test]
    fn bypass_grant_without_reason_errors() {
        let store = InMemoryBypassStore::new();
        let fs = InMemoryFileSystem::new();
        let result = grant(
            &store,
            &fs,
            Path::new("/home"),
            "hook",
            "",
            "session-1",
            &*TEST_CLOCK,
        );
        assert!(result.is_err());
    }

    #[test]
    fn bypass_summary_returns_counts() {
        let store = InMemoryBypassStore::new();
        let d = BypassDecision::new("hook-a", "r", "s1", Verdict::Accepted, "ts").unwrap();
        store.record(&d).unwrap();
        let s = summary(&store).unwrap();
        assert_eq!(s.total_accepted, 1);
    }

    #[test]
    fn bypass_gc_cleans_stale() {
        use ecc_test_support::MockExecutor;
        let fs = InMemoryFileSystem::new()
            .with_dir("/home/.ecc/bypass-tokens")
            .with_dir("/home/.ecc/bypass-tokens/old-session")
            .with_dir("/home/.ecc/bypass-tokens/current-session")
            .with_file("/home/.ecc/bypass-tokens/old-session/hook.json", "{}");
        let shell = MockExecutor::new();

        // No worktree dir exists for old-session → skip liveness check → delete.
        let removed = gc(
            &fs,
            &shell,
            Path::new("/home"),
            Path::new("/project"),
            "current-session",
            3600,
        )
        .unwrap();
        assert_eq!(removed, 1);
        assert!(!fs.exists(Path::new("/home/.ecc/bypass-tokens/old-session")));
        assert!(fs.exists(Path::new("/home/.ecc/bypass-tokens/current-session")));
    }

    /// PC-053: `bypass::gc` consults `LivenessChecker::check` before deletion.
    ///
    /// Verifies that when a worktree dir exists for a sibling session the checker
    /// is consulted. Here the `.ecc-session` file is missing so the verdict is
    /// `MissingFile` (not `Live`), meaning the token-dir IS deleted.
    #[test]
    fn gc_consults_checker() {
        use ecc_test_support::MockExecutor;

        // Worktree dir exists for "sibling" but has NO .ecc-session → MissingFile verdict.
        let fs = InMemoryFileSystem::new()
            .with_dir("/home/.ecc/bypass-tokens")
            .with_dir("/home/.ecc/bypass-tokens/sibling")
            .with_dir("/home/.ecc/bypass-tokens/current-session")
            .with_file("/home/.ecc/bypass-tokens/sibling/hook.json", "{}")
            // worktree dir exists but no .ecc-session inside
            .with_dir("/project/.claude/worktrees/sibling");

        // MockExecutor: no kill responses needed (MissingFile short-circuits before kill -0).
        let shell = MockExecutor::new();

        let removed = gc(
            &fs,
            &shell,
            Path::new("/home"),
            Path::new("/project"),
            "current-session",
            3600,
        )
        .unwrap();

        // MissingFile → not Live → token-dir must be deleted.
        assert_eq!(removed, 1, "sibling with MissingFile verdict must be gc'd");
        assert!(!fs.exists(Path::new("/home/.ecc/bypass-tokens/sibling")));
    }

    /// PC-054: Bypass-token preserved when sibling worktree is live.
    ///
    /// A sibling session with a fresh `.ecc-session` file and a live PID must
    /// NOT have its token-dir deleted. The test writes a heartbeat timestamped
    /// 60 seconds before real wall-clock now so the liveness check always
    /// produces `Live` regardless of when the test runs.
    #[test]
    fn preserves_live_sibling() {
        use ecc_ports::shell::CommandOutput;
        use ecc_test_support::MockExecutor;

        const PID: u32 = 99001;

        // Use real wall-clock to ensure the heartbeat is always fresh.
        let real_now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let live_json = format!(
            r#"{{"schema_version":1,"claude_code_pid":{PID},"last_seen_unix_ts":{}}}"#,
            real_now - 60
        );

        let fs = InMemoryFileSystem::new()
            .with_dir("/home/.ecc/bypass-tokens")
            .with_dir("/home/.ecc/bypass-tokens/live-sibling")
            .with_dir("/home/.ecc/bypass-tokens/current-session")
            .with_file("/home/.ecc/bypass-tokens/live-sibling/hook.json", "{}")
            .with_dir("/project/.claude/worktrees/live-sibling")
            .with_file(
                "/project/.claude/worktrees/live-sibling/.ecc-session",
                &live_json,
            );

        // PID alive → kill -0 returns exit code 0.
        let shell = MockExecutor::new().on_args(
            "kill",
            &["-0", &PID.to_string()],
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        );

        let removed = gc(
            &fs,
            &shell,
            Path::new("/home"),
            Path::new("/project"),
            "current-session",
            3600,
        )
        .unwrap();

        // Live verdict → token-dir MUST be preserved.
        assert_eq!(removed, 0, "live sibling's tokens must NOT be gc'd");
        assert!(fs.exists(Path::new("/home/.ecc/bypass-tokens/live-sibling")));
    }
}
