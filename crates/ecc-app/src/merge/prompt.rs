use crate::config::merge as config_merge;
use crate::smart_merge;
use ecc_domain::config::merge::{FileToReview, MergeReport};
use ecc_domain::diff::{formatter, lcs};
use ecc_ports::env::Environment;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::terminal::TerminalIO;

use super::ReviewChoice;

/// Display a diff and prompt the user for a review choice.
///
/// Returns `(choice, apply_to_all)`. When `apply_to_all` is true, the choice
/// should be applied to all remaining files.
pub fn prompt_file_review(
    terminal: &dyn TerminalIO,
    fs: &dyn FileSystem,
    env: &dyn Environment,
    file: &FileToReview,
    progress: &str,
) -> Result<(ReviewChoice, bool), String> {
    let colored = env.var("NO_COLOR").is_none();

    if file.is_new {
        terminal.stdout_write(&format!(
            "\n{progress} NEW: {}\n",
            file.filename
        ));
        // Show preview of incoming file
        if let Ok(content) = fs.read_to_string(&file.src_path) {
            let preview: Vec<&str> = content.lines().take(10).collect();
            terminal.stdout_write(&format!("  Preview: {} lines\n", content.lines().count()));
            for line in &preview {
                terminal.stdout_write(&format!("    {line}\n"));
            }
            if content.lines().count() > 10 {
                terminal.stdout_write("    ...\n");
            }
        }
    } else {
        terminal.stdout_write(&format!(
            "\n{progress} CHANGED: {}\n",
            file.filename
        ));
        // Show diff
        let existing = fs.read_to_string(&file.dest_path).unwrap_or_default();
        let incoming = fs.read_to_string(&file.src_path).unwrap_or_default();
        let old_lines: Vec<&str> = existing.lines().collect();
        let new_lines: Vec<&str> = incoming.lines().collect();
        let diff = lcs::compute_line_diff(&old_lines, &new_lines);
        let diff_output = formatter::generate_diff(&diff, colored);
        terminal.stdout_write(&format!("{diff_output}\n"));
    }

    terminal.stdout_write("  [a]ccept | [k]eep | [s]mart-merge | [A]ccept all | [K]eep all: ");

    let input = terminal
        .prompt("")
        .map_err(|_| "Prompt cancelled".to_string())?;

    match input.trim() {
        "a" => Ok((ReviewChoice::Accept, false)),
        "k" => Ok((ReviewChoice::Keep, false)),
        "s" => Ok((ReviewChoice::SmartMerge, false)),
        "A" => Ok((ReviewChoice::Accept, true)),
        "K" => Ok((ReviewChoice::Keep, true)),
        _ => Ok((ReviewChoice::Accept, false)), // Default to accept
    }
}

/// Apply a review choice for a single file, updating the merge report.
///
/// Returns updated options (may have `apply_all` set if user chose "all").
pub fn apply_review_choice(
    fs: &dyn FileSystem,
    shell: &dyn ShellExecutor,
    choice: ReviewChoice,
    file: &FileToReview,
    dry_run: bool,
    report: &mut MergeReport,
) {
    match choice {
        ReviewChoice::Accept => {
            match config_merge::apply_accept(fs, &file.src_path, &file.dest_path, dry_run) {
                Ok(()) => {
                    if file.is_new {
                        report.added.push(file.filename.clone());
                    } else {
                        report.updated.push(file.filename.clone());
                    }
                }
                Err(e) => {
                    report.errors.push(format!("{}: {e}", file.filename));
                }
            }
        }
        ReviewChoice::Keep => {
            report.skipped.push(file.filename.clone());
        }
        ReviewChoice::SmartMerge => {
            let existing = fs
                .read_to_string(&file.dest_path)
                .unwrap_or_default();
            let incoming = fs
                .read_to_string(&file.src_path)
                .unwrap_or_default();

            let result = smart_merge::smart_merge(shell, &existing, &incoming, &file.filename);
            if result.success {
                if let Some(merged) = &result.merged {
                    if !dry_run
                        && let Err(e) = fs.write(&file.dest_path, merged)
                    {
                        report
                            .errors
                            .push(format!("{}: write error: {e}", file.filename));
                        return;
                    }
                    report.smart_merged.push(file.filename.clone());
                }
            } else {
                // Smart merge failed — fall back to accept
                let err = result.error.unwrap_or_else(|| "unknown error".to_string());
                report
                    .errors
                    .push(format!("{}: smart merge failed: {err}", file.filename));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_domain::config::merge::{self, FileToReview};
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};

    fn no_color_env() -> MockEnvironment {
        MockEnvironment::new().with_var("NO_COLOR", "1")
    }

    fn changed_file() -> FileToReview {
        FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        }
    }

    // --- prompt_file_review ---

    #[test]
    fn prompt_unknown_input_defaults_to_accept() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agent.md", "new content")
            .with_file("/dest/agent.md", "old content");
        let terminal = BufferedTerminal::new().with_input("x");
        let env = no_color_env();

        let (choice, apply_all) =
            prompt_file_review(&terminal, &fs, &env, &changed_file(), "[1/1]").unwrap();

        assert_eq!(choice, ReviewChoice::Accept);
        assert!(!apply_all);
    }

    #[test]
    fn prompt_shows_diff_for_changed_file() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agent.md", "new line")
            .with_file("/dest/agent.md", "old line");
        let terminal = BufferedTerminal::new().with_input("a");
        let env = no_color_env();

        prompt_file_review(&terminal, &fs, &env, &changed_file(), "[2/5]").unwrap();

        let output = terminal.stdout_output().join("");
        assert!(output.contains("CHANGED: agent.md"));
        assert!(output.contains("[2/5]"));
    }

    #[test]
    fn prompt_new_file_shows_preview_truncated() {
        // File with more than 10 lines — preview should be truncated with "..."
        let many_lines: String = (1..=15).map(|i| format!("Line {i}\n")).collect();
        let fs = InMemoryFileSystem::new().with_file("/src/new.md", &many_lines);
        let terminal = BufferedTerminal::new().with_input("a");
        let env = no_color_env();
        let file = FileToReview {
            filename: "new.md".into(),
            src_path: "/src/new.md".into(),
            dest_path: "/dest/new.md".into(),
            is_new: true,
        };

        prompt_file_review(&terminal, &fs, &env, &file, "[1/1]").unwrap();

        let output = terminal.stdout_output().join("");
        assert!(output.contains("NEW: new.md"));
        assert!(output.contains("15 lines"));
        assert!(output.contains("..."));
    }

    #[test]
    fn prompt_new_file_short_no_truncation() {
        let fs = InMemoryFileSystem::new().with_file("/src/new.md", "# Header\nBody line");
        let terminal = BufferedTerminal::new().with_input("k");
        let env = no_color_env();
        let file = FileToReview {
            filename: "new.md".into(),
            src_path: "/src/new.md".into(),
            dest_path: "/dest/new.md".into(),
            is_new: true,
        };

        prompt_file_review(&terminal, &fs, &env, &file, "[1/1]").unwrap();

        let output = terminal.stdout_output().join("");
        assert!(output.contains("NEW: new.md"));
        assert!(!output.contains("..."));
    }

    // --- apply_review_choice ---

    #[test]
    fn apply_accept_existing_file_records_updated() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agent.md", "updated content")
            .with_file("/dest/agent.md", "old content");
        let shell = MockExecutor::new();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        };
        let mut report = merge::empty_report();

        apply_review_choice(&fs, &shell, ReviewChoice::Accept, &file, false, &mut report);

        assert!(report.added.is_empty());
        assert_eq!(report.updated, vec!["agent.md"]);
    }

    #[test]
    fn apply_smart_merge_dry_run_does_not_write() {
        let fs = InMemoryFileSystem::new()
            .with_file("/dest/agent.md", "old")
            .with_file("/src/agent.md", "new");
        let shell = MockExecutor::new()
            .with_command("claude")
            .on("claude", CommandOutput {
                stdout: "merged".to_string(),
                stderr: String::new(),
                exit_code: 0,
            });
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        };
        let mut report = merge::empty_report();

        apply_review_choice(&fs, &shell, ReviewChoice::SmartMerge, &file, true, &mut report);

        assert_eq!(report.smart_merged, vec!["agent.md"]);
        // dry_run: dest file must remain unchanged
        assert_eq!(fs.read_to_string(std::path::Path::new("/dest/agent.md")).unwrap(), "old");
    }
}
