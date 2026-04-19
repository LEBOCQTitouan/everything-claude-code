//! Tests for agent interaction and output validation — PC-029 through PC-037.
//! Covers enriched context dispatch, journey/flow schema validation, link generation,
//! delta-merge preservation, GAP markers, external I/O detection, and commit scoping.
use super::*;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::CommandOutput;
use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};

use super::start_test_helpers::make_ports;

// ────────────────────────────────────────────────────────────────────────
// PC-029 through PC-037: agent interaction and output validation tests
// ────────────────────────────────────────────────────────────────────────

/// Build a valid journey file content string for tests.
fn make_journey_content(extra: &str) -> String {
    format!(
        "# Test Journey\n\n## Overview\nA test actor does something.\n\n## Mermaid Diagram\n```mermaid\nflowchart LR\n  A --> B\n```\n\n## Steps\n1. Step one\n{}\n## Related Flows\n- [test-flow](../flows/test-flow.md)\n",
        extra
    )
}

/// Build a valid flow file content string for tests.
fn make_flow_content(extra: &str) -> String {
    format!(
        "# Test Flow\n\n## Overview\nData moves from A to B.\n\n## Mermaid Diagram\n```mermaid\nflowchart LR\n  A --> B\n```\n\n## Source-Destination\nSource: Service A\nDestination: Service B\n\n## Transformation Steps\n1. Transform input\n{}\n## Error Paths\n- On error: retry\n",
        extra
    )
}

/// Helper: build a SessionDelta with a specific changed file path.
fn make_delta_with_file(session_id: &str, timestamp: u64, file_path: &str) -> SessionDelta {
    SessionDelta {
        session_id: session_id.to_string(),
        timestamp,
        changed_files: vec![ChangedFile {
            path: file_path.to_string(),
            classification: "ecc-app".to_string(),
        }],
        project_type: ProjectType::Rust,
    }
}

/// Helper: common MockExecutor setup for process_cartography with enriched agent input.
/// The agent receives enriched_input_json as the --input arg value and returns journey_output.
fn make_shell_for_agent(enriched_input_json: &str, agent_output: &str) -> MockExecutor {
    MockExecutor::new()
        .on_args(
            "git",
            &["status", "--porcelain", "docs/cartography/"],
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        )
        .on_args(
            "claude",
            &["--agent", "cartographer", "--input", enriched_input_json],
            CommandOutput {
                stdout: agent_output.to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        )
        // Element generator dispatch — succeeds by default for tests using this helper.
        // The exact prompt arg varies, so use command-level fallback via on().
        // NOTE: on_args takes priority over on(), so the cartographer call above is unaffected.
        .on(
            "claude",
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        )
        .on_args(
            "git",
            &["add", "docs/cartography/"],
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        )
        .on_args(
            "git",
            &["commit", "-m", "docs(cartography): update"],
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        )
}

/// PC-029: agent dispatch passes delta context including existing journey content for merge.
///
/// When a journey file already exists, the handler must include its content in the
/// enriched context passed to the agent so the agent can delta-merge new steps.
#[test]
fn agent_receives_existing_content_for_merge() {
    let existing_journey = make_journey_content("2. Step two\n");
    let delta = make_delta_with_file("session-merge", 1000, "crates/ecc-app/src/handler.rs");
    let delta_json = serde_json::to_string(&delta).unwrap();

    // Pre-populate an existing journey file
    let fs = InMemoryFileSystem::new()
        .with_file(
            "/project/.claude/cartography/pending-delta-session-merge.json",
            &delta_json,
        )
        .with_file(
            "/project/docs/cartography/journeys/ecc-app.md",
            &existing_journey,
        )
        .with_dir("/project/docs/cartography/journeys")
        .with_dir("/project/docs/cartography/flows");

    // Build the enriched context the handler should pass to the agent
    let enriched_json = serde_json::to_string(&AgentContext {
        delta: &delta,
        existing_journey: Some(existing_journey.clone()),
        existing_flow: None,
        flow_files: vec![],
        external_io_patterns: vec![],
    })
    .unwrap();

    // New journey content returned by the agent (with step appended inside marker)
    let updated_journey = make_journey_content(
        "2. Step two\n<!-- CARTOGRAPHY: step-3 -->\n3. Step three (new)\n<!-- /CARTOGRAPHY: step-3 -->\n",
    );

    let shell = make_shell_for_agent(&enriched_json, &updated_journey);
    let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
    let term = BufferedTerminal::new();
    let ports = make_ports(&fs, &shell, &env, &term);

    let result = process_cartography("{}", &ports);

    // Handler should succeed (agent was called with enriched context matching mock)
    assert_eq!(
        result.exit_code, 0,
        "handler should succeed when agent receives existing content"
    );
    // Delta should be archived (successful run)
    assert!(
        fs.exists(std::path::Path::new(
            "/project/.claude/cartography/processed/pending-delta-session-merge.json"
        )),
        "delta should be archived after successful agent dispatch with existing content"
    );
}

/// PC-030: agent output journey file is validated for required sections before write.
///
/// The handler must validate the agent's output contains ## Mermaid Diagram and ## Steps
/// sections before persisting it.
#[test]
fn agent_output_validates_journey_schema() {
    let delta = make_delta_with_file(
        "session-validate-journey",
        1000,
        "crates/ecc-app/src/handler.rs",
    );
    let delta_json = serde_json::to_string(&delta).unwrap();

    let fs = InMemoryFileSystem::new()
        .with_file(
            "/project/.claude/cartography/pending-delta-session-validate-journey.json",
            &delta_json,
        )
        .with_dir("/project/docs/cartography/journeys")
        .with_dir("/project/docs/cartography/flows");

    // Agent returns a fully valid journey file
    let valid_journey = make_journey_content("");
    let enriched_json = serde_json::to_string(&AgentContext {
        delta: &delta,
        existing_journey: None,
        existing_flow: None,
        flow_files: vec![],
        external_io_patterns: vec![],
    })
    .unwrap();

    let shell = make_shell_for_agent(&enriched_json, &valid_journey);
    let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
    let term = BufferedTerminal::new();
    let ports = make_ports(&fs, &shell, &env, &term);

    let result = process_cartography("{}", &ports);
    assert_eq!(result.exit_code, 0);

    // The written journey file must contain ## Mermaid Diagram and ## Steps
    let written_path = std::path::Path::new("/project/docs/cartography/journeys/ecc-app.md");
    assert!(
        fs.exists(written_path),
        "journey file should have been written"
    );
    let written = fs.read_to_string(written_path).expect("journey file");
    assert!(
        written.contains("## Mermaid Diagram"),
        "journey file must contain ## Mermaid Diagram section"
    );
    assert!(
        written.contains("## Steps"),
        "journey file must contain ## Steps section"
    );
}

/// PC-031: agent output journey file contains relative path links to flow files.
///
/// When flow files exist in docs/cartography/flows/, the journey file written by the
/// handler must contain relative path links like [flow-name](../flows/flow-slug.md).
#[test]
fn journey_links_to_flows() {
    let delta = make_delta_with_file("session-links", 1000, "crates/ecc-app/src/handler.rs");
    let delta_json = serde_json::to_string(&delta).unwrap();

    // Pre-populate an existing flow file
    let flow_content = make_flow_content("");
    let fs = InMemoryFileSystem::new()
        .with_file(
            "/project/.claude/cartography/pending-delta-session-links.json",
            &delta_json,
        )
        .with_dir("/project/docs/cartography/journeys")
        .with_file(
            "/project/docs/cartography/flows/ecc-app-handler.md",
            &flow_content,
        );

    // Agent receives enriched context with flow_files populated
    let enriched_json = serde_json::to_string(&AgentContext {
        delta: &delta,
        existing_journey: None,
        existing_flow: None,
        flow_files: vec!["ecc-app-handler".to_string()],
        external_io_patterns: vec![],
    })
    .unwrap();

    // Agent returns journey with a relative link to the flow
    let journey_with_link =
        make_journey_content("[ecc-app-handler](../flows/ecc-app-handler.md)\n");
    let shell = make_shell_for_agent(&enriched_json, &journey_with_link);
    let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
    let term = BufferedTerminal::new();
    let ports = make_ports(&fs, &shell, &env, &term);

    let result = process_cartography("{}", &ports);
    assert_eq!(result.exit_code, 0);

    // The written journey file must contain relative links to the flow
    let written_path = std::path::Path::new("/project/docs/cartography/journeys/ecc-app.md");
    assert!(
        fs.exists(written_path),
        "journey file should have been written"
    );
    let written = fs.read_to_string(written_path).expect("journey file");
    assert!(
        written.contains("../flows/ecc-app-handler.md"),
        "journey file must contain relative link to flow: {}",
        written
    );
}

/// PC-032: on first run with no existing journeys, only delta-referenced files get entries.
///
/// The handler must not do a full project scan — only create journey entries for
/// files referenced in the current delta.
#[test]
fn no_backfill_on_first_run() {
    let delta = make_delta_with_file("session-first-run", 1000, "crates/ecc-app/src/handler.rs");
    let delta_json = serde_json::to_string(&delta).unwrap();

    // Fresh filesystem — no existing journeys
    let fs = InMemoryFileSystem::new()
        .with_file(
            "/project/.claude/cartography/pending-delta-session-first-run.json",
            &delta_json,
        )
        .with_dir("/project/docs/cartography/journeys")
        .with_dir("/project/docs/cartography/flows");

    let enriched_json = serde_json::to_string(&AgentContext {
        delta: &delta,
        existing_journey: None,
        existing_flow: None,
        flow_files: vec![],
        external_io_patterns: vec![],
    })
    .unwrap();
    let new_journey = make_journey_content("");
    let shell = make_shell_for_agent(&enriched_json, &new_journey);

    let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
    let term = BufferedTerminal::new();
    let ports = make_ports(&fs, &shell, &env, &term);

    let result = process_cartography("{}", &ports);
    assert_eq!(result.exit_code, 0);

    // Only one journey file should be created (for the delta's classification: ecc-app)
    let journeys_dir = std::path::Path::new("/project/docs/cartography/journeys");
    let journey_entries = fs.read_dir(journeys_dir).unwrap_or_default();
    let journey_files: Vec<_> = journey_entries
        .iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("md"))
        .collect();
    assert_eq!(
        journey_files.len(),
        1,
        "only one journey file should be created on first run, got: {:?}",
        journey_files
    );
}

/// PC-033: agent output includes GAP markers for unknown actors/triggers.
///
/// When the agent cannot determine the actor, the written file must preserve
/// <!-- GAP: ... --> markers.
#[test]
fn gap_markers_for_unknown_actors() {
    let delta = make_delta_with_file("session-gap", 1000, "crates/ecc-app/src/unknown.rs");
    let delta_json = serde_json::to_string(&delta).unwrap();

    let fs = InMemoryFileSystem::new()
        .with_file(
            "/project/.claude/cartography/pending-delta-session-gap.json",
            &delta_json,
        )
        .with_dir("/project/docs/cartography/journeys")
        .with_dir("/project/docs/cartography/flows");

    let enriched_json = serde_json::to_string(&AgentContext {
        delta: &delta,
        existing_journey: None,
        existing_flow: None,
        flow_files: vec![],
        external_io_patterns: vec![],
    })
    .unwrap();

    // Agent returns journey with GAP marker for unknown actor
    let journey_with_gap = "# Unknown Journey\n\n## Overview\n<!-- GAP: actor unknown, infer from context -->\n\n## Mermaid Diagram\n```mermaid\nflowchart LR\n  A --> B\n```\n\n## Steps\n1. Unknown step\n\n## Related Flows\n".to_string();
    let shell = make_shell_for_agent(&enriched_json, &journey_with_gap);
    let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
    let term = BufferedTerminal::new();
    let ports = make_ports(&fs, &shell, &env, &term);

    let result = process_cartography("{}", &ports);
    assert_eq!(result.exit_code, 0);

    // The written journey file must preserve GAP markers
    let written_path = std::path::Path::new("/project/docs/cartography/journeys/ecc-app.md");
    assert!(
        fs.exists(written_path),
        "journey file should have been written"
    );
    let written = fs.read_to_string(written_path).expect("journey file");
    assert!(
        written.contains("<!-- GAP:"),
        "journey file must preserve GAP markers for unknown actors: {}",
        written
    );
}

/// PC-034: agent dispatch for flows includes external I/O detection patterns.
///
/// When changed files contain paths indicative of external I/O (http, fs::, database, api),
/// the enriched context passed to the agent must include these detected patterns.
#[test]
fn flow_captures_external_io() {
    let delta = SessionDelta {
        session_id: "session-external-io".to_string(),
        timestamp: 1000,
        changed_files: vec![
            ChangedFile {
                path: "crates/ecc-infra/src/http_client.rs".to_string(),
                classification: "ecc-infra".to_string(),
            },
            ChangedFile {
                path: "crates/ecc-infra/src/database_store.rs".to_string(),
                classification: "ecc-infra".to_string(),
            },
        ],
        project_type: ProjectType::Rust,
    };
    let delta_json = serde_json::to_string(&delta).unwrap();

    let fs = InMemoryFileSystem::new()
        .with_file(
            "/project/.claude/cartography/pending-delta-session-external-io.json",
            &delta_json,
        )
        .with_dir("/project/docs/cartography/journeys")
        .with_dir("/project/docs/cartography/flows");

    // The enriched context must include detected I/O patterns from the file paths
    let enriched_json = serde_json::to_string(&AgentContext {
        delta: &delta,
        existing_journey: None,
        existing_flow: None,
        flow_files: vec![],
        external_io_patterns: vec!["database".to_string(), "http".to_string()],
    })
    .unwrap();

    let flow_output = make_flow_content("");
    let shell = make_shell_for_agent(&enriched_json, &flow_output);
    let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
    let term = BufferedTerminal::new();
    let ports = make_ports(&fs, &shell, &env, &term);

    let result = process_cartography("{}", &ports);

    // Handler must succeed — the agent was called with external_io_patterns in context
    assert_eq!(
        result.exit_code, 0,
        "handler should succeed with external I/O patterns in enriched context"
    );
    // Delta should be archived — proves the agent was called with the correct enriched context
    assert!(
        fs.exists(std::path::Path::new(
            "/project/.claude/cartography/processed/pending-delta-session-external-io.json"
        )),
        "delta should be archived, proving agent was dispatched with external I/O patterns"
    );
}

/// PC-035: agent output flow file contains ## Mermaid Diagram and ## Transformation Steps.
///
/// The handler validates flow output before writing.
#[test]
fn agent_output_validates_flow_schema() {
    let delta = make_delta_with_file(
        "session-validate-flow",
        1000,
        "crates/ecc-app/src/handler.rs",
    );
    let delta_json = serde_json::to_string(&delta).unwrap();

    let fs = InMemoryFileSystem::new()
        .with_file(
            "/project/.claude/cartography/pending-delta-session-validate-flow.json",
            &delta_json,
        )
        .with_dir("/project/docs/cartography/journeys")
        .with_dir("/project/docs/cartography/flows");

    let enriched_json = serde_json::to_string(&AgentContext {
        delta: &delta,
        existing_journey: None,
        existing_flow: None,
        flow_files: vec![],
        external_io_patterns: vec![],
    })
    .unwrap();

    // Agent returns a valid flow file
    let valid_flow = make_flow_content("");
    let shell = make_shell_for_agent(&enriched_json, &valid_flow);
    let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
    let term = BufferedTerminal::new();
    let ports = make_ports(&fs, &shell, &env, &term);

    let result = process_cartography("{}", &ports);
    assert_eq!(result.exit_code, 0);

    // The written flow file must contain ## Mermaid Diagram and ## Transformation Steps
    let written_path = std::path::Path::new("/project/docs/cartography/flows/ecc-app.md");
    assert!(
        fs.exists(written_path),
        "flow file should have been written"
    );
    let written = fs.read_to_string(written_path).expect("flow file");
    assert!(
        written.contains("## Mermaid Diagram"),
        "flow file must contain ## Mermaid Diagram section"
    );
    assert!(
        written.contains("## Transformation Steps"),
        "flow file must contain ## Transformation Steps section"
    );
}

/// PC-036: flow delta-merge only updates changed steps inside markers; unchanged preserved.
///
/// When a flow file exists with section markers, delta-merge must preserve unchanged
/// sections and only update the changed ones.
#[test]
fn flow_delta_merge_preserves_unchanged() {
    let existing_flow = "# Test Flow\n\n## Overview\nData flow.\n\n## Mermaid Diagram\n```mermaid\nflowchart LR\n  A --> B\n```\n\n## Source-Destination\nSource: A\nDestination: B\n\n## Transformation Steps\n<!-- CARTOGRAPHY: step-1 -->\nOld step 1 content.\n<!-- /CARTOGRAPHY: step-1 -->\n<!-- CARTOGRAPHY: step-2 -->\nUnchanged step 2 content.\n<!-- /CARTOGRAPHY: step-2 -->\n\n## Error Paths\n- On failure: retry\n".to_string();

    let delta = make_delta_with_file("session-flow-merge", 1000, "crates/ecc-app/src/data.rs");
    let delta_json = serde_json::to_string(&delta).unwrap();

    let fs = InMemoryFileSystem::new()
        .with_file(
            "/project/.claude/cartography/pending-delta-session-flow-merge.json",
            &delta_json,
        )
        .with_dir("/project/docs/cartography/journeys")
        .with_file("/project/docs/cartography/flows/ecc-app.md", &existing_flow);

    let enriched_json = serde_json::to_string(&AgentContext {
        delta: &delta,
        existing_journey: None,
        existing_flow: Some(existing_flow.clone()),
        flow_files: vec!["ecc-app".to_string()],
        external_io_patterns: vec![],
    })
    .unwrap();

    // Agent returns a flow that only updates step-1, step-2 remains unchanged
    let updated_flow = "# Test Flow\n\n## Overview\nData flow.\n\n## Mermaid Diagram\n```mermaid\nflowchart LR\n  A --> B\n```\n\n## Source-Destination\nSource: A\nDestination: B\n\n## Transformation Steps\n<!-- CARTOGRAPHY: step-1 -->\nUpdated step 1 content.\n<!-- /CARTOGRAPHY: step-1 -->\n<!-- CARTOGRAPHY: step-2 -->\nUnchanged step 2 content.\n<!-- /CARTOGRAPHY: step-2 -->\n\n## Error Paths\n- On failure: retry\n".to_string();
    let shell = make_shell_for_agent(&enriched_json, &updated_flow);
    let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
    let term = BufferedTerminal::new();
    let ports = make_ports(&fs, &shell, &env, &term);

    let result = process_cartography("{}", &ports);
    assert_eq!(result.exit_code, 0);

    // The written flow file must have step-1 updated and step-2 preserved
    let written_path = std::path::Path::new("/project/docs/cartography/flows/ecc-app.md");
    assert!(
        fs.exists(written_path),
        "flow file should have been written"
    );
    let written = fs.read_to_string(written_path).expect("flow file");
    assert!(
        written.contains("Updated step 1 content."),
        "step-1 should be updated: {}",
        written
    );
    assert!(
        written.contains("Unchanged step 2 content."),
        "step-2 should be preserved: {}",
        written
    );
    assert!(
        !written.contains("Old step 1 content."),
        "old step-1 content should be replaced: {}",
        written
    );
}

/// PC-037: commit command uses `git add docs/cartography/` specifically.
///
/// The handler must stage only docs/cartography/ and never use `git add .` or `git add -A`.
#[test]
fn commit_stages_only_cartography_dir() {
    let delta = make_delta_with_file(
        "session-commit-scope",
        1000,
        "crates/ecc-app/src/handler.rs",
    );
    let delta_json = serde_json::to_string(&delta).unwrap();

    let fs = InMemoryFileSystem::new()
        .with_file(
            "/project/.claude/cartography/pending-delta-session-commit-scope.json",
            &delta_json,
        )
        .with_dir("/project/docs/cartography/journeys")
        .with_dir("/project/docs/cartography/flows");

    let enriched_json = serde_json::to_string(&AgentContext {
        delta: &delta,
        existing_journey: None,
        existing_flow: None,
        flow_files: vec![],
        external_io_patterns: vec![],
    })
    .unwrap();
    let journey = make_journey_content("");

    // Register `git add docs/cartography/` as a known command that succeeds.
    // If the handler uses any other git-add variant (e.g., `git add .`), the
    // MockExecutor will return ShellError::NotFound and the agent success branch
    // will fail, causing the delta to NOT be archived.
    let shell = MockExecutor::new()
        .on_args(
            "git",
            &["status", "--porcelain", "docs/cartography/"],
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        )
        .on_args(
            "claude",
            &["--agent", "cartographer", "--input", &enriched_json],
            CommandOutput {
                stdout: journey.clone(),
                stderr: String::new(),
                exit_code: 0,
            },
        )
        // Element generator dispatch — delta contains crates/ path (element target).
        // on_args takes priority over on(), so the cartographer on_args above is unaffected.
        .on(
            "claude",
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        )
        // Only register `git add docs/cartography/` — no `git add .` registration.
        // Using `git add .` would result in ShellError::NotFound and prevent archiving.
        .on_args(
            "git",
            &["add", "docs/cartography/"],
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        )
        .on_args(
            "git",
            &["commit", "-m", "docs(cartography): update"],
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        );

    let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
    let term = BufferedTerminal::new();
    let ports = make_ports(&fs, &shell, &env, &term);

    let result = process_cartography("{}", &ports);
    assert_eq!(result.exit_code, 0);

    // Delta must be archived — proves git add docs/cartography/ succeeded
    // (any other git add variant would fail, leaving delta unarchived)
    assert!(
        fs.exists(std::path::Path::new(
            "/project/.claude/cartography/processed/pending-delta-session-commit-scope.json"
        )),
        "delta should be archived, proving git add docs/cartography/ was used"
    );
}
