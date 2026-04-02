//! Renderer that generates tasks.md content from design Pass Conditions.
//!
//! `render_tasks` produces a well-formed tasks.md string from a slice of
//! `PassCondition` values extracted from a design file.

use crate::spec::pc::PassCondition;

/// The fixed Post-TDD checklist entries, in canonical order.
const POST_TDD_ENTRIES: &[&str] = &[
    "E2E tests",
    "Code review",
    "Doc updates",
    "Supplemental docs",
    "Write implement-done.md",
];

/// Generate tasks.md content from design Pass Conditions.
///
/// # Arguments
///
/// * `pcs` — Pass Conditions in dependency order (input order is preserved).
/// * `feature_title` — Human-readable title used in the `# Tasks:` header.
/// * `timestamp` — ISO-8601 timestamp used for all initial `pending@<timestamp>` entries.
///
/// # Returns
///
/// A `String` containing the full tasks.md content, using the `→` separator
/// for status trails and a fixed five-entry Post-TDD section.
pub fn render_tasks(pcs: &[PassCondition], feature_title: &str, timestamp: &str) -> String {
    let mut out = String::new();

    // Title
    out.push_str(&format!("# Tasks: {feature_title}\n"));
    out.push('\n');

    // Pass Conditions section
    out.push_str("## Pass Conditions\n");
    out.push('\n');
    for pc in pcs {
        out.push_str(&format!(
            "- [ ] {}: {} | `{}` | pending@{}\n",
            pc.id, pc.description, pc.command, timestamp
        ));
    }
    out.push('\n');

    // Post-TDD section
    out.push_str("## Post-TDD\n");
    out.push('\n');
    for label in POST_TDD_ENTRIES {
        out.push_str(&format!("- [ ] {} | pending@{}\n", label, timestamp));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::pc::{PassCondition, PcId};

    fn make_pc(num: u16, description: &str, command: &str) -> PassCondition {
        PassCondition {
            id: PcId(num),
            pc_type: "unit".to_owned(),
            description: description.to_owned(),
            verifies_acs: vec![],
            command: command.to_owned(),
            expected: "PASS".to_owned(),
        }
    }

    fn sample_pcs() -> Vec<PassCondition> {
        vec![
            make_pc(
                1,
                "First pass condition",
                "cargo test --lib -p ecc-domain first",
            ),
            make_pc(
                2,
                "Second pass condition",
                "cargo test --lib -p ecc-domain second",
            ),
        ]
    }

    #[test]
    fn renders_title_header() {
        let output = render_tasks(&sample_pcs(), "My Feature", "2026-03-29T10:00:00Z");
        assert!(
            output.starts_with("# Tasks: My Feature\n"),
            "output should start with title header, got:\n{output}"
        );
    }

    #[test]
    fn renders_pass_conditions_section() {
        let output = render_tasks(&sample_pcs(), "My Feature", "2026-03-29T10:00:00Z");
        assert!(
            output.contains("## Pass Conditions\n"),
            "output should contain '## Pass Conditions' section"
        );
    }

    #[test]
    fn renders_pc_entry_format() {
        let output = render_tasks(&sample_pcs(), "My Feature", "2026-03-29T10:00:00Z");
        assert!(
            output.contains("- [ ] PC-001: First pass condition | `cargo test --lib -p ecc-domain first` | pending@2026-03-29T10:00:00Z"),
            "PC-001 entry format incorrect, got:\n{output}"
        );
    }

    #[test]
    fn renders_both_pcs() {
        let output = render_tasks(&sample_pcs(), "My Feature", "2026-03-29T10:00:00Z");
        assert!(output.contains("PC-001"), "output should contain PC-001");
        assert!(output.contains("PC-002"), "output should contain PC-002");
    }

    #[test]
    fn renders_post_tdd_section() {
        let output = render_tasks(&sample_pcs(), "My Feature", "2026-03-29T10:00:00Z");
        assert!(
            output.contains("## Post-TDD\n"),
            "output should contain '## Post-TDD' section"
        );
    }

    #[test]
    fn renders_all_five_post_tdd_entries() {
        let output = render_tasks(&sample_pcs(), "My Feature", "2026-03-29T10:00:00Z");
        for label in &[
            "E2E tests",
            "Code review",
            "Doc updates",
            "Supplemental docs",
            "Write implement-done.md",
        ] {
            assert!(
                output.contains(label),
                "Post-TDD section should contain '{label}', got:\n{output}"
            );
        }
    }

    #[test]
    fn post_tdd_entries_have_pending_timestamp() {
        let ts = "2026-03-29T10:00:00Z";
        let output = render_tasks(&sample_pcs(), "My Feature", ts);
        assert!(
            output.contains(&format!("- [ ] E2E tests | pending@{ts}")),
            "E2E tests entry should have pending@timestamp, got:\n{output}"
        );
    }

    #[test]
    fn pc_entries_have_pending_timestamp() {
        let ts = "2026-03-29T10:00:00Z";
        let output = render_tasks(&sample_pcs(), "My Feature", ts);
        assert!(
            output.contains(&format!("pending@{ts}")),
            "PC entries should have pending@timestamp"
        );
    }

    #[test]
    fn uses_arrow_separator_not_pipe() {
        let output = render_tasks(&sample_pcs(), "My Feature", "2026-03-29T10:00:00Z");
        let pc1_line = output
            .lines()
            .find(|l| l.contains("PC-001"))
            .expect("PC-001 line must be present");
        assert!(
            pc1_line.ends_with("pending@2026-03-29T10:00:00Z"),
            "initial trail should end with pending@timestamp (no → separator yet), line: {pc1_line}"
        );
    }

    pub mod order {
        use super::*;

        #[test]
        fn order_preserved() {
            // Give PCs in non-numeric order to verify input order is preserved
            let pcs = vec![
                make_pc(3, "Third first", "cmd-three"),
                make_pc(1, "First second", "cmd-one"),
                make_pc(2, "Second third", "cmd-two"),
            ];
            let output = render_tasks(&pcs, "Order Test", "2026-03-29T10:00:00Z");
            let pc3_pos = output.find("PC-003").expect("PC-003 must be present");
            let pc1_pos = output.find("PC-001").expect("PC-001 must be present");
            let pc2_pos = output.find("PC-002").expect("PC-002 must be present");
            assert!(
                pc3_pos < pc1_pos,
                "PC-003 (input position 0) must appear before PC-001 (input position 1)"
            );
            assert!(
                pc1_pos < pc2_pos,
                "PC-001 (input position 1) must appear before PC-002 (input position 2)"
            );
        }
    }
}
