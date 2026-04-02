//! Delta merge algorithm for cartography section markers.
//!
//! Pure string processing — zero I/O.
//! Section markers format:
//!   - Open:  `<!-- CARTOGRAPHY: <section-id> -->`
//!   - Close: `<!-- /CARTOGRAPHY: <section-id> -->`

/// Returns `true` if the document already contains the given section marker pair.
pub fn has_section(content: &str, section_id: &str) -> bool {
    let open = format!("<!-- CARTOGRAPHY: {} -->", section_id);
    content.contains(&open)
}

/// Insert or replace a section in `existing`.
///
/// - If `section_id` does not exist: append the new section (with markers)
///   immediately after the last closing marker found in the document, or at
///   the end of the document if there are no existing markers.
/// - If `section_id` already exists: replace the content between the matching
///   open and close markers in-place. Content outside all markers is never
///   modified.
pub fn merge_section(existing: &str, section_id: &str, new_content: &str) -> String {
    let open_marker = format!("<!-- CARTOGRAPHY: {} -->", section_id);
    let close_marker = format!("<!-- /CARTOGRAPHY: {} -->", section_id);
    let section_block = format!("{}\n{}{}\n", open_marker, new_content, close_marker);

    if has_section(existing, section_id) {
        // Replace: find open marker, then close marker after it.
        let open_pos = existing
            .find(&open_marker)
            .expect("has_section guarantees open marker exists");
        let after_open = &existing[open_pos..];
        let close_relative = after_open
            .find(&close_marker)
            .expect("malformed document: open marker without close marker");
        let close_pos = open_pos + close_relative + close_marker.len();

        let before = &existing[..open_pos];
        let after = &existing[close_pos..];
        format!("{}{}{}", before, section_block, after)
    } else {
        // Append: find the last closing marker of any section.
        let any_close = "<!-- /CARTOGRAPHY:";
        if let Some(last_close_pos) = find_last_close_marker(existing, any_close) {
            let after_last_close = &existing[last_close_pos..];
            let end_of_line = after_last_close
                .find('\n')
                .map(|i| i + 1)
                .unwrap_or(after_last_close.len());
            let insert_at = last_close_pos + end_of_line;
            let before = &existing[..insert_at];
            let after = &existing[insert_at..];
            format!("{}\n{}{}", before, section_block, after)
        } else {
            // No existing markers — append at end.
            let separator = if existing.ends_with('\n') { "" } else { "\n" };
            format!("{}{}\n{}", existing, separator, section_block)
        }
    }
}

/// Find the byte position of the start of the last line that begins an
/// arbitrary CARTOGRAPHY closing marker (`<!-- /CARTOGRAPHY: ...`).
fn find_last_close_marker(content: &str, prefix: &str) -> Option<usize> {
    let mut last: Option<usize> = None;
    let mut search_from = 0;
    while let Some(pos) = content[search_from..].find(prefix) {
        let abs_pos = search_from + pos;
        last = Some(abs_pos);
        search_from = abs_pos + prefix.len();
    }
    last
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // has_section
    // -----------------------------------------------------------------------

    #[test]
    fn has_section_returns_true_when_present() {
        let doc = "# Doc\n<!-- CARTOGRAPHY: step-1 -->\ncontent\n<!-- /CARTOGRAPHY: step-1 -->\n";
        assert!(has_section(doc, "step-1"));
    }

    #[test]
    fn has_section_returns_false_when_absent() {
        let doc = "# Doc\nSome content here.\n";
        assert!(!has_section(doc, "step-1"));
    }

    #[test]
    fn has_section_is_exact_match() {
        let doc =
            "<!-- CARTOGRAPHY: step-10 -->\ncontent\n<!-- /CARTOGRAPHY: step-10 -->\n";
        // "step-1" must NOT match inside "step-10"
        assert!(!has_section(doc, "step-1"));
    }

    // -----------------------------------------------------------------------
    // merge_section — append (new section)
    // -----------------------------------------------------------------------

    #[test]
    fn append_new_section_after_last_closing_marker() {
        let existing = "\
# My Journey

Some manual content here.

<!-- CARTOGRAPHY: step-1 -->
### Step 1: Do the thing
Details here.
<!-- /CARTOGRAPHY: step-1 -->

More manual content.
";

        let new_content = "### Step 2: Second thing\nMore details.\n";
        let result = merge_section(existing, "step-2", new_content);

        // The new section must appear after step-1's closing marker.
        let step1_close_pos = result
            .find("<!-- /CARTOGRAPHY: step-1 -->")
            .expect("step-1 closing marker must still be present");
        let step2_open_pos = result
            .find("<!-- CARTOGRAPHY: step-2 -->")
            .expect("new step-2 opening marker must be present");
        assert!(
            step2_open_pos > step1_close_pos,
            "step-2 must appear after step-1 closing marker"
        );

        // The new content must be between the new markers.
        assert!(result.contains("### Step 2: Second thing"));
        assert!(result.contains("<!-- /CARTOGRAPHY: step-2 -->"));

        // Manual content that was after step-1 must still be present.
        assert!(
            result.contains("More manual content."),
            "manual content after last marker must be preserved"
        );

        // Manual content before step-1 must still be present.
        assert!(
            result.contains("Some manual content here."),
            "manual content before markers must be preserved"
        );
    }

    #[test]
    fn append_new_section_when_no_existing_markers() {
        let existing = "# Empty Doc\n\nNo steps yet.\n";
        let new_content = "### Step 1: First step\nDetails.\n";
        let result = merge_section(existing, "step-1", new_content);

        assert!(result.contains("<!-- CARTOGRAPHY: step-1 -->"));
        assert!(result.contains("### Step 1: First step"));
        assert!(result.contains("<!-- /CARTOGRAPHY: step-1 -->"));
        // Original content preserved.
        assert!(result.contains("# Empty Doc"));
        assert!(result.contains("No steps yet."));
    }

    // -----------------------------------------------------------------------
    // merge_section — replace (existing section with same ID)
    // -----------------------------------------------------------------------

    #[test]
    fn replace_existing_section_in_place() {
        let existing = "\
# My Journey

Manual intro.

<!-- CARTOGRAPHY: step-1 -->
### Step 1: Old content
Old details.
<!-- /CARTOGRAPHY: step-1 -->

Manual outro.
";

        let new_content = "### Step 1: Updated content\nNew details.\n";
        let result = merge_section(existing, "step-1", new_content);

        // New content present.
        assert!(result.contains("### Step 1: Updated content"));
        assert!(result.contains("New details."));

        // Old content gone.
        assert!(!result.contains("### Step 1: Old content"));
        assert!(!result.contains("Old details."));

        // Markers still present.
        assert!(result.contains("<!-- CARTOGRAPHY: step-1 -->"));
        assert!(result.contains("<!-- /CARTOGRAPHY: step-1 -->"));

        // Manual content unchanged.
        assert!(result.contains("Manual intro."));
        assert!(result.contains("Manual outro."));
    }

    #[test]
    fn replace_middle_section_preserves_adjacent_sections() {
        let existing = "\
<!-- CARTOGRAPHY: step-1 -->
Content 1.
<!-- /CARTOGRAPHY: step-1 -->

<!-- CARTOGRAPHY: step-2 -->
Content 2 old.
<!-- /CARTOGRAPHY: step-2 -->

<!-- CARTOGRAPHY: step-3 -->
Content 3.
<!-- /CARTOGRAPHY: step-3 -->
";

        let result = merge_section(existing, "step-2", "Content 2 new.\n");

        // step-2 updated.
        assert!(result.contains("Content 2 new."));
        assert!(!result.contains("Content 2 old."));

        // step-1 and step-3 untouched.
        assert!(result.contains("Content 1."));
        assert!(result.contains("Content 3."));
    }

    // -----------------------------------------------------------------------
    // merge_section — manual content preservation (AC-002.5)
    // -----------------------------------------------------------------------

    #[test]
    fn manual_content_outside_all_markers_preserved_byte_for_byte() {
        let before_all = "# Title\n\nIntroduction paragraph.\n\n";
        let after_all = "\n## Conclusion\n\nHand-written notes.\n";
        let section_block = "<!-- CARTOGRAPHY: step-1 -->\nOld.\n<!-- /CARTOGRAPHY: step-1 -->\n";
        let existing = format!("{}{}{}", before_all, section_block, after_all);

        let result = merge_section(&existing, "step-1", "New.\n");

        assert!(
            result.starts_with(before_all),
            "content before markers must be byte-for-byte identical"
        );
        assert!(
            result.ends_with(after_all),
            "content after markers must be byte-for-byte identical"
        );
    }
}
