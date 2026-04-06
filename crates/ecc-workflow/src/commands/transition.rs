use std::path::Path;
use std::str::FromStr;

use ecc_domain::metrics::MetricOutcome;
use ecc_domain::workflow::concern::Concern;
use ecc_domain::workflow::phase::Phase;
use ecc_domain::workflow::state::Completion;
use ecc_domain::workflow::transition::resolve_transition_by_name;
use ecc_ports::metrics_store::MetricsStore;

use crate::io::{read_state, with_state_lock, write_state_atomic};
use crate::output::WorkflowOutput;
use crate::time::utc_now_iso8601;

#[cfg(test)]
mod tests {
    use ecc_domain::workflow::{
        concern::Concern,
        phase::Phase,
        state::{Artifacts, Toolchain, WorkflowState},
        timestamp::Timestamp,
    };
    use tempfile::TempDir;

    /// Write a Plan-phase state.json into a temp dir, ready for transition tests.
    fn write_plan_state(dir: &TempDir) {
        let wf_dir = dir.path().join(".claude/workflow");
        std::fs::create_dir_all(&wf_dir).unwrap();
        let state = WorkflowState {
            phase: Phase::Plan,
            concern: Concern::Dev,
            feature: "BL-068".to_owned(),
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

    // PC-031: try_record_transition() with success records PhaseTransition/Success event
    #[test]
    fn transition_records_success_metric() {
        use ecc_domain::metrics::{MetricEventType, MetricOutcome};
        use ecc_ports::metrics_store::MetricsStore;
        use ecc_test_support::InMemoryMetricsStore;

        let store = InMemoryMetricsStore::new();
        super::try_record_transition(
            Some(&store as &dyn MetricsStore),
            "workflow-BL-068",
            "plan",
            "solution",
            MetricOutcome::Success,
            None,
            false,
        );
        let events = store.snapshot();
        assert_eq!(events.len(), 1, "expected exactly 1 event");
        assert_eq!(events[0].event_type, MetricEventType::PhaseTransition);
        assert_eq!(events[0].outcome, MetricOutcome::Success);
        assert_eq!(events[0].session_id, "workflow-BL-068");
        assert_eq!(events[0].from_phase.as_deref(), Some("plan"));
        assert_eq!(events[0].to_phase.as_deref(), Some("solution"));
    }

    // PC-032: try_record_transition() with rejection records PhaseTransition/Rejected event
    #[test]
    fn transition_records_rejected_metric() {
        use ecc_domain::metrics::{MetricEventType, MetricOutcome};
        use ecc_ports::metrics_store::MetricsStore;
        use ecc_test_support::InMemoryMetricsStore;

        let store = InMemoryMetricsStore::new();
        super::try_record_transition(
            Some(&store as &dyn MetricsStore),
            "workflow-BL-068",
            "plan",
            "solution",
            MetricOutcome::Rejected,
            Some("Illegal transition".to_owned()),
            false,
        );
        let events = store.snapshot();
        assert_eq!(events.len(), 1, "expected exactly 1 event");
        assert_eq!(events[0].event_type, MetricEventType::PhaseTransition);
        assert_eq!(events[0].outcome, MetricOutcome::Rejected);
        assert_eq!(
            events[0].rejection_reason.as_deref(),
            Some("Illegal transition")
        );
    }

    // PC-033: try_record_transition() with disabled=true records zero events
    #[test]
    fn transition_metrics_disabled() {
        use ecc_domain::metrics::MetricOutcome;
        use ecc_ports::metrics_store::MetricsStore;
        use ecc_test_support::InMemoryMetricsStore;

        let store = InMemoryMetricsStore::new();
        super::try_record_transition(
            Some(&store as &dyn MetricsStore),
            "workflow-BL-068",
            "plan",
            "solution",
            MetricOutcome::Success,
            None,
            true, // disabled
        );
        let events = store.snapshot();
        assert_eq!(events.len(), 0, "expected zero events when disabled=true");
    }

    // PC-034: transition::run with store=None completes normally (fire-and-forget)
    #[test]
    fn transition_metrics_store_unavailable() {
        let dir = TempDir::new().unwrap();
        write_plan_state(&dir);
        std::fs::create_dir_all(dir.path().join("docs/memory/work-items")).unwrap();
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .expect("git init failed");
        let state_dir = dir.path().join(".claude/workflow");

        // Pass None store — should complete without panic
        let output =
            super::run_with_store("solution", Some("plan"), None, dir.path(), &state_dir, None);

        assert!(
            !matches!(output.status, crate::output::Status::Block),
            "expected non-block status when store=None, got {:?}: {}",
            output.status,
            output.message
        );
    }

    // PC-035: When no state.json exists, transition::run records no PhaseTransition metric event
    #[test]
    fn transition_no_state_no_metric() {
        use ecc_ports::metrics_store::MetricsStore;
        use ecc_test_support::InMemoryMetricsStore;

        let dir = TempDir::new().unwrap();
        let state_dir = dir.path().join(".claude/workflow");
        std::fs::create_dir_all(&state_dir).unwrap();
        // No state.json written

        let store = InMemoryMetricsStore::new();
        let _output = super::run_with_store(
            "solution",
            Some("plan"),
            None,
            dir.path(),
            &state_dir,
            Some(&store as &dyn MetricsStore),
        );

        let events = store.snapshot();
        assert_eq!(
            events.len(),
            0,
            "expected zero metric events when no state.json, got: {}",
            events.len()
        );
    }

    // PC-027: When memory writes fail, transition returns warn (exit 0), not block.
    #[test]
    fn transition_memory_fail_warns() {
        let dir = TempDir::new().unwrap();
        write_plan_state(&dir);
        block_memory_dir(&dir);
        let state_dir = dir.path().join(".claude/workflow");

        let output = super::run("solution", Some("plan"), None, dir.path(), &state_dir);

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
        let state_dir = dir.path().join(".claude/workflow");

        let output = super::run("solution", Some("plan"), None, dir.path(), &state_dir);

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
        // Initialize as git repo so resolve_repo_root succeeds for daily/memory-index.
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .expect("git init failed");
        let state_dir = dir.path().join(".claude/workflow");

        let output = super::run("solution", Some("plan"), None, dir.path(), &state_dir);

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
        let state_dir = dir.path().join(".claude/workflow");

        super::run("solution", Some("plan"), None, dir.path(), &state_dir);

        let state_path = state_dir.join("state.json");
        assert!(
            state_path.exists(),
            "state.json must exist after transition"
        );
        let content = std::fs::read_to_string(&state_path).unwrap();
        let state = WorkflowState::from_json(&content).unwrap();
        assert_eq!(
            state.phase,
            Phase::Solution,
            "phase must be Solution after transition, even when memory writes fail"
        );
    }
}

/// Fire-and-forget helper: build and record a PhaseTransition metric event.
///
/// - If `disabled` is true, skips immediately.
/// - If `store` is None, skips immediately.
/// - On store error, logs a warning and returns without panicking.
pub(crate) fn try_record_transition(
    store: Option<&dyn MetricsStore>,
    session_id: &str,
    from: &str,
    to: &str,
    outcome: MetricOutcome,
    rejection_reason: Option<String>,
    disabled: bool,
) {
    use ecc_domain::metrics::MetricEvent;
    let ts = utc_now_iso8601();
    let event = match MetricEvent::phase_transition(
        session_id.to_owned(),
        ts,
        from.to_owned(),
        to.to_owned(),
        outcome,
        rejection_reason,
    ) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("try_record_transition: failed to build event: {e}");
            return;
        }
    };
    if let Err(e) = ecc_app::metrics_mgmt::record_if_enabled(store, &event, disabled) {
        tracing::warn!("try_record_transition: record_if_enabled failed: {e}");
    }
}

/// Run the `transition` subcommand with an optional metrics store.
///
/// This is the primary entry point. [`run`] delegates here with `store = None`.
/// Call this directly (e.g., from tests or main) when you need to inject a store.
pub fn run_with_store(
    target: &str,
    artifact: Option<&str>,
    path: Option<&str>,
    project_dir: &Path,
    state_dir: &Path,
    store: Option<&dyn MetricsStore>,
) -> WorkflowOutput {
    let target = target.to_owned();
    let artifact = artifact.map(str::to_owned);
    let path = path.map(str::to_owned);

    let metrics_disabled = std::env::var("ECC_METRICS_DISABLED").as_deref() == Ok("1");

    let result = with_state_lock(state_dir, || {
        let mut state = match read_state(state_dir) {
            Ok(None) => {
                return (
                    WorkflowOutput::warn("No state.json found — workflow not initialized"),
                    None,
                    None::<(String, String, String, MetricOutcome, Option<String>)>,
                );
            }
            Ok(Some(s)) => s,
            Err(e) => {
                return (
                    WorkflowOutput::warn(format!("Failed to read state: {e}")),
                    None,
                    None,
                );
            }
        };
        let from = state.phase;
        tracing::info!(from_phase = %from, target = %target, "transition: attempting phase change");
        let to = match resolve_transition_by_name(from, &target) {
            Ok(t) => t,
            Err(e) => {
                let reason = format!("Illegal transition: {e}");
                let feature = state.feature.clone();
                let metric_info = Some((
                    format!("workflow-{feature}"),
                    from.to_string(),
                    target.clone(),
                    MetricOutcome::Rejected,
                    Some(reason.clone()),
                ));
                return (WorkflowOutput::block(reason), None, metric_info);
            }
        };
        state.phase = to;
        if let Some(ref artifact_name) = artifact
            && let Err(output) =
                apply_artifact_stamp(&mut state, artifact_name, path.as_deref(), to)
        {
            return (output, None, None);
        }
        match write_state_atomic(state_dir, &state) {
            Ok(()) => {
                tracing::info!(from = %from, to = %to, feature = %state.feature, "workflow transition");
                let memory_info = artifact
                    .as_ref()
                    .map(|a| (a.clone(), state.feature.clone(), state.concern));
                let feature = state.feature.clone();
                let metric_info = Some((
                    format!("workflow-{feature}"),
                    from.to_string(),
                    to.to_string(),
                    MetricOutcome::Success,
                    None,
                ));
                (
                    WorkflowOutput::pass(format!("Phase transition: {from} -> {to}")),
                    memory_info,
                    metric_info,
                )
            }
            Err(e) => (
                WorkflowOutput::block(format!("Failed to write state.json: {e}")),
                None,
                None,
            ),
        }
    });

    match result {
        Ok((output, memory_info_opt, metric_info_opt)) => {
            if let Some((session_id, from, to, outcome, rejection_reason)) = metric_info_opt {
                try_record_transition(
                    store,
                    &session_id,
                    &from,
                    &to,
                    outcome,
                    rejection_reason,
                    metrics_disabled,
                );
            }
            match memory_info_opt {
                Some((artifact_name, feature, concern)) => {
                    write_memory_best_effort(&artifact_name, &feature, concern, project_dir, output)
                }
                None => output,
            }
        }
        Err(e) => WorkflowOutput::block(format!("Lock error: {e}")),
    }
}

/// Stamp artifact fields on `state` and validate the artifact name.
///
/// Returns `Err(WorkflowOutput)` if `artifact_name` is unknown.
fn apply_artifact_stamp(
    state: &mut ecc_domain::workflow::state::WorkflowState,
    artifact_name: &str,
    path: Option<&str>,
    to: Phase,
) -> Result<(), WorkflowOutput> {
    let now = utc_now_iso8601();
    match artifact_name {
        "plan" => state.artifacts.plan = Some(now),
        "solution" => state.artifacts.solution = Some(now),
        "implement" => state.artifacts.implement = Some(now),
        other => {
            return Err(WorkflowOutput::block(format!(
                "Unknown artifact '{other}' — expected plan, solution, or implement"
            )));
        }
    }
    if let Some(p) = path {
        match artifact_name {
            "plan" => state.artifacts.spec_path = Some(p.to_owned()),
            "solution" => state.artifacts.design_path = Some(p.to_owned()),
            "implement" => state.artifacts.tasks_path = Some(p.to_owned()),
            _ => {}
        }
    }
    if to == Phase::Done {
        state.completed.push(Completion {
            phase: Phase::from_str(artifact_name).unwrap_or(Phase::Unknown),
            file: "implement-done.md".to_owned(),
            at: utc_now_iso8601(),
        });
    }
    Ok(())
}

/// Perform best-effort memory writes for a completed transition.
///
/// Collects failures into a warnings list (never blocks the transition).
fn write_memory_best_effort(
    artifact_name: &str,
    feature: &str,
    concern: Concern,
    project_dir: &Path,
    base_output: WorkflowOutput,
) -> WorkflowOutput {
    let wi_phase = match artifact_name {
        "plan" => "plan",
        "solution" => "solution",
        "implement" => "implementation",
        _ => artifact_name,
    };
    let concern_str = concern.to_string();
    let mut warnings: Vec<String> = Vec::new();

    if let Err(e) = crate::commands::memory_write::write_action(
        artifact_name,
        feature,
        "success",
        "[]",
        project_dir,
    ) {
        warnings.push(format!("write_action failed: {e}"));
    }
    if let Err(e) =
        crate::commands::memory_write::write_work_item(wi_phase, feature, &concern_str, project_dir)
    {
        warnings.push(format!("write_work_item failed: {e}"));
    }
    if let Err(e) =
        crate::commands::memory_write::write_daily(wi_phase, feature, &concern_str, project_dir)
    {
        warnings.push(format!("write_daily failed: {e}"));
    }
    if let Err(e) = crate::commands::memory_write::write_memory_index(project_dir) {
        warnings.push(format!("write_memory_index failed: {e}"));
    }

    if warnings.is_empty() {
        base_output
    } else {
        let warn_text = warnings.join("; ");
        WorkflowOutput::warn(format!("{} [warnings: {warn_text}]", base_output.message))
    }
}

/// Run the `transition` subcommand: advance the workflow to the target phase.
///
/// Delegates to [`run_with_store`] with `store = None`.
/// Construct a [`SqliteMetricsStore`] in `main.rs` and call [`run_with_store`] directly
/// when metrics instrumentation is needed.
pub fn run(
    target: &str,
    artifact: Option<&str>,
    path: Option<&str>,
    project_dir: &Path,
    state_dir: &Path,
) -> WorkflowOutput {
    run_with_store(target, artifact, path, project_dir, state_dir, None)
}
