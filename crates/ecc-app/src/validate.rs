//! Validate use case — validates content files (agents, commands, hooks, skills, rules, paths).

use ecc_domain::config::validate::{extract_frontmatter, validate_hook_entry};
use ecc_ports::fs::FileSystem;
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
///
/// Errors are printed to stderr via `eprintln!`.
pub fn run_validate(fs: &dyn FileSystem, target: &ValidateTarget, root: &Path) -> bool {
    match target {
        ValidateTarget::Agents => validate_agents(root, fs),
        ValidateTarget::Commands => validate_commands(root, fs),
        ValidateTarget::Hooks => validate_hooks(root, fs),
        ValidateTarget::Skills => validate_skills(root, fs),
        ValidateTarget::Rules => validate_rules(root, fs),
        ValidateTarget::Paths => validate_paths(root, fs),
    }
}

fn validate_agents(root: &Path, fs: &dyn FileSystem) -> bool {
    let agents_dir = root.join("agents");
    if !fs.exists(&agents_dir) {
        println!("No agents directory found, skipping validation");
        return true;
    }

    let files = match fs.read_dir(&agents_dir) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("ERROR: Cannot read agents directory: {e}");
            return false;
        }
    };
    let md_files: Vec<_> = files
        .iter()
        .filter(|f| f.to_string_lossy().ends_with(".md"))
        .collect();

    let required_fields = ["model", "tools"];
    let valid_models = ["haiku", "sonnet", "opus"];
    let mut has_errors = false;

    for file in &md_files {
        let content = match fs.read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("ERROR: {} - {}", file.display(), e);
                has_errors = true;
                continue;
            }
        };

        let frontmatter = match extract_frontmatter(&content) {
            Some(fm) => fm,
            None => {
                let name = file.file_name().unwrap_or_default().to_string_lossy();
                eprintln!("ERROR: {} - Missing frontmatter", name);
                has_errors = true;
                continue;
            }
        };

        let name = file.file_name().unwrap_or_default().to_string_lossy();

        for field in &required_fields {
            match frontmatter.get(*field) {
                Some(v) if !v.trim().is_empty() => {}
                _ => {
                    eprintln!("ERROR: {} - Missing required field: {}", name, field);
                    has_errors = true;
                }
            }
        }

        if let Some(model) = frontmatter.get("model")
            && !valid_models.contains(&model.as_str())
        {
            eprintln!(
                "ERROR: {} - Invalid model '{}'. Must be one of: {}",
                name,
                model,
                valid_models.join(", ")
            );
            has_errors = true;
        }
    }

    if has_errors {
        return false;
    }

    println!("Validated {} agent files", md_files.len());
    true
}

fn validate_commands(root: &Path, fs: &dyn FileSystem) -> bool {
    let commands_dir = root.join("commands");
    if !fs.exists(&commands_dir) {
        println!("No commands directory found, skipping validation");
        return true;
    }

    let files = match fs.read_dir(&commands_dir) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("ERROR: Cannot read commands directory: {e}");
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
                eprintln!("ERROR: {} - {}", file.display(), e);
                has_errors = true;
                continue;
            }
        };

        let name = file.file_name().unwrap_or_default().to_string_lossy();

        if content.trim().is_empty() {
            eprintln!("ERROR: {} - Empty command file", name);
            has_errors = true;
        }
    }

    if has_errors {
        return false;
    }

    println!("Validated {} command files", md_files.len());
    true
}

fn validate_hooks(root: &Path, fs: &dyn FileSystem) -> bool {
    let hooks_file = root.join("hooks").join("hooks.json");
    if !fs.exists(&hooks_file) {
        println!("No hooks.json found, skipping validation");
        return true;
    }

    let content = match fs.read_to_string(&hooks_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ERROR: Cannot read hooks.json: {e}");
            return false;
        }
    };
    let data: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("ERROR: Invalid JSON in hooks.json: {e}");
            return false;
        }
    };

    let valid_events = [
        "PreToolUse",
        "PostToolUse",
        "PreCompact",
        "SessionStart",
        "SessionEnd",
        "Stop",
        "Notification",
        "SubagentStop",
    ];

    let hooks = data.get("hooks").unwrap_or(&data);
    let mut has_errors = false;
    let mut total_matchers = 0;

    if let Some(obj) = hooks.as_object() {
        for (event_type, matchers) in obj {
            if !valid_events.contains(&event_type.as_str()) {
                eprintln!("ERROR: Invalid event type: {}", event_type);
                has_errors = true;
                continue;
            }

            let matchers = match matchers.as_array() {
                Some(a) => a,
                None => {
                    eprintln!("ERROR: {} must be an array", event_type);
                    has_errors = true;
                    continue;
                }
            };

            for (i, matcher) in matchers.iter().enumerate() {
                let obj = match matcher.as_object() {
                    Some(o) => o,
                    None => {
                        eprintln!("ERROR: {}[{}] is not an object", event_type, i);
                        has_errors = true;
                        continue;
                    }
                };

                if obj.get("matcher").and_then(|v| v.as_str()).is_none() {
                    eprintln!("ERROR: {}[{}] missing 'matcher' field", event_type, i);
                    has_errors = true;
                }

                match obj.get("hooks").and_then(|v| v.as_array()) {
                    Some(hooks) => {
                        for (j, hook) in hooks.iter().enumerate() {
                            let label = format!("{}[{}].hooks[{}]", event_type, i, j);
                            if validate_hook_entry(hook, &label) {
                                has_errors = true;
                            }
                        }
                    }
                    None => {
                        eprintln!("ERROR: {}[{}] missing 'hooks' array", event_type, i);
                        has_errors = true;
                    }
                }

                total_matchers += 1;
            }
        }
    }

    if has_errors {
        return false;
    }

    println!("Validated {} hook matchers", total_matchers);
    true
}

fn validate_skills(root: &Path, fs: &dyn FileSystem) -> bool {
    let skills_dir = root.join("skills");
    if !fs.exists(&skills_dir) {
        println!("No skills directory found, skipping validation");
        return true;
    }

    let entries = match fs.read_dir(&skills_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("ERROR: Cannot read skills directory: {e}");
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
            eprintln!("ERROR: {}/ - Missing SKILL.md", name);
            has_errors = true;
            continue;
        }

        match fs.read_to_string(&skill_md) {
            Ok(c) if c.trim().is_empty() => {
                eprintln!("ERROR: {}/SKILL.md - Empty file", name);
                has_errors = true;
            }
            Err(e) => {
                eprintln!("ERROR: {}/SKILL.md - {}", name, e);
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

    println!("Validated {} skill directories", valid_count);
    true
}

fn validate_rules(root: &Path, fs: &dyn FileSystem) -> bool {
    let rules_dir = root.join("rules");
    if !fs.exists(&rules_dir) {
        println!("No rules directory found, skipping validation");
        return true;
    }

    let files = match fs.read_dir_recursive(&rules_dir) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("ERROR: Cannot read rules directory: {e}");
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
                eprintln!("ERROR: {} - Empty rule file", rel.display());
                has_errors = true;
            }
            Err(e) => {
                let rel = file.strip_prefix(root).unwrap_or(file);
                eprintln!("ERROR: {} - {}", rel.display(), e);
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

    println!("Validated {} rule files", validated);
    true
}

fn validate_paths(root: &Path, fs: &dyn FileSystem) -> bool {
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
            fs.read_dir_recursive(&target_path).unwrap_or_default()
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
                        eprintln!("ERROR: personal path detected in {}", rel.display());
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

    println!("Validated: no personal absolute paths in shipped docs/skills/commands");
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;

    // --- validate_agents ---

    #[test]
    fn agents_no_dir_succeeds() {
        let fs = InMemoryFileSystem::new();
        assert!(run_validate(&fs, &ValidateTarget::Agents, Path::new("/root")));
    }

    #[test]
    fn agents_valid_file() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/test.md",
            "---\nmodel: sonnet\ntools: Read\n---\n# Agent",
        );
        assert!(run_validate(&fs, &ValidateTarget::Agents, Path::new("/root")));
    }

    #[test]
    fn agents_missing_frontmatter() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/agents/bad.md", "# No frontmatter");
        assert!(!run_validate(&fs, &ValidateTarget::Agents, Path::new("/root")));
    }

    #[test]
    fn agents_missing_required_field() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/bad.md",
            "---\nmodel: sonnet\n---\n# Agent",
        );
        assert!(!run_validate(&fs, &ValidateTarget::Agents, Path::new("/root")));
    }

    #[test]
    fn agents_invalid_model() {
        let fs = InMemoryFileSystem::new().with_file(
            "/root/agents/bad.md",
            "---\nmodel: gpt4\ntools: Read\n---\n# Agent",
        );
        assert!(!run_validate(&fs, &ValidateTarget::Agents, Path::new("/root")));
    }

    // --- validate_commands ---

    #[test]
    fn commands_no_dir_succeeds() {
        let fs = InMemoryFileSystem::new();
        assert!(run_validate(&fs, &ValidateTarget::Commands, Path::new("/root")));
    }

    #[test]
    fn commands_valid_file() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/commands/test.md", "# Command");
        assert!(run_validate(&fs, &ValidateTarget::Commands, Path::new("/root")));
    }

    #[test]
    fn commands_empty_file() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/commands/bad.md", "   ");
        assert!(!run_validate(&fs, &ValidateTarget::Commands, Path::new("/root")));
    }

    // --- validate_hooks ---

    #[test]
    fn hooks_no_file_succeeds() {
        let fs = InMemoryFileSystem::new();
        assert!(run_validate(&fs, &ValidateTarget::Hooks, Path::new("/root")));
    }

    #[test]
    fn hooks_valid() {
        let json = r#"{"PreToolUse": [{"matcher": "Write", "hooks": [{"type": "command", "command": "echo ok"}]}]}"#;
        let fs = InMemoryFileSystem::new()
            .with_file("/root/hooks/hooks.json", json);
        assert!(run_validate(&fs, &ValidateTarget::Hooks, Path::new("/root")));
    }

    #[test]
    fn hooks_invalid_json() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/hooks/hooks.json", "not json");
        assert!(!run_validate(&fs, &ValidateTarget::Hooks, Path::new("/root")));
    }

    #[test]
    fn hooks_invalid_event() {
        let json = r#"{"InvalidEvent": [{"matcher": "x", "hooks": [{"type": "command", "command": "echo"}]}]}"#;
        let fs = InMemoryFileSystem::new()
            .with_file("/root/hooks/hooks.json", json);
        assert!(!run_validate(&fs, &ValidateTarget::Hooks, Path::new("/root")));
    }

    // --- validate_skills ---

    #[test]
    fn skills_no_dir_succeeds() {
        let fs = InMemoryFileSystem::new();
        assert!(run_validate(&fs, &ValidateTarget::Skills, Path::new("/root")));
    }

    #[test]
    fn skills_valid_dir() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/skills/tdd/SKILL.md", "# TDD Skill")
            .with_dir("/root/skills/tdd");
        assert!(run_validate(&fs, &ValidateTarget::Skills, Path::new("/root")));
    }

    #[test]
    fn skills_missing_skill_md() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/skills")
            .with_dir("/root/skills/empty-skill");
        assert!(!run_validate(&fs, &ValidateTarget::Skills, Path::new("/root")));
    }

    // --- validate_rules ---

    #[test]
    fn rules_no_dir_succeeds() {
        let fs = InMemoryFileSystem::new();
        assert!(run_validate(&fs, &ValidateTarget::Rules, Path::new("/root")));
    }

    #[test]
    fn rules_valid_file() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/rules")
            .with_file("/root/rules/common/test.md", "# Rule");
        assert!(run_validate(&fs, &ValidateTarget::Rules, Path::new("/root")));
    }

    #[test]
    fn rules_empty_file() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/rules")
            .with_file("/root/rules/common/bad.md", "  ");
        assert!(!run_validate(&fs, &ValidateTarget::Rules, Path::new("/root")));
    }

    // --- validate_paths ---

    #[test]
    fn paths_no_targets_succeeds() {
        let fs = InMemoryFileSystem::new();
        assert!(run_validate(&fs, &ValidateTarget::Paths, Path::new("/root")));
    }

    #[test]
    fn paths_clean_files() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/README.md", "# Project\nClean content.");
        assert!(run_validate(&fs, &ValidateTarget::Paths, Path::new("/root")));
    }

    #[test]
    fn paths_personal_path_detected() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/README.md", "See /Users/affoon/code for details.");
        assert!(!run_validate(&fs, &ValidateTarget::Paths, Path::new("/root")));
    }
}
