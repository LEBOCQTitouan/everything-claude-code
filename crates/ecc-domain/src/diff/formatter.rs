use crate::ansi;
use crate::diff::lcs::{DiffLine, DiffType};

/// Minimum terminal width to use side-by-side format.
const MIN_SIDE_BY_SIDE_WIDTH: u16 = 60;

/// Generate a colored unified diff string from diff lines.
pub fn generate_diff(diff: &[DiffLine], colors_enabled: bool) -> String {
    let mut lines = Vec::new();

    for d in diff {
        match d.kind {
            DiffType::Same => {
                lines.push(ansi::dim(&format!("  {}", d.content), colors_enabled));
            }
            DiffType::Added => {
                lines.push(ansi::green(&format!("+ {}", d.content), colors_enabled));
            }
            DiffType::Removed => {
                lines.push(ansi::red(&format!("- {}", d.content), colors_enabled));
            }
        }
    }

    lines.join("\n")
}

/// Group consecutive removed/added lines into paired chunks for side-by-side display.
struct DiffChunk {
    removed: Vec<String>,
    added: Vec<String>,
    same: Vec<String>,
}

fn group_diff_chunks(diff: &[DiffLine]) -> Vec<DiffChunk> {
    let mut chunks = Vec::new();
    let mut removed = Vec::new();
    let mut added = Vec::new();

    for d in diff {
        match d.kind {
            DiffType::Removed => {
                if !added.is_empty() {
                    // Flush pending removed+added as a chunk
                    chunks.push(DiffChunk {
                        removed: std::mem::take(&mut removed),
                        added: std::mem::take(&mut added),
                        same: Vec::new(),
                    });
                }
                removed.push(d.content.clone());
            }
            DiffType::Added => {
                added.push(d.content.clone());
            }
            DiffType::Same => {
                // Flush any pending changes
                if !removed.is_empty() || !added.is_empty() {
                    chunks.push(DiffChunk {
                        removed: std::mem::take(&mut removed),
                        added: std::mem::take(&mut added),
                        same: Vec::new(),
                    });
                }
                chunks.push(DiffChunk {
                    removed: Vec::new(),
                    added: Vec::new(),
                    same: vec![d.content.clone()],
                });
            }
        }
    }

    // Flush remaining
    if !removed.is_empty() || !added.is_empty() {
        chunks.push(DiffChunk {
            removed,
            added,
            same: Vec::new(),
        });
    }

    chunks
}

/// Format a side-by-side diff for terminal display.
/// Falls back to unified format if terminal is too narrow.
pub fn format_side_by_side_diff(
    diff: &[DiffLine],
    terminal_width: Option<u16>,
    colors_enabled: bool,
) -> String {
    let width = terminal_width.unwrap_or(80);

    if width < MIN_SIDE_BY_SIDE_WIDTH {
        return generate_diff(diff, colors_enabled);
    }

    let gutter = 6; // line number gutter width
    let separator = 3; // " │ "
    let col_width = ((width as usize) - separator - 2 * gutter) / 2;
    let chunks = group_diff_chunks(diff);
    let mut output = Vec::new();
    let mut left_num = 1usize;
    let mut right_num = 1usize;

    for chunk in &chunks {
        if !chunk.same.is_empty() {
            for line in &chunk.same {
                let left = truncate(line, col_width);
                let right = truncate(line, col_width);
                output.push(format!(
                    "{:>gutter$} {:<col_width$} │ {:>gutter$} {:<col_width$}",
                    left_num,
                    ansi::dim(&left, colors_enabled),
                    right_num,
                    ansi::dim(&right, colors_enabled),
                    gutter = gutter,
                    col_width = col_width,
                ));
                left_num += 1;
                right_num += 1;
            }
        } else {
            let max_lines = chunk.removed.len().max(chunk.added.len());
            for i in 0..max_lines {
                let left_part = if let Some(line) = chunk.removed.get(i) {
                    let num = format!("{:>gutter$}", left_num, gutter = gutter);
                    left_num += 1;
                    format!(
                        "{} {}",
                        num,
                        ansi::red(&truncate(line, col_width), colors_enabled)
                    )
                } else {
                    format!(
                        "{:>gutter$} {:col_width$}",
                        "",
                        "",
                        gutter = gutter,
                        col_width = col_width
                    )
                };

                let right_part = if let Some(line) = chunk.added.get(i) {
                    let num = format!("{:>gutter$}", right_num, gutter = gutter);
                    right_num += 1;
                    format!(
                        "{} {}",
                        num,
                        ansi::green(&truncate(line, col_width), colors_enabled)
                    )
                } else {
                    String::new()
                };

                output.push(format!("{left_part} │ {right_part}"));
            }
        }
    }

    output.join("\n")
}

/// Count additions and removals in a diff.
pub fn diff_stats(diff: &[DiffLine]) -> (usize, usize) {
    let added = diff.iter().filter(|d| d.kind == DiffType::Added).count();
    let removed = diff.iter().filter(|d| d.kind == DiffType::Removed).count();
    (added, removed)
}

fn truncate(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else if max_width > 1 {
        format!("{}…", &s[..max_width - 1])
    } else {
        "…".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_diff() -> Vec<DiffLine> {
        vec![
            DiffLine {
                kind: DiffType::Same,
                content: "unchanged".into(),
            },
            DiffLine {
                kind: DiffType::Removed,
                content: "old line".into(),
            },
            DiffLine {
                kind: DiffType::Added,
                content: "new line".into(),
            },
            DiffLine {
                kind: DiffType::Same,
                content: "also unchanged".into(),
            },
        ]
    }

    #[test]
    fn generate_diff_no_colors() {
        let output = generate_diff(&make_diff(), false);
        assert!(output.contains("  unchanged"));
        assert!(output.contains("- old line"));
        assert!(output.contains("+ new line"));
    }

    #[test]
    fn generate_diff_with_colors() {
        let output = generate_diff(&make_diff(), true);
        assert!(output.contains("\x1b["));
    }

    #[test]
    fn diff_stats_counts_correctly() {
        let (added, removed) = diff_stats(&make_diff());
        assert_eq!(added, 1);
        assert_eq!(removed, 1);
    }

    #[test]
    fn truncate_short_string() {
        assert_eq!(truncate("hi", 10), "hi");
    }

    #[test]
    fn truncate_long_string() {
        assert_eq!(truncate("hello world", 6), "hello…");
    }

    #[test]
    fn side_by_side_fallback_narrow() {
        let output = format_side_by_side_diff(&make_diff(), Some(30), false);
        // Falls back to unified when too narrow
        assert!(output.contains("- old line"));
    }
}
