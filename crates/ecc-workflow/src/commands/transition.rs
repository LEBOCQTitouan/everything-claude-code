use std::path::Path;

use ecc_domain::workflow::phase::Phase;
use ecc_domain::workflow::state::Completion;
use ecc_domain::workflow::transition::resolve_transition_by_name;

use crate::io::{read_state, with_state_lock, write_state_atomic};
use crate::output::WorkflowOutput;
use crate::time::utc_now_iso8601;

/// Run the `transition` subcommand: advance the workflow to the target phase.
///
/// - Reads state.json from project_dir under the state lock.
/// - If missing, returns a warn (exit 0).
/// - Resolves the transition via domain rules.
/// - If illegal, returns a block (exit 2).
/// - Stamps the artifact timestamp for --artifact <name>.
/// - Stores the path for --path <value> into the matching artifact path field.
/// - Writes state.json atomically under the same state lock.
/// - Memory writes are performed OUTSIDE the lock (they use their own per-type locks).
pub fn run(
    target: &str,
    artifact: Option<&str>,
    path: Option<&str>,
    project_dir: &Path,
) -> WorkflowOutput {
    // Owned copies for the closure.
    let target = target.to_owned();
    let artifact = artifact.map(str::to_owned);
    let path = path.map(str::to_owned);

    let result = with_state_lock(project_dir, || {
        let mut state = match read_state(project_dir) {
            Ok(None) => {
                return (
                    WorkflowOutput::warn("No state.json found — workflow not initialized"),
                    None,
                );
            }
            Ok(Some(s)) => s,
            Err(e) => {
                return (
                    WorkflowOutput::warn(format!("Failed to read state: {e}")),
                    None,
                );
            }
        };

        let from = state.phase;
        let to = match resolve_transition_by_name(from, &target) {
            Ok(t) => t,
            Err(e) => {
                return (
                    WorkflowOutput::block(format!("Illegal transition: {e}")),
                    None,
                );
            }
        };

        // Update phase
        state.phase = to;

        // Stamp artifact timestamp and handle side-effects of the named artifact.
        if let Some(ref artifact_name) = artifact {
            let now = utc_now_iso8601();
            match artifact_name.as_str() {
                "plan" => state.artifacts.plan = Some(now),
                "solution" => state.artifacts.solution = Some(now),
                "implement" => state.artifacts.implement = Some(now),
                other => {
                    return (
                        WorkflowOutput::block(format!(
                            "Unknown artifact '{other}' — expected plan, solution, or implement"
                        )),
                        None,
                    );
                }
            }

            // Store optional path into the corresponding path field.
            if let Some(ref p) = path {
                match artifact_name.as_str() {
                    "plan" => state.artifacts.spec_path = Some(p.clone()),
                    "solution" => state.artifacts.design_path = Some(p.clone()),
                    "implement" => state.artifacts.tasks_path = Some(p.clone()),
                    _ => {}
                }
            }

            // On done transition, append a completion record to the completed array.
            if to == Phase::Done {
                state.completed.push(Completion {
                    phase: artifact_name.clone(),
                    file: "implement-done.md".to_owned(),
                    at: utc_now_iso8601(),
                });
            }
        }

        match write_state_atomic(project_dir, &state) {
            Ok(()) => {
                let output = WorkflowOutput::pass(format!("Phase transition: {from} -> {to}"));
                let memory_info = artifact
                    .as_ref()
                    .map(|a| (a.clone(), state.feature.clone(), state.concern.clone()));
                (output, memory_info)
            }
            Err(e) => (
                WorkflowOutput::block(format!("Failed to write state.json: {e}")),
                None,
            ),
        }
    });

    match result {
        Ok((output, Some((artifact_name, feature, concern)))) => {
            // Best-effort memory writes outside the state lock (they use their own locks).
            let wi_phase = match artifact_name.as_str() {
                "plan" => "plan",
                "solution" => "solution",
                "implement" => "implementation",
                _ => artifact_name.as_str(),
            };
            let _ = crate::commands::memory_write::write_action(
                &artifact_name,
                &feature,
                "success",
                "[]",
                project_dir,
            );
            let _ = crate::commands::memory_write::write_work_item(
                wi_phase,
                &feature,
                &concern,
                project_dir,
            );
            let _ = crate::commands::memory_write::write_daily(
                wi_phase,
                &feature,
                &concern,
                project_dir,
            );
            let _ = crate::commands::memory_write::write_memory_index(project_dir);
            output
        }
        Ok((output, None)) => output,
        Err(e) => WorkflowOutput::block(format!("Lock error: {e}")),
    }
}
