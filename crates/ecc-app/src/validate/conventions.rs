use ecc_domain::config::validate::{
    LintFinding, LintSeverity, check_naming_consistency, check_tool_values, extract_frontmatter,
};
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

/// Tool vocabulary for validation — sourced from manifest/tool-manifest.yaml.
/// Phase 1 bridge: will be replaced by injected ToolManifest in Phase 3 (BL-146 US-003).
const TOOL_VOCAB: &[&str] = &[
    "Read",
    "Write",
    "Edit",
    "MultiEdit",
    "Bash",
    "Glob",
    "Grep",
    "Agent",
    "Task",
    "WebSearch",
    "TodoWrite",
    "TodoRead",
    "AskUserQuestion",
    "LS",
    "Skill",
    "EnterPlanMode",
    "ExitPlanMode",
    "TaskCreate",
    "TaskUpdate",
    "TaskGet",
    "TaskList",
];

pub(super) fn validate_conventions(
    root: &Path,
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
) -> bool {
    let mut findings: Vec<LintFinding> = Vec::new();
    let mut total_checked: usize = 0;

    // 1. Agents
    let agents_dir = root.join("agents");
    if let Ok(files) = fs.read_dir(&agents_dir) {
        {
            for file in files
                .iter()
                .filter(|f| f.to_string_lossy().ends_with(".md"))
            {
                total_checked += 1;
                let content = match fs.read_to_string(file) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let stem = file.file_stem().unwrap_or_default().to_string_lossy();
                let fm = extract_frontmatter(&content);
                let fm_name = fm.as_ref().and_then(|m| m.get("name")).map(|s| s.as_str());
                findings.extend(check_naming_consistency(&stem, fm_name, "agent"));
                if let Some(tools) = fm.as_ref().and_then(|m| m.get("tools")) {
                    findings.extend(check_tool_values(&stem, tools, "tools", TOOL_VOCAB));
                }
            }
        }
    }

    // 2. Commands
    let commands_dir = root.join("commands");
    if let Ok(files) = fs.read_dir(&commands_dir) {
        {
            for file in files
                .iter()
                .filter(|f| f.to_string_lossy().ends_with(".md"))
            {
                total_checked += 1;
                let content = match fs.read_to_string(file) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let stem = file.file_stem().unwrap_or_default().to_string_lossy();
                let fm = extract_frontmatter(&content);
                let fm_name = fm.as_ref().and_then(|m| m.get("name")).map(|s| s.as_str());
                findings.extend(check_naming_consistency(&stem, fm_name, "command"));
                if let Some(tools) = fm.as_ref().and_then(|m| m.get("allowed-tools")) {
                    findings.extend(check_tool_values(&stem, tools, "allowed-tools", TOOL_VOCAB));
                }
            }
        }
    }

    // 3. Skills — check directories
    let skills_dir = root.join("skills");
    if let Ok(entries) = fs.read_dir(&skills_dir) {
        {
            for entry in entries.iter().filter(|e| fs.is_dir(e)) {
                total_checked += 1;
                let dir_name = entry
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                // Check for empty directory (no .md files)
                if let Ok(children) = fs.read_dir(entry) {
                    let has_md = children
                        .iter()
                        .any(|c| c.to_string_lossy().ends_with(".md"));
                    if !has_md {
                        findings.push(LintFinding {
                            severity: LintSeverity::Warn,
                            file: dir_name.clone(),
                            message: format!("skill directory '{dir_name}/' contains no .md files"),
                        });
                        continue;
                    }
                }

                // Read SKILL.md for naming check
                let skill_md = entry.join("SKILL.md");
                if let Ok(content) = fs.read_to_string(&skill_md) {
                    {
                        let fm = extract_frontmatter(&content);
                        let fm_name = fm.as_ref().and_then(|m| m.get("name")).map(|s| s.as_str());
                        findings.extend(check_naming_consistency(&dir_name, fm_name, "skill"));
                    }
                }
            }
        }
    }

    // 4. Report findings
    let error_count = findings
        .iter()
        .filter(|f| f.severity == LintSeverity::Error)
        .count();
    let warn_count = findings
        .iter()
        .filter(|f| f.severity == LintSeverity::Warn)
        .count();

    for f in &findings {
        match f.severity {
            LintSeverity::Error => terminal.stderr_write(&format!("ERROR: {}\n", f.message)),
            LintSeverity::Warn => terminal.stdout_write(&format!("WARN: {}\n", f.message)),
        }
    }

    if error_count == 0 {
        terminal.stdout_write(&format!(
            "Convention check OK: {total_checked} files checked, {warn_count} warnings\n"
        ));
        true
    } else {
        terminal.stderr_write(&format!(
            "Convention check FAILED: {error_count} errors, {warn_count} warnings in {total_checked} files\n"
        ));
        false
    }
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
    fn conventions_no_dirs_returns_true() {
        let fs = InMemoryFileSystem::new();
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Conventions,
            Path::new("/root")
        ));
    }

    #[test]
    fn conventions_agent_mismatched_name_returns_false() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/my-agent.md",
            "---\nname: other-agent\ntools: Read\n---\n# Agent",
        );
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Conventions,
            Path::new("/root")
        ));
        assert!(t.stderr_output().iter().any(|s| s.contains("ERROR")));
    }

    #[test]
    fn conventions_agent_invalid_tool_returns_false() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/my-agent.md",
            "---\nname: my-agent\ntools: UnknownTool\n---\n# Agent",
        );
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Conventions,
            Path::new("/root")
        ));
        assert!(t.stderr_output().iter().any(|s| s.contains("ERROR")));
    }

    #[test]
    fn conventions_command_invalid_allowed_tool_returns_false() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/commands/my-command.md",
            "---\nname: my-command\nallowed-tools: FakeTool\n---\n# Command",
        );
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Conventions,
            Path::new("/root")
        ));
        assert!(t.stderr_output().iter().any(|s| s.contains("ERROR")));
    }

    #[test]
    fn conventions_all_valid_returns_true() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/root/agents/my-agent.md",
                "---\nname: my-agent\ntools: Read\n---\n# Agent",
            )
            .with_file(
                "/root/commands/my-command.md",
                "---\nname: my-command\nallowed-tools: Read\n---\n# Command",
            );
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Conventions,
            Path::new("/root")
        ));
        assert!(t.stdout_output().iter().any(|s| s.contains("OK")));
    }

    #[test]
    fn conventions_skill_dir_no_md_files_returns_true_with_warn() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/skills")
            .with_dir("/root/skills/empty-skill");
        let t = term();
        // WARN-only: should return true
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Conventions,
            Path::new("/root")
        ));
        assert!(t.stdout_output().iter().any(|s| s.contains("WARN")));
    }

    #[test]
    fn conventions_warn_only_returns_true() {
        // Agent with missing name field -> WARN (not ERROR)
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/my-agent.md",
            "---\ntools: Read\n---\n# Agent without name",
        );
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Conventions,
            Path::new("/root")
        ));
    }

    #[test]
    fn conventions_mixed_error_and_warn_returns_false() {
        // Agent with invalid tool (ERROR) + missing name (WARN)
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/my-agent.md",
            "---\ntools: UnknownTool\n---\n# Agent",
        );
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Conventions,
            Path::new("/root")
        ));
    }

    #[test]
    fn conventions_skill_dir_with_skill_md_runs_naming_check() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/skills")
            .with_dir("/root/skills/my-skill")
            .with_file(
                "/root/skills/my-skill/SKILL.md",
                "---\nname: my-skill\ndescription: A skill\norigin: ECC\n---\n# Skill",
            );
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Conventions,
            Path::new("/root")
        ));
    }

    #[test]
    fn conventions_skill_dir_all_clean_no_warnings() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/skills")
            .with_dir("/root/skills/my-skill")
            .with_file(
                "/root/skills/my-skill/SKILL.md",
                "---\nname: my-skill\ndescription: A skill\norigin: ECC\n---\n# Skill",
            );
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Conventions,
            Path::new("/root"),
        );
        assert!(result);
        assert!(t.stderr_output().is_empty());
    }

    // Helper: write the canonical tool manifest fixture into the in-memory fs
    fn with_manifest(fs: &InMemoryFileSystem, root: &Path) {
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
        fs.write(
            &root.join("manifest/tool-manifest.yaml"),
            yaml,
        )
        .expect("write manifest");
    }

    // ── PC-019: allowed_tool_set_validates ───────────────────────────────────

    #[test]
    fn allowed_tool_set_validates() {
        let root = Path::new("/root");
        let fs = InMemoryFileSystem::new().with_file(
            "/root/commands/my-command.md",
            "---\nname: my-command\nallowed-tool-set: audit-command\n---\n# Command",
        );
        with_manifest(&fs, root);
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Conventions,
            root,
        );
        assert!(result, "command with allowed-tool-set should validate; stderr: {:?}", t.stderr_output());
    }

    // ── PC-030: unknown_allowed_tool_set_reported ────────────────────────────

    #[test]
    fn unknown_allowed_tool_set_reported() {
        let root = Path::new("/root");
        let fs = InMemoryFileSystem::new().with_file(
            "/root/commands/bad-command.md",
            "---\nname: bad-command\nallowed-tool-set: no-such-preset\n---\n# Command",
        );
        with_manifest(&fs, root);
        let t = term();
        let result = run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Conventions,
            root,
        );
        assert!(!result, "unknown allowed-tool-set should fail");
        let stderr = t.stderr_output().join("");
        assert!(
            stderr.contains("no-such-preset"),
            "error must name the preset, got: {stderr}"
        );
        assert!(
            stderr.contains("bad-command"),
            "error must name the file, got: {stderr}"
        );
    }
}
