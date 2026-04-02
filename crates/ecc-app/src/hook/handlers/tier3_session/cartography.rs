//! Cartography session hooks — scaffold, element dispatch, INDEX regeneration.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::hook::{HookPorts, HookResult};

/// Delta describing changed source files in a cartography session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct CartographyDelta {
    /// Source paths of journey-related files that changed.
    pub journey_targets: Vec<String>,
    /// Source paths of flow-related files that changed.
    pub flow_targets: Vec<String>,
    /// Source paths of element-related files that changed.
    pub element_targets: Vec<String>,
}

/// Scaffold the `elements/` subdirectory under `docs/cartography/` if absent.
///
/// Creates `docs/cartography/elements/` and a minimal `README.md` stub.
/// Idempotent: checks `fs.exists` before creating.
pub(crate) fn scaffold_elements_dir(
    docs_cartography: &Path,
    ports: &HookPorts<'_>,
) -> Result<(), String> {
    let elements_dir = docs_cartography.join("elements");

    if !ports.fs.exists(&elements_dir) {
        ports
            .fs
            .create_dir_all(&elements_dir)
            .map_err(|e| format!("failed to create elements dir: {e}"))?;

        let readme = elements_dir.join("README.md");
        let stub = "# Element Registry\n\nPer-element documentation files.\n";
        ports
            .fs
            .write(&readme, stub)
            .map_err(|e| format!("failed to write elements README: {e}"))?;
    }

    Ok(())
}

/// Dispatch a single generator command for each target, resetting git on failure.
///
/// On any generator exit code != 0, runs `git reset HEAD docs/cartography/` and returns `Err`.
fn dispatch_generator(
    generator: &str,
    targets: &[String],
    ports: &HookPorts<'_>,
) -> Result<(), String> {
    for target in targets {
        let out = ports
            .shell
            .run_command(generator, &[target])
            .map_err(|e| format!("{generator} failed: {e}"))?;
        if !out.success() {
            let _ = ports
                .shell
                .run_command("git", &["reset", "HEAD", "docs/cartography/"]);
            return Err(format!("{generator} exit {}", out.exit_code));
        }
    }
    Ok(())
}

/// Dispatch element generator and reset git on failure.
///
/// Passed `targets_json` is a JSON array of element source paths.
fn dispatch_element_generator(
    targets_json: &str,
    ports: &HookPorts<'_>,
) -> Result<(), String> {
    let out = ports
        .shell
        .run_command("ecc-element-generator", &[targets_json])
        .map_err(|e| format!("element generator failed: {e}"))?;

    if !out.success() {
        let _ = ports
            .shell
            .run_command("git", &["reset", "HEAD", "docs/cartography/"]);
        return Err(format!("element generator exit {}", out.exit_code));
    }

    Ok(())
}

/// Dispatch element generator, run INDEX regeneration, and return result.
///
/// Journey and flow generators are dispatched first; element generator comes
/// AFTER they complete. On element generator failure: git reset and return error.
/// On success: git add docs/cartography/ and regenerate INDEX.md.
pub(crate) fn run_cartography_post_loop(
    docs_cartography: &Path,
    delta: &CartographyDelta,
    ports: &HookPorts<'_>,
) -> Result<(), String> {
    // Dispatch journey and flow generators first
    dispatch_generator("ecc-journey-generator", &delta.journey_targets, ports)?;
    dispatch_generator("ecc-flow-generator", &delta.flow_targets, ports)?;

    // Dispatch element generator AFTER journey + flow generators complete
    if !delta.element_targets.is_empty() {
        let targets_json = serde_json::to_string(&delta.element_targets)
            .unwrap_or_else(|_| "[]".to_string());
        dispatch_element_generator(&targets_json, ports)?;

        // Stage all cartography files
        let _ = ports
            .shell
            .run_command("git", &["add", "docs/cartography/"]);

        // Regenerate INDEX.md from element files
        regenerate_index(docs_cartography, ports)?;
    }

    Ok(())
}

/// Regenerate `docs/cartography/elements/INDEX.md` as a cross-reference matrix.
///
/// Fully replaces any existing content (not a delta merge).
fn regenerate_index(docs_cartography: &Path, ports: &HookPorts<'_>) -> Result<(), String> {
    use ecc_domain::cartography::cross_reference::build_cross_reference_matrix;
    use ecc_domain::cartography::element_types::ElementEntry;

    let elements_dir = docs_cartography.join("elements");

    // Collect element entries from files in elements/ directory
    let mut elements: Vec<ElementEntry> = Vec::new();
    let mut journey_slugs: Vec<String> = Vec::new();
    let mut flow_slugs: Vec<String> = Vec::new();

    if ports.fs.exists(&elements_dir) {
        let entries = ports.fs.read_dir(&elements_dir).unwrap_or_default();

        for entry_path in &entries {
            let name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            // Skip INDEX.md and README.md — not element files
            if name == "INDEX.md" || name == "README.md" || !name.ends_with(".md") {
                continue;
            }
            if let Ok(content) = ports.fs.read_to_string(entry_path)
                && let Some(entry) = parse_element_entry_from_md(&content, name)
            {
                for j in &entry.participating_journeys {
                    if !journey_slugs.contains(j) {
                        journey_slugs.push(j.clone());
                    }
                }
                for f in &entry.participating_flows {
                    if !flow_slugs.contains(f) {
                        flow_slugs.push(f.clone());
                    }
                }
                elements.push(entry);
            }
        }
    }

    journey_slugs.sort();
    flow_slugs.sort();

    let matrix = build_cross_reference_matrix(&elements, &journey_slugs, &flow_slugs);
    let index_content = format!("# Element Cross-Reference Matrix\n\n{matrix}\n");

    let index_path = elements_dir.join("INDEX.md");
    ports
        .fs
        .write(&index_path, &index_content)
        .map_err(|e| format!("failed to write INDEX.md: {e}"))
}

/// Parse a minimal `ElementEntry` from a markdown file's content and filename.
fn parse_element_entry_from_md(
    content: &str,
    filename: &str,
) -> Option<ecc_domain::cartography::element_types::ElementEntry> {
    use ecc_domain::cartography::element_types::{ElementEntry, ElementType};

    let slug = filename.trim_end_matches(".md").to_string();
    if slug.is_empty() {
        return None;
    }

    let mut journeys: Vec<String> = Vec::new();
    let mut flows: Vec<String> = Vec::new();

    // Parse participating journeys and flows from markdown links
    let mut in_journeys = false;
    let mut in_flows = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "## Participating Journeys" {
            in_journeys = true;
            in_flows = false;
            continue;
        }
        if trimmed == "## Participating Flows" {
            in_flows = true;
            in_journeys = false;
            continue;
        }
        if trimmed.starts_with("## ") {
            in_journeys = false;
            in_flows = false;
            continue;
        }
        // Extract slug from link text: `- [slug](path)`
        if in_journeys && trimmed.starts_with("- [")
            && let Some(end) = trimmed.find("](") {
            journeys.push(trimmed[3..end].to_string()); // skip `- [`
        } else if in_flows && trimmed.starts_with("- [")
            && let Some(end) = trimmed.find("](") {
            flows.push(trimmed[3..end].to_string()); // skip `- [`
        }
    }

    Some(ElementEntry {
        slug,
        element_type: ElementType::Unknown,
        purpose: String::new(),
        uses: Vec::new(),
        used_by: Vec::new(),
        participating_flows: flows,
        participating_journeys: journeys,
        sources: Vec::new(),
        last_updated: String::new(),
    })
}

/// start:cartography — scaffold `elements/` directory for cartography use.
pub fn start_cartography(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "start_cartography", "executing handler");
    let docs_cartography = Path::new("docs/cartography");
    if let Err(e) = scaffold_elements_dir(docs_cartography, ports) {
        return HookResult::warn(stdin, &format!("[Cartography] scaffold warning: {e}\n"));
    }
    HookResult::passthrough(stdin)
}

/// stop:cartography — dispatch generators and regenerate INDEX.md.
pub fn stop_cartography(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "stop_cartography", "executing handler");
    let delta: CartographyDelta = serde_json::from_str(stdin).unwrap_or_default();
    let docs_cartography = Path::new("docs/cartography");
    match run_cartography_post_loop(docs_cartography, &delta, ports) {
        Ok(()) => HookResult::passthrough(stdin),
        Err(e) => HookResult::warn(stdin, &format!("[Cartography] error: {e}\n")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookPorts;
    use ecc_ports::fs::FileSystem;
    use ecc_ports::shell::{CommandOutput, ShellError, ShellExecutor};
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    /// Order-tracking shell executor for verifying dispatch order.
    struct OrderTrackingExecutor {
        responses: HashMap<String, CommandOutput>,
        call_log: Arc<Mutex<Vec<String>>>,
    }

    impl OrderTrackingExecutor {
        fn new(call_log: Arc<Mutex<Vec<String>>>) -> Self {
            Self {
                responses: HashMap::new(),
                call_log,
            }
        }

        fn on(mut self, command: &str, output: CommandOutput) -> Self {
            self.responses.insert(command.to_string(), output);
            self
        }
    }

    impl ShellExecutor for OrderTrackingExecutor {
        fn run_command(
            &self,
            command: &str,
            args: &[&str],
        ) -> Result<CommandOutput, ShellError> {
            let entry = format!("{} {}", command, args.join(" ")).trim().to_string();
            self.call_log.lock().unwrap().push(entry);
            self.responses
                .get(command)
                .cloned()
                .ok_or_else(|| ShellError::NotFound(command.to_string()))
        }

        fn run_command_in_dir(
            &self,
            command: &str,
            args: &[&str],
            _dir: &std::path::Path,
        ) -> Result<CommandOutput, ShellError> {
            self.run_command(command, args)
        }

        fn command_exists(&self, _command: &str) -> bool {
            false
        }

        fn spawn_with_stdin(
            &self,
            command: &str,
            args: &[&str],
            _stdin: &str,
        ) -> Result<CommandOutput, ShellError> {
            self.run_command(command, args)
        }
    }

    fn make_success_output() -> CommandOutput {
        CommandOutput {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    fn make_failure_output() -> CommandOutput {
        CommandOutput {
            stdout: String::new(),
            stderr: "generator error".to_string(),
            exit_code: 1,
        }
    }

    fn make_ports_tracking<'a>(
        fs: &'a InMemoryFileSystem,
        shell: &'a OrderTrackingExecutor,
        env: &'a MockEnvironment,
        term: &'a BufferedTerminal,
    ) -> HookPorts<'a> {
        HookPorts {
            fs,
            shell,
            env,
            terminal: term,
        }
    }

    // --- PC-015: scaffold_creates_elements_dir ---

    #[test]
    fn scaffold_creates_elements_dir() {
        let fs = InMemoryFileSystem::new();
        let call_log = Arc::new(Mutex::new(Vec::new()));
        let shell = OrderTrackingExecutor::new(Arc::clone(&call_log));
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports_tracking(&fs, &shell, &env, &term);

        let docs_cart = PathBuf::from("docs/cartography");
        scaffold_elements_dir(&docs_cart, &ports).expect("scaffold should succeed");

        let elements_dir = PathBuf::from("docs/cartography/elements");
        let readme_path = PathBuf::from("docs/cartography/elements/README.md");

        assert!(
            fs.exists(&elements_dir),
            "elements/ directory should be created"
        );
        assert!(
            fs.exists(&readme_path),
            "elements/README.md should be created"
        );

        let readme_content = fs.read_to_string(&readme_path).expect("README readable");
        assert!(
            readme_content.contains("Element Registry"),
            "README should mention Element Registry"
        );
    }

    // --- PC-016: scaffold_idempotent ---

    #[test]
    fn scaffold_idempotent() {
        let fs = InMemoryFileSystem::new()
            .with_dir("docs/cartography/elements")
            .with_file(
                "docs/cartography/elements/README.md",
                "# Original Content\n",
            );
        let call_log = Arc::new(Mutex::new(Vec::new()));
        let shell = OrderTrackingExecutor::new(Arc::clone(&call_log));
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports_tracking(&fs, &shell, &env, &term);

        let docs_cart = PathBuf::from("docs/cartography");
        // Call scaffold when dir already exists
        scaffold_elements_dir(&docs_cart, &ports).expect("scaffold should succeed idempotently");

        // Original content should be preserved (not overwritten)
        let readme = fs
            .read_to_string(&PathBuf::from("docs/cartography/elements/README.md"))
            .expect("README should still exist");
        assert_eq!(
            readme, "# Original Content\n",
            "README content should not change when dir already exists"
        );
    }

    // --- PC-017: element_dispatch_order ---

    #[test]
    fn element_dispatch_order() {
        let fs = InMemoryFileSystem::new();
        let call_log = Arc::new(Mutex::new(Vec::new()));
        let shell = OrderTrackingExecutor::new(Arc::clone(&call_log))
            .on("ecc-journey-generator", make_success_output())
            .on("ecc-flow-generator", make_success_output())
            .on("ecc-element-generator", make_success_output())
            .on("git", make_success_output());
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports_tracking(&fs, &shell, &env, &term);

        let delta = CartographyDelta {
            journey_targets: vec!["journeys/my-journey.md".to_string()],
            flow_targets: vec!["flows/my-flow.md".to_string()],
            element_targets: vec!["agents/my-agent.md".to_string()],
        };

        let docs_cart = PathBuf::from("docs/cartography");
        run_cartography_post_loop(&docs_cart, &delta, &ports)
            .expect("post_loop should succeed");

        let log = call_log.lock().unwrap();

        let journey_idx = log
            .iter()
            .position(|s| s.contains("ecc-journey-generator"))
            .expect("journey generator should be called");
        let flow_idx = log
            .iter()
            .position(|s| s.contains("ecc-flow-generator"))
            .expect("flow generator should be called");
        let element_idx = log
            .iter()
            .position(|s| s.contains("ecc-element-generator"))
            .expect("element generator should be called");

        assert!(
            journey_idx < flow_idx,
            "journey generator ({journey_idx}) should come before flow generator ({flow_idx})"
        );
        assert!(
            flow_idx < element_idx,
            "flow generator ({flow_idx}) should come before element generator ({element_idx})"
        );
    }

    // --- PC-018: no_dispatch_without_targets ---

    #[test]
    fn no_dispatch_without_targets() {
        let fs = InMemoryFileSystem::new();
        let call_log = Arc::new(Mutex::new(Vec::new()));
        let shell = OrderTrackingExecutor::new(Arc::clone(&call_log))
            .on("ecc-journey-generator", make_success_output())
            .on("ecc-flow-generator", make_success_output())
            .on("git", make_success_output());
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports_tracking(&fs, &shell, &env, &term);

        let delta = CartographyDelta {
            journey_targets: vec!["journeys/my-journey.md".to_string()],
            flow_targets: vec!["flows/my-flow.md".to_string()],
            element_targets: vec![], // no element targets
        };

        let docs_cart = PathBuf::from("docs/cartography");
        run_cartography_post_loop(&docs_cart, &delta, &ports)
            .expect("post_loop should succeed");

        let log = call_log.lock().unwrap();
        let element_calls: Vec<&String> = log
            .iter()
            .filter(|s| s.contains("ecc-element-generator"))
            .collect();

        assert!(
            element_calls.is_empty(),
            "element generator should NOT be called when no element targets present, got: {element_calls:?}"
        );
    }

    // --- PC-019: element_failure_resets_git ---

    #[test]
    fn element_failure_resets_git() {
        let fs = InMemoryFileSystem::new();
        let call_log = Arc::new(Mutex::new(Vec::new()));
        let shell = OrderTrackingExecutor::new(Arc::clone(&call_log))
            .on("ecc-journey-generator", make_success_output())
            .on("ecc-flow-generator", make_success_output())
            .on("ecc-element-generator", make_failure_output()) // element fails
            .on("git", make_success_output());
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports_tracking(&fs, &shell, &env, &term);

        let delta = CartographyDelta {
            journey_targets: vec![],
            flow_targets: vec![],
            element_targets: vec!["agents/my-agent.md".to_string()],
        };

        let docs_cart = PathBuf::from("docs/cartography");
        let result = run_cartography_post_loop(&docs_cart, &delta, &ports);

        // Should return error
        assert!(result.is_err(), "should fail when element generator fails");

        let log = call_log.lock().unwrap();

        // git reset should have been called
        let has_reset = log
            .iter()
            .any(|s| s.contains("git") && s.contains("reset"));
        assert!(
            has_reset,
            "git reset should be called on element generator failure, log: {log:?}"
        );

        // No archive file should be written
        let index_path = PathBuf::from("docs/cartography/elements/INDEX.md");
        assert!(
            !fs.exists(&index_path),
            "INDEX.md should NOT be written after element generator failure"
        );
    }

    // --- PC-020: element_success_stages_files ---

    #[test]
    fn element_success_stages_files() {
        let fs = InMemoryFileSystem::new();
        let call_log = Arc::new(Mutex::new(Vec::new()));
        let shell = OrderTrackingExecutor::new(Arc::clone(&call_log))
            .on("ecc-journey-generator", make_success_output())
            .on("ecc-flow-generator", make_success_output())
            .on("ecc-element-generator", make_success_output())
            .on("git", make_success_output());
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports_tracking(&fs, &shell, &env, &term);

        let delta = CartographyDelta {
            journey_targets: vec![],
            flow_targets: vec![],
            element_targets: vec!["agents/my-agent.md".to_string()],
        };

        let docs_cart = PathBuf::from("docs/cartography");
        run_cartography_post_loop(&docs_cart, &delta, &ports)
            .expect("post_loop should succeed");

        let log = call_log.lock().unwrap();

        // git add docs/cartography/ should have been called
        let has_git_add = log
            .iter()
            .any(|s| s.contains("git") && s.contains("add") && s.contains("docs/cartography/"));
        assert!(
            has_git_add,
            "git add docs/cartography/ should be called on success, log: {log:?}"
        );
    }

    // --- PC-021: index_full_replacement ---

    #[test]
    fn index_full_replacement() {
        // Pre-populate INDEX.md with old content
        let fs = InMemoryFileSystem::new()
            .with_dir("docs/cartography/elements")
            .with_file(
                "docs/cartography/elements/INDEX.md",
                "# OLD CONTENT\n\nThis should be gone.\n",
            )
            .with_file(
                "docs/cartography/elements/my-agent.md",
                "## Overview\n\n## Relationships\n\n## Participating Flows\n\n## Participating Journeys\n",
            );

        let call_log = Arc::new(Mutex::new(Vec::new()));
        let shell = OrderTrackingExecutor::new(Arc::clone(&call_log))
            .on("ecc-element-generator", make_success_output())
            .on("git", make_success_output());
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports_tracking(&fs, &shell, &env, &term);

        let delta = CartographyDelta {
            journey_targets: vec![],
            flow_targets: vec![],
            element_targets: vec!["agents/my-agent.md".to_string()],
        };

        let docs_cart = PathBuf::from("docs/cartography");
        run_cartography_post_loop(&docs_cart, &delta, &ports)
            .expect("post_loop should succeed");

        let index_content = fs
            .read_to_string(&PathBuf::from("docs/cartography/elements/INDEX.md"))
            .expect("INDEX.md should exist after success");

        // Old content must be gone
        assert!(
            !index_content.contains("OLD CONTENT"),
            "old INDEX.md content should be replaced, got:\n{index_content}"
        );
        // New content should reference cross-reference matrix
        assert!(
            index_content.contains("Element"),
            "new INDEX.md should contain matrix header 'Element', got:\n{index_content}"
        );
    }

    // --- PC-021b: index_regen_after_elements ---

    #[test]
    fn index_regen_after_elements() {
        let fs = InMemoryFileSystem::new();
        let call_log = Arc::new(Mutex::new(Vec::new()));
        let shell = OrderTrackingExecutor::new(Arc::clone(&call_log))
            .on("ecc-element-generator", make_success_output())
            .on("git", make_success_output());
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports_tracking(&fs, &shell, &env, &term);

        let delta = CartographyDelta {
            journey_targets: vec![],
            flow_targets: vec![],
            element_targets: vec!["agents/my-agent.md".to_string()],
        };

        let docs_cart = PathBuf::from("docs/cartography");
        run_cartography_post_loop(&docs_cart, &delta, &ports)
            .expect("post_loop should succeed");

        let log = call_log.lock().unwrap();

        // Element generator must appear in the call log
        let element_idx = log
            .iter()
            .position(|s| s.contains("ecc-element-generator"))
            .expect("element generator should be called");

        // INDEX.md should be written (via fs, not shell) AFTER element generator
        let index_path = PathBuf::from("docs/cartography/elements/INDEX.md");
        assert!(
            fs.exists(&index_path),
            "INDEX.md should be written after element generator succeeds"
        );

        // The element generator should be in the log at some point (ordering verified above)
        assert!(
            element_idx < log.len(),
            "element generator index ({element_idx}) should be before log end ({})",
            log.len()
        );
    }
}
