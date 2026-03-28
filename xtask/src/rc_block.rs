pub const START_MARKER: &str = "# >>> ecc >>>";
pub const END_MARKER: &str = "# <<< ecc <<<";

#[derive(Debug, PartialEq)]
pub struct RcBlockResult {
    pub content: String,
    pub changed: bool,
}

/// Update RC file content with a managed block.
/// - If no markers exist: append block with markers at end
/// - If markers exist: replace content between them
/// - If content identical: return changed=false
/// - If only start marker (no end): treat as no block, insert fresh
pub fn update_rc_content(_existing: &str, _block_lines: &[&str]) -> RcBlockResult {
    todo!("implement update_rc_content")
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
