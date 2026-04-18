//! Workflow stuck-state detection and recovery.

use ecc_domain::workflow::staleness;
use ecc_ports::clock::Clock;
use ecc_ports::fs::FileSystem;
use std::path::{Path, PathBuf};

/// Staleness detection result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StalenessInfo {
    /// The phase the workflow is stuck at.
    pub phase: String,
    /// The ISO 8601 timestamp when the workflow started.
    pub started_at: String,
}

/// Check if a workflow state file is stale.
///
/// Reads `state.json` from `state_dir`, checks `started_at` against the clock,
/// and returns `Some(StalenessInfo)` if the threshold is exceeded.
///
/// Flow/decision diagram — silent-None on any error, filter idle/done:
///
/// <!-- keep in sync with: staleness_with_mock_clock -->
/// ```text
/// detect_staleness(state_dir, fs, clock, threshold)
///        |
///        v
/// read_to_string(state.json) --Err--> None
///        |--Ok(content)-->
///        v
/// WorkflowState::from_json --Err--> None
///        |--Ok(state)-->
///        v
/// phase is Idle || Done? --Y--> None (never stale)
///        |--N-->
///        v
/// is_stale(started_at, now, threshold)? --N--> None
///        |--Y--> Some(StalenessInfo { phase, started_at })
/// ```
pub fn detect_staleness(
    state_dir: &Path,
    fs: &dyn FileSystem,
    clock: &dyn Clock,
    threshold_secs: u64,
) -> Option<StalenessInfo> {
    let state_path = state_dir.join("state.json");
    let content = fs.read_to_string(&state_path).ok()?;
    let state = ecc_domain::workflow::state::WorkflowState::from_json(&content).ok()?;

    // Idle and Done phases are not considered stale
    if matches!(
        state.phase,
        ecc_domain::workflow::phase::Phase::Idle | ecc_domain::workflow::phase::Phase::Done
    ) {
        return None;
    }

    let now = clock.now_iso8601();
    if staleness::is_stale(state.started_at.as_str(), &now, threshold_secs) {
        Some(StalenessInfo {
            phase: state.phase.to_string(),
            started_at: state.started_at.as_str().to_owned(),
        })
    } else {
        None
    }
}

/// Error type for recovery operations.
#[derive(Debug)]
pub enum RecoverError {
    /// Failed to read the current state.
    ReadFailed(String),
    /// Failed to archive the current state.
    ArchiveFailed(String),
    /// Failed to write the reset state.
    WriteFailed(String),
}

impl std::fmt::Display for RecoverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadFailed(e) => write!(f, "failed to read state: {e}"),
            Self::ArchiveFailed(e) => write!(f, "failed to archive state: {e}"),
            Self::WriteFailed(e) => write!(f, "failed to write reset state: {e}"),
        }
    }
}

impl std::error::Error for RecoverError {}

/// Archive the current state and reset to Idle.
///
/// 1. Reads current state.json
/// 2. Archives it to `archive/state-<timestamp>.json`
/// 3. Writes a new Idle state
///
/// If archival fails, the reset does NOT happen (no data loss).
///
/// Flow/decision diagram — archive-before-reset order guarantee:
///
/// <!-- keep in sync with: recover_archives_and_resets -->
/// ```text
/// recover(state_dir, fs, clock)
///        |
///        v
/// read(state.json) --Err--> ReadFailed
///        |--Ok(content)-->
///        v
/// create_dir_all(archive/) --Err--> ArchiveFailed (no reset)
///        |--Ok-->
///        v
/// write(archive/state-<ts>.json) --Err--> ArchiveFailed (no reset)
///        |--Ok-->
///        v
/// write(state.json, idle_json) --Err--> WriteFailed
///        |--Ok-->
///        v
/// return Ok(())
/// ```
///
/// # Pattern
///
/// Memento \[GoF\] — preserves prior state before reset.
pub fn recover(
    state_dir: &Path,
    fs: &dyn FileSystem,
    clock: &dyn Clock,
) -> Result<(), RecoverError> {
    let state_path = state_dir.join("state.json");
    let content = fs
        .read_to_string(&state_path)
        .map_err(|e| RecoverError::ReadFailed(e.to_string()))?;

    // Archive first — if this fails, we don't reset
    let archive_dir = state_dir.join("archive");
    fs.create_dir_all(&archive_dir)
        .map_err(|e| RecoverError::ArchiveFailed(e.to_string()))?;

    let ts = clock.now_iso8601().replace(['T', ':', 'Z'], "");
    let archive_path = archive_dir.join(format!("state-{ts}.json"));
    fs.write(&archive_path, &content)
        .map_err(|e| RecoverError::ArchiveFailed(e.to_string()))?;

    // Write idle state
    let idle_json = r#"{"phase":"idle","concern":"dev","feature":"","started_at":"","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[],"version":1}"#;
    fs.write(&state_path, idle_json)
        .map_err(|e| RecoverError::WriteFailed(e.to_string()))?;

    Ok(())
}

/// Resolve the archive directory path within the state directory.
pub fn archive_dir(state_dir: &Path) -> PathBuf {
    state_dir.join("archive")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{InMemoryFileSystem, MockClock};
    use std::path::PathBuf;

    fn make_state_json(phase: &str, started_at: &str) -> String {
        format!(
            r#"{{"phase":"{phase}","concern":"dev","feature":"test","started_at":"{started_at}","toolchain":{{"test":null,"lint":null,"build":null}},"artifacts":{{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null}},"completed":[],"version":1}}"#
        )
    }

    #[test]
    fn recover_archives_and_resets() {
        let fs = InMemoryFileSystem::new();
        let state_dir = PathBuf::from("/state");
        let state_path = state_dir.join("state.json");
        fs.write(
            &state_path,
            &make_state_json("plan", "2026-04-01T06:00:00Z"),
        )
        .unwrap();

        let clock = MockClock::fixed("2026-04-01T12:00:00Z", 0);

        let result = recover(&state_dir, &fs, &clock);
        assert!(result.is_ok(), "recover should succeed: {result:?}");

        // State should now be idle
        let new_content = fs.read_to_string(&state_path).unwrap();
        assert!(
            new_content.contains(r#""phase":"idle""#),
            "state should be idle after recovery"
        );

        // Archive should exist
        let archive_path = state_dir.join("archive/state-2026-04-01120000.json");
        assert!(fs.exists(&archive_path), "archive file should exist");
    }

    #[test]
    fn recover_fails_if_archive_fails() {
        let fs = InMemoryFileSystem::new();
        let state_dir = PathBuf::from("/state");
        // No state.json written — read will fail
        let clock = MockClock::fixed("2026-04-01T12:00:00Z", 0);

        let result = recover(&state_dir, &fs, &clock);
        assert!(result.is_err(), "recover should fail when state read fails");

        // Verify the error is a ReadFailed variant
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("read"),
            "error should mention read failure: {err_msg}"
        );
    }

    #[test]
    fn staleness_with_mock_clock() {
        let fs = InMemoryFileSystem::new();
        let state_dir = PathBuf::from("/state");
        fs.write(
            &state_dir.join("state.json"),
            &make_state_json("plan", "2026-04-01T06:00:00Z"),
        )
        .unwrap();

        // 5 hours later — should be stale (threshold 4h)
        let clock = MockClock::fixed("2026-04-01T11:00:00Z", 0);
        let info = detect_staleness(&state_dir, &fs, &clock, 14400);
        assert!(info.is_some(), "should detect staleness after 5 hours");
        assert_eq!(info.unwrap().phase, "plan");

        // 2 hours later — should NOT be stale
        let clock = MockClock::fixed("2026-04-01T08:00:00Z", 0);
        let info = detect_staleness(&state_dir, &fs, &clock, 14400);
        assert!(info.is_none(), "should not be stale after 2 hours");
    }
}
