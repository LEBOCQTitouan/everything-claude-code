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
