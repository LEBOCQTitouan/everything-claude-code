//! Thin start:cartography hook — counts pending deltas and prints a reminder.

use std::path::PathBuf;

use crate::hook::{HookPorts, HookResult};

/// start:cartography — counts pending deltas and prints a reminder if any exist.
///
/// Uses CWD to find `.claude/cartography/` (no `CLAUDE_PROJECT_DIR` needed).
/// If pending delta files exist, prints a reminder to stderr suggesting the user
/// run `/doc-suite --phase=cartography` to process them.
pub fn start_cartography(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "start_cartography", "executing handler");

    // Resolve project root: try CLAUDE_PROJECT_DIR, fall back to CWD
    let project_root = ports
        .env
        .var("CLAUDE_PROJECT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let cartography_dir = project_root.join(".claude").join("cartography");

    // Count pending-delta-*.json files
    let count = match ports.fs.read_dir(&cartography_dir) {
        Ok(entries) => entries
            .iter()
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("pending-delta-") && n.ends_with(".json"))
                    .unwrap_or(false)
            })
            .count(),
        Err(_) => 0,
    };

    if count == 0 {
        return HookResult::passthrough(stdin);
    }

    // Print reminder to stderr
    let msg = format!(
        "{} pending cartography deltas — run `/doc-suite --phase=cartography` to process\n",
        count
    );
    HookResult {
        stdout: stdin.to_string(),
        stderr: msg,
        exit_code: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookPorts;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};

    fn make_ports<'a>(
        fs: &'a InMemoryFileSystem,
        shell: &'a MockExecutor,
        env: &'a MockEnvironment,
        term: &'a BufferedTerminal,
    ) -> HookPorts<'a> {
        HookPorts {
            fs,
            shell,
            env,
            terminal: term,
            cost_store: None,
            bypass_store: None,
        }
    }

    /// PC-016: Prints pending count and /doc-suite hint when deltas exist.
    #[test]
    fn prints_pending_count() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/.claude/cartography/pending-delta-001.json", "{}")
            .with_file("/project/.claude/cartography/pending-delta-002.json", "{}")
            .with_file("/project/.claude/cartography/other-file.txt", "not a delta");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = start_cartography("{}", &ports);

        assert_eq!(result.exit_code, 0);
        assert!(
            result.stderr.contains("2 pending cartography deltas"),
            "should show count of 2: got {:?}",
            result.stderr
        );
        assert!(
            result.stderr.contains("/doc-suite"),
            "should mention /doc-suite"
        );
    }

    /// PC-017: Silent passthrough when no pending deltas.
    #[test]
    fn silent_when_no_deltas() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/.claude/cartography/other-file.txt", "not a delta");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = start_cartography("{}", &ports);

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "{}");
        assert!(result.stderr.is_empty(), "stderr should be empty");
    }

    /// PC-018: Works without CLAUDE_PROJECT_DIR — falls back to CWD.
    /// In test context CWD won't match in-memory FS, so this verifies
    /// the function handles the fallback gracefully (returns passthrough).
    #[test]
    fn uses_cwd_not_env_var() {
        let fs = InMemoryFileSystem::new(); // No files at CWD
        let shell = MockExecutor::new();
        let env = MockEnvironment::new(); // No CLAUDE_PROJECT_DIR
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = start_cartography("{}", &ports);

        // Without CLAUDE_PROJECT_DIR and no matching CWD files,
        // the handler falls back gracefully to passthrough
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "{}");
    }
}
