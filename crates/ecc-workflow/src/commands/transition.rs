use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use ecc_domain::workflow::transition::resolve_transition_by_name;

use crate::io::{read_state, write_state_atomic};
use crate::output::WorkflowOutput;

/// Run the `transition` subcommand: advance the workflow to the target phase.
///
/// - Reads state.json from project_dir.
/// - If missing, returns a warn (exit 0).
/// - Resolves the transition via domain rules.
/// - If illegal, returns a block (exit 2).
/// - Stamps the artifact timestamp for --artifact <name>.
/// - Stores the path for --path <value> into the matching artifact path field.
/// - Writes state.json atomically.
pub fn run(
    target: &str,
    artifact: Option<&str>,
    path: Option<&str>,
    project_dir: &Path,
) -> WorkflowOutput {
    let mut state = match read_state(project_dir) {
        Ok(None) => {
            return WorkflowOutput::warn(
                "No state.json found — workflow not initialized",
            )
        }
        Ok(Some(s)) => s,
        Err(e) => return WorkflowOutput::warn(format!("Failed to read state: {e}")),
    };

    let from = state.phase;
    let to = match resolve_transition_by_name(from, target) {
        Ok(t) => t,
        Err(e) => return WorkflowOutput::block(format!("Illegal transition: {e}")),
    };

    // Update phase
    state.phase = to;

    // Stamp artifact timestamp
    if let Some(artifact_name) = artifact {
        let now = utc_now_iso8601();
        match artifact_name {
            "plan" => state.artifacts.plan = Some(now.clone()),
            "solution" => state.artifacts.solution = Some(now.clone()),
            "implement" => state.artifacts.implement = Some(now.clone()),
            other => {
                return WorkflowOutput::block(format!(
                    "Unknown artifact '{other}' — expected plan, solution, or implement"
                ))
            }
        }

        // Store optional path into the corresponding path field
        if let Some(p) = path {
            match artifact_name {
                "plan" => state.artifacts.spec_path = Some(p.to_owned()),
                "solution" => state.artifacts.design_path = Some(p.to_owned()),
                "implement" => state.artifacts.tasks_path = Some(p.to_owned()),
                _ => {}
            }
        }
    }

    match write_state_atomic(project_dir, &state) {
        Ok(()) => WorkflowOutput::pass(format!(
            "Phase transition: {from} -> {to}"
        )),
        Err(e) => WorkflowOutput::block(format!("Failed to write state.json: {e}")),
    }
}

/// Return the current UTC time formatted as ISO 8601: `YYYY-MM-DDTHH:MM:SSZ`.
fn utc_now_iso8601() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let (year, month, day, hour, min, sec) = unix_secs_to_calendar(secs);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

/// Convert a Unix epoch (seconds) to (year, month, day, hour, min, sec) in UTC.
fn unix_secs_to_calendar(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let sec = secs % 60;
    let mins = secs / 60;
    let min = mins % 60;
    let hours = mins / 60;
    let hour = hours % 24;
    let days = hours / 24;

    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z % 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    (y, m, d, hour, min, sec)
}
