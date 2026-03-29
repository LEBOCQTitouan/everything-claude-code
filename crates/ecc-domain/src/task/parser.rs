//! Parser for tasks.md files.
//!
//! `parse_tasks` reads a tasks.md string and returns a structured `TaskReport`.

use crate::task::entry::{EntryKind, StatusSegment, TaskEntry, TaskReport};
use crate::task::error::TaskError;
use crate::task::status::TaskStatus;

/// Parse a tasks.md string into a [`TaskReport`].
///
/// # Errors
///
/// Returns [`TaskError::ParseError`] with a descriptive message when the input
/// is structurally malformed (e.g. missing required sections, unparseable lines).
pub fn parse_tasks(_content: &str) -> Result<TaskReport, TaskError> {
    Err(TaskError::ParseError {
        line: 0,
        message: "not yet implemented".to_owned(),
    })
}

/// Parse a single status trail string into an ordered `Vec<StatusSegment>`.
///
/// Supports the new `→` separator only (old `|` separator is handled in Wave 3).
fn parse_trail(trail_str: &str) -> Result<Vec<StatusSegment>, TaskError> {
    let _ = (trail_str, TaskStatus::Pending, EntryKind::PostTdd(String::new()));
    Ok(vec![])
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
}
