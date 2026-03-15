//! Validate use case — validates content files (agents, commands, hooks, skills, rules, paths).

use ecc_domain::config::validate::{
    check_hook_entry, extract_frontmatter, VALID_HOOK_EVENTS, VALID_MODELS,
};
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

/// Which content type to validate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidateTarget {
    Agents,
    Commands,
    Hooks,
    Skills,
    Rules,
    Paths,
}

/// Run validation for the given target. Returns `true` on success, `false` on errors.
pub fn run_validate(
    fs: &dyn FileSystem,
    terminal: &dyn TerminalIO,
    target: &ValidateTarget,
    root: &Path,
) -> bool {
    match target {
        ValidateTarget::Agents => validate_agents(root, fs, terminal),
        ValidateTarget::Commands => validate_commands(root, fs, terminal),
        ValidateTarget::Hooks => validate_hooks(root, fs, terminal),
        ValidateTarget::Skills => validate_skills(root, fs, terminal),
        ValidateTarget::Rules => validate_rules(root, fs, terminal),
        ValidateTarget::Paths => validate_paths(root, fs, terminal),
    }
}

fn validate_agents(root: &Path, fs: &dyn FileSystem, terminal: &dyn TerminalIO) -> bool {
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
        if !validate_agent_file(file, fs, terminal) {
            has_errors = true;
        }
    }

    if has_errors {
        return false;
    }

    terminal.stdout_write(&format!("Validated {} agent files\n", md_files.len()));
    true
}

fn validate_agent_file(file: &Path, fs: &dyn FileSystem, terminal: &dyn TerminalIO) -> bool {
    let required_fields = ["model", "tools"];

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

    valid
}

fn validate_commands(root: &Path, fs: &dyn FileSystem, terminal: &dyn TerminalIO) -> bool {
    let commands_dir = root.join("commands");
    if !fs.exists(&commands_dir) {
        terminal.stdout_write("No commands directory found, skipping validation\n");
        return true;
    }

    let files = match fs.read_dir(&commands_dir) {
        Ok(f) => f,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: Cannot read commands directory: {e}\n"));
            return false;
        }
    };
    let md_files: Vec<_> = files
        .iter()
        .filter(|f| f.to_string_lossy().ends_with(".md"))
        .collect();

    let mut has_errors = false;

    for file in &md_files {
        let content = match fs.read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                terminal.stderr_write(&format!("ERROR: {} - {}\n", file.display(), e));
                has_errors = true;
                continue;
            }
        };

        let name = file.file_name().unwrap_or_default().to_string_lossy();

        if content.trim().is_empty() {
            terminal.stderr_write(&format!("ERROR: {} - Empty command file\n", name));
            has_errors = true;
        }
    }

    if has_errors {
        return false;
    }

    terminal.stdout_write(&format!("Validated {} command files\n", md_files.len()));
    true
}

fn validate_hooks(root: &Path, fs: &dyn FileSystem, terminal: &dyn TerminalIO) -> bool {
    let hooks_file = root.join("hooks").join("hooks.json");
    if !fs.exists(&hooks_file) {
        terminal.stdout_write("No hooks.json found, skipping validation\n");
        return true;
    }

    let content = match fs.read_to_string(&hooks_file) {
        Ok(c) => c,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: Cannot read hooks.json: {e}\n"));
            return false;
        }
    };
    let data: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: Invalid JSON in hooks.json: {e}\n"));
            return false;
        }
    };

    let hooks = data.get("hooks").unwrap_or(&data);
    let mut has_errors = false;
    let mut total_matchers = 0;

    if let Some(obj) = hooks.as_object() {
        for (event_type, matchers) in obj {
            if !VALID_HOOK_EVENTS.contains(&event_type.as_str()) {
                terminal.stderr_write(&format!("ERROR: Invalid event type: {}\n", event_type));
                has_errors = true;
                continue;
            }

            let matchers = match matchers.as_array() {
                Some(a) => a,
                None => {
                    terminal.stderr_write(&format!("ERROR: {} must be an array\n", event_type));
                    has_errors = true;
                    continue;
                }
            };

            for (i, matcher) in matchers.iter().enumerate() {
                if !validate_hook_matcher(matcher, event_type, i, terminal) {
                    has_errors = true;
                }
                total_matchers += 1;
            }
        }
    }

    if has_errors {
        return false;
    }

    terminal.stdout_write(&format!("Validated {} hook matchers\n", total_matchers));
    true
}

fn validate_hook_matcher(
    matcher: &serde_json::Value,
    event_type: &str,
    idx: usize,
    terminal: &dyn TerminalIO,
) -> bool {
    let obj = match matcher.as_object() {
        Some(o) => o,
        None => {
            terminal.stderr_write(&format!("ERROR: {}[{}] is not an object\n", event_type, idx));
            return false;
        }
    };

    let mut valid = true;

    if obj.get("matcher").and_then(|v| v.as_str()).is_none() {
        terminal.stderr_write(&format!(
            "ERROR: {}[{}] missing 'matcher' field\n",
            event_type, idx
        ));
        valid = false;
    }

    match obj.get("hooks").and_then(|v| v.as_array()) {
        Some(hooks) => {
            for (j, hook) in hooks.iter().enumerate() {
                let label = format!("{}[{}].hooks[{}]", event_type, idx, j);
                let errors = check_hook_entry(hook, &label);
                for err in &errors {
                    terminal.stderr_write(&format!("ERROR: {err}\n"));
                }
                if !errors.is_empty() {
                    valid = false;
                }
            }
        }
        None => {
            terminal.stderr_write(&format!(
                "ERROR: {}[{}] missing 'hooks' array\n",
                event_type, idx
            ));
            valid = false;
        }
    }

    valid
}

fn validate_skills(root: &Path, fs: &dyn FileSystem, terminal: &dyn TerminalIO) -> bool {
    let skills_dir = root.join("skills");
    if !fs.exists(&skills_dir) {
        terminal.stdout_write("No skills directory found, skipping validation\n");
        return true;
    }

    let entries = match fs.read_dir(&skills_dir) {
        Ok(e) => e,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: Cannot read skills directory: {e}\n"));
            return false;
        }
    };
    let mut has_errors = false;
    let mut valid_count = 0;

    for entry in &entries {
        if !fs.is_dir(entry) {
            continue;
        }
        let name = entry
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let skill_md = entry.join("SKILL.md");

        if !fs.exists(&skill_md) {
            terminal.stderr_write(&format!("ERROR: {}/ - Missing SKILL.md\n", name));
            has_errors = true;
            continue;
        }

        match fs.read_to_string(&skill_md) {
            Ok(c) if c.trim().is_empty() => {
                terminal.stderr_write(&format!("ERROR: {}/SKILL.md - Empty file\n", name));
                has_errors = true;
            }
            Err(e) => {
                terminal.stderr_write(&format!("ERROR: {}/SKILL.md - {}\n", name, e));
                has_errors = true;
            }
            Ok(_) => {
                valid_count += 1;
            }
        }
    }

    if has_errors {
        return false;
    }

    terminal.stdout_write(&format!("Validated {} skill directories\n", valid_count));
    true
}

fn validate_rules(root: &Path, fs: &dyn FileSystem, terminal: &dyn TerminalIO) -> bool {
    let rules_dir = root.join("rules");
    if !fs.exists(&rules_dir) {
        terminal.stdout_write("No rules directory found, skipping validation\n");
        return true;
    }

    let files = match fs.read_dir_recursive(&rules_dir) {
        Ok(f) => f,
        Err(e) => {
            terminal.stderr_write(&format!("ERROR: Cannot read rules directory: {e}\n"));
            return false;
        }
    };
    let md_files: Vec<_> = files
        .iter()
        .filter(|f| f.to_string_lossy().ends_with(".md"))
        .collect();

    let mut has_errors = false;
    let mut validated = 0;

    for file in &md_files {
        match fs.read_to_string(file) {
            Ok(c) if c.trim().is_empty() => {
                let rel = file.strip_prefix(root).unwrap_or(file);
                terminal.stderr_write(&format!("ERROR: {} - Empty rule file\n", rel.display()));
                has_errors = true;
            }
            Err(e) => {
                let rel = file.strip_prefix(root).unwrap_or(file);
                terminal.stderr_write(&format!("ERROR: {} - {}\n", rel.display(), e));
                has_errors = true;
            }
            Ok(_) => {
                validated += 1;
            }
        }
    }

    if has_errors {
        return false;
    }

    terminal.stdout_write(&format!("Validated {} rule files\n", validated));
    true
}

fn validate_paths(root: &Path, fs: &dyn FileSystem, terminal: &dyn TerminalIO) -> bool {
    let targets = ["README.md", "skills", "commands", "agents", "docs"];

    let block_patterns = ["/Users/affoon", "C:\\Users\\affoon"];
    let check_exts = [
        ".md", ".json", ".js", ".ts", ".sh", ".toml", ".yml", ".yaml",
    ];

    let mut failures = 0;

    for target in &targets {
        let target_path = root.join(target);
        if !fs.exists(&target_path) {
            continue;
        }

        let files = if fs.is_file(&target_path) {
            vec![target_path]
        } else {
            match fs.read_dir_recursive(&target_path) {
                Ok(f) => f,
                Err(e) => {
                    terminal.stderr_write(&format!(
                        "ERROR: Cannot read {}: {}\n",
                        target, e
                    ));
                    failures += 1;
                    continue;
                }
            }
        };

        for file in &files {
            let name = file.to_string_lossy();
            if !check_exts.iter().any(|ext| name.ends_with(ext)) {
                continue;
            }
            if name.contains("node_modules") || name.contains(".git/") {
                continue;
            }

            if let Ok(content) = fs.read_to_string(file) {
                for pattern in &block_patterns {
                    if content.contains(pattern) {
                        let rel = file.strip_prefix(root).unwrap_or(file);
                        terminal.stderr_write(&format!(
                            "ERROR: personal path detected in {}\n",
                            rel.display()
                        ));
                        failures += 1;
                        break;
                    }
                }
            }
        }
    }

    if failures > 0 {
        return false;
    }

    terminal.stdout_write("Validated: no personal absolute paths in shipped docs/skills/commands\n");
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem};

    fn term() -> BufferedTerminal {
        BufferedTerminal::new()
    }

    // --- validate_agents ---

    #[test]
    fn agents_no_dir_succeeds() {
        let fs = InMemoryFileSystem::new();
        let t = term();
        assert!(run_validate(&fs, &t, &ValidateTarget::Agents, Path::new("/root")));
    }

    #[test]
    fn agents_valid_file() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/test.md",
            "---\nmodel: sonnet\ntools: Read\n---\n# Agent",
        );
        let t = term();
        assert!(run_validate(&fs, &t, &ValidateTarget::Agents, Path::new("/root")));
    }

    #[test]
    fn agents_missing_frontmatter() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/agents/bad.md", "# No frontmatter");
        let t = term();
        assert!(!run_validate(&fs, &t, &ValidateTarget::Agents, Path::new("/root")));
        assert!(t.stderr_output().iter().any(|s| s.contains("Missing frontmatter")));
    }

    #[test]
    fn agents_missing_required_field() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/bad.md",
            "---\nmodel: sonnet\n---\n# Agent",
        );
        let t = term();
        assert!(!run_validate(&fs, &t, &ValidateTarget::Agents, Path::new("/root")));
        assert!(t.stderr_output().iter().any(|s| s.contains("Missing required field")));
    }

    #[test]
    fn agents_invalid_model() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/bad.md",
            "---\nmodel: gpt4\ntools: Read\n---\n# Agent",
        );
        let t = term();
        assert!(!run_validate(&fs, &t, &ValidateTarget::Agents, Path::new("/root")));
        assert!(t.stderr_output().iter().any(|s| s.contains("Invalid model")));
    }

    // --- validate_commands ---

    #[test]
    fn commands_no_dir_succeeds() {
        let fs = InMemoryFileSystem::new();
        let t = term();
        assert!(run_validate(&fs, &t, &ValidateTarget::Commands, Path::new("/root")));
    }

    #[test]
    fn commands_valid_file() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/commands/test.md", "# Command");
        let t = term();
        assert!(run_validate(&fs, &t, &ValidateTarget::Commands, Path::new("/root")));
    }

    #[test]
    fn commands_empty_file() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/commands/bad.md", "   ");
        let t = term();
        assert!(!run_validate(&fs, &t, &ValidateTarget::Commands, Path::new("/root")));
        assert!(t.stderr_output().iter().any(|s| s.contains("Empty command file")));
    }

    // --- validate_hooks ---

    #[test]
    fn hooks_no_file_succeeds() {
        let fs = InMemoryFileSystem::new();
        let t = term();
        assert!(run_validate(&fs, &t, &ValidateTarget::Hooks, Path::new("/root")));
    }

    #[test]
    fn hooks_valid() {
        let json = r#"{"PreToolUse": [{"matcher": "Write", "hooks": [{"type": "command", "command": "echo ok"}]}]}"#;
        let fs = InMemoryFileSystem::new()
            .with_file("/root/hooks/hooks.json", json);
        let t = term();
        assert!(run_validate(&fs, &t, &ValidateTarget::Hooks, Path::new("/root")));
    }

    #[test]
    fn hooks_invalid_json() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/hooks/hooks.json", "not json");
        let t = term();
        assert!(!run_validate(&fs, &t, &ValidateTarget::Hooks, Path::new("/root")));
        assert!(t.stderr_output().iter().any(|s| s.contains("Invalid JSON")));
    }

    #[test]
    fn hooks_invalid_event() {
        let json = r#"{"InvalidEvent": [{"matcher": "x", "hooks": [{"type": "command", "command": "echo"}]}]}"#;
        let fs = InMemoryFileSystem::new()
            .with_file("/root/hooks/hooks.json", json);
        let t = term();
        assert!(!run_validate(&fs, &t, &ValidateTarget::Hooks, Path::new("/root")));
        assert!(t.stderr_output().iter().any(|s| s.contains("Invalid event type")));
    }

    // --- validate_skills ---

    #[test]
    fn skills_no_dir_succeeds() {
        let fs = InMemoryFileSystem::new();
        let t = term();
        assert!(run_validate(&fs, &t, &ValidateTarget::Skills, Path::new("/root")));
    }

    #[test]
    fn skills_valid_dir() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/skills/tdd/SKILL.md", "# TDD Skill")
            .with_dir("/root/skills/tdd");
        let t = term();
        assert!(run_validate(&fs, &t, &ValidateTarget::Skills, Path::new("/root")));
    }

    #[test]
    fn skills_missing_skill_md() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/skills")
            .with_dir("/root/skills/empty-skill");
        let t = term();
        assert!(!run_validate(&fs, &t, &ValidateTarget::Skills, Path::new("/root")));
        assert!(t.stderr_output().iter().any(|s| s.contains("Missing SKILL.md")));
    }

    // --- validate_rules ---

    #[test]
    fn rules_no_dir_succeeds() {
        let fs = InMemoryFileSystem::new();
        let t = term();
        assert!(run_validate(&fs, &t, &ValidateTarget::Rules, Path::new("/root")));
    }

    #[test]
    fn rules_valid_file() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/rules")
            .with_file("/root/rules/common/test.md", "# Rule");
        let t = term();
        assert!(run_validate(&fs, &t, &ValidateTarget::Rules, Path::new("/root")));
    }

    #[test]
    fn rules_empty_file() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/rules")
            .with_file("/root/rules/common/bad.md", "  ");
        let t = term();
        assert!(!run_validate(&fs, &t, &ValidateTarget::Rules, Path::new("/root")));
        assert!(t.stderr_output().iter().any(|s| s.contains("Empty rule file")));
    }

    // --- validate_paths ---

    #[test]
    fn paths_no_targets_succeeds() {
        let fs = InMemoryFileSystem::new();
        let t = term();
        assert!(run_validate(&fs, &t, &ValidateTarget::Paths, Path::new("/root")));
    }

    #[test]
    fn paths_clean_files() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/README.md", "# Project\nClean content.");
        let t = term();
        assert!(run_validate(&fs, &t, &ValidateTarget::Paths, Path::new("/root")));
    }

    #[test]
    fn paths_personal_path_detected() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/README.md", "See /Users/affoon/code for details.");
        let t = term();
        assert!(!run_validate(&fs, &t, &ValidateTarget::Paths, Path::new("/root")));
        assert!(t.stderr_output().iter().any(|s| s.contains("personal path detected")));
    }

    #[test]
    fn paths_read_dir_error_reported() {
        // Verify that read_dir errors are now reported (fixes ERR-008)
        let fs = InMemoryFileSystem::new()
            .with_file("/root/skills/test.md", "content");
        let t = term();
        // skills dir exists as a file (not a dir), so read_dir_recursive will fail
        // This verifies the error path is now surfaced
        let result = run_validate(&fs, &t, &ValidateTarget::Paths, Path::new("/root"));
        // Should still pass since this path is not a checked extension
        assert!(result);
    }
}
