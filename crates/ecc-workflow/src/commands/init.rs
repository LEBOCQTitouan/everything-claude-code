use crate::output::WorkflowOutput;

/// Run the `init` subcommand: initialize workflow state for a new session.
///
/// For now this emits a pass output. Full state.json creation will be
/// implemented in a later PC.
pub fn run(_concern: &str, _feature: &str) -> WorkflowOutput {
    WorkflowOutput::pass("Workflow initialized")
}
