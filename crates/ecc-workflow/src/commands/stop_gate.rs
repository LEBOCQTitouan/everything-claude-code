use std::path::Path;

use ecc_domain::workflow::phase::Phase;

use crate::output::WorkflowOutput;

/// Run the `stop-gate` subcommand.
///
/// Called by the Stop hook when a Claude session ends.
/// Warns on stderr if the workflow is in an incomplete phase, exits 0 always.
///
/// Exit behavior:
/// - No state.json → exit 0, silent
/// - Phase `Done`  → exit 0, silent
/// - Any other phase → exit 0, warn on stderr:
///   "WARNING: Workflow is in '<phase>' phase (not done)."
pub fn run(project_dir: &Path) -> WorkflowOutput {
    let state = match crate::io::read_state(project_dir) {
        Ok(None) => return WorkflowOutput::pass(""),
        Ok(Some(s)) => s,
        Err(_) => return WorkflowOutput::pass(""),
    };

    if state.phase == Phase::Done {
        return WorkflowOutput::pass("");
    }

    WorkflowOutput::warn(format!(
        "WARNING: Workflow is in '{}' phase (not done). Feature: '{}'. \
         Complete the workflow or run `ecc-workflow transition done`.",
        state.phase, state.feature
    ))
}
