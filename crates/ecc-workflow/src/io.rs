use std::io::Read;
use std::path::{Path, PathBuf};

use ecc_domain::workflow::state::WorkflowState;

/// Read the current workflow phase from state.json.
///
/// Returns `None` when state.json does not exist or cannot be parsed.
/// Returns `Some(phase_string)` when the phase field is present.
pub fn read_phase(project_dir: &Path) -> Option<String> {
    let state_path = project_dir.join(".claude/workflow/state.json");
    if !state_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&state_path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    v.get("phase")
        .and_then(|p| p.as_str())
        .map(|s| s.to_owned())
}

/// Read the workflow state from the project directory.
///
/// Returns `Ok(None)` when state.json does not exist (workflow not initialized).
/// Returns `Ok(Some(state))` when state.json exists and parses correctly.
/// Returns `Err` when the file exists but cannot be read or parsed.
pub fn read_state(project_dir: &Path) -> Result<Option<WorkflowState>, anyhow::Error> {
    let state_path = project_dir.join(".claude/workflow/state.json");
    if !state_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&state_path)
        .map_err(|e| anyhow::anyhow!("Failed to read state.json: {e}"))?;
    let state = WorkflowState::from_json(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse state.json: {e}"))?;
    Ok(Some(state))
}

/// Ensure the `.claude/workflow/` directory exists inside the project directory.
/// Returns the path to the workflow directory.
pub fn ensure_workflow_dir(project_dir: &Path) -> Result<PathBuf, anyhow::Error> {
    let dir = project_dir.join(".claude/workflow");
    std::fs::create_dir_all(&dir)
        .map_err(|e| anyhow::anyhow!("Failed to create workflow directory: {e}"))?;
    Ok(dir)
}

/// Read all of stdin into a string (used by hook subcommands).
pub fn read_stdin() -> String {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).unwrap_or(0);
    buf
}

/// Write the workflow state to state.json atomically (temp file + rename).
pub fn write_state_atomic(project_dir: &Path, state: &WorkflowState) -> Result<(), anyhow::Error> {
    let dir = ensure_workflow_dir(project_dir)?;
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
