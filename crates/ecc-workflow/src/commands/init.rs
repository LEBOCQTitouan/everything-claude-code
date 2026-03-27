use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use ecc_domain::workflow::phase::Phase;
use ecc_domain::workflow::state::{Artifacts, Toolchain, WorkflowState};

use crate::io::write_state_atomic;
use crate::output::WorkflowOutput;

/// Run the `init` subcommand: initialize workflow state for a new session.
///
/// Creates `.claude/workflow/state.json` with phase=plan, the given concern and feature,
/// a current UTC timestamp, and all toolchain/artifact fields set to null.
pub fn run(concern: &str, feature: &str, project_dir: &Path) -> WorkflowOutput {
    let started_at = utc_now_iso8601();

    let state = WorkflowState {
        phase: Phase::Plan,
        concern: concern.to_owned(),
        feature: feature.to_owned(),
        started_at,
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
    };

    match write_state_atomic(project_dir, &state) {
        Ok(()) => WorkflowOutput::pass("Workflow initialized"),
        Err(e) => WorkflowOutput::block(format!("Failed to write state.json: {e}")),
    }
}

/// Return the current UTC time formatted as ISO 8601: `YYYY-MM-DDTHH:MM:SSZ`.
fn utc_now_iso8601() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Convert Unix timestamp to calendar fields (no external deps).
    let (year, month, day, hour, min, sec) = unix_secs_to_calendar(secs);

    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

/// Convert a Unix epoch (seconds) to (year, month, day, hour, min, sec) in UTC.
///
/// Uses the standard calendar algorithm for Gregorian dates.
fn unix_secs_to_calendar(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let sec = secs % 60;
    let mins = secs / 60;
    let min = mins % 60;
    let hours = mins / 60;
    let hour = hours % 24;
    let days = hours / 24;

    // Convert days since 1970-01-01 to Gregorian calendar.
    // Algorithm: http://howardhinnant.github.io/date_algorithms.html (civil_from_days)
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
