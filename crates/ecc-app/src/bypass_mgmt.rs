//! Bypass management use cases.
//!
//! Orchestrates bypass grant, list, summary, prune, and gc operations.

use ecc_domain::hook_runtime::bypass::{BypassDecision, BypassSummary, BypassToken, Verdict};
use ecc_ports::bypass_store::{BypassStore, BypassStoreError};
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// Grant a bypass for a specific hook in the current session.
///
/// Creates a bypass token file AND records an Accepted decision in the audit trail.
pub fn grant(
    store: &dyn BypassStore,
    fs: &dyn FileSystem,
    home: &Path,
    hook_id: &str,
    reason: &str,
    session_id: &str,
) -> Result<BypassToken, BypassMgmtError> {
    let timestamp = chrono_like_now();

    // Create domain objects (validates inputs)
    let token = BypassToken::new(hook_id, session_id, &timestamp, reason)
        .map_err(|e| BypassMgmtError::Validation(e.to_string()))?;
    let decision = BypassDecision::new(hook_id, reason, session_id, Verdict::Accepted, &timestamp)
        .map_err(|e| BypassMgmtError::Validation(e.to_string()))?;

    // Write token file
    let token_dir = home
        .join(".ecc")
        .join("bypass-tokens")
        .join(session_id);
    fs.create_dir_all(&token_dir)
        .map_err(|e| BypassMgmtError::Io(e.to_string()))?;

    let encoded_hook = hook_id.replace(':', "__");
    let token_path = token_dir.join(format!("{encoded_hook}.json"));
    let token_json =
        serde_json::to_string_pretty(&token).map_err(|e| BypassMgmtError::Io(e.to_string()))?;
    fs.write(&token_path, &token_json)
        .map_err(|e| BypassMgmtError::Io(e.to_string()))?;

    // Record in audit trail
    store
        .record(&decision)
        .map_err(BypassMgmtError::Store)?;

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
            store.query_by_hook("", limit).map_err(BypassMgmtError::Store)
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
pub fn gc(
    fs: &dyn FileSystem,
    home: &Path,
    current_session_id: &str,
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
            if dir_name != current_session_id {
                fs.remove_dir_all(&entry)
                    .map_err(|e| BypassMgmtError::Io(e.to_string()))?;
                removed += 1;
            }
        }
    }
    Ok(removed)
}

/// Generate an ISO-8601 timestamp string without chrono dependency.
fn chrono_like_now() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    // Simple UTC timestamp format
    let secs = now.as_secs();
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    // Approximate date from days since epoch (good enough for timestamps)
    let (year, month, day) = days_to_date(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_date(days: u64) -> (u64, u64, u64) {
    // Simplified calendar calculation
    let mut y = 1970;
    let mut remaining = days;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let days_in_months: [u64; 12] = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 0;
    for (i, &dim) in days_in_months.iter().enumerate() {
        if remaining < dim {
            m = i;
            break;
        }
        remaining -= dim;
    }
    (y, (m + 1) as u64, remaining + 1)
}

#[allow(clippy::manual_is_multiple_of)]
fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
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
    use ecc_test_support::{InMemoryBypassStore, InMemoryFileSystem};
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
        );
        assert!(result.is_ok());

        // Token file should exist
        let token_path = "/home/user/.ecc/bypass-tokens/session-abc/pre__write-edit__worktree-guard.json";
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
        let result = grant(&store, &fs, Path::new("/home"), "hook", "", "session-1");
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
        let fs = InMemoryFileSystem::new()
            .with_dir("/home/.ecc/bypass-tokens")
            .with_dir("/home/.ecc/bypass-tokens/old-session")
            .with_dir("/home/.ecc/bypass-tokens/current-session")
            .with_file("/home/.ecc/bypass-tokens/old-session/hook.json", "{}");

        let removed = gc(&fs, Path::new("/home"), "current-session").unwrap();
        assert_eq!(removed, 1);
        assert!(!fs.exists(Path::new("/home/.ecc/bypass-tokens/old-session")));
        assert!(fs.exists(Path::new("/home/.ecc/bypass-tokens/current-session")));
    }
}
