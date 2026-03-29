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
    let is_post_tdd = !entry_id.starts_with("PC-");
    let mut lines: Vec<String> = content.lines().map(str::to_owned).collect();
    let mut in_post_tdd = false;

    for line in &mut lines {
        let trimmed = line.trim();
        if trimmed == "## Post-TDD" {
            in_post_tdd = true;
            continue;
        }
        if trimmed.starts_with("## ") {
            in_post_tdd = false;
            continue;
        }

        if !trimmed.starts_with("- [ ]") && !trimmed.starts_with("- [x]") {
            continue;
        }

        // Check if this line matches entry_id
        let rest = &trimmed[6..]; // skip "- [ ] " or "- [x] "
        let matches = if is_post_tdd {
            // PostTdd: label is the text before " | "
            rest.starts_with(entry_id)
                && rest[entry_id.len()..].starts_with(" |")
        } else {
            // PC entry: "PC-NNN: ..."
            rest.starts_with(&format!("{entry_id}: "))
        };

        if !matches {
            continue;
        }

        // Entry found — determine is_post_tdd from actual section context
        let entry_is_post_tdd = in_post_tdd || is_post_tdd;

        // Extract current status from last trail segment
        let current_status = extract_last_status(rest)?;

        // Validate transition
        TaskStatus::can_transition(current_status, new_status, entry_is_post_tdd)?;

        // Append ` → new_status@timestamp` to the line
        let new_segment = format!(" → {new_status}@{timestamp}");
        let updated_line = format!("{line}{new_segment}");
        *line = updated_line;
        // Done — reconstruct content (preserve trailing newline)
        let mut result = lines.join("\n");
        if content.ends_with('\n') {
            result.push('\n');
        }
        return Ok(result);
    }

    // Entry not found — return content unchanged (PC-012 will add EntryNotFound error)
    Ok(content.to_owned())
}

/// Extract the last trail segment's status from an entry line's `rest` portion.
fn extract_last_status(rest: &str) -> Result<TaskStatus, TaskError> {
    // Find the trail section: after the second ` | ` for PC entries, or after first ` | ` for PostTdd
    // The trail is the last pipe-delimited field in a PC line or last space-delimited segment.
    // We split by ` | ` to get parts and take the last one as the trail string.
    let parts: Vec<&str> = rest.splitn(4, " | ").collect();
    let trail_str = match parts.len() {
        1 => return Ok(TaskStatus::Pending), // no trail at all
        2 => parts[1].trim(),                // PostTdd: [label, trail]
        3 => parts[2].trim(),                // PC: [desc, cmd, trail]
        _ => parts[parts.len() - 1].trim(),
    };

    if trail_str.is_empty() {
        return Ok(TaskStatus::Pending);
    }

    // Find the last segment by splitting on ` → ` or ` | `
    let separator = if trail_str.contains(" → ") { " → " } else { " | " };
    let last_seg = trail_str.split(separator).last().unwrap_or(trail_str).trim();

    // Parse `status@timestamp`
    let at_pos = last_seg.find('@').ok_or_else(|| TaskError::ParseError {
        line: 0,
        message: format!("trail segment missing '@': {last_seg}"),
    })?;
    let status_str = &last_seg[..at_pos];
    TaskStatus::from_str(status_str)
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

    #[test]
    fn done_checkbox() {
        // Start from a green state so done transition is valid
        let green_content = r#"# Tasks: Test Feature

## Pass Conditions

- [ ] PC-001: Test description | `cargo test` | pending@2026-03-29T14:00:00Z → red@2026-03-29T14:01:00Z → green@2026-03-29T14:02:00Z

## Post-TDD

- [ ] E2E tests | pending@2026-03-29T14:00:00Z
"#;
        let updated = apply_update(
            green_content,
            "PC-001",
            TaskStatus::Done,
            "2026-03-29T14:03:00Z",
        )
        .expect("apply_update should succeed for green -> done");

        assert!(
            updated.contains("- [x]"),
            "checkbox should be flipped to [x] on done transition, got:\n{updated}"
        );
        assert!(
            !updated.contains("- [ ] PC-001"),
            "original [ ] checkbox should be replaced, got:\n{updated}"
        );
    }
}
