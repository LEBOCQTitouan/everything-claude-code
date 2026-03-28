use std::path::Path;

use ecc_domain::workflow::phase::Phase;
use ecc_domain::workflow::state::Completion;
use ecc_domain::workflow::transition::resolve_transition_by_name;

use crate::io::{read_state, with_state_lock, write_state_atomic};
use crate::output::WorkflowOutput;
use crate::time::utc_now_iso8601;

#[cfg(test)]
mod tests {
    use ecc_domain::workflow::{
        phase::Phase,
        state::{Artifacts, Toolchain, WorkflowState},
    };
    use tempfile::TempDir;

    /// Write a Plan-phase state.json into a temp dir, ready for transition tests.
    fn write_plan_state(dir: &TempDir) {
        let wf_dir = dir.path().join(".claude/workflow");
        std::fs::create_dir_all(&wf_dir).unwrap();
        let state = WorkflowState {
            phase: Phase::Plan,
            concern: "dev".to_owned(),
            feature: "BL-068".to_owned(),
            started_at: "2026-01-01T00:00:00Z".to_owned(),
            toolchain: Toolchain { test: None, lint: None, build: None },
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
        let json = serde_json::to_string_pretty(&state).unwrap();
        std::fs::write(wf_dir.join("state.json"), json).unwrap();
    }

    /// Block memory writes by creating a file at docs/memory (prevents create_dir_all).
    fn block_memory_dir(dir: &TempDir) {
        let docs_dir = dir.path().join("docs");
        std::fs::create_dir_all(&docs_dir).unwrap();
        // Create a file at docs/memory to prevent create_dir_all from succeeding.
        std::fs::write(docs_dir.join("memory"), b"blocker").unwrap();
    }

    // PC-027: When memory writes fail, transition returns warn (exit 0), not block.
    #[test]
    fn transition_memory_fail_warns() {
        let dir = TempDir::new().unwrap();
        write_plan_state(&dir);
        block_memory_dir(&dir);

        let output = super::run("solution", Some("plan"), None, dir.path());

        assert!(
            matches!(output.status, crate::output::Status::Warn),
            "expected Warn status when memory writes fail, got {:?}: {}",
            output.status,
            output.message
        );
        assert!(
            output.message.contains("[warnings:"),
            "expected warnings in message, got: {}",
            output.message
        );
    }

    // PC-028: Each memory error is captured individually in the warning output.
    #[test]
    fn transition_captures_each_memory_error() {
        let dir = TempDir::new().unwrap();
        write_plan_state(&dir);
        block_memory_dir(&dir);

        let output = super::run("solution", Some("plan"), None, dir.path());

        // write_action and write_work_item both write to docs/memory and should fail.
        // The warning must contain both error descriptions.
        assert!(
            output.message.contains("write_action"),
            "expected 'write_action' in warnings, got: {}",
            output.message
        );
        assert!(
            output.message.contains("write_work_item"),
            "expected 'write_work_item' in warnings, got: {}",
            output.message
        );
    }

    // PC-029: When all memory writes succeed, output has no warnings.
    #[test]
    fn transition_success_no_warnings() {
        let dir = TempDir::new().unwrap();
        write_plan_state(&dir);
        // Create the docs/memory and work-items dirs so write_action and write_work_item succeed.
        std::fs::create_dir_all(dir.path().join("docs/memory/work-items")).unwrap();

        let output = super::run("solution", Some("plan"), None, dir.path());

        assert!(
            matches!(output.status, crate::output::Status::Pass),
            "expected Pass status when memory writes succeed, got {:?}: {}",
            output.status,
            output.message
        );
        assert!(
            !output.message.contains("[warnings:"),
            "expected no warnings in message, got: {}",
            output.message
        );
    }

    // PC-030: state.json retains the new phase even after memory write failures.
    #[test]
    fn state_persists_after_memory_failure() {
        let dir = TempDir::new().unwrap();
        write_plan_state(&dir);
        block_memory_dir(&dir);

        super::run("solution", Some("plan"), None, dir.path());

        let state_path = dir.path().join(".claude/workflow/state.json");
        assert!(state_path.exists(), "state.json must exist after transition");
        let content = std::fs::read_to_string(&state_path).unwrap();
        let state = WorkflowState::from_json(&content).unwrap();
        assert_eq!(
            state.phase,
            Phase::Solution,
            "phase must be Solution after transition, even when memory writes fail"
        );
    }
}

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
