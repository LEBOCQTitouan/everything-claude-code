//! Parser for tasks.md files.
//!
//! `parse_tasks` reads a tasks.md string and returns a structured `TaskReport`.

use std::str::FromStr;

use crate::spec::pc::PcId;
use crate::task::entry::{EntryKind, StatusSegment, TaskEntry, TaskReport};
use crate::task::error::TaskError;
use crate::task::status::TaskStatus;

/// Active section while scanning tasks.md lines.
#[derive(Debug, PartialEq, Eq)]
enum Section {
    None,
    PassConditions,
    PostTdd,
}

/// Parse a tasks.md string into a [`TaskReport`].
///
/// # Errors
///
/// Returns [`TaskError::ParseError`] with a descriptive message when the input
/// is structurally malformed (e.g. unparseable entry lines).
pub fn parse_tasks(content: &str) -> Result<TaskReport, TaskError> {
    let mut entries: Vec<TaskEntry> = Vec::new();
    let mut section = Section::None;

    for (line_idx, line) in content.lines().enumerate() {
        let line_no = line_idx + 1;
        let trimmed = line.trim();

        // Section headers
        if trimmed == "## Pass Conditions" {
            section = Section::PassConditions;
            continue;
        }
        if trimmed == "## Post-TDD" {
            section = Section::PostTdd;
            continue;
        }
        // Any other `##` header resets the section
        if trimmed.starts_with("## ") {
            section = Section::None;
            continue;
        }

        // Entry lines start with `- [ ]` or `- [x]`
        if !trimmed.starts_with("- [ ]") && !trimmed.starts_with("- [x]") {
            continue;
        }

        let completed = trimmed.starts_with("- [x]");
        // Strip the checkbox prefix (`- [ ] ` or `- [x] ` — 6 chars)
        let rest = &trimmed[6..];

        match section {
            Section::PassConditions => {
                let entry = parse_pc_entry(rest, completed, line_no)?;
                entries.push(entry);
            }
            Section::PostTdd => {
                let entry = parse_post_tdd_entry(rest, completed, line_no)?;
                entries.push(entry);
            }
            Section::None => {
                // Entries outside known sections are ignored
            }
        }
    }

    Ok(build_report(entries))
}

/// Parse one Pass Condition entry line (after stripping the checkbox prefix).
///
/// Format: `PC-NNN: <description> | \`<command>\` | <trail>`
fn parse_pc_entry(rest: &str, completed: bool, line_no: usize) -> Result<TaskEntry, TaskError> {
    // Split on first `: ` to separate PC-NNN from the rest
    let colon_pos = rest.find(": ").ok_or_else(|| TaskError::ParseError {
        line: line_no,
        message: format!("expected 'PC-NNN: <desc>' format, got: {rest}"),
    })?;

    let id_str = &rest[..colon_pos];
    let after_id = &rest[colon_pos + 2..];

    let pc_id = PcId::parse(id_str).map_err(|_| TaskError::ParseError {
        line: line_no,
        message: format!("invalid PC ID: {id_str}"),
    })?;

    // Split on ` | ` to get [description, command, trail]
    let parts: Vec<&str> = after_id.splitn(3, " | ").collect();
    if parts.len() < 2 {
        return Err(TaskError::ParseError {
            line: line_no,
            message: format!("expected '{{desc}} | {{cmd}} | {{trail}}', got: {after_id}"),
        });
    }

    let description = parts[0].trim().to_owned();
    let command = parts[1].trim().trim_matches('`').to_owned();
    let trail_str = if parts.len() >= 3 { parts[2].trim() } else { "" };
    let trail = parse_trail(trail_str, line_no)?;

    Ok(TaskEntry {
        kind: EntryKind::Pc(pc_id),
        description,
        command: Some(command),
        completed,
        trail,
    })
}

/// Parse one Post-TDD entry line (after stripping the checkbox prefix).
///
/// Format: `<label> | <trail>`
fn parse_post_tdd_entry(
    rest: &str,
    completed: bool,
    line_no: usize,
) -> Result<TaskEntry, TaskError> {
    let parts: Vec<&str> = rest.splitn(2, " | ").collect();
    let label = parts[0].trim().to_owned();
    let trail_str = if parts.len() >= 2 { parts[1].trim() } else { "" };
    let trail = parse_trail(trail_str, line_no)?;

    Ok(TaskEntry {
        kind: EntryKind::PostTdd(label.clone()),
        description: label,
        command: None,
        completed,
        trail,
    })
}

/// Parse a status trail string into an ordered `Vec<StatusSegment>`.
///
/// Segments are separated by ` → `. Each segment has the form `status@timestamp`
/// with an optional `[error detail]` suffix for `Failed` segments.
fn parse_trail(trail_str: &str, line_no: usize) -> Result<Vec<StatusSegment>, TaskError> {
    if trail_str.is_empty() {
        return Ok(vec![]);
    }

    let mut segments = Vec::new();
    // Support both new (`→`) and old (`|`) separator formats (AC-001.6).
    let separator = if trail_str.contains(" → ") { " → " } else { " | " };
    for raw in trail_str.split(separator) {
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }
        let seg = parse_trail_segment(raw, line_no)?;
        segments.push(seg);
    }
    Ok(segments)
}

/// Parse one trail segment: `status@timestamp` or `failed@timestamp[error detail]`.
fn parse_trail_segment(raw: &str, line_no: usize) -> Result<StatusSegment, TaskError> {
    let at_pos = raw.find('@').ok_or_else(|| TaskError::ParseError {
        line: line_no,
        message: format!("trail segment missing '@': {raw}"),
    })?;

    let status_str = &raw[..at_pos];
    let rest = &raw[at_pos + 1..];

    let status = TaskStatus::from_str(status_str).map_err(|_| TaskError::ParseError {
        line: line_no,
        message: format!("unknown status '{status_str}' in trail segment: {raw}"),
    })?;

    // Extract optional `[error detail]` from the timestamp remainder
    let (timestamp, error_detail) = if let Some(bracket_pos) = rest.find('[') {
        let ts = rest[..bracket_pos].trim().to_owned();
        let detail_end = rest.rfind(']').unwrap_or(rest.len() - 1);
        let detail = rest[bracket_pos + 1..detail_end].to_owned();
        (ts, Some(detail))
    } else {
        (rest.to_owned(), None)
    };

    Ok(StatusSegment {
        status,
        timestamp,
        error_detail,
    })
}

/// Compute summary counters from a list of entries and construct a `TaskReport`.
fn build_report(entries: Vec<TaskEntry>) -> TaskReport {
    let total = entries.len();
    let mut completed = 0usize;
    let mut pending = 0usize;
    let mut in_progress = 0usize;
    let mut failed = 0usize;

    for entry in &entries {
        let last_status = entry
            .trail
            .last()
            .map(|s| s.status)
            .unwrap_or(TaskStatus::Pending);

        match last_status {
            TaskStatus::Done => completed += 1,
            TaskStatus::Pending => pending += 1,
            TaskStatus::Red | TaskStatus::Green => in_progress += 1,
            TaskStatus::Failed => failed += 1,
        }
    }

    let progress_pct = if total == 0 {
        0.0
    } else {
        completed as f64 * 100.0 / total as f64
    };

    TaskReport {
        entries,
        total,
        completed,
        pending,
        in_progress,
        failed,
        progress_pct,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::entry::EntryKind;
    use crate::task::status::TaskStatus;

    fn well_formed_tasks() -> &'static str {
        r#"# Tasks: Example Feature

## Pass Conditions

- [ ] PC-001: First pass condition | `cargo test --lib -p ecc-domain` | pending@2026-03-29T13:59:20Z
- [x] PC-002: Second pass condition | `cargo test --lib -p ecc-domain second` | pending@2026-03-29T13:59:20Z → done@2026-03-29T14:00:00Z

## Post-TDD

- [ ] E2E tests | pending@2026-03-29T13:59:20Z
"#
    }

    #[test]
    fn well_formed_returns_task_report() {
        let report = parse_tasks(well_formed_tasks()).expect("should parse well-formed tasks.md");
        assert_eq!(report.total, 3, "expected 3 total entries");
    }

    #[test]
    fn well_formed_correct_completed_count() {
        let report = parse_tasks(well_formed_tasks()).expect("should parse well-formed tasks.md");
        assert_eq!(report.completed, 1, "one entry has [x] checkbox");
    }

    #[test]
    fn well_formed_correct_pending_count() {
        let report = parse_tasks(well_formed_tasks()).expect("should parse well-formed tasks.md");
        // PC-001 is pending (trail ends with pending), E2E tests is pending
        assert_eq!(report.pending, 2, "two entries have last status pending");
    }

    #[test]
    fn checkbox_completed_detected() {
        let report = parse_tasks(well_formed_tasks()).expect("should parse well-formed tasks.md");
        let pc2 = report
            .entries
            .iter()
            .find(|e| matches!(&e.kind, EntryKind::Pc(id) if id.number() == 2))
            .expect("PC-002 must be present");
        assert!(pc2.completed, "PC-002 checkbox should be [x]");
    }

    #[test]
    fn checkbox_pending_detected() {
        let report = parse_tasks(well_formed_tasks()).expect("should parse well-formed tasks.md");
        let pc1 = report
            .entries
            .iter()
            .find(|e| matches!(&e.kind, EntryKind::Pc(id) if id.number() == 1))
            .expect("PC-001 must be present");
        assert!(!pc1.completed, "PC-001 checkbox should be [ ]");
    }

    #[test]
    fn pc_entry_has_command() {
        let report = parse_tasks(well_formed_tasks()).expect("should parse well-formed tasks.md");
        let pc1 = report
            .entries
            .iter()
            .find(|e| matches!(&e.kind, EntryKind::Pc(id) if id.number() == 1))
            .expect("PC-001 must be present");
        assert_eq!(
            pc1.command.as_deref(),
            Some("cargo test --lib -p ecc-domain"),
            "command should be stripped of backticks"
        );
    }

    #[test]
    fn pc_entry_description_extracted() {
        let report = parse_tasks(well_formed_tasks()).expect("should parse well-formed tasks.md");
        let pc1 = report
            .entries
            .iter()
            .find(|e| matches!(&e.kind, EntryKind::Pc(id) if id.number() == 1))
            .expect("PC-001 must be present");
        assert_eq!(pc1.description, "First pass condition");
    }

    #[test]
    fn progress_pct_one_of_two_pcs_done() {
        // 2 PC entries + 1 Post-TDD; 1 done => 1/3 = 33.3%
        let report = parse_tasks(well_formed_tasks()).expect("should parse well-formed tasks.md");
        let expected = 100.0 / 3.0;
        assert!(
            (report.progress_pct - expected).abs() < 0.01,
            "progress_pct={} expected~={expected}",
            report.progress_pct
        );
    }

    pub mod multi_segment {
        use super::*;

        fn multi_segment_input() -> &'static str {
            r#"# Tasks: Multi-segment test

## Pass Conditions

- [x] PC-001: Full TDD cycle | `cargo test` | pending@2026-03-29T10:00:00Z → red@2026-03-29T10:05:00Z → green@2026-03-29T10:10:00Z → done@2026-03-29T10:15:00Z
"#
        }

        #[test]
        fn extracts_all_four_segments() {
            let report =
                parse_tasks(multi_segment_input()).expect("should parse multi-segment tasks.md");
            let pc1 = report
                .entries
                .iter()
                .find(|e| matches!(&e.kind, EntryKind::Pc(id) if id.number() == 1))
                .expect("PC-001 must be present");
            assert_eq!(
                pc1.trail.len(),
                4,
                "expected 4 trail segments, got {}",
                pc1.trail.len()
            );
        }

        #[test]
        fn segments_in_order() {
            let report =
                parse_tasks(multi_segment_input()).expect("should parse multi-segment tasks.md");
            let pc1 = &report.entries[0];
            let statuses: Vec<TaskStatus> = pc1.trail.iter().map(|s| s.status).collect();
            assert_eq!(
                statuses,
                vec![
                    TaskStatus::Pending,
                    TaskStatus::Red,
                    TaskStatus::Green,
                    TaskStatus::Done
                ],
                "trail segments should be in order"
            );
        }

        #[test]
        fn timestamps_preserved() {
            let report =
                parse_tasks(multi_segment_input()).expect("should parse multi-segment tasks.md");
            let pc1 = &report.entries[0];
            assert_eq!(pc1.trail[0].timestamp, "2026-03-29T10:00:00Z");
            assert_eq!(pc1.trail[3].timestamp, "2026-03-29T10:15:00Z");
        }
    }

    pub mod post_tdd {
        use super::*;

        fn post_tdd_input() -> &'static str {
            r#"# Tasks: Post-TDD test

## Pass Conditions

- [x] PC-001: A PC | `cargo test` | pending@2026-03-29T10:00:00Z → done@2026-03-29T10:05:00Z

## Post-TDD

- [ ] E2E tests | pending@2026-03-29T10:00:00Z
- [ ] Code review | pending@2026-03-29T10:00:00Z
- [x] Doc updates | pending@2026-03-29T10:00:00Z → done@2026-03-29T10:30:00Z
"#
        }

        #[test]
        fn post_tdd_entries_parsed() {
            let report = parse_tasks(post_tdd_input()).expect("should parse post-TDD tasks.md");
            let post_tdd_count = report
                .entries
                .iter()
                .filter(|e| matches!(&e.kind, EntryKind::PostTdd(_)))
                .count();
            assert_eq!(post_tdd_count, 3, "expected 3 Post-TDD entries");
        }

        #[test]
        fn post_tdd_kind_is_distinct() {
            let report = parse_tasks(post_tdd_input()).expect("should parse post-TDD tasks.md");
            let e2e = report
                .entries
                .iter()
                .find(|e| matches!(&e.kind, EntryKind::PostTdd(label) if label == "E2E tests"))
                .expect("E2E tests entry must be present");
            assert!(
                matches!(&e2e.kind, EntryKind::PostTdd(_)),
                "E2E tests must be EntryKind::PostTdd"
            );
        }

        #[test]
        fn post_tdd_has_no_command() {
            let report = parse_tasks(post_tdd_input()).expect("should parse post-TDD tasks.md");
            let e2e = report
                .entries
                .iter()
                .find(|e| matches!(&e.kind, EntryKind::PostTdd(label) if label == "E2E tests"))
                .expect("E2E tests entry must be present");
            assert!(e2e.command.is_none(), "Post-TDD entries must have no command");
        }

        #[test]
        fn post_tdd_completed_detected() {
            let report = parse_tasks(post_tdd_input()).expect("should parse post-TDD tasks.md");
            let doc = report
                .entries
                .iter()
                .find(|e| matches!(&e.kind, EntryKind::PostTdd(label) if label == "Doc updates"))
                .expect("Doc updates entry must be present");
            assert!(doc.completed, "Doc updates should be [x]");
        }
    }

    pub mod old_format {
        use super::*;

        fn old_format_input() -> &'static str {
            r#"# Tasks: Old format test

## Pass Conditions

- [ ] PC-001: Full TDD cycle | `cargo test` | pending@2026-03-29T10:00:00Z | red@2026-03-29T10:05:00Z | done@2026-03-29T10:15:00Z
"#
        }

        #[test]
        fn old_pipe_separator_reads_trail_segments() {
            let report =
                parse_tasks(old_format_input()).expect("should parse old-format tasks.md");
            let pc1 = report
                .entries
                .iter()
                .find(|e| matches!(&e.kind, EntryKind::Pc(id) if id.number() == 1))
                .expect("PC-001 must be present");
            assert_eq!(
                pc1.trail.len(),
                3,
                "expected 3 trail segments from old pipe-separated format, got {}",
                pc1.trail.len()
            );
        }

        #[test]
        fn old_pipe_separator_trail_statuses_in_order() {
            let report =
                parse_tasks(old_format_input()).expect("should parse old-format tasks.md");
            let pc1 = &report.entries[0];
            let statuses: Vec<TaskStatus> = pc1.trail.iter().map(|s| s.status).collect();
            assert_eq!(
                statuses,
                vec![TaskStatus::Pending, TaskStatus::Red, TaskStatus::Done],
                "trail segments should be in order: pending, red, done"
            );
        }

        #[test]
        fn old_pipe_separator_post_tdd_trail() {
            let input = r#"# Tasks: Old format PostTDD

## Post-TDD

- [ ] E2E tests | pending@2026-03-29T10:00:00Z | red@2026-03-29T10:05:00Z
"#;
            let report = parse_tasks(input).expect("should parse old-format PostTDD");
            let e2e = report
                .entries
                .iter()
                .find(|e| matches!(&e.kind, EntryKind::PostTdd(label) if label == "E2E tests"))
                .expect("E2E tests entry must be present");
            assert_eq!(
                e2e.trail.len(),
                2,
                "expected 2 trail segments in old PostTDD format, got {}",
                e2e.trail.len()
            );
        }
    }

    pub mod empty {
        use super::*;

        #[test]
        fn header_only_returns_zero_entries() {
            let input = r#"# Tasks: Some Feature

## Pass Conditions

## Post-TDD
"#;
            let report = parse_tasks(input).expect("header-only tasks.md should parse without error");
            assert_eq!(report.total, 0, "expected zero total entries");
            assert_eq!(report.completed, 0, "expected zero completed");
            assert_eq!(report.pending, 0, "expected zero pending");
            assert_eq!(report.in_progress, 0, "expected zero in_progress");
            assert_eq!(report.failed, 0, "expected zero failed");
            assert!(report.entries.is_empty(), "expected empty entries vec");
        }
    }

    pub mod malformed {
        use super::*;

        #[test]
        fn malformed_pc_id_returns_error() {
            let input = r#"# Tasks: Bad

## Pass Conditions

- [ ] INVALID-001: desc | `cmd` | pending@2026-03-29T10:00:00Z
"#;
            let result = parse_tasks(input);
            assert!(
                result.is_err(),
                "invalid PC ID should produce an error, got: {result:?}"
            );
        }

        #[test]
        fn malformed_trail_segment_returns_error() {
            let input = r#"# Tasks: Bad trail

## Pass Conditions

- [ ] PC-001: desc | `cmd` | BADTRAIL
"#;
            let result = parse_tasks(input);
            assert!(
                result.is_err(),
                "malformed trail segment should produce an error"
            );
        }

        #[test]
        fn malformed_error_is_descriptive() {
            let input = r#"# Tasks: Bad

## Pass Conditions

- [ ] INVALID-001: desc | `cmd` | pending@2026-03-29T10:00:00Z
"#;
            let err = parse_tasks(input).unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.contains("INVALID-001") || msg.contains("invalid") || msg.contains("PC"),
                "error message should reference the problematic token, got: {msg}"
            );
        }

        #[test]
        fn parser_does_not_panic_on_garbage() {
            let inputs = [
                "",
                "random garbage",
                "- [ ] no section",
                "## Pass Conditions\n- [ ] no pipe separator here",
            ];
            for input in inputs {
                // Should either succeed (no entries) or return an Err — must not panic
                let _ = parse_tasks(input);
            }
        }

        pub mod failed_detail {
            use super::*;

            #[test]
            fn failed_segment_with_error_detail() {
                let input = r#"# Tasks: Failed detail

## Pass Conditions

- [ ] PC-001: desc | `cmd` | pending@2026-03-29T10:00:00Z → failed@2026-03-29T10:05:00Z[compilation error]
"#;
                let report = parse_tasks(input).expect("should parse failed detail input");
                let pc1 = &report.entries[0];
                let failed_seg = pc1
                    .trail
                    .iter()
                    .find(|s| s.status == TaskStatus::Failed)
                    .expect("Failed segment must be present");
                assert_eq!(
                    failed_seg.error_detail.as_deref(),
                    Some("compilation error"),
                    "error_detail should be extracted from brackets"
                );
            }
        }
    }
}
