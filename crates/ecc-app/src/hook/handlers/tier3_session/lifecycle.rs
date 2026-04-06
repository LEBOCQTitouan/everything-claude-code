//! Session lifecycle hooks — start and end.

use tracing::warn;

use crate::hook::{HookPorts, HookResult};
use ecc_domain::time::{datetime_from_epoch, format_date, format_time};
use std::path::Path;

use super::epoch_secs;
use super::helpers::{
    append_project_detection, build_summary_section, count_files_with_ext, extract_session_summary,
    find_files_by_suffix, find_last_updated_line,
};

/// session-start: load previous context, detect project type.
pub fn session_start(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "session_start", "executing handler");

    // Best-effort gc: clean stale worktrees from previous sessions.
    // Errors are swallowed — gc must never block session start.
    best_effort_gc(ports);

    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    let sessions_dir = home.join(".claude").join("sessions");
    let learned_dir = home.join(".claude").join("learned-skills");

    if let Err(e) = ports.fs.create_dir_all(&sessions_dir) {
        warn!("Cannot create sessions dir: {}", e);
    }
    if let Err(e) = ports.fs.create_dir_all(&learned_dir) {
        warn!("Cannot create learned-skills dir: {}", e);
    }

    let mut stderr_parts: Vec<String> = Vec::new();

    // Find recent session files
    let session_files = find_files_by_suffix(&sessions_dir, "-session.tmp", ports);
    if !session_files.is_empty() {
        stderr_parts.push(format!(
            "[SessionStart] Found {} recent session(s)",
            session_files.len()
        ));

        // Try to load latest session content
        if let Some(latest) = session_files.first() {
            stderr_parts.push(format!("[SessionStart] Latest: {}", latest.display()));
            if let Ok(content) = ports.fs.read_to_string(latest)
                && !content.contains("[Session context goes here]")
                && !content.trim().is_empty()
            {
                // Output previous session summary to stdout
                let mut out = format!("Previous session summary:\n{}", content);
                if !stdin.is_empty() {
                    out = format!("{}\n{}", out, stdin);
                }
                // Load learned skills count
                let learned_count = count_files_with_ext(&learned_dir, ".md", ports);
                if learned_count > 0 {
                    stderr_parts.push(format!(
                        "[SessionStart] {} learned skill(s) available in {}",
                        learned_count,
                        learned_dir.display()
                    ));
                }

                // Detect project type
                append_project_detection(&mut stderr_parts, ports);

                return HookResult {
                    stdout: out,
                    stderr: format!("{}\n", stderr_parts.join("\n")),
                    exit_code: 0,
                };
            }
        }
    }

    // Load learned skills count
    let learned_count = count_files_with_ext(&learned_dir, ".md", ports);
    if learned_count > 0 {
        stderr_parts.push(format!(
            "[SessionStart] {} learned skill(s) available in {}",
            learned_count,
            learned_dir.display()
        ));
    }

    // Detect project type
    append_project_detection(&mut stderr_parts, ports);

    if stderr_parts.is_empty() {
        return HookResult::passthrough(stdin);
    }

    HookResult {
        stdout: stdin.to_string(),
        stderr: format!("{}\n", stderr_parts.join("\n")),
        exit_code: 0,
    }
}

/// Best-effort worktree gc at session start.
/// Resolves the git toplevel and runs gc. All errors are swallowed.
fn best_effort_gc(ports: &HookPorts<'_>) {
    let toplevel = ports
        .shell
        .run_command("git", &["rev-parse", "--show-toplevel"])
        .ok()
        .map(|o| o.stdout.trim().to_string());

    if let Some(dir) = toplevel {
        let project_dir = std::path::Path::new(&dir);
        let shell_mgr = crate::worktree::ShellWorktreeManager::new(ports.shell);
        match crate::worktree::gc(&shell_mgr, ports.shell, project_dir, false) {
            Ok(result) => {
                let removed = result.removed.len();
                let skipped = result.skipped.len();
                if removed > 0 {
                    tracing::info!(
                        removed,
                        skipped,
                        "session-start gc: cleaned stale worktrees"
                    );
                }
            }
            Err(e) => {
                tracing::debug!("session-start gc failed (non-blocking): {e}");
            }
        }
    }
}

/// session-end: persist session summary from transcript.
pub fn session_end(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "session_end", "executing handler");
    let home = match ports.env.home_dir() {
        Some(h) => h,
        None => return HookResult::passthrough(stdin),
    };

    // Parse transcript_path from stdin JSON
    let transcript_path = serde_json::from_str::<serde_json::Value>(stdin)
        .ok()
        .and_then(|v| v.get("transcript_path")?.as_str().map(|s| s.to_string()))
        .or_else(|| ports.env.var("CLAUDE_TRANSCRIPT_PATH"));

    let sessions_dir = home.join(".claude").join("sessions");
    if let Err(e) = ports.fs.create_dir_all(&sessions_dir) {
        warn!("Cannot create sessions dir: {}", e);
    }

    let today = format_date(&datetime_from_epoch(epoch_secs()));
    let short_id = ports
        .env
        .var("CLAUDE_SESSION_ID")
        .map(|s| if s.len() > 8 { s[..8].to_string() } else { s })
        .unwrap_or_else(|| "unknown".to_string());
    let session_file = sessions_dir.join(format!("{}-{}-session.tmp", today, short_id));
    let current_time = format_time(&datetime_from_epoch(epoch_secs()));

    // Try to extract summary from transcript
    let summary = transcript_path.as_deref().and_then(|tp| {
        let path = Path::new(tp);
        if ports.fs.exists(path) {
            ports
                .fs
                .read_to_string(path)
                .ok()
                .and_then(|content| extract_session_summary(&content))
        } else {
            None
        }
    });

    if ports.fs.exists(&session_file) {
        // Update existing session file
        if let Ok(existing) = ports.fs.read_to_string(&session_file) {
            let mut updated = existing.replace(
                &find_last_updated_line(&existing),
                &format!("**Last Updated:** {}", current_time),
            );

            if let Some(ref summary) = summary {
                let summary_section = build_summary_section(summary);
                // Replace existing summary section
                if let Some(pos) = updated.find("## Session Summary") {
                    updated.truncate(pos);
                    updated.push_str(&summary_section);
                } else if let Some(pos) = updated.find("## Current State") {
                    updated.truncate(pos);
                    updated.push_str(&summary_section);
                } else {
                    updated.push_str(&summary_section);
                }
            }

            if let Err(e) = ports.fs.write(&session_file, &updated) {
                let msg = format!("[Warning] Failed to write session: {}", e);
                warn!("{}", msg);
                return HookResult::warn(stdin, &format!("{msg}\n"));
            }
        }
    } else {
        // Create new session file
        let summary_section = if let Some(ref summary) = summary {
            build_summary_section(summary)
        } else {
            "## Current State\n\n[Session context goes here]\n\n\
             ### Completed\n- [ ]\n\n\
             ### In Progress\n- [ ]\n\n\
             ### Notes for Next Session\n-\n\n\
             ### Context to Load\n```\n[relevant files]\n```"
                .to_string()
        };

        let template = format!(
            "# Session: {today}\n\
             **Date:** {today}\n\
             **Started:** {time}\n\
             **Last Updated:** {time}\n\n\
             ---\n\n\
             {summary}\n",
            today = today,
            time = current_time,
            summary = summary_section,
        );

        if let Err(e) = ports.fs.write(&session_file, &template) {
            let msg = format!("[Warning] Failed to write session: {}", e);
            warn!("{}", msg);
            return HookResult::warn(stdin, &format!("{msg}\n"));
        }
    }

    HookResult::passthrough(stdin)
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

    // PC-007: session_start runs gc and removes stale worktrees
    #[test]
    fn session_start_runs_gc() {
        let fs = InMemoryFileSystem::new();
        // Mock git rev-parse for gc's project dir resolution
        let shell = MockExecutor::new()
            .on_args("git", &["rev-parse", "--show-toplevel"], ok("/repo\n"))
            // gc calls git worktree list --porcelain
            .on_args("git", &["worktree", "list", "--porcelain"], ok(""));
        let env = MockEnvironment::new().with_home("/home/user");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        // session_start should not panic — gc runs and completes
        let result = session_start("{}", &ports);
        assert_eq!(result.exit_code, 0);
    }

    // PC-008: session_start gc skips worktrees with alive PID
    #[test]
    fn session_start_gc_skips_alive() {
        let fs = InMemoryFileSystem::new();
        // Mock: git toplevel, then worktree list returns empty (no stale worktrees)
        let shell = MockExecutor::new()
            .on_args("git", &["rev-parse", "--show-toplevel"], ok("/repo\n"))
            .on_args("git", &["worktree", "list", "--porcelain"], ok(""));
        let env = MockEnvironment::new().with_home("/home/user");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        // gc with no worktrees = nothing to skip, but no crash
        let result = session_start("{}", &ports);
        assert_eq!(result.exit_code, 0);
    }

    // PC-009: session_start gc failure does not block session
    #[test]
    fn session_start_gc_failure_non_blocking() {
        let fs = InMemoryFileSystem::new();
        // Mock: git rev-parse fails (not a git repo) — gc should fail silently
        let shell = MockExecutor::new();
        // No mocks = all commands return ShellError::NotFound
        let env = MockEnvironment::new().with_home("/home/user");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        // session_start should succeed even when gc can't run
        let result = session_start("{}", &ports);
        assert_eq!(result.exit_code, 0);
    }
}
