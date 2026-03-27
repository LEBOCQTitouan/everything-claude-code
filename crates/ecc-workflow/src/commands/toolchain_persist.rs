//! `toolchain-persist` subcommand — write toolchain commands to state.json.

use std::path::Path;

use crate::{io, output::WorkflowOutput};

/// Persist toolchain commands into state.json.
///
/// Reads state.json, updates `toolchain.test`, `toolchain.lint`, and `toolchain.build`,
/// then writes back atomically. If state.json does not exist, returns a warn.
pub fn run(test_cmd: &str, lint_cmd: &str, build_cmd: &str, project_dir: &Path) -> WorkflowOutput {
    let state = match io::read_state(project_dir) {
        Ok(Some(s)) => s,
        Ok(None) => {
            return WorkflowOutput::warn(
                "state.json not found — workflow not initialized",
            );
        }
        Err(e) => {
            return WorkflowOutput::warn(format!("Failed to read state.json: {e}"));
        }
    };

    let mut updated = state;
    updated.toolchain.test = Some(test_cmd.to_owned());
    updated.toolchain.lint = Some(lint_cmd.to_owned());
    updated.toolchain.build = Some(build_cmd.to_owned());

    if let Err(e) = io::write_state_atomic(project_dir, &updated) {
        return WorkflowOutput::warn(format!("Failed to write state.json: {e}"));
    }

    WorkflowOutput::pass(format!(
        "Toolchain persisted: test={test_cmd}, lint={lint_cmd}, build={build_cmd}"
    ))
}
