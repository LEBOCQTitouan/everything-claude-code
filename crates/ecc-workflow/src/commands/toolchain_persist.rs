//! `toolchain-persist` subcommand — write toolchain commands to state.json.

use std::path::Path;

use crate::{io, output::WorkflowOutput};

/// Persist toolchain commands into state.json.
///
/// Reads state.json under the state lock, updates `toolchain.test`, `toolchain.lint`,
/// and `toolchain.build`, then writes back atomically — all within the same lock scope.
/// If state.json does not exist, returns a warn.
pub fn run(test_cmd: &str, lint_cmd: &str, build_cmd: &str, state_dir: &Path) -> WorkflowOutput {
    let test_cmd = test_cmd.to_owned();
    let lint_cmd = lint_cmd.to_owned();
    let build_cmd = build_cmd.to_owned();

    let result = io::with_state_lock(state_dir, || {
        let state = match io::read_state(state_dir) {
            Ok(Some(s)) => s,
            Ok(None) => {
                return WorkflowOutput::warn("state.json not found — workflow not initialized");
            }
            Err(e) => {
                return WorkflowOutput::warn(format!("Failed to read state.json: {e}"));
            }
        };

        let mut updated = state;
        updated.toolchain.test = Some(test_cmd.clone());
        updated.toolchain.lint = Some(lint_cmd.clone());
        updated.toolchain.build = Some(build_cmd.clone());

        if let Err(e) = io::write_state_atomic(state_dir, &updated) {
            return WorkflowOutput::warn(format!("Failed to write state.json: {e}"));
        }

        WorkflowOutput::pass(format!(
            "Toolchain persisted: test={test_cmd}, lint={lint_cmd}, build={build_cmd}"
        ))
    });

    match result {
        Ok(output) => output,
        Err(e) => WorkflowOutput::block(format!("Lock error: {e}")),
    }
}
