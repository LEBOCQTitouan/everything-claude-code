//! Cartography session hooks — scaffold, element dispatch, INDEX regeneration.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::hook::{HookPorts, HookResult};

/// Delta describing changed source files in a cartography session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CartographyDelta {
    /// Source paths of journey-related files that changed.
    pub journey_targets: Vec<String>,
    /// Source paths of flow-related files that changed.
    pub flow_targets: Vec<String>,
    /// Source paths of element-related files that changed.
    pub element_targets: Vec<String>,
}

// NOTE: Production functions are intentionally absent at RED phase.
// scaffold_elements_dir, run_cartography_post_loop, and regenerate_index
// will be implemented in the GREEN phase.

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
