use ecc_ports::fs::FileSystem;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

pub(super) fn validate_conventions(
    _root: &Path,
    _fs: &dyn FileSystem,
    _terminal: &dyn TerminalIO,
) -> bool {
    // Stub: always returns true (RED — tests for error cases will fail)
    true
}

#[cfg(test)]
mod tests {
    use super::super::{ValidateTarget, run_validate};
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem};
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
            &ValidateTarget::Conventions,
            Path::new("/root")
        );
        assert!(result);
        assert!(t.stderr_output().is_empty());
    }
}
