//! Install summary display.

use ecc_domain::ansi;
use ecc_ports::terminal::TerminalIO;

use crate::install::InstallSummary;

pub(in crate::install) fn print_summary(
    terminal: &dyn TerminalIO,
    summary: &InstallSummary,
    colored: bool,
    dry_run: bool,
) {
    let prefix = if dry_run { "[DRY RUN] " } else { "" };

    terminal.stdout_write(&format!(
        "\n{prefix}{}\n",
        ansi::bold("Install Summary", colored)
    ));

    if summary.added > 0 {
        terminal.stdout_write(&format!(
            "  {} {}\n",
            ansi::green(&format!("{}", summary.added), colored),
            "added"
        ));
    }
    if summary.updated > 0 {
        terminal.stdout_write(&format!(
            "  {} {}\n",
            ansi::yellow(&format!("{}", summary.updated), colored),
            "updated"
        ));
    }
    if summary.unchanged > 0 {
        terminal.stdout_write(&format!(
            "  {} unchanged\n",
            summary.unchanged
        ));
    }
    if summary.skipped > 0 {
        terminal.stdout_write(&format!(
            "  {} skipped\n",
            summary.skipped
        ));
    }
    if summary.smart_merged > 0 {
        terminal.stdout_write(&format!(
            "  {} smart-merged\n",
            summary.smart_merged
        ));
    }
    if !summary.errors.is_empty() {
        terminal.stdout_write(&format!(
            "  {} {}\n",
            ansi::red(&format!("{}", summary.errors.len()), colored),
            "errors"
        ));
        for err in &summary.errors {
            terminal.stdout_write(&format!("    - {err}\n"));
        }
    }

    if summary.success {
        terminal.stdout_write(&format!(
            "\n{}\n",
            ansi::green("Install complete!", colored)
        ));
    }
}
