//! CLI command: `ecc validate <target>`
//!
//! Validates content files (agents, commands, hooks, skills, rules, paths).

use clap::{Args, Subcommand};
use ecc_infra::os_fs::OsFileSystem;
use ecc_ports::fs::FileSystem;
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct ValidateArgs {
    #[command(subcommand)]
    pub target: ValidateTarget,

    /// ECC root directory (defaults to current directory)
    #[arg(long, default_value = ".")]
    pub ecc_root: PathBuf,
}

#[derive(Subcommand)]
pub enum ValidateTarget {
    /// Validate agent markdown files
    Agents,
    /// Validate command markdown files
    Commands,
    /// Validate hooks.json schema
    Hooks,
    /// Validate skill directories
    Skills,
    /// Validate rule markdown files
    Rules,
    /// Check for personal paths in shipped files
    Paths,
}

pub fn run(args: ValidateArgs) -> anyhow::Result<()> {
    let fs = OsFileSystem;
    let root = &args.ecc_root;

    match args.target {
        ValidateTarget::Agents => validate_agents(root, &fs),
        ValidateTarget::Commands => validate_commands(root, &fs),
        ValidateTarget::Hooks => validate_hooks(root, &fs),
        ValidateTarget::Skills => validate_skills(root, &fs),
        ValidateTarget::Rules => validate_rules(root, &fs),
        ValidateTarget::Paths => validate_paths(root, &fs),
    }
}

fn validate_agents(root: &Path, fs: &dyn FileSystem) -> anyhow::Result<()> {
    let agents_dir = root.join("agents");
    if !fs.exists(&agents_dir) {
        println!("No agents directory found, skipping validation");
        return Ok(());
    }

    let files = fs.read_dir(&agents_dir)?;
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
            && !valid_models.contains(&model.as_str()) {
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
        std::process::exit(1);
    }

    println!("Validated {} agent files", md_files.len());
    Ok(())
}

fn validate_commands(root: &Path, fs: &dyn FileSystem) -> anyhow::Result<()> {
    let commands_dir = root.join("commands");
    if !fs.exists(&commands_dir) {
        println!("No commands directory found, skipping validation");
        return Ok(());
    }

    let files = fs.read_dir(&commands_dir)?;
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
        std::process::exit(1);
    }

    println!("Validated {} command files", md_files.len());
    Ok(())
}

fn validate_hooks(root: &Path, fs: &dyn FileSystem) -> anyhow::Result<()> {
    let hooks_file = root.join("hooks").join("hooks.json");
    if !fs.exists(&hooks_file) {
        println!("No hooks.json found, skipping validation");
        return Ok(());
    }

    let content = fs.read_to_string(&hooks_file)?;
    let data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Invalid JSON in hooks.json: {}", e))?;

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
        std::process::exit(1);
    }

    println!("Validated {} hook matchers", total_matchers);
    Ok(())
}

fn validate_hook_entry(hook: &serde_json::Value, label: &str) -> bool {
    let mut has_errors = false;

    match hook.get("type").and_then(|v| v.as_str()) {
        Some(t) if !t.is_empty() => {}
        _ => {
            eprintln!("ERROR: {} missing or invalid 'type' field", label);
            has_errors = true;
        }
    }

    if let Some(a) = hook.get("async")
        && !a.is_boolean() {
            eprintln!("ERROR: {} 'async' must be a boolean", label);
            has_errors = true;
        }

    if let Some(t) = hook.get("timeout") {
        match t.as_f64() {
            Some(v) if v >= 0.0 => {}
            _ => {
                eprintln!("ERROR: {} 'timeout' must be a non-negative number", label);
                has_errors = true;
            }
        }
    }

    match hook.get("command") {
        Some(serde_json::Value::String(s)) if !s.trim().is_empty() => {}
        Some(serde_json::Value::Array(arr)) if !arr.is_empty() => {
            if !arr.iter().all(|v| matches!(v, serde_json::Value::String(s) if !s.is_empty())) {
                eprintln!("ERROR: {} invalid 'command' array entries", label);
                has_errors = true;
            }
        }
        _ => {
            eprintln!("ERROR: {} missing or invalid 'command' field", label);
            has_errors = true;
        }
    }

    has_errors
}

fn validate_skills(root: &Path, fs: &dyn FileSystem) -> anyhow::Result<()> {
    let skills_dir = root.join("skills");
    if !fs.exists(&skills_dir) {
        println!("No skills directory found, skipping validation");
        return Ok(());
    }

    let entries = fs.read_dir(&skills_dir)?;
    let mut has_errors = false;
    let mut valid_count = 0;

    for entry in &entries {
        if !fs.is_dir(entry) {
            continue;
        }
        let name = entry.file_name().unwrap_or_default().to_string_lossy().to_string();
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
        std::process::exit(1);
    }

    println!("Validated {} skill directories", valid_count);
    Ok(())
}

fn validate_rules(root: &Path, fs: &dyn FileSystem) -> anyhow::Result<()> {
    let rules_dir = root.join("rules");
    if !fs.exists(&rules_dir) {
        println!("No rules directory found, skipping validation");
        return Ok(());
    }

    let files = fs.read_dir_recursive(&rules_dir)?;
    let md_files: Vec<_> = files
        .iter()
        .filter(|f| f.to_string_lossy().ends_with(".md"))
        .collect();

    let mut has_errors = false;
    let mut validated = 0;

    for file in &md_files {
        match fs.read_to_string(file) {
            Ok(c) if c.trim().is_empty() => {
                let rel = file
                    .strip_prefix(root)
                    .unwrap_or(file);
                eprintln!("ERROR: {} - Empty rule file", rel.display());
                has_errors = true;
            }
            Err(e) => {
                let rel = file
                    .strip_prefix(root)
                    .unwrap_or(file);
                eprintln!("ERROR: {} - {}", rel.display(), e);
                has_errors = true;
            }
            Ok(_) => {
                validated += 1;
            }
        }
    }

    if has_errors {
        std::process::exit(1);
    }

    println!("Validated {} rule files", validated);
    Ok(())
}

fn validate_paths(root: &Path, fs: &dyn FileSystem) -> anyhow::Result<()> {
    let targets = [
        "README.md",
        "skills",
        "commands",
        "agents",
        "docs",
    ];

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
        std::process::exit(1);
    }

    println!("Validated: no personal absolute paths in shipped docs/skills/commands");
    Ok(())
}

/// Extract YAML frontmatter from markdown content into a key-value map.
fn extract_frontmatter(content: &str) -> Option<std::collections::HashMap<String, String>> {
    let clean = content.strip_prefix('\u{FEFF}').unwrap_or(content);
    let rest = clean.strip_prefix("---")?;
    let end = rest.find("\n---")?;
    let frontmatter_str = &rest[..end];

    let mut map = std::collections::HashMap::new();
    for line in frontmatter_str.lines() {
        if let Some(colon_idx) = line.find(':') {
            let key = line[..colon_idx].trim().to_string();
            let value = line[colon_idx + 1..].trim().to_string();
            if !key.is_empty() {
                map.insert(key, value);
            }
        }
    }

    Some(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- extract_frontmatter ---

    #[test]
    fn extracts_frontmatter_fields() {
        let content = "---\nname: test-agent\nmodel: sonnet\ntools: Read, Write\n---\n# Body";
        let fm = extract_frontmatter(content).unwrap();
        assert_eq!(fm.get("name").unwrap(), "test-agent");
        assert_eq!(fm.get("model").unwrap(), "sonnet");
        assert_eq!(fm.get("tools").unwrap(), "Read, Write");
    }

    #[test]
    fn missing_frontmatter_returns_none() {
        assert!(extract_frontmatter("# No frontmatter here").is_none());
    }

    #[test]
    fn bom_stripped() {
        let content = "\u{FEFF}---\nmodel: haiku\n---\n# Body";
        let fm = extract_frontmatter(content).unwrap();
        assert_eq!(fm.get("model").unwrap(), "haiku");
    }

    #[test]
    fn empty_value_preserved() {
        let content = "---\nmodel: \ntools: Read\n---\n";
        let fm = extract_frontmatter(content).unwrap();
        assert_eq!(fm.get("model").unwrap(), "");
    }

    // --- validate_hook_entry ---

    #[test]
    fn valid_hook_entry() {
        let hook = serde_json::json!({
            "type": "command",
            "command": "echo hello"
        });
        assert!(!validate_hook_entry(&hook, "test"));
    }

    #[test]
    fn hook_missing_type() {
        let hook = serde_json::json!({
            "command": "echo hello"
        });
        assert!(validate_hook_entry(&hook, "test"));
    }

    #[test]
    fn hook_missing_command() {
        let hook = serde_json::json!({
            "type": "command"
        });
        assert!(validate_hook_entry(&hook, "test"));
    }

    #[test]
    fn hook_invalid_async() {
        let hook = serde_json::json!({
            "type": "command",
            "command": "echo hello",
            "async": "yes"
        });
        assert!(validate_hook_entry(&hook, "test"));
    }

    #[test]
    fn hook_valid_array_command() {
        let hook = serde_json::json!({
            "type": "command",
            "command": ["echo", "hello"]
        });
        assert!(!validate_hook_entry(&hook, "test"));
    }

    #[test]
    fn hook_invalid_timeout() {
        let hook = serde_json::json!({
            "type": "command",
            "command": "echo hello",
            "timeout": -5
        });
        assert!(validate_hook_entry(&hook, "test"));
    }
}
