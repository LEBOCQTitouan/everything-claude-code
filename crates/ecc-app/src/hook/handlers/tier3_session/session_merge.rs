use crate::hook::handlers::tier1_simple::worktree_guard::is_in_worktree;
use crate::hook::{HookPorts, HookResult};

/// session:end:worktree-merge — merge worktree back to main at session end.
///
/// Calls `ecc-workflow merge` which handles rebase + verify + ff-only merge.
/// Worktree cleanup is deferred to `ecc worktree gc` at next session start.
/// If the merge fails, the worktree is preserved and a recovery file is written.
pub fn session_end_merge(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "session_end_merge", "executing handler");

    // Check if we're in a worktree
    let in_worktree = match is_in_worktree(ports) {
        Ok(true) => true,
        Ok(false) | Err(()) => {
            // Not in worktree or not a git repo — skip merge
            return HookResult::passthrough(stdin);
        }
    };

    if !in_worktree {
        return HookResult::passthrough(stdin);
    }

    // Check if there are commits ahead of main
    let commit_count = ports
        .shell
        .run_command("git", &["rev-list", "HEAD", "^main", "--count"])
        .ok()
        .map(|o| o.stdout.trim().to_string())
        .unwrap_or_default();

    if commit_count == "0" {
        // No commits — defer cleanup to session-start gc (worktree directory preserved)
        return HookResult::warn(
            stdin,
            "[Session] Empty worktree (no commits to merge). Cleanup deferred to next session gc.\n",
        );
    }

    // Call ecc-workflow merge
    let merge_result = ports.shell.run_command("ecc-workflow", &["merge"]);

    match merge_result {
        Ok(output) if output.exit_code == 0 => HookResult::warn(
            stdin,
            &format!(
                "[Session] Worktree merged to main successfully.\n{}",
                output.stderr
            ),
        ),
        Ok(output) => {
            // Non-zero exit — preserve worktree, write recovery file, warn
            let cwd = ports
                .shell
                .run_command("pwd", &[])
                .ok()
                .map(|o| o.stdout.trim().to_string())
                .unwrap_or_else(|| ".".to_string());

            let recovery_content = format!(
                "# ECC Merge Recovery\n\
                 # Merge failed at: {}\n\
                 # Worktree: {}\n\
                 # Exit code: {}\n\
                 #\n\
                 # To retry: cd {} && ecc-workflow merge\n\
                 # To clean up: ecc worktree gc --force\n",
                chrono_like_now(),
                cwd,
                output.exit_code,
                cwd,
            );

            let recovery_path = std::path::Path::new(&cwd).join(".ecc-merge-recovery");
            let _ = ports.fs.write(&recovery_path, &recovery_content);

            HookResult::warn(
                stdin,
                &format!(
                    "[Session] Worktree merge failed (exit {}). Worktree preserved.\n\
                     Recovery file: {}\n\
                     To retry: cd {} && ecc-workflow merge\n\
                     {}\n",
                    output.exit_code,
                    recovery_path.display(),
                    cwd,
                    output.stderr,
                ),
            )
        }
        Err(e) => {
            // ecc-workflow not found or other error
            HookResult::warn(
                stdin,
                &format!(
                    "[Session] Could not run ecc-workflow merge: {e}\n\
                     Run manually: ecc-workflow merge\n"
                ),
            )
        }
    }
}

/// Simple timestamp without requiring chrono crate.
fn chrono_like_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("epoch:{secs}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookPorts;
    use ecc_ports::fs::FileSystem;
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
            bypass_store: None,
            metrics_store: None,
        }
    }

    fn ok(stdout: &str) -> CommandOutput {
        CommandOutput {
            stdout: stdout.to_string(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    fn fail(exit_code: i32, stderr: &str) -> CommandOutput {
        CommandOutput {
            stdout: String::new(),
            stderr: stderr.to_string(),
            exit_code,
        }
    }

    fn in_worktree_shell() -> MockExecutor {
        MockExecutor::new()
            .on_args(
                "git",
                &["rev-parse", "--show-toplevel"],
                ok("/repo/.claude/worktrees/session-123\n"),
            )
            .on_args(
                "git",
                &["rev-parse", "--git-common-dir"],
                ok("/repo/.git\n"),
            )
    }

    fn not_in_worktree_shell() -> MockExecutor {
        MockExecutor::new()
            .on_args("git", &["rev-parse", "--show-toplevel"], ok("/repo\n"))
            .on_args(
                "git",
                &["rev-parse", "--git-common-dir"],
                ok("/repo/.git\n"),
            )
    }

    // PC-002a: calls merge in worktree
    #[test]
    fn calls_merge_in_worktree() {
        let fs = InMemoryFileSystem::new();
        let shell = in_worktree_shell()
            .on_args("git", &["rev-list", "HEAD", "^main", "--count"], ok("3\n"))
            .on("ecc-workflow", ok("Merged successfully\n"))
            .on("pwd", ok("/repo/.claude/worktrees/session-123\n"));
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = session_end_merge("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.contains("merged to main"));
    }

    // PC-002b: skips when not in worktree
    #[test]
    fn skips_when_not_in_worktree() {
        let fs = InMemoryFileSystem::new();
        let shell = not_in_worktree_shell();
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = session_end_merge("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    // PC-002c: rebase conflict preserves worktree
    #[test]
    fn rebase_conflict_preserves_worktree() {
        let fs = InMemoryFileSystem::new();
        let shell = in_worktree_shell()
            .on_args("git", &["rev-list", "HEAD", "^main", "--count"], ok("3\n"))
            .on("ecc-workflow", fail(1, "Rebase conflict detected\n"))
            .on("pwd", ok("/repo/.claude/worktrees/session-123\n"));
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = session_end_merge("{}", &ports);
        assert_eq!(result.exit_code, 0); // warn, not block
        assert!(result.stderr.contains("merge failed"));
        assert!(result.stderr.contains("Worktree preserved"));
    }

    // PC-002d: verify failure preserves worktree
    #[test]
    fn verify_failure_preserves_worktree() {
        let fs = InMemoryFileSystem::new();
        let shell = in_worktree_shell()
            .on_args("git", &["rev-list", "HEAD", "^main", "--count"], ok("3\n"))
            .on("ecc-workflow", fail(2, "verify failed: clippy error\n"))
            .on("pwd", ok("/repo/.claude/worktrees/session-123\n"));
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = session_end_merge("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.contains("merge failed"));
    }

    // PC-002e: lock held warns
    #[test]
    fn lock_held_warns() {
        let fs = InMemoryFileSystem::new();
        let shell = in_worktree_shell()
            .on_args("git", &["rev-list", "HEAD", "^main", "--count"], ok("3\n"))
            .on(
                "ecc-workflow",
                fail(3, "merge lock held by another session\n"),
            )
            .on("pwd", ok("/repo/.claude/worktrees/session-123\n"));
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = session_end_merge("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.contains("merge failed"));
    }

    // PC-001: empty worktree defers to gc (no removal, no "cleaned up")
    #[test]
    fn empty_worktree_defers_to_gc() {
        let fs = InMemoryFileSystem::new();
        // No mock for "git worktree remove" — if code calls it, MockExecutor panics
        let shell = in_worktree_shell().on_args(
            "git",
            &["rev-list", "HEAD", "^main", "--count"],
            ok("0\n"),
        );
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = session_end_merge("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert!(
            !result.stderr.contains("cleaned up"),
            "should not claim 'cleaned up'"
        );
        assert!(
            result.stderr.contains("deferred"),
            "should mention deferred cleanup"
        );
    }

    // PC-002: merge success message has no "cleaned up" claim
    #[test]
    fn merge_success_message_no_cleanup_claim() {
        let fs = InMemoryFileSystem::new();
        let shell = in_worktree_shell()
            .on_args("git", &["rev-list", "HEAD", "^main", "--count"], ok("3\n"))
            .on("ecc-workflow", ok("Merged successfully\n"))
            .on("pwd", ok("/repo/.claude/worktrees/session-123\n"));
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = session_end_merge("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert!(
            result.stderr.contains("merged"),
            "should mention merge success"
        );
        assert!(
            !result.stderr.contains("cleaned up"),
            "should not claim 'cleaned up'"
        );
    }

    // PC-002g: bypass skips merge
    #[test]
    fn bypass_skips_merge() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        // Handler no longer checks ECC_WORKFLOW_BYPASS — bypass is at dispatch level
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = session_end_merge("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    // PC-002h: writes recovery file on failure
    #[test]
    fn writes_recovery_file_on_unexpected_failure() {
        let fs = InMemoryFileSystem::new();
        let shell = in_worktree_shell()
            .on_args("git", &["rev-list", "HEAD", "^main", "--count"], ok("3\n"))
            .on("ecc-workflow", fail(1, "unexpected error\n"))
            .on("pwd", ok("/tmp/worktree\n"));
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = session_end_merge("{}", &ports);
        assert_eq!(result.exit_code, 0);

        let recovery = fs
            .read_to_string(std::path::Path::new("/tmp/worktree/.ecc-merge-recovery"))
            .unwrap();
        assert!(recovery.contains("ECC Merge Recovery"));
        assert!(recovery.contains("ecc-workflow merge"));
    }

    // PC-002i: exit code mapping
    #[test]
    fn maps_exit_codes_correctly() {
        // Exit 0 = success (tested in calls_merge_in_worktree)
        // Any non-zero = preserve + warn (tested in rebase_conflict, verify_failure, lock_held)
        // This test verifies the general non-zero path
        let fs = InMemoryFileSystem::new();
        let shell = in_worktree_shell()
            .on_args("git", &["rev-list", "HEAD", "^main", "--count"], ok("1\n"))
            .on("ecc-workflow", fail(42, "unknown error\n"))
            .on("pwd", ok("/tmp/wt\n"));
        let env = MockEnvironment::new();
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = session_end_merge("{}", &ports);
        assert_eq!(result.exit_code, 0); // warn, not block
        assert!(result.stderr.contains("exit 42"));
    }
}
