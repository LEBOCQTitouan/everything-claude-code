//! AI-assisted merge via Claude CLI invocation.

use ecc_ports::shell::ShellExecutor;

/// Result of a smart merge attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartMergeResult {
    pub success: bool,
    pub merged: Option<String>,
    pub error: Option<String>,
}

/// Check if the `claude` CLI is available on the system.
pub fn is_claude_available(shell: &dyn ShellExecutor) -> bool {
    shell.command_exists("claude")
}

/// Build the prompt sent to Claude for merging two file versions.
pub fn build_merge_prompt(existing: &str, incoming: &str, filename: &str) -> String {
    format!(
        "You are merging configuration files for Claude Code.\n\
         File: {filename}\n\n\
         EXISTING content (user may have customized):\n\
         ```\n{existing}\n```\n\n\
         INCOMING content (new version from ECC):\n\
         ```\n{incoming}\n```\n\n\
         Merge these two versions. Keep user customizations where they don't conflict \
         with structural changes in the incoming version. Output ONLY the merged file \
         content, no explanation."
    )
}

/// Attempt a smart merge using the Claude CLI.
///
/// Spawns `claude -p` with the merge prompt via stdin.
/// Returns the merged content on success, or an error description on failure.
pub fn smart_merge(
    shell: &dyn ShellExecutor,
    existing: &str,
    incoming: &str,
    filename: &str,
) -> SmartMergeResult {
    if !is_claude_available(shell) {
        return SmartMergeResult {
            success: false,
            merged: None,
            error: Some("Claude CLI not found. Install it with: npm install -g @anthropic-ai/claude-code".to_string()),
        };
    }

    let prompt = build_merge_prompt(existing, incoming, filename);

    match shell.spawn_with_stdin("claude", &["-p"], &prompt) {
        Ok(output) if output.success() => {
            let merged = output.stdout.trim().to_string();
            if merged.is_empty() {
                SmartMergeResult {
                    success: false,
                    merged: None,
                    error: Some("Claude returned empty output".to_string()),
                }
            } else {
                SmartMergeResult {
                    success: true,
                    merged: Some(merged),
                    error: None,
                }
            }
        }
        Ok(output) => SmartMergeResult {
            success: false,
            merged: None,
            error: Some(format!("Claude exited with code {}: {}", output.exit_code, output.stderr.trim())),
        },
        Err(e) => SmartMergeResult {
            success: false,
            merged: None,
            error: Some(format!("Failed to spawn Claude: {e}")),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::MockExecutor;

    #[test]
    fn is_claude_available_true() {
        let shell = MockExecutor::new().with_command("claude");
        assert!(is_claude_available(&shell));
    }

    #[test]
    fn is_claude_available_false() {
        let shell = MockExecutor::new();
        assert!(!is_claude_available(&shell));
    }

    #[test]
    fn build_merge_prompt_contains_filename() {
        let prompt = build_merge_prompt("old", "new", "agents/planner.md");
        assert!(prompt.contains("agents/planner.md"));
    }

    #[test]
    fn build_merge_prompt_contains_existing_content() {
        let prompt = build_merge_prompt("existing content here", "new stuff", "test.md");
        assert!(prompt.contains("existing content here"));
    }

    #[test]
    fn build_merge_prompt_contains_incoming_content() {
        let prompt = build_merge_prompt("old", "incoming content here", "test.md");
        assert!(prompt.contains("incoming content here"));
    }

    #[test]
    fn build_merge_prompt_has_merge_instructions() {
        let prompt = build_merge_prompt("a", "b", "c");
        assert!(prompt.contains("Merge these two versions"));
        assert!(prompt.contains("Keep user customizations"));
    }

    #[test]
    fn smart_merge_claude_not_available() {
        let shell = MockExecutor::new();
        let result = smart_merge(&shell, "old", "new", "test.md");
        assert!(!result.success);
        assert!(result.merged.is_none());
        assert!(result.error.unwrap().contains("not found"));
    }

    #[test]
    fn smart_merge_success() {
        let shell = MockExecutor::new()
            .with_command("claude")
            .on(
                "claude",
                CommandOutput {
                    stdout: "merged content\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
        let result = smart_merge(&shell, "old", "new", "test.md");
        assert!(result.success);
        assert_eq!(result.merged.unwrap(), "merged content");
        assert!(result.error.is_none());
    }

    #[test]
    fn smart_merge_claude_exits_nonzero() {
        let shell = MockExecutor::new()
            .with_command("claude")
            .on(
                "claude",
                CommandOutput {
                    stdout: String::new(),
                    stderr: "rate limited".to_string(),
                    exit_code: 1,
                },
            );
        let result = smart_merge(&shell, "old", "new", "test.md");
        assert!(!result.success);
        assert!(result.merged.is_none());
        assert!(result.error.unwrap().contains("rate limited"));
    }

    #[test]
    fn smart_merge_empty_output() {
        let shell = MockExecutor::new()
            .with_command("claude")
            .on(
                "claude",
                CommandOutput {
                    stdout: "   \n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
        let result = smart_merge(&shell, "old", "new", "test.md");
        assert!(!result.success);
        assert!(result.error.unwrap().contains("empty output"));
    }

    #[test]
    fn smart_merge_spawn_error() {
        // MockExecutor returns ShellError::NotFound when no response is registered
        // but command_exists returns true
        let shell = MockExecutor::new().with_command("claude");
        let result = smart_merge(&shell, "old", "new", "test.md");
        assert!(!result.success);
        assert!(result.error.unwrap().contains("spawn"));
    }
}
