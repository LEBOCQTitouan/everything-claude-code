use std::io::Read;
use std::path::{Path, PathBuf};

use ecc_domain::workflow::phase::Phase;
use ecc_domain::workflow::state::WorkflowState;

/// Acquire an exclusive state lock, run `f`, then release the lock.
///
/// The lock file is `<state_dir>/.locks/state.lock`.
/// The lock is released when the `FlockGuard` is dropped at the end of this
/// function.
pub fn with_state_lock<F, R>(state_dir: &Path, f: F) -> Result<R, anyhow::Error>
where
    F: FnOnce() -> R,
{
    let _guard = ecc_flock::acquire_for(state_dir, "state")
        .map_err(|e| anyhow::anyhow!("Failed to acquire state lock: {e}"))?;
    Ok(f())
}

/// Read the current workflow phase from state.json.
///
/// Returns `None` when state.json does not exist or cannot be parsed.
/// Returns `Some(phase_string)` when the phase field is present.
pub fn read_phase(state_dir: &Path) -> Option<String> {
    let state_path = state_dir.join("state.json");
    if !state_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&state_path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    v.get("phase")
        .and_then(|p| p.as_str())
        .map(|s| s.to_owned())
}

/// Read the workflow state from the state directory.
///
/// Returns `Ok(None)` when state.json does not exist (workflow not initialized).
/// Returns `Ok(Some(state))` when state.json exists and parses correctly.
/// Returns `Err` when the file exists but cannot be read or parsed.
pub fn read_state(state_dir: &Path) -> Result<Option<WorkflowState>, anyhow::Error> {
    let state_path = state_dir.join("state.json");
    if !state_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&state_path)
        .map_err(|e| anyhow::anyhow!("Failed to read state.json: {e}"))?;
    let state = WorkflowState::from_json(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse state.json: {e}"))?;
    Ok(Some(state))
}

/// Ensure the state directory exists.
/// Returns the path to the state directory.
pub fn ensure_state_dir(state_dir: &Path) -> Result<PathBuf, anyhow::Error> {
    std::fs::create_dir_all(state_dir).map_err(|e| {
        anyhow::anyhow!(
            "Failed to create state directory {}: {e}",
            state_dir.display()
        )
    })?;
    Ok(state_dir.to_path_buf())
}

pub(crate) const MAX_STDIN_BYTES: u64 = 1_048_576; // 1 MB

/// Read from `reader` up to `limit` bytes. Returns `(content, Some(bytes_read))` when the
/// input exceeded the limit (indicating truncation), or `(content, None)` when within bounds.
pub(crate) fn read_bounded(reader: impl Read, limit: u64) -> (String, Option<usize>) {
    let mut buf = String::new();
    let bytes_read = reader.take(limit + 1).read_to_string(&mut buf).unwrap_or(0);
    if bytes_read > limit as usize {
        buf.truncate(limit as usize);
        (buf, Some(bytes_read))
    } else {
        (buf, None)
    }
}

/// Read all of stdin into a string (used by hook subcommands).
///
/// Input is bounded at 1 MB. If the input exceeds this limit it is truncated
/// and a warning is emitted via `tracing::warn!`.
pub fn read_stdin() -> String {
    let (content, truncated_at) = read_bounded(std::io::stdin(), MAX_STDIN_BYTES);
    if let Some(original) = truncated_at {
        tracing::warn!(
            "read_stdin: input truncated from {} bytes to {} bytes",
            original,
            MAX_STDIN_BYTES
        );
    }
    content
}

/// Archive state.json to archive/state-YYYYMMDDHHMMSS.json.
///
/// When `include_done` is false, done-phase states are NOT archived (init behavior).
/// When `include_done` is true, ALL states are archived (reset behavior).
pub fn archive_state(workflow_dir: &Path, include_done: bool) -> Result<(), anyhow::Error> {
    let state_path = workflow_dir.join("state.json");
    if !state_path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&state_path)
        .map_err(|e| anyhow::anyhow!("Failed to read state.json: {e}"))?;
    let is_done = WorkflowState::from_json(&content)
        .map(|s| s.phase == Phase::Done)
        .unwrap_or(false); // corrupt state -> archive it

    if is_done && !include_done {
        return Ok(());
    }

    let archive_dir = workflow_dir.join("archive");
    std::fs::create_dir_all(&archive_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create archive directory: {e}"))?;
    let ts = crate::time::utc_now_iso8601().replace(['T', ':', 'Z'], "");
    let archive_name = format!("state-{ts}.json");
    std::fs::rename(&state_path, archive_dir.join(&archive_name))
        .map_err(|e| anyhow::anyhow!("Failed to archive state.json to {archive_name}: {e}"))?;
    Ok(())
}

/// Write the workflow state to state.json atomically (temp file + rename).
pub fn write_state_atomic(state_dir: &Path, state: &WorkflowState) -> Result<(), anyhow::Error> {
    let dir = ensure_state_dir(state_dir)?;
    let state_path = dir.join("state.json");
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| anyhow::anyhow!("Failed to serialize state: {e}"))?;
    let tmp_path = dir.join(".state.json.tmp");
    std::fs::write(&tmp_path, &json)
        .map_err(|e| anyhow::anyhow!("Failed to write temp state file: {e}"))?;
    std::fs::rename(&tmp_path, &state_path)
        .map_err(|e| anyhow::anyhow!("Failed to rename state file: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tempfile::TempDir;

    #[test]
    fn with_state_lock_raii() {
        let tmp = TempDir::new().unwrap();
        let state_dir = tmp.path();
        let flag_path = state_dir.join("inside_closure.flag");
        let flag_path_clone = flag_path.clone();

        with_state_lock(state_dir, || {
            std::fs::write(&flag_path_clone, b"ran").unwrap();
        })
        .unwrap();

        assert!(
            flag_path.exists(),
            "closure did not run inside with_state_lock"
        );

        let lock_file = ecc_flock::lock_dir_for(state_dir).join("state.lock");
        assert!(lock_file.exists(), "state.lock file was not created");

        ecc_flock::acquire_for(state_dir, "state")
            .expect("lock should be free after with_state_lock returns");
    }

    #[test]
    fn read_stdin_bounded_truncates() {
        let oversized = "x".repeat(1_048_577);
        let cursor = Cursor::new(oversized.as_bytes().to_vec());
        let (content, truncated) = read_bounded(cursor, MAX_STDIN_BYTES);
        assert!(truncated.is_some(), "expected truncation indicator");
        assert_eq!(content.len(), 1_048_576, "content should be exactly 1 MB");
    }

    #[test]
    fn read_stdin_bounded_exact() {
        let exactly_1mb = "y".repeat(1_048_576);
        let cursor = Cursor::new(exactly_1mb.as_bytes().to_vec());
        let (content, truncated) = read_bounded(cursor, MAX_STDIN_BYTES);
        assert!(truncated.is_none(), "exactly 1 MB should NOT be truncated");
        assert_eq!(content.len(), 1_048_576);
    }

    #[test]
    fn read_stdin_bounded_logs_truncation() {
        let oversized = "z".repeat(1_048_577);
        let cursor = Cursor::new(oversized.as_bytes().to_vec());
        let (_, truncated) = read_bounded(cursor, MAX_STDIN_BYTES);
        assert!(
            truncated.is_some(),
            "truncation indicator must be Some to trigger tracing::warn!"
        );
    }

    fn make_state_json(phase: ecc_domain::workflow::phase::Phase) -> String {
        use ecc_domain::workflow::{
            concern::Concern,
            state::{Artifacts, Toolchain, WorkflowState},
            timestamp::Timestamp,
        };
        let state = WorkflowState {
            phase,
            concern: Concern::Dev,
            feature: "test-feature".to_owned(),
            started_at: Timestamp::new("2026-01-01T00:00:00Z"),
            toolchain: Toolchain {
                test: None,
                lint: None,
                build: None,
            },
            artifacts: Artifacts {
                plan: None,
                solution: None,
                implement: None,
                campaign_path: None,
                spec_path: None,
                design_path: None,
                tasks_path: None,
            },
            completed: vec![],
            version: 1,
            history: vec![],
        };
        serde_json::to_string_pretty(&state).unwrap()
    }

    #[test]
    fn archive_state_includes_done() {
        let tmp = TempDir::new().unwrap();
        let workflow_dir = tmp.path().join(".claude/workflow");
        std::fs::create_dir_all(&workflow_dir).unwrap();
        let state_path = workflow_dir.join("state.json");
        std::fs::write(
            &state_path,
            make_state_json(ecc_domain::workflow::phase::Phase::Done),
        )
        .unwrap();

        // include_done=true: done state MUST be archived
        archive_state(&workflow_dir, true).unwrap();

        // state.json must no longer exist (moved to archive)
        assert!(!state_path.exists(), "state.json should have been archived");
        // archive dir must contain a file
        let archive_dir = workflow_dir.join("archive");
        let entries: Vec<_> = std::fs::read_dir(&archive_dir).unwrap().collect();
        assert!(
            !entries.is_empty(),
            "archive dir must contain the archived state"
        );
    }

    #[test]
    fn archive_state_skips_done() {
        let tmp = TempDir::new().unwrap();
        let workflow_dir = tmp.path().join(".claude/workflow");
        std::fs::create_dir_all(&workflow_dir).unwrap();
        let state_path = workflow_dir.join("state.json");
        std::fs::write(
            &state_path,
            make_state_json(ecc_domain::workflow::phase::Phase::Done),
        )
        .unwrap();

        // include_done=false: done state must NOT be archived
        archive_state(&workflow_dir, false).unwrap();

        // state.json must still exist (not moved)
        assert!(
            state_path.exists(),
            "state.json should NOT have been archived when include_done=false and phase=done"
        );
        // archive dir must NOT have been created (or be empty)
        let archive_dir = workflow_dir.join("archive");
        if archive_dir.exists() {
            let entries: Vec<_> = std::fs::read_dir(&archive_dir).unwrap().collect();
            assert!(
                entries.is_empty(),
                "no files should be archived when include_done=false"
            );
        }
    }

    /// PC-008: `read_phase(state_dir)` reads from `state_dir/state.json`
    #[test]
    fn read_phase_from_state_dir() {
        let tmp = TempDir::new().unwrap();
        // state_dir is the directory that directly contains state.json
        let state_dir = tmp.path().join("my-state-dir");
        std::fs::create_dir_all(&state_dir).unwrap();
        let state_path = state_dir.join("state.json");
        std::fs::write(&state_path, r#"{"phase":"implement","concern":"dev","feature":"f","started_at":"2026-01-01T00:00:00Z","toolchain":{"test":null,"lint":null,"build":null},"artifacts":{"plan":null,"solution":null,"implement":null,"campaign_path":null,"spec_path":null,"design_path":null,"tasks_path":null},"completed":[],"version":1}"#).unwrap();

        // Pass state_dir directly — not a parent that contains .claude/workflow/
        let phase = read_phase(&state_dir);
        assert_eq!(
            phase,
            Some("implement".to_owned()),
            "read_phase must read from state_dir/state.json"
        );
    }

    /// PC-009: `write_state_atomic(state_dir, state)` writes to `state_dir/state.json`
    #[test]
    fn write_state_to_state_dir() {
        use ecc_domain::workflow::{
            concern::Concern,
            phase::Phase,
            state::{Artifacts, Toolchain, WorkflowState},
            timestamp::Timestamp,
        };

        let tmp = TempDir::new().unwrap();
        let state_dir = tmp.path().join("my-state-dir");
        // state_dir does NOT need to exist yet — ensure_state_dir creates it

        let state = WorkflowState {
            phase: Phase::Plan,
            concern: Concern::Dev,
            feature: "test-write".to_owned(),
            started_at: Timestamp::new("2026-01-01T00:00:00Z"),
            toolchain: Toolchain {
                test: None,
                lint: None,
                build: None,
            },
            artifacts: Artifacts {
                plan: None,
                solution: None,
                implement: None,
                campaign_path: None,
                spec_path: None,
                design_path: None,
                tasks_path: None,
            },
            completed: vec![],
            version: 1,
            history: vec![],
        };

        write_state_atomic(&state_dir, &state).unwrap();

        // The file must be at state_dir/state.json — NOT state_dir/.claude/workflow/state.json
        let expected_path = state_dir.join("state.json");
        assert!(
            expected_path.exists(),
            "write_state_atomic must write to state_dir/state.json"
        );

        let content = std::fs::read_to_string(&expected_path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(v["phase"], "plan");
    }

    /// PC-010: `with_state_lock(state_dir)` locks under `state_dir/.locks/`
    #[test]
    fn with_state_lock_uses_state_dir() {
        let tmp = TempDir::new().unwrap();
        let state_dir = tmp.path().join("my-state-dir");
        std::fs::create_dir_all(&state_dir).unwrap();

        with_state_lock(&state_dir, || {}).unwrap();

        // Lock file must be at state_dir/.locks/state.lock — NOT state_dir/.claude/workflow/.locks/state.lock
        let lock_file = state_dir.join(".locks").join("state.lock");
        assert!(
            lock_file.exists(),
            "with_state_lock must create lock at state_dir/.locks/state.lock"
        );
    }

    /// PC-022: archive_state archives to state_dir/archive/ (AC-004.6)
    #[test]
    fn archive_uses_state_dir() {
        let tmp = TempDir::new().unwrap();
        // Use a non-default state_dir
        let custom_state_dir = tmp.path().join(".git/ecc-workflow");
        std::fs::create_dir_all(&custom_state_dir).unwrap();

        let state_path = custom_state_dir.join("state.json");
        std::fs::write(
            &state_path,
            make_state_json(ecc_domain::workflow::phase::Phase::Plan),
        )
        .unwrap();

        archive_state(&custom_state_dir, true).unwrap();

        // state.json must have been moved (archived)
        assert!(
            !state_path.exists(),
            "state.json should have been archived from custom state_dir"
        );

        // archive dir must be under custom state_dir, NOT under .claude/workflow/
        let archive_dir = custom_state_dir.join("archive");
        assert!(
            archive_dir.exists(),
            "archive dir must be created under custom state_dir"
        );
        let entries: Vec<_> = std::fs::read_dir(&archive_dir).unwrap().collect();
        assert!(
            !entries.is_empty(),
            "archive dir must contain the archived state file"
        );

        // .claude/workflow/ must NOT have been touched
        assert!(
            !tmp.path().join(".claude/workflow").exists(),
            ".claude/workflow must NOT be created when using custom state_dir"
        );
    }

    /// PC-028: `ensure_state_dir` error message contains the target directory path
    #[test]
    fn ensure_state_dir_error_contains_path() {
        // Use a path that cannot be created (parent is a file, not a directory)
        let tmp = TempDir::new().unwrap();
        let blocker = tmp.path().join("blocker");
        std::fs::write(&blocker, b"I am a file").unwrap();
        // state_dir = blocker/subdir — cannot be created because blocker is a file
        let state_dir = blocker.join("subdir");

        let err = ensure_state_dir(&state_dir).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains(state_dir.to_str().unwrap()),
            "ensure_state_dir error must contain the target path. Got: {msg}"
        );
    }
}
