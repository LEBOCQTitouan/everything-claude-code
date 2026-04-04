use crate::hook::{HookPorts, HookResult};

/// Detect whether the current working directory is inside a git worktree.
///
/// Compares `git rev-parse --show-toplevel` against the parent of
/// `git rev-parse --git-common-dir`. If they differ, we're in a worktree.
///
/// Returns:
/// - `Ok(true)` if in a worktree
/// - `Ok(false)` if NOT in a worktree (main repo checkout)
/// - `Err(())` if not a git repository (graceful degradation)
pub fn is_in_worktree(ports: &HookPorts<'_>) -> Result<bool, ()> {
    let toplevel = ports
        .shell
        .run_command("git", &["rev-parse", "--show-toplevel"])
        .map_err(|_| ())?;

    if !toplevel.success() {
        return Err(()); // not a git repo
    }

    let common_dir = ports
        .shell
        .run_command("git", &["rev-parse", "--git-common-dir"])
        .map_err(|_| ())?;

    if !common_dir.success() {
        return Err(());
    }

    let toplevel_path = toplevel.stdout.trim();
    let common_dir_raw = common_dir.stdout.trim();

    // Resolve the parent of git-common-dir to get the main repo root.
    // git-common-dir may return a relative path (e.g., "../../.git") or absolute.
    let common_parent = if std::path::Path::new(common_dir_raw).is_absolute() {
        // Absolute: take parent directly
        std::path::Path::new(common_dir_raw)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    } else {
        // Relative: resolve relative to toplevel, then take parent
        let resolved = std::path::Path::new(toplevel_path).join(common_dir_raw);
        resolved
            .canonicalize()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_string_lossy().to_string()))
            .unwrap_or_default()
    };

    // If toplevel differs from the parent of git-common-dir, we're in a worktree
    Ok(toplevel_path != common_parent)
}

/// pre:write-edit:worktree-guard — block Write/Edit/MultiEdit outside a worktree.
///
/// Forces Claude to call `EnterWorktree` before making file changes.
/// Respects `ECC_WORKFLOW_BYPASS=1` and passes through for non-git directories.
pub fn pre_worktree_write_guard(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(
        handler = "pre_worktree_write_guard",
        "executing handler"
    );

    // Bypass check
    if ports.env.var("ECC_WORKFLOW_BYPASS").as_deref() == Some("1") {
        return HookResult::passthrough(stdin);
    }

    match is_in_worktree(ports) {
        Ok(true) => {
            // In a worktree — allow the write
            HookResult::passthrough(stdin)
        }
        Ok(false) => {
            // NOT in a worktree — block with actionable message
            HookResult::block(
                stdin,
                "[Hook] BLOCKED: Not in a worktree. Call EnterWorktree before making changes.\n\
                 If EnterWorktree is unavailable, set ECC_WORKFLOW_BYPASS=1 to proceed on main.\n",
            )
        }
        Err(()) => {
            // Not a git repo — allow (graceful degradation)
            HookResult::passthrough(stdin)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookPorts;
    use ecc_ports::shell::CommandOutput;
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

    fn git_output(stdout: &str) -> CommandOutput {
        CommandOutput {
            stdout: stdout.to_string(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    fn git_failure() -> CommandOutput {
        CommandOutput {
            stdout: String::new(),
            stderr: "fatal: not a git repository".to_string(),
            exit_code: 128,
        }
    }

    // PC-001a: blocks write outside worktree
    #[test]
    fn blocks_write_outside_worktree() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new()
            .on_args(
                "git",
                &["rev-parse", "--show-toplevel"],
                git_output("/repo\n"),
            )
            .on_args(
                "git",
                &["rev-parse", "--git-common-dir"],
                git_output("/repo/.git\n"),
            );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = pre_worktree_write_guard(r#"{"file_path":"src/main.rs"}"#, &ports);
        assert_eq!(result.exit_code, 2);
        assert!(result.stderr.contains("BLOCKED"));
        assert!(result.stderr.contains("EnterWorktree"));
    }

    // PC-001b: allows write inside worktree
    #[test]
    fn allows_write_inside_worktree() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new()
            .on_args(
                "git",
                &["rev-parse", "--show-toplevel"],
                git_output("/repo/.claude/worktrees/session-123\n"),
            )
            .on_args(
                "git",
                &["rev-parse", "--git-common-dir"],
                git_output("/repo/.git\n"),
            );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = pre_worktree_write_guard(r#"{"file_path":"src/main.rs"}"#, &ports);
        assert_eq!(result.exit_code, 0);
    }

    // PC-001c: bypass allows write
    #[test]
    fn bypass_allows_write() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new(); // no git responses — shouldn't even be called
        let env = MockEnvironment::new().with_var("ECC_WORKFLOW_BYPASS", "1");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = pre_worktree_write_guard(r#"{"file_path":"src/main.rs"}"#, &ports);
        assert_eq!(result.exit_code, 0);
    }

    // PC-001d: non-git passthrough
    #[test]
    fn non_git_passthrough() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on_args(
            "git",
            &["rev-parse", "--show-toplevel"],
            git_failure(),
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = pre_worktree_write_guard(r#"{"file_path":"src/main.rs"}"#, &ports);
        assert_eq!(result.exit_code, 0);
    }

    // PC-001e: worktree detection logic
    #[test]
    fn detects_worktree_via_git_common_dir() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new()
            .on_args(
                "git",
                &["rev-parse", "--show-toplevel"],
                git_output("/repo/.claude/worktrees/session-abc\n"),
            )
            .on_args(
                "git",
                &["rev-parse", "--git-common-dir"],
                git_output("/repo/.git\n"),
            );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = is_in_worktree(&ports);
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn detects_not_worktree() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new()
            .on_args(
                "git",
                &["rev-parse", "--show-toplevel"],
                git_output("/repo\n"),
            )
            .on_args(
                "git",
                &["rev-parse", "--git-common-dir"],
                git_output("/repo/.git\n"),
            );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = is_in_worktree(&ports);
        assert_eq!(result, Ok(false));
    }

    // PC-001f: pipeline worktree passes
    #[test]
    fn pipeline_worktree_passes() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new()
            .on_args(
                "git",
                &["rev-parse", "--show-toplevel"],
                git_output("/repo/.claude/worktrees/ecc-session-20260402-spec-dev-1234\n"),
            )
            .on_args(
                "git",
                &["rev-parse", "--git-common-dir"],
                git_output("/repo/.git\n"),
            );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = pre_worktree_write_guard(r#"{"file_path":"src/main.rs"}"#, &ports);
        assert_eq!(result.exit_code, 0);
    }

    // PC-001g: block message includes fallback
    #[test]
    fn block_message_includes_fallback() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new()
            .on_args(
                "git",
                &["rev-parse", "--show-toplevel"],
                git_output("/repo\n"),
            )
            .on_args(
                "git",
                &["rev-parse", "--git-common-dir"],
                git_output("/repo/.git\n"),
            );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = pre_worktree_write_guard(r#"{"file_path":"src/main.rs"}"#, &ports);
        assert_eq!(result.exit_code, 2);
        assert!(result.stderr.contains("ECC_WORKFLOW_BYPASS=1"));
    }

    // PC-001i: coexists with branch guard (both can pass independently)
    #[test]
    fn coexists_with_branch_guard() {
        // In a worktree = write-guard passes. Branch guard is a separate hook.
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new()
            .on_args(
                "git",
                &["rev-parse", "--show-toplevel"],
                git_output("/repo/.claude/worktrees/session-xyz\n"),
            )
            .on_args(
                "git",
                &["rev-parse", "--git-common-dir"],
                git_output("/repo/.git\n"),
            );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = pre_worktree_write_guard(r#"{"file_path":".github/workflows/ci.yml"}"#, &ports);
        // Write-guard allows (we're in a worktree). Branch-guard is a separate hook.
        assert_eq!(result.exit_code, 0);
    }
}
