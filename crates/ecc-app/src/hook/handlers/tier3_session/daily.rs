//! Daily memory file handler — appends session summary to daily file (BL-047).

use std::path::PathBuf;
use tracing::warn;

use crate::hook::{HookPorts, HookResult};
use ecc_domain::time::{datetime_from_epoch, format_date, format_time};

use super::epoch_secs;

/// Resolve the daily memory directory for the current project.
///
/// Path: `~/.claude/projects/<project-hash>/memory/daily/`
/// where `<project-hash>` is the absolute project path with `/` replaced by `-`.
fn resolve_daily_dir(ports: &HookPorts<'_>) -> Option<PathBuf> {
    let project_dir = ports.env.var("CLAUDE_PROJECT_DIR")?;
    let home = ports.env.home_dir()?;

    // Build project hash: strip leading / and replace remaining / with -
    let abs_path = if project_dir.starts_with('/') {
        project_dir.clone()
    } else {
        // Fallback — treat as-is
        project_dir.clone()
    };
    let project_hash = abs_path.trim_start_matches('/').replace('/', "-");

    let daily_dir = home
        .join(".claude")
        .join("projects")
        .join(project_hash)
        .join("memory")
        .join("daily");

    Some(daily_dir)
}

/// stop:daily-summary — append session summary to daily memory file.
pub fn daily_summary(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "daily_summary", "executing handler");
    let daily_dir = match resolve_daily_dir(ports) {
        Some(d) => d,
        None => return HookResult::passthrough(stdin),
    };

    if let Err(e) = ports.fs.create_dir_all(&daily_dir) {
        warn!("Cannot create daily dir: {}", e);
        return HookResult::passthrough(stdin);
    }

    let now = datetime_from_epoch(epoch_secs());
    let today = format_date(&now);
    let time = format_time(&now);

    let daily_file = daily_dir.join(format!("{}.md", today));

    let content = if ports.fs.exists(&daily_file) {
        match ports.fs.read_to_string(&daily_file) {
            Ok(c) => c,
            Err(e) => {
                warn!("Cannot read daily file: {}", e);
                return HookResult::passthrough(stdin);
            }
        }
    } else {
        // Create new daily file with sections
        format!(
            "# Daily: {today}\n\n## Activity\n\n## Insights\n",
            today = today,
        )
    };

    // Append session entry under ## Activity
    let entry = format!("- [{}] **session-end** Session complete", time);
    let updated = insert_after_heading(&content, "## Activity", &entry);

    if let Err(e) = ports.fs.write(&daily_file, &updated) {
        warn!("Cannot write daily file: {}", e);
        return HookResult::passthrough(stdin);
    }

    HookResult::passthrough(stdin)
}

/// Insert a line after the first blank line following a heading.
fn insert_after_heading(content: &str, heading: &str, entry: &str) -> String {
    let mut lines: Vec<&str> = content.lines().collect();
    let mut insert_idx = None;

    for (i, line) in lines.iter().enumerate() {
        if line.trim() == heading {
            // Insert after the heading + one blank line
            let idx = if i + 1 < lines.len() && lines[i + 1].trim().is_empty() {
                i + 2
            } else {
                i + 1
            };
            insert_idx = Some(idx);
            break;
        }
    }

    match insert_idx {
        Some(idx) => {
            lines.insert(idx, entry);
            let mut result = lines.join("\n");
            // Preserve trailing newline if original had one
            if content.ends_with('\n') && !result.ends_with('\n') {
                result.push('\n');
            }
            result
        }
        None => {
            // Heading not found — append at end
            format!("{}\n{}\n", content.trim_end(), entry)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookPorts;
    use ecc_ports::fs::FileSystem;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};
    #[test]
    fn daily_summary_creates_file_when_missing() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_var("CLAUDE_PROJECT_DIR", "/home/user/myproject")
            .with_home("/home/user");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let result = daily_summary("{}", &ports);
        assert_eq!(result.exit_code, 0);

        // Verify a daily file was created
        let dir =
            std::path::Path::new("/home/user/.claude/projects/home-user-myproject/memory/daily");
        let entries = fs.read_dir(dir).expect("daily dir should exist");
        assert_eq!(entries.len(), 1);

        let content = fs.read_to_string(&entries[0]).expect("should read file");
        assert!(content.contains("## Activity"));
        assert!(content.contains("## Insights"));
        assert!(content.contains("**session-end** Session complete"));
    }

    #[test]
    fn daily_summary_appends_to_existing() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new()
            .with_var("CLAUDE_PROJECT_DIR", "/home/user/myproject")
            .with_home("/home/user");
        let term = BufferedTerminal::new();

        // Pre-create daily dir and file
        let now = datetime_from_epoch(epoch_secs());
        let today = format_date(&now);
        let dir_path =
            std::path::Path::new("/home/user/.claude/projects/home-user-myproject/memory/daily");
        fs.create_dir_all(dir_path).unwrap();
        let file_path = dir_path.join(format!("{}.md", today));
        let existing = format!(
            "# Daily: {}\n\n## Activity\n\n- [09:00] **plan** Feature X — dev\n\n## Insights\n",
            today,
        );
        fs.write(&file_path, &existing).unwrap();

        let ports = HookPorts::test_default(&fs, &shell, &env, &term);
        let result = daily_summary("{}", &ports);
        assert_eq!(result.exit_code, 0);

        let content = fs.read_to_string(&file_path).expect("should read file");
        assert!(content.contains("**session-end** Session complete"));
        assert!(content.contains("**plan** Feature X"));
    }

    #[test]
    fn daily_summary_no_project_dir_passthrough() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let env = MockEnvironment::new().with_home("/home/user");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let result = daily_summary("hello", &ports);
        assert_eq!(result.stdout, "hello");
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    /// PC-025: when all changed files are noise paths, `daily_summary` returns passthrough
    /// without appending a daily entry (noise-only session skip).
    #[test]
    fn skips_append_on_noise_only_session() {
        use ecc_ports::shell::CommandOutput;

        let fs = InMemoryFileSystem::new();
        // git diff returns only noise paths
        let shell = MockExecutor::new().on_args(
            "git",
            &["diff", "--name-only", "HEAD"],
            CommandOutput {
                stdout: ".claude/workflow/state.json\ndocs/specs/2026-04-18-foo/spec.md\n"
                    .to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env = MockEnvironment::new()
            .with_var("CLAUDE_PROJECT_DIR", "/home/user/myproject")
            .with_home("/home/user");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let result = daily_summary("{}", &ports);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "{}");

        // No daily file should have been written — session was noise-only
        let dir = std::path::Path::new(
            "/home/user/.claude/projects/home-user-myproject/memory/daily",
        );
        let no_file_written = if fs.exists(dir) {
            fs.read_dir(dir)
                .map(|entries| entries.is_empty())
                .unwrap_or(true)
        } else {
            true
        };
        assert!(
            no_file_written,
            "daily file must NOT be written when all changed files are noise paths"
        );
    }
}
