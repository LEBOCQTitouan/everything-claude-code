#[allow(dead_code)]
pub const START_MARKER: &str = "# >>> ecc >>>";
#[allow(dead_code)]
pub const END_MARKER: &str = "# <<< ecc <<<";

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub struct RcBlockResult {
    pub content: String,
    pub changed: bool,
}

#[allow(dead_code)]
/// Update RC file content with a managed block.
/// - If no markers exist: append block with markers at end
/// - If markers exist: replace content between them
/// - If content identical: return changed=false
/// - If only start marker (no end): treat as no block, insert fresh
pub fn update_rc_content(existing: &str, block_lines: &[&str]) -> RcBlockResult {
    let new_block = build_block(block_lines);

    let start_pos = existing.find(START_MARKER);
    let end_pos = existing.find(END_MARKER);

    match (start_pos, end_pos) {
        (Some(s), Some(e)) if s < e => {
            // Both markers present in correct order — replace between them.
            // Consume one trailing newline after END_MARKER if present so
            // build_block's own trailing newline is not doubled.
            let end_of_end_marker = e + END_MARKER.len();
            let after = if existing[end_of_end_marker..].starts_with('\n') {
                &existing[end_of_end_marker + 1..]
            } else {
                &existing[end_of_end_marker..]
            };
            let replaced = format!("{}{}{}", &existing[..s], new_block, after);
            let changed = replaced != existing;
            RcBlockResult { content: replaced, changed }
        }
        _ => {
            // No valid block — append fresh block at end
            let separator = if existing.is_empty() || existing.ends_with('\n') {
                String::new()
            } else {
                "\n".to_string()
            };
            let content = format!("{}{}{}", existing, separator, new_block);
            RcBlockResult { content, changed: true }
        }
    }
}

fn build_block(block_lines: &[&str]) -> String {
    let mut block = String::new();
    block.push_str(START_MARKER);
    block.push('\n');
    for line in block_lines {
        block.push_str(line);
        block.push('\n');
    }
    block.push_str(END_MARKER);
    block.push('\n');
    block
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_empty() {
        let result = update_rc_content("", &["export PATH=\"$HOME/.ecc/bin:$PATH\""]);
        assert!(result.changed, "changed should be true for empty file");
        assert!(
            result.content.contains(START_MARKER),
            "content should contain start marker"
        );
        assert!(
            result.content.contains(END_MARKER),
            "content should contain end marker"
        );
        assert!(
            result.content.contains("export PATH=\"$HOME/.ecc/bin:$PATH\""),
            "content should contain the block line"
        );
    }

    #[test]
    fn insert_preserving() {
        let existing = "# existing\nfoo=bar\n";
        let result = update_rc_content(existing, &["export PATH=\"$HOME/.ecc/bin:$PATH\""]);
        assert!(result.changed, "changed should be true");
        assert!(
            result.content.starts_with("# existing\nfoo=bar\n"),
            "content should start with existing content"
        );
        assert!(
            result.content.contains(START_MARKER),
            "content should contain start marker"
        );
        assert!(
            result.content.contains(END_MARKER),
            "content should contain end marker"
        );
        assert!(
            result.content.contains("export PATH=\"$HOME/.ecc/bin:$PATH\""),
            "content should contain the block line"
        );
    }

    #[test]
    fn replace_existing() {
        let existing = format!(
            "# preamble\n{}\nold line\n{}\n# epilogue\n",
            START_MARKER, END_MARKER
        );
        let result = update_rc_content(&existing, &["new line"]);
        assert!(result.changed, "changed should be true when replacing content");
        assert!(
            result.content.contains("new line"),
            "content should contain new line"
        );
        assert!(
            !result.content.contains("old line"),
            "content should not contain old line"
        );
        assert!(
            result.content.contains("# preamble"),
            "content should preserve preamble"
        );
        assert!(
            result.content.contains("# epilogue"),
            "content should preserve epilogue"
        );
    }

    #[test]
    fn unchanged() {
        let block_line = "export PATH=\"$HOME/.ecc/bin:$PATH\"";
        let existing = format!(
            "{}\n{}\n{}\n",
            START_MARKER, block_line, END_MARKER
        );
        let result = update_rc_content(&existing, &[block_line]);
        assert!(!result.changed, "changed should be false when content is identical");
    }

    #[test]
    fn missing_marker() {
        // Has start marker but no end marker — treat as no block, append fresh
        let existing = format!("# before\n{}\nsome content\n", START_MARKER);
        let result = update_rc_content(&existing, &["export PATH=\"$HOME/.ecc/bin:$PATH\""]);
        assert!(result.changed, "changed should be true");
        // Should have both markers in fresh block
        assert!(
            result.content.contains(START_MARKER),
            "content should contain start marker"
        );
        assert!(
            result.content.contains(END_MARKER),
            "content should contain end marker"
        );
        assert!(
            result.content.contains("export PATH=\"$HOME/.ecc/bin:$PATH\""),
            "content should contain the block line"
        );
    }

    #[test]
    fn empty_lines() {
        let result = update_rc_content("", &[]);
        assert!(result.changed, "changed should be true even for empty block");
        assert!(
            result.content.contains(START_MARKER),
            "content should contain start marker"
        );
        assert!(
            result.content.contains(END_MARKER),
            "content should contain end marker"
        );
    }
}
