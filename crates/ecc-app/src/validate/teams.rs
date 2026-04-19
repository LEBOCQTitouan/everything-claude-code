//! Validate team manifests — cross-references agents, checks tool escalation.

use ecc_domain::config::team::{parse_team_manifest, validate_team_manifest};
use ecc_domain::config::tool_manifest::ToolManifest;
use ecc_domain::config::validate::extract_frontmatter;
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub(super) fn validate_teams(root: &Path, fs: &dyn FileSystem, terminal: &dyn TerminalIO) -> bool {
    let teams_dir = root.join("teams");
    if !fs.exists(&teams_dir) {
        terminal.stdout_write("No teams directory found, skipping validation\n");
        return true; // AC-002.4
    }

    let files = match fs.read_dir(&teams_dir) {
        Ok(f) => f,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: Cannot read teams directory: {e}\n"));
            return false;
        }
    };
    let md_files: Vec<_> = files
        .iter()
        .filter(|f| f.to_string_lossy().ends_with(".md"))
        .collect();

    // Load manifest (best-effort: missing manifest falls back to legacy behaviour)
    let manifest_opt = super::tool_manifest_loader::load_tool_manifest(fs, root).ok();

    // Collect known agent names + their tools from agents/ directory
    let known_agents = collect_agent_info(root, fs, manifest_opt.as_ref());

    let mut has_errors = false;
    for file in &md_files {
        if !validate_team_file(file, fs, terminal, &known_agents, manifest_opt.as_ref()) {
            has_errors = true;
        }
    }

    if has_errors {
        return false;
    }

    terminal.stdout_write(&format!("Validated {} team manifests\n", md_files.len()));
    true
}

/// Agent info: name -> set of tools defined in the agent's frontmatter.
///
/// When a `tool-set:` key is present and a manifest is provided, the preset is
/// resolved to its atomic tools. Inline `tools:` and preset tools are merged.
fn collect_agent_info(
    root: &Path,
    fs: &dyn FileSystem,
    manifest: Option<&ToolManifest>,
) -> HashMap<String, HashSet<String>> {
    let agents_dir = root.join("agents");
    let mut agents = HashMap::new();

    let files = match fs.read_dir(&agents_dir) {
        Ok(f) => f,
        Err(_) => return agents,
    };

    for file in &files {
        let name = file
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if name.is_empty() {
            continue;
        }

        let mut tools = HashSet::new();
        if let Ok(content) = fs.read_to_string(file)
            && let Some(fm) = extract_frontmatter(&content)
        {
            // Resolve tool-set: via manifest if available
            if let (Some(preset_name), Some(m)) = (fm.get("tool-set"), manifest) {
                let preset_name = preset_name.trim().trim_matches('"').trim_matches('\'');
                if let Some(preset_tools) = m.presets.get(preset_name) {
                    for t in preset_tools {
                        tools.insert(t.clone());
                    }
                }
            }

            // Merge inline tools: if present
            if let Some(tools_str) = fm.get("tools") {
                // Parse tools list: "Read, Write, Edit" or "[Read, Write]"
                let cleaned = tools_str.trim_matches(|c| c == '[' || c == ']').to_string();
                for tool in cleaned.split(',') {
                    let t = tool.trim().trim_matches('"').trim_matches('\'').to_string();
                    if !t.is_empty() {
                        tools.insert(t);
                    }
                }
            }
        }
        agents.insert(name, tools);
    }

    agents
}

fn validate_team_file(
    file: &Path,
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    known_agents: &HashMap<String, HashSet<String>>,
    tool_manifest: Option<&ToolManifest>,
) -> bool {
    let file_name = file.file_name().unwrap_or_default().to_string_lossy();

    let content = match fs.read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: {file_name} - {e}\n"));
            return false;
        }
    };

    // Parse team manifest
    let manifest = match parse_team_manifest(&content) {
        Ok(m) => m,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: {file_name} - {e}\n"));
            return false;
        }
    };

    // Run domain validation
    let domain_errors = validate_team_manifest(&manifest);
    let mut valid = domain_errors.is_empty();
    for err in &domain_errors {
        terminal.stderr_write(&format!("ERROR: {file_name} - {err}\n"));
    }

    // Cross-reference: check agent names exist (AC-002.1)
    for agent in &manifest.agents {
        if !known_agents.contains_key(&agent.name) {
            terminal.stderr_write(&format!(
                "ERROR: {file_name} - Agent '{}' not found in agents/\n",
                agent.name
            ));
            valid = false;
        }
    }

    // Tool escalation check (AC-002.3) — warning only, exit 0
    for agent in &manifest.agents {
        // Resolve the effective team-side tool list from allowed-tools or allowed-tool-set
        let team_tools: Option<Vec<String>> = if let Some(ref tools) = agent.allowed_tools {
            Some(tools.clone())
        } else if let (Some(preset_name), Some(m)) = (&agent.allowed_tool_set, tool_manifest) {
            m.presets.get(preset_name.as_str()).cloned()
        } else {
            None
        };

        if let Some(ref team_tool_list) = team_tools
            && let Some(agent_tools) = known_agents.get(&agent.name)
        {
            for tool in team_tool_list {
                if !agent_tools.contains(tool) {
                    // Name the preset if one was used, for better diagnostics
                    let preset_info = agent
                        .allowed_tool_set
                        .as_ref()
                        .map(|p| format!(" (from preset '{p}')"))
                        .unwrap_or_default();
                    terminal.stdout_write(&format!(
                        "WARNING: {file_name} - Tool '{}'{} in team manifest exceeds agent '{}' allowed tools (privilege escalation)\n",
                        tool, preset_info, agent.name
                    ));
                }
            }
        }
    }

    valid
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem};
    use std::path::PathBuf;

    fn setup_agents(fs: &InMemoryFileSystem) {
        let root = PathBuf::from("/project");
        fs.write(
            &root.join("agents/tdd-executor.md"),
            "---\nname: tdd-executor\ndescription: TDD executor\nmodel: sonnet\ntools: Read, Write, Edit, Bash, Grep, Glob\n---\n",
        ).unwrap();
        fs.write(
            &root.join("agents/code-reviewer.md"),
            "---\nname: code-reviewer\ndescription: Code reviewer\nmodel: opus\ntools: Read, Grep, Glob, Bash\n---\n",
        ).unwrap();
    }

    fn valid_team_content() -> String {
        r#"---
name: test-team
description: A test team
coordination: wave-dispatch
agents:
  - name: tdd-executor
    role: implementer
  - name: code-reviewer
    role: reviewer
---
# Test Team
"#
        .to_string()
    }

    #[test]
    fn no_dir_succeeds() {
        let fs = InMemoryFileSystem::new();
        let terminal = BufferedTerminal::new();
        let result = validate_teams(&PathBuf::from("/project"), &fs, &terminal);
        assert!(result, "should succeed when no teams/ directory");
        assert!(
            terminal
                .stdout_output()
                .join("")
                .contains("No teams directory found")
        );
    }

    #[test]
    fn rejects_unknown_agent() {
        let fs = InMemoryFileSystem::new();
        let terminal = BufferedTerminal::new();
        let root = PathBuf::from("/project");
        // No agents/ at all — so all agent refs are unknown
        fs.write(&root.join("teams/test-team.md"), &valid_team_content())
            .unwrap();

        let result = validate_teams(&root, &fs, &terminal);
        assert!(!result, "should fail when agent not found");
        let stderr = terminal.stderr_output().join("");
        assert!(
            stderr.contains("not found in agents/"),
            "should report missing agent, got: {stderr}"
        );
    }

    #[test]
    fn warns_on_tool_escalation() {
        let fs = InMemoryFileSystem::new();
        let terminal = BufferedTerminal::new();
        let root = PathBuf::from("/project");
        setup_agents(&fs);

        // Team manifest gives code-reviewer Write tool (not in its agent definition)
        let content = r#"---
name: escalation-team
description: Tool escalation test
coordination: sequential
agents:
  - name: code-reviewer
    role: reviewer
    allowed-tools: ["Read", "Grep", "Glob", "Write"]
---
"#;
        fs.write(&root.join("teams/escalation-team.md"), content)
            .unwrap();

        let result = validate_teams(&root, &fs, &terminal);
        assert!(result, "tool escalation should warn, not fail");
        let stdout = terminal.stdout_output().join("");
        assert!(
            stdout.contains("privilege escalation"),
            "should warn about privilege escalation, got: {stdout}"
        );
    }

    #[test]
    fn valid_manifest_passes() {
        let fs = InMemoryFileSystem::new();
        let terminal = BufferedTerminal::new();
        let root = PathBuf::from("/project");
        setup_agents(&fs);
        fs.write(&root.join("teams/test-team.md"), &valid_team_content())
            .unwrap();

        let result = validate_teams(&root, &fs, &terminal);
        assert!(result, "valid manifest should pass");
        assert!(
            terminal
                .stdout_output()
                .join("")
                .contains("Validated 1 team manifests")
        );
    }

    #[test]
    fn reports_parse_error_with_path() {
        let fs = InMemoryFileSystem::new();
        let terminal = BufferedTerminal::new();
        let root = PathBuf::from("/project");
        setup_agents(&fs);
        fs.write(
            &root.join("teams/broken.md"),
            "No frontmatter at all, just text.",
        )
        .unwrap();

        let result = validate_teams(&root, &fs, &terminal);
        assert!(!result, "should fail for broken manifest");
        let stderr = terminal.stderr_output().join("");
        assert!(
            stderr.contains("broken.md"),
            "should include file name in error, got: {stderr}"
        );
    }

    // Helper: write tool manifest fixture
    fn with_manifest(fs: &InMemoryFileSystem, root: &Path) {
        let yaml = r#"tools:
  - Read
  - Write
  - Edit
  - Bash
  - Grep
  - Glob
presets:
  readonly-analyzer:
    - Read
    - Grep
    - Glob
  code-writer:
    - Read
    - Write
    - Edit
    - Bash
"#;
        fs.write(&root.join("manifest/tool-manifest.yaml"), yaml)
            .expect("write manifest");
    }

    // ── PC-036: team_allowed_tools_cross_ref_manifest ───────────────────────
    // Team validator cross-refs manifest for `allowed-tools`

    #[test]
    fn team_allowed_tools_cross_ref_manifest() {
        let fs = InMemoryFileSystem::new();
        let terminal = BufferedTerminal::new();
        let root = PathBuf::from("/project");
        with_manifest(&fs, &root);

        // Agent has tools: Read, Grep, Glob (inline)
        fs.write(
            &root.join("agents/code-reviewer.md"),
            "---\nname: code-reviewer\ndescription: Code reviewer\nmodel: opus\ntools: Read, Grep, Glob\n---\n",
        ).unwrap();

        // Team gives code-reviewer Write (not in its tools → escalation)
        let content = r#"---
name: cross-ref-team
description: Team for cross-ref test
coordination: sequential
agents:
  - name: code-reviewer
    role: reviewer
    allowed-tools: ["Read", "Grep", "Glob", "Write"]
---
"#;
        fs.write(&root.join("teams/cross-ref-team.md"), content)
            .unwrap();

        let result = validate_teams(&root, &fs, &terminal);
        assert!(result, "tool escalation should warn, not fail");
        let stdout = terminal.stdout_output().join("");
        // Should emit a privilege escalation warning naming Write
        assert!(
            stdout.contains("Write") || stdout.contains("privilege escalation"),
            "warning should name escalated tool; got: {stdout}"
        );
    }

    // ── PC-038: collect_agent_info_resolves_tool_set ─────────────────────────
    // `collect_agent_info` resolves `tool-set:` via manifest

    #[test]
    fn collect_agent_info_resolves_tool_set() {
        let fs = InMemoryFileSystem::new();
        let root = PathBuf::from("/project");
        with_manifest(&fs, &root);

        // Agent with tool-set: readonly-analyzer (no inline tools:)
        fs.write(
            &root.join("agents/code-reviewer.md"),
            "---\nname: code-reviewer\ndescription: Code reviewer\nmodel: opus\ntool-set: readonly-analyzer\n---\n",
        ).unwrap();

        // Load manifest to pass to collect_agent_info
        let yaml = r#"tools:
  - Read
  - Grep
  - Glob
presets:
  readonly-analyzer:
    - Read
    - Grep
    - Glob
"#;
        let tm = ecc_domain::config::tool_manifest::parse_tool_manifest(yaml).unwrap();
        let known_agents = collect_agent_info(&root, &fs, Some(&tm));
        let tools = known_agents
            .get("code-reviewer")
            .expect("code-reviewer should be in known_agents");

        // After resolving readonly-analyzer: [Read, Grep, Glob]
        assert!(
            tools.contains("Read"),
            "resolved tools must contain Read; got: {tools:?}"
        );
        assert!(
            tools.contains("Grep"),
            "resolved tools must contain Grep; got: {tools:?}"
        );
        assert!(
            tools.contains("Glob"),
            "resolved tools must contain Glob; got: {tools:?}"
        );
    }

    // ── PC-027: escalation_warn_names_preset_and_missing_tool ────────────────

    #[test]
    fn escalation_warn_names_preset_and_missing_tool() {
        let fs = InMemoryFileSystem::new();
        let terminal = BufferedTerminal::new();
        let root = PathBuf::from("/project");

        // Agent that uses a preset (tool-set: readonly-analyzer)
        fs.write(
            &root.join("agents/code-reviewer.md"),
            "---\nname: code-reviewer\ndescription: Code reviewer\nmodel: opus\ntool-set: readonly-analyzer\n---\n",
        ).unwrap();

        with_manifest(&fs, &root);

        // Team manifest gives code-reviewer Write (not in readonly-analyzer → escalation)
        let content = r#"---
name: escalation-team
description: Tool escalation test
coordination: sequential
agents:
  - name: code-reviewer
    role: reviewer
    allowed-tools: ["Read", "Grep", "Glob", "Write"]
---
"#;
        fs.write(&root.join("teams/escalation-team.md"), content)
            .unwrap();

        let result = validate_teams(&root, &fs, &terminal);
        assert!(result, "tool escalation should warn, not fail");
        let stdout = terminal.stdout_output().join("");
        // Warning must name the preset AND the missing tool
        assert!(
            stdout.contains("Write") || stdout.contains("privilege escalation"),
            "warning should mention the escalated tool; got: {stdout}"
        );
        assert!(
            stdout.contains("readonly-analyzer") || stdout.contains("Write"),
            "warning should name the preset or the tool; got: {stdout}"
        );
    }
}
