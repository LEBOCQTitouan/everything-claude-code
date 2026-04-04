//! Effort enforcement hook — maps agent `effort` frontmatter field to `MAX_THINKING_TOKENS`.
//!
//! Fires on `SubagentStart`. Reads the agent's `.md` file, extracts the `effort` field,
//! and outputs `MAX_THINKING_TOKENS={value}` to stdout for Claude Code to apply.
//! Uses the authoritative `EFFORT_TOKENS` lookup table from ecc-domain.

use crate::hook::{HookPorts, HookResult};
use ecc_domain::config::validate::{EFFORT_TOKENS, VALID_EFFORT_LEVELS, extract_frontmatter};
use std::path::Path;

/// Slug regex check: only lowercase alphanumeric and hyphens, no path separators.
fn is_valid_agent_slug(s: &str) -> bool {
    if s.is_empty() || s.len() > 100 {
        return false;
    }
    let bytes = s.as_bytes();
    if !bytes[0].is_ascii_lowercase() && !bytes[0].is_ascii_digit() {
        return false;
    }
    bytes
        .iter()
        .all(|&b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Look up the MAX_THINKING_TOKENS value for an effort level.
fn tokens_for_effort(effort: &str) -> Option<u32> {
    EFFORT_TOKENS
        .iter()
        .find(|(level, _)| *level == effort)
        .map(|(_, tokens)| *tokens)
}

/// SubagentStart effort enforcement handler.
///
/// 1. Checks ECC_EFFORT_BYPASS → passthrough
/// 2. Checks MAX_THINKING_TOKENS already set → passthrough (user override)
/// 3. Parses agent_type from stdin JSON
/// 4. Validates agent_type as slug (security: no path traversal)
/// 5. Reads agent .md file, extracts effort frontmatter
/// 6. Maps effort → tokens via EFFORT_TOKENS
/// 7. Outputs MAX_THINKING_TOKENS={value} to stdout
pub fn subagent_start_effort(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "subagent_start_effort", "executing handler");

    // 1. Bypass check
    if ports.env.var("ECC_EFFORT_BYPASS").as_deref() == Some("1") {
        tracing::debug!("effort enforcement bypassed via ECC_EFFORT_BYPASS=1");
        return HookResult::passthrough(stdin);
    }

    // 2. User override check
    if ports.env.var("MAX_THINKING_TOKENS").is_some() {
        tracing::debug!("MAX_THINKING_TOKENS already set, preserving user value");
        return HookResult::passthrough(stdin);
    }

    // 3. Parse agent_type from stdin
    let agent_type = match serde_json::from_str::<serde_json::Value>(stdin)
        .ok()
        .and_then(|v| v.get("agent_type")?.as_str().map(|s| s.to_string()))
    {
        Some(t) => t,
        None => {
            tracing::debug!("no agent_type in stdin, passthrough");
            return HookResult::passthrough(stdin);
        }
    };

    // 4. Slug validation (security: prevent path traversal)
    if !is_valid_agent_slug(&agent_type) {
        tracing::debug!(agent_type = %agent_type, "invalid agent slug, passthrough");
        return HookResult::passthrough(stdin);
    }

    // 5. Resolve and read agent file
    let project_dir = ports
        .env
        .var("CLAUDE_PROJECT_DIR")
        .unwrap_or_else(|| ".".to_string());
    let agent_path = Path::new(&project_dir)
        .join("agents")
        .join(format!("{agent_type}.md"));

    let content = match ports.fs.read_to_string(&agent_path) {
        Ok(c) => c,
        Err(_) => {
            tracing::debug!(path = %agent_path.display(), "agent file not found, passthrough");
            return HookResult::passthrough(stdin);
        }
    };

    // 6. Extract effort from frontmatter
    let frontmatter = match extract_frontmatter(&content) {
        Some(fm) => fm,
        None => {
            tracing::debug!("no frontmatter in agent file, passthrough");
            return HookResult::passthrough(stdin);
        }
    };

    let effort = match frontmatter.get("effort") {
        Some(e) if !e.trim().is_empty() => e.trim().to_string(),
        _ => {
            tracing::debug!("no effort field in agent, passthrough");
            return HookResult::passthrough(stdin);
        }
    };

    // Validate effort value (reject unrecognized values)
    if !VALID_EFFORT_LEVELS.contains(&effort.as_str()) {
        tracing::debug!(effort = %effort, "unrecognized effort value, passthrough");
        return HookResult::passthrough(stdin);
    }

    // 7. Map to tokens and output
    match tokens_for_effort(&effort) {
        Some(tokens) => {
            tracing::debug!(effort = %effort, tokens, "setting MAX_THINKING_TOKENS");
            HookResult {
                stdout: format!("MAX_THINKING_TOKENS={tokens}"),
                stderr: String::new(),
                exit_code: 0,
            }
        }
        None => HookResult::passthrough(stdin),
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
        }
    }

    fn agent_content(effort: &str) -> String {
        format!(
            "---\nname: test-agent\ndescription: Test\nmodel: sonnet\ntools: Read\neffort: {effort}\n---\n# Agent"
        )
    }

    fn agent_content_no_effort() -> String {
        "---\nname: test-agent\ndescription: Test\nmodel: sonnet\ntools: Read\n---\n# Agent"
            .to_string()
    }

    #[test]
    fn effort_hook_low_maps_to_2048() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/agents/my-agent.md", &agent_content("low"));
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = subagent_start_effort(r#"{"agent_type":"my-agent"}"#, &ports);
        assert_eq!(result.stdout, "MAX_THINKING_TOKENS=2048");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn effort_hook_medium_maps_to_8192() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/agents/my-agent.md", &agent_content("medium"));
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = subagent_start_effort(r#"{"agent_type":"my-agent"}"#, &ports);
        assert_eq!(result.stdout, "MAX_THINKING_TOKENS=8192");
    }

    #[test]
    fn effort_hook_high_maps_to_16384() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/agents/my-agent.md", &agent_content("high"));
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = subagent_start_effort(r#"{"agent_type":"my-agent"}"#, &ports);
        assert_eq!(result.stdout, "MAX_THINKING_TOKENS=16384");
    }

    #[test]
    fn effort_hook_max_maps_to_32768() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/agents/my-agent.md", &agent_content("max"));
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = subagent_start_effort(r#"{"agent_type":"my-agent"}"#, &ports);
        assert_eq!(result.stdout, "MAX_THINKING_TOKENS=32768");
    }

    #[test]
    fn effort_hook_no_effort_passthrough() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/agents/my-agent.md", &agent_content_no_effort());
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"agent_type":"my-agent"}"#;
        let result = subagent_start_effort(stdin, &ports);
        assert_eq!(result.stdout, stdin);
    }

    #[test]
    fn effort_hook_missing_file_passthrough() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"agent_type":"nonexistent"}"#;
        let result = subagent_start_effort(stdin, &ports);
        assert_eq!(result.stdout, stdin);
    }

    #[test]
    fn effort_hook_bypass_passthrough() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/agents/my-agent.md", &agent_content("max"));
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_var("CLAUDE_PROJECT_DIR", "/project")
            .with_var("ECC_EFFORT_BYPASS", "1");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"agent_type":"my-agent"}"#;
        let result = subagent_start_effort(stdin, &ports);
        assert_eq!(result.stdout, stdin);
    }

    #[test]
    fn effort_hook_user_override_preserved() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/agents/my-agent.md", &agent_content("low"));
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_var("CLAUDE_PROJECT_DIR", "/project")
            .with_var("MAX_THINKING_TOKENS", "4096");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"agent_type":"my-agent"}"#;
        let result = subagent_start_effort(stdin, &ports);
        assert_eq!(result.stdout, stdin);
    }

    #[test]
    fn effort_hook_bad_yaml_passthrough() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/agents/my-agent.md", "not yaml frontmatter");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"agent_type":"my-agent"}"#;
        let result = subagent_start_effort(stdin, &ports);
        assert_eq!(result.stdout, stdin);
    }

    #[test]
    fn effort_hook_rejects_path_traversal() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/agents/../../etc/passwd", "secret");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"agent_type":"../../etc/passwd"}"#;
        let result = subagent_start_effort(stdin, &ports);
        assert_eq!(result.stdout, stdin);
    }

    #[test]
    fn effort_hook_rejects_invalid_effort() {
        let fs = InMemoryFileSystem::new()
            .with_file("/project/agents/my-agent.md", &agent_content("banana"));
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let stdin = r#"{"agent_type":"my-agent"}"#;
        let result = subagent_start_effort(stdin, &ports);
        assert_eq!(result.stdout, stdin);
    }
}
