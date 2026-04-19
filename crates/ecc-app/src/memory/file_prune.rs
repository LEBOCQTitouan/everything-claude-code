//! File pruning utilities for the memory system.
//!
//! Provides BL-ID pattern matching for memory file cleanup.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bl_id_regex_collision_safety() {
        // BL-10 matches project_bl10* and project_bl010* but NOT project_bl100*
        assert!(matches_bl_id("project_bl10.md", "BL-10"));
        assert!(matches_bl_id("project_bl010_foo.md", "BL-10"));
        assert!(
            !matches_bl_id("project_bl100.md", "BL-10"),
            "collision risk: bl100 must not match BL-10"
        );
        assert!(!matches_bl_id("project_bl100_foo.md", "BL-10"));

        // BL-100 matches bl100* but NOT bl10*
        assert!(matches_bl_id("project_bl100.md", "BL-100"));
        assert!(matches_bl_id("project_bl100_bar.md", "BL-100"));
        assert!(!matches_bl_id("project_bl10.md", "BL-100"));

        // BL-031 matches with or without suffix
        assert!(matches_bl_id("project_bl031.md", "BL-031"));
        assert!(matches_bl_id("project_bl031_foo.md", "BL-031"));
        assert!(matches_bl_id("project_bl31.md", "BL-031"), "zero-pad allowed via 0*");

        // Invalid BL ID returns false
        assert!(!matches_bl_id("project_bl001.md", "invalid"));
    }
}
