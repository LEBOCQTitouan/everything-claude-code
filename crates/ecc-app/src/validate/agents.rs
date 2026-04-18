use ecc_domain::config::tool_manifest::ToolManifest;
use ecc_domain::config::tool_manifest_resolver::{FrontmatterToolSpec, resolve_effective_tools};
use ecc_domain::config::validate::{
    VALID_EFFORT_LEVELS, VALID_MODELS, check_tool_values, extract_frontmatter, parse_tool_list,
};
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

pub(super) fn validate_agents(
    root: &Path,
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    manifest: Option<&ToolManifest>,
) -> bool {
    let agents_dir = root.join("agents");
    if !fs.exists(&agents_dir) {
        terminal.stdout_write("No agents directory found, skipping validation\n");
        return true;
    }

    let files = match fs.read_dir(&agents_dir) {
        Ok(f) => f,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: Cannot read agents directory: {e}\n"));
            return false;
        }
    };
    let md_files: Vec<_> = files
        .iter()
        .filter(|f| f.to_string_lossy().ends_with(".md"))
        .collect();

    let mut has_errors = false;
    for file in &md_files {
        if !validate_agent_file(file, root, fs, terminal, manifest) {
            has_errors = true;
        }
    }

    if has_errors {
        return false;
    }

    terminal.stdout_write(&format!("Validated {} agent files\n", md_files.len()));
    true
}

fn validate_agent_file(
    file: &Path,
    root: &Path,
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    manifest: Option<&ToolManifest>,
) -> bool {
    let required_fields = ["model"];

    let content = match fs.read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: {} - {}\n", file.display(), e));
            return false;
        }
    };

    let frontmatter = match extract_frontmatter(&content) {
        Some(fm) => fm,
        None => {
            let name = file.file_name().unwrap_or_default().to_string_lossy();
            terminal.stderr_write(&format!("ERROR: {} - Missing frontmatter\n", name));
            return false;
        }
    };

    let name = file.file_name().unwrap_or_default().to_string_lossy();
    let mut valid = true;

    for field in &required_fields {
        match frontmatter.get(*field) {
            Some(v) if !v.trim().is_empty() => {}
            _ => {
                terminal.stderr_write(&format!(
                    "ERROR: {} - Missing required field: {}\n",
                    name, field
                ));
                valid = false;
            }
        }
    }

    // Tool resolution: tool-set and/or tools field
    if let Some(tool_errors) = resolve_tool_set_for_agent(&frontmatter, manifest, &name, terminal) {
        if !tool_errors {
            valid = false;
        }
    } else {
        // Neither field present: legacy error
        terminal.stderr_write(&format!(
            "ERROR: {} - Missing required field: tools\n",
            name
        ));
        valid = false;
    }

    if let Some(model) = frontmatter.get("model")
        && !VALID_MODELS.contains(&model.as_str())
    {
        terminal.stderr_write(&format!(
            "ERROR: {} - Invalid model '{}'. Must be one of: {}\n",
            name,
            model,
            VALID_MODELS.join(", ")
        ));
        valid = false;
    }

    // Reject deprecated budget_tokens / budget-tokens
    if frontmatter.contains_key("budget_tokens") || frontmatter.contains_key("budget-tokens") {
        terminal.stderr_write(&format!(
            "ERROR: {} - budget_tokens is deprecated — use effort field instead\n",
            name
        ));
        valid = false;
    }

    // Validate effort if present
    if let Some(effort) = frontmatter.get("effort") {
        let trimmed = effort.trim();
        if !trimmed.is_empty() && !VALID_EFFORT_LEVELS.contains(&trimmed) {
            terminal.stderr_write(&format!(
                "ERROR: {} - Invalid effort '{}'. Must be one of: {}\n",
                name,
                trimmed,
                VALID_EFFORT_LEVELS.join(", ")
            ));
            valid = false;
        }

        // Model/effort cross-validation (warnings only)
        if let Some(model) = frontmatter.get("model") {
            let model = model.trim();
            let effort = trimmed;
            match model {
                "haiku" if effort != "low" && !effort.is_empty() => {
                    terminal.stdout_write(&format!(
                        "WARNING: {} - model/effort mismatch: haiku should use effort: low\n",
                        name
                    ));
                }
                "opus" if (effort == "low" || effort == "medium") && !effort.is_empty() => {
                    terminal.stdout_write(&format!(
                        "WARNING: {} - underutilized effort for opus: consider high or max\n",
                        name
                    ));
                }
                "sonnet" if effort != "medium" && effort != "high" && !effort.is_empty() => {
                    terminal.stdout_write(&format!(
                        "WARNING: {} - model/effort mismatch: sonnet should use medium or high\n",
                        name
                    ));
                }
                _ => {}
            }
        }
    }

    // Optional: validate patterns field if present
    if let Some(raw_patterns) = frontmatter.get("patterns") {
        let categories = parse_tool_list(raw_patterns.trim());
        for category in &categories {
            let category_dir = root.join("patterns").join(category);
            if !fs.is_dir(&category_dir) {
                terminal.stderr_write(&format!(
                    "WARNING: {} - patterns category '{}' not found\n",
                    name, category
                ));
            }
        }
    }

    valid
}

/// Resolve and validate the tool-set/tools fields for an agent.
///
/// Returns `None` if neither `tool-set` nor `tools` is present (caller emits legacy error).
/// Returns `Some(true)` if resolution succeeded with no errors, `Some(false)` if errors were found.
fn resolve_tool_set_for_agent(
    frontmatter: &std::collections::HashMap<String, String>,
    manifest: Option<&ToolManifest>,
    name: &str,
    terminal: &dyn TerminalIO,
) -> Option<bool> {
    let has_tool_set = frontmatter
        .get("tool-set")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    let has_tools = frontmatter
        .get("tools")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);

    if !has_tool_set && !has_tools {
        return None;
    }

    let m = match manifest {
        Some(m) => m,
        None => return Some(true), // No manifest: legacy pass-through
    };

    let tool_set_val = frontmatter.get("tool-set").map(|s| s.trim().to_string());
    let inline_tools = frontmatter
        .get("tools")
        .map(|raw| parse_tool_list(raw.trim()));

    let spec = FrontmatterToolSpec {
        tool_set: tool_set_val,
        inline_tools,
    };

    let mut valid = true;

    match resolve_effective_tools(&spec, m) {
        Ok(resolved) => {
            for warn in &resolved.warnings {
                terminal.stdout_write(&format!("WARNING: {} - {}\n", name, warn));
            }
            if let Some(raw_tools) = frontmatter.get("tools") {
                let atomic_tools: Vec<&str> = m.tools.iter().map(String::as_str).collect();
                let findings = check_tool_values(name, raw_tools.trim(), "tools", &atomic_tools);
                for f in &findings {
                    terminal.stderr_write(&format!("ERROR: {}\n", f.message));
                    valid = false;
                }
            }
        }
        Err(e) => {
            use ecc_domain::config::tool_manifest_resolver::ResolveError;
            match &e {
                ResolveError::UnknownPreset(preset) => {
                    terminal.stderr_write(&format!(
                        "ERROR: {} - unknown tool-set '{}'\n",
                        name, preset
                    ));
                }
                ResolveError::InvalidToolSetReference(v) => {
                    terminal.stderr_write(&format!(
                        "ERROR: {} - invalid tool-set value '{}'\n",
                        name, v
                    ));
                }
                ResolveError::ArrayNotSupported => {
                    terminal.stderr_write(&format!(
                        "ERROR: {} - tool-set must be a single string, not an array\n",
                        name
                    ));
                }
                _ => {
                    terminal.stderr_write(&format!("ERROR: {} - {}\n", name, e));
                }
            }
            valid = false;
        }
    }

    Some(valid)
}

#[cfg(test)]
mod tests {
    use super::super::{ValidateTarget, run_validate};
    use ecc_ports::fs::FileSystem;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment};
    use std::path::Path;

    fn term() -> BufferedTerminal {
        BufferedTerminal::new()
    }

    #[test]
    fn agents_no_dir_succeeds() {
        let fs = InMemoryFileSystem::new();
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
    }

    #[test]
    fn agents_valid_file() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/test.md",
            "---\nmodel: sonnet\ntools: Read\n---\n# Agent",
        );
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
    }

    #[test]
    fn agents_missing_frontmatter() {
        let fs = InMemoryFileSystem::new().with_file("/root/agents/bad.md", "# No frontmatter");
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("Missing frontmatter"))
        );
    }

    #[test]
    fn agents_missing_required_field() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/agents/bad.md", "---\nmodel: sonnet\n---\n# Agent");
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("Missing required field"))
        );
    }

    #[test]
    fn agents_budget_tokens_rejected() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/bad.md",
            "---\nmodel: sonnet\ntools: Read\nbudget_tokens: 8000\n---\n# Agent",
        );
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("budget_tokens is deprecated"))
        );
    }

    #[test]
    fn agents_budget_tokens_kebab_rejected() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/bad.md",
            "---\nmodel: sonnet\ntools: Read\nbudget-tokens: 8000\n---\n# Agent",
        );
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("budget_tokens is deprecated"))
        );
    }

    #[test]
    fn agents_no_budget_tokens_no_warning() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/good.md",
            "---\nmodel: sonnet\ntools: Read\n---\n# Agent",
        );
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
        assert!(
            !t.stderr_output()
                .iter()
                .any(|s| s.contains("budget_tokens"))
        );
    }

    #[test]
    fn agents_haiku_high_effort_warns() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/bad.md",
            "---\nmodel: haiku\ntools: Read\neffort: high\n---\n# Agent",
        );
        let t = term();
        // Validation still passes (warning, not error)
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
        assert!(
            t.stdout_output()
                .iter()
                .any(|s| s.contains("model/effort mismatch"))
        );
    }

    #[test]
    fn agents_opus_low_effort_warns() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/bad.md",
            "---\nmodel: opus\ntools: Read\neffort: low\n---\n# Agent",
        );
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
        assert!(
            t.stdout_output()
                .iter()
                .any(|s| s.contains("underutilized effort"))
        );
    }

    #[test]
    fn agents_valid_model_effort_no_warning() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/good.md",
            "---\nmodel: sonnet\ntools: Read\neffort: medium\n---\n# Agent",
        );
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
        assert!(!t.stdout_output().iter().any(|s| s.contains("WARNING")));
    }

    #[test]
    fn agents_invalid_model() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/bad.md",
            "---\nmodel: gpt4\ntools: Read\n---\n# Agent",
        );
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("Invalid model"))
        );
    }

    #[test]
    fn agent_patterns_invalid_category_warns() {
        // Agent references a patterns category that doesn't exist as a dir → WARNING
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/planner.md",
            "---\nmodel: sonnet\ntools: Read\npatterns: nonexistent-category\n---\n# Planner",
        );
        let t = term();
        // Validation should still succeed (warning, not error)
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root"),
        );
        assert!(result, "should pass (warning only, not error)");
        // But a warning should be emitted
        let stderr = t.stderr_output();
        assert!(
            stderr.iter().any(|s| s.contains("nonexistent-category")),
            "expected warning about missing pattern category, got: {stderr:?}"
        );
    }

    #[test]
    fn agents_domain_subdir_valid_file() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/root/agents/top.md",
                "---\nmodel: sonnet\ntools: Read\n---\n# Top Agent",
            )
            .with_file(
                "/root/agents/domain/backlog.md",
                "---\nmodel: sonnet\ntools: Read\ngenerated: true\ngenerated_at: 2026-04-17T00:00:00Z\n---\n# Backlog Domain Agent",
            );
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
    }

    #[test]
    fn agents_domain_subdir_missing_model_fails() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/domain/backlog.md",
            "---\ntools: Read\n---\n# Missing model",
        );
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
        let stderr = t.stderr_output();
        assert!(
            stderr
                .iter()
                .any(|s| s.contains("backlog.md") || s.contains("Missing required field") || s.contains("model")),
            "expected error mentioning backlog.md or model field, got: {stderr:?}"
        );
    }

    #[test]
    fn agents_domain_subdir_absent_succeeds() {
        // No agents/domain/ dir — should succeed silently
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/top.md",
            "---\nmodel: sonnet\ntools: Read\n---\n# Top",
        );
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
    }

    #[test]
    fn agents_count_includes_domain_subdir() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/root/agents/top.md",
                "---\nmodel: sonnet\ntools: Read\n---\n# Top",
            )
            .with_file(
                "/root/agents/domain/config.md",
                "---\nmodel: sonnet\ntools: Read\ngenerated: true\ngenerated_at: 2026-04-17T00:00:00Z\n---\n# Config Domain Agent",
            );
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root")
        ));
        // Count should include both top-level and domain subdir agents (2 total)
        let stdout = t.stdout_output();
        assert!(
            stdout.iter().any(|s| s.contains('2')),
            "expected count 2 in stdout, got: {stdout:?}"
        );
    }

    #[test]
    fn agent_no_patterns_field_ok() {
        // Agent without patterns field — no warning, validation passes
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/simple.md",
            "---\nmodel: sonnet\ntools: Read\n---\n# Simple",
        );
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root"),
        );
        assert!(result, "should pass when patterns field is absent");
        // No stderr output about patterns
        let stderr = t.stderr_output();
        assert!(
            !stderr.iter().any(|s| s.to_lowercase().contains("pattern")),
            "unexpected pattern warning when field absent: {stderr:?}"
        );
    }

    // Helper: write the canonical tool manifest fixture into the in-memory fs
    fn with_manifest(fs: &InMemoryFileSystem) {
        let yaml = r#"tools:
  - Read
  - Write
  - Edit
  - MultiEdit
  - Bash
  - Glob
  - Grep
  - Agent
  - Task
  - WebSearch
  - TodoWrite
  - TodoRead
  - AskUserQuestion
  - LS
  - Skill
  - EnterPlanMode
  - ExitPlanMode
  - TaskCreate
  - TaskUpdate
  - TaskGet
  - TaskList
presets:
  readonly-analyzer:
    - Read
    - Grep
    - Glob
  readonly-analyzer-shell:
    - Read
    - Grep
    - Glob
    - Bash
  tdd-executor:
    - Read
    - Write
    - Edit
    - Bash
  code-writer:
    - Read
    - Write
    - Edit
    - MultiEdit
    - Bash
  orchestrator:
    - Read
    - Write
    - Edit
    - Bash
    - Agent
    - Task
  audit-command:
    - Task
    - Read
    - Grep
    - Glob
    - LS
    - Bash
    - Write
    - TodoWrite
"#;
        fs.write(Path::new("/root/manifest/tool-manifest.yaml"), yaml)
            .expect("write manifest");
    }

    // ── PC-015: tool_set_only_validates ──────────────────────────────────────

    #[test]
    fn tool_set_only_validates() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/readonly-agent.md",
            "---\nmodel: sonnet\ntool-set: readonly-analyzer\n---\n# Agent",
        );
        with_manifest(&fs);
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root"),
        );
        assert!(
            result,
            "agent with tool-set only should validate; stderr: {:?}",
            t.stderr_output()
        );
    }

    // ── PC-016: unknown_preset_names_file_and_preset ─────────────────────────

    #[test]
    fn unknown_preset_names_file_and_preset() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/bad-agent.md",
            "---\nmodel: sonnet\ntool-set: nonexistent\n---\n# Agent",
        );
        with_manifest(&fs);
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root"),
        );
        assert!(!result, "unknown preset should fail");
        let stderr = t.stderr_output().join("");
        assert!(
            stderr.contains("nonexistent"),
            "error must name the unknown preset, got: {stderr}"
        );
        assert!(
            stderr.contains("bad-agent"),
            "error must name the file, got: {stderr}"
        );
    }

    // ── PC-018: neither_field_preserves_legacy_error ─────────────────────────

    #[test]
    fn neither_field_preserves_legacy_error() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/no-tools.md",
            "---\nmodel: sonnet\n---\n# Agent with no tools",
        );
        with_manifest(&fs);
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root"),
        );
        assert!(!result, "agent with neither tool-set nor tools should fail");
        let stderr = t.stderr_output().join("");
        assert!(
            stderr.to_lowercase().contains("missing required field")
                || stderr.to_lowercase().contains("tools"),
            "should report missing tools field; got: {stderr}"
        );
    }

    // ── PC-023: unknown_atomic_tool_reported ─────────────────────────────────

    #[test]
    fn unknown_atomic_tool_reported() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/bad-tools.md",
            "---\nmodel: sonnet\ntools: Read, FakeTool\n---\n# Agent",
        );
        with_manifest(&fs);
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root"),
        );
        assert!(!result, "agent with unknown tool should fail");
        let stderr = t.stderr_output().join("");
        assert!(
            stderr.contains("FakeTool"),
            "error must name the unknown tool, got: {stderr}"
        );
        assert!(
            stderr.contains("bad-tools"),
            "error must name the file, got: {stderr}"
        );
    }

    // ── PC-024: valid_preset_passes ──────────────────────────────────────────

    #[test]
    fn valid_preset_passes() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/valid-agent.md",
            "---\nmodel: sonnet\ntool-set: readonly-analyzer\n---\n# Agent",
        );
        with_manifest(&fs);
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Agents,
            Path::new("/root"),
        );
        assert!(
            result,
            "valid preset should pass; stderr: {:?}",
            t.stderr_output()
        );
    }
}
