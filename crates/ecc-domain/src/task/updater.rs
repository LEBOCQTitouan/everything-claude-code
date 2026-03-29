//! Updater for tasks.md content.
//!
//! `apply_update` finds the matching entry line, validates the FSM transition,
//! appends a new trail segment, and optionally flips the checkbox to `[x]`.

use crate::task::error::TaskError;
use crate::task::status::TaskStatus;

/// Apply a status update to a tasks.md entry in-place (string surgery).
///
/// Returns a new `String` with the updated content.
///
/// # Errors
///
/// - [`TaskError::EntryNotFound`] when no line matches `entry_id`
/// - [`TaskError::InvalidTransition`] when the FSM rejects the transition
/// - [`TaskError::SameState`] when the entry is already in `new_status`
pub fn apply_update(
    content: &str,
    entry_id: &str,
    new_status: TaskStatus,
    timestamp: &str,
) -> Result<String, TaskError> {
    // Stub: always returns unchanged content (RED — tests will fail)
    let _ = (entry_id, new_status, timestamp);
    Ok(content.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::status::TaskStatus;

    fn fixture_pending() -> &'static str {
        r#"# Tasks: Test Feature

## Pass Conditions

- [ ] PC-001: Test description | `cargo test` | pending@2026-03-29T14:00:00Z

## Post-TDD

- [ ] E2E tests | pending@2026-03-29T14:00:00Z
"#
    }

    #[test]
    fn append_trail() {
        let updated = apply_update(
            fixture_pending(),
            "PC-001",
            TaskStatus::Red,
            "2026-03-29T14:01:00Z",
        )
        .expect("apply_update should succeed for pending -> red");

        assert!(
            updated.contains("pending@2026-03-29T14:00:00Z → red@2026-03-29T14:01:00Z"),
            "trail segment should be appended with ' → ' separator, got:\n{updated}"
        );
    }
}
