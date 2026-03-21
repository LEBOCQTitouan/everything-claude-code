//! Skill hot-loading — read skill files from the skills directory.

use super::ClawPorts;

/// Load a skill by name. Searches for `skills/<name>/SKILL.md` relative to home.
pub fn load_skill(name: &str, ports: &ClawPorts<'_>) -> Result<String, String> {
    let home = ports
        .env
        .home_dir()
        .ok_or_else(|| "No home directory".to_string())?;

    // Try multiple locations
    let candidates = [
        home.join(".claude")
            .join("skills")
            .join(name)
            .join("SKILL.md"),
        home.join(".claude")
            .join("skills")
            .join(format!("{name}.md")),
    ];

    for path in &candidates {
        if let Ok(content) = ports.fs.read_to_string(path) {
            return Ok(content);
        }
    }

    Err(format!("Skill '{name}' not found"))
}

/// List available skills by scanning the skills directory.
pub fn list_skills(ports: &ClawPorts<'_>) -> Vec<String> {
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return Vec::new(),
    };

    let skills_dir = home.join(".claude").join("skills");
    match ports.fs.read_dir(&skills_dir) {
        Ok(entries) => entries
            .iter()
            .filter_map(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.to_string())
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{
        BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor, ScriptedInput,
    };

    fn make_ports<'a>(
        fs: &'a InMemoryFileSystem,
        shell: &'a MockExecutor,
        env: &'a MockEnvironment,
        term: &'a BufferedTerminal,
        input: &'a ScriptedInput,
    ) -> ClawPorts<'a> {
        ClawPorts {
            fs,
            shell,
            env,
            terminal: term,
            repl_input: input,
        }
    }

    #[test]
    fn load_skill_from_skill_dir() {
        let fs = InMemoryFileSystem::new()
            .with_file("/home/test/.claude/skills/tdd/SKILL.md", "# TDD Workflow");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let result = load_skill("tdd", &ports);
        assert_eq!(result.unwrap(), "# TDD Workflow");
    }

    #[test]
    fn load_skill_from_flat_file() {
        let fs = InMemoryFileSystem::new()
            .with_file("/home/test/.claude/skills/security.md", "# Security");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let result = load_skill("security", &ports);
        assert_eq!(result.unwrap(), "# Security");
    }

    #[test]
    fn load_skill_not_found() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let result = load_skill("nonexistent", &ports);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn load_skill_no_home() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home_none();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let result = load_skill("any", &ports);
        assert!(result.is_err());
    }

    #[test]
    fn list_skills_returns_entries() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/home/test/.claude/skills/tdd")
            .with_dir("/home/test/.claude/skills/security");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let skills = list_skills(&ports);
        assert!(skills.contains(&"tdd".to_string()));
        assert!(skills.contains(&"security".to_string()));
    }

    #[test]
    fn list_skills_empty() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let skills = list_skills(&ports);
        assert!(skills.is_empty());
    }

    #[test]
    fn list_skills_no_home() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home_none();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let skills = list_skills(&ports);
        assert!(skills.is_empty());
    }

    #[test]
    fn load_skill_prefers_skill_dir_over_flat() {
        let fs = InMemoryFileSystem::new()
            .with_file("/home/test/.claude/skills/tdd/SKILL.md", "# From dir")
            .with_file("/home/test/.claude/skills/tdd.md", "# From flat");
        let shell = MockExecutor::new();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let input = ScriptedInput::new();
        let ports = make_ports(&fs, &shell, &env, &term, &input);

        let result = load_skill("tdd", &ports);
        assert_eq!(result.unwrap(), "# From dir");
    }
}
