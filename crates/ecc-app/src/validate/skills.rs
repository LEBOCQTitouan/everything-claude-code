use ecc_domain::config::validate::extract_frontmatter;
use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

pub(super) fn validate_skills(root: &Path, fs: &dyn FileSystem, terminal: &dyn TerminalIO) -> bool {
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
            Ok(content) => {
                if validate_skill_file(&name, &content, terminal) {
                    valid_count += 1;
                } else {
                    has_errors = true;
                }
            }
        }
    }

    if has_errors {
        return false;
    }

    terminal.stdout_write(&format!("Validated {} skill directories\n", valid_count));
    true
}

fn validate_skill_file(name: &str, content: &str, terminal: &dyn TerminalIO) -> bool {
    let fm = extract_frontmatter(content);
    let required_fields = ["name", "description", "origin"];
    let mut has_errors = false;

    match fm {
        Some(ref map) => {
            for field in &required_fields {
                match map.get(*field) {
                    Some(v) if !v.trim().is_empty() => {}
                    _ => {
                        terminal.stderr_write(&format!(
                            "ERROR: {}/SKILL.md - Missing required frontmatter field '{}'\n",
                            name, field
                        ));
                        has_errors = true;
                    }
                }
            }
            for warn_field in &["model", "tools"] {
                if map.contains_key(*warn_field) {
                    terminal.stdout_write(&format!(
                        "WARNING: {}/SKILL.md - Skills should not have '{}' field (use agents for behavioral configuration)\n",
                        name, warn_field
                    ));
                }
            }
        }
        None => {
            terminal.stderr_write(&format!(
                "ERROR: {}/SKILL.md - No frontmatter found (requires name, description, origin)\n",
                name
            ));
            has_errors = true;
        }
    }

    !has_errors
}

#[cfg(test)]
mod tests {
    use super::super::{ValidateTarget, run_validate};
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment};
    use std::path::Path;

    fn term() -> BufferedTerminal {
        BufferedTerminal::new()
    }

    #[test]
    fn skills_no_dir_succeeds() {
        let fs = InMemoryFileSystem::new();
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Skills,
            Path::new("/root")
        ));
    }

    #[test]
    fn skills_valid_dir() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/skills")
            .with_file(
                "/root/skills/tdd/SKILL.md",
                "---\nname: tdd\ndescription: TDD skill\norigin: ECC\n---\n# TDD Skill",
            )
            .with_dir("/root/skills/tdd");
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Skills,
            Path::new("/root")
        ));
    }

    #[test]
    fn skills_missing_skill_md() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/skills")
            .with_dir("/root/skills/empty-skill");
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Skills,
            Path::new("/root")
        ));
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("Missing SKILL.md"))
        );
    }

    #[test]
    fn skills_missing_name_field() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/skills")
            .with_file(
                "/root/skills/bad/SKILL.md",
                "---\ndescription: test\norigin: ECC\n---\n# Bad",
            )
            .with_dir("/root/skills/bad");
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Skills,
            Path::new("/root")
        ));
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("Missing required frontmatter field 'name'"))
        );
    }

    #[test]
    fn skills_missing_description_field() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/skills")
            .with_file(
                "/root/skills/bad/SKILL.md",
                "---\nname: bad\norigin: ECC\n---\n# Bad",
            )
            .with_dir("/root/skills/bad");
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Skills,
            Path::new("/root")
        ));
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("Missing required frontmatter field 'description'"))
        );
    }

    #[test]
    fn skills_missing_origin_field() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/skills")
            .with_file(
                "/root/skills/bad/SKILL.md",
                "---\nname: bad\ndescription: test\n---\n# Bad",
            )
            .with_dir("/root/skills/bad");
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Skills,
            Path::new("/root")
        ));
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("Missing required frontmatter field 'origin'"))
        );
    }

    #[test]
    fn skills_valid_frontmatter() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/skills")
            .with_file(
                "/root/skills/good/SKILL.md",
                "---\nname: good\ndescription: A good skill\norigin: ECC\n---\n# Good",
            )
            .with_dir("/root/skills/good");
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Skills,
            Path::new("/root")
        ));
    }

    #[test]
    fn skills_warns_on_model_or_tools() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/skills")
            .with_file(
                "/root/skills/warn/SKILL.md",
                "---\nname: warn\ndescription: test\norigin: ECC\nmodel: opus\ntools: [Read]\n---\n# Warn",
            )
            .with_dir("/root/skills/warn");
        let t = term();
        assert!(run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Skills,
            Path::new("/root")
        ));
        assert!(
            t.stdout_output()
                .iter()
                .any(|s| s.contains("WARNING") && s.contains("model"))
        );
        assert!(
            t.stdout_output()
                .iter()
                .any(|s| s.contains("WARNING") && s.contains("tools"))
        );
    }

    #[test]
    fn skills_no_frontmatter() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/skills")
            .with_file(
                "/root/skills/nofm/SKILL.md",
                "# No Frontmatter\nJust content, no --- delimiters",
            )
            .with_dir("/root/skills/nofm");
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Skills,
            Path::new("/root")
        ));
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("No frontmatter found"))
        );
    }

    #[test]
    fn skills_valid_count_accuracy() {
        let fs = InMemoryFileSystem::new()
            .with_dir("/root/skills")
            .with_file(
                "/root/skills/good1/SKILL.md",
                "---\nname: good1\ndescription: test\norigin: ECC\n---\n# G1",
            )
            .with_dir("/root/skills/good1")
            .with_file(
                "/root/skills/bad1/SKILL.md",
                "---\ndescription: test\norigin: ECC\n---\n# Missing name",
            )
            .with_dir("/root/skills/bad1");
        let t = term();
        assert!(!run_validate(
            &fs,
            &t,
            &MockEnvironment::default(),
            &ValidateTarget::Skills,
            Path::new("/root")
        ));
        assert!(
            t.stderr_output()
                .iter()
                .any(|s| s.contains("Missing required frontmatter field 'name'"))
        );
    }
}
