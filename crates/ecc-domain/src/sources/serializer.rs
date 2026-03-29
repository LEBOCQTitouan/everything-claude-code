//! Pure serializer for `docs/sources.md` format.
//!
//! No I/O: operates on `&SourcesRegistry` and returns an owned `String`.

use super::entry::{Quadrant, SourceEntry};
use super::registry::SourcesRegistry;

/// Serialize a `SourcesRegistry` to canonical `docs/sources.md` format.
///
/// Canonical order:
/// 1. `# Knowledge Sources`
/// 2. `## Inbox` section
/// 3. `## Adopt`, `## Trial`, `## Assess`, `## Hold` sections with subject subsections
/// 4. `## Module Mapping` table
///
/// Within each quadrant section, subjects are sorted alphabetically.
/// Within each subject subsection, entries appear in their original order.
pub fn serialize_sources(registry: &SourcesRegistry) -> String {
    let mut out = String::new();

    out.push_str("# Knowledge Sources\n\n");

    // Inbox section
    out.push_str("## Inbox\n\n");
    for entry in &registry.inbox {
        out.push_str(&serialize_entry(entry, true));
        out.push('\n');
    }
    out.push('\n');

    // Four quadrant sections in canonical order
    for quadrant in &[Quadrant::Adopt, Quadrant::Trial, Quadrant::Assess, Quadrant::Hold] {
        let quadrant_display = quadrant_title(quadrant);
        out.push_str(&format!("## {quadrant_display}\n\n"));

        let mut quadrant_entries: Vec<&SourceEntry> = registry
            .entries
            .iter()
            .filter(|e| &e.quadrant == quadrant)
            .collect();

        // Sort by subject alphabetically, then preserve relative order within subject
        quadrant_entries.sort_by(|a, b| a.subject.cmp(&b.subject));

        // Group by subject
        let mut subjects: Vec<String> = vec![];
        for entry in &quadrant_entries {
            if !subjects.contains(&entry.subject) {
                subjects.push(entry.subject.clone());
            }
        }

        for subject in &subjects {
            out.push_str(&format!("### {subject}\n\n"));
            for entry in quadrant_entries.iter().filter(|e| &e.subject == subject) {
                out.push_str(&serialize_entry(entry, false));
                out.push('\n');
            }
            out.push('\n');
        }
    }

    // Module Mapping section
    out.push_str("## Module Mapping\n\n");
    if !registry.module_mappings.is_empty() {
        out.push_str("| Module | Subjects |\n");
        out.push_str("|--------|----------|\n");
        for mapping in &registry.module_mappings {
            let subjects_str = mapping.subjects.join(", ");
            out.push_str(&format!("| {} | {} |\n", mapping.module_path, subjects_str));
        }
    }

    out
}

/// Format the quadrant name with title case.
fn quadrant_title(q: &Quadrant) -> &'static str {
    match q {
        Quadrant::Adopt => "Adopt",
        Quadrant::Trial => "Trial",
        Quadrant::Assess => "Assess",
        Quadrant::Hold => "Hold",
    }
}

/// Serialize a single `SourceEntry` to its markdown list item format.
///
/// For Inbox entries (`include_quadrant = true`), the quadrant is included in the metadata.
/// For quadrant-section entries, the quadrant is implied by the section heading.
fn serialize_entry(entry: &SourceEntry, include_quadrant: bool) -> String {
    let mut parts: Vec<String> = vec![];

    parts.push(format!("type: {}", entry.source_type));

    if include_quadrant {
        parts.push(format!("quadrant: {}", entry.quadrant));
    }

    parts.push(format!("subject: {}", entry.subject));
    parts.push(format!("added: {}", entry.added_date));
    parts.push(format!("by: {}", entry.added_by));

    if let Some(ref checked) = entry.last_checked {
        parts.push(format!("checked: {checked}"));
    }

    if entry.stale {
        parts.push("stale".to_owned());
    }

    if let Some(ref reason) = entry.deprecation_reason {
        parts.push(format!("deprecated: {reason}"));
    }

    let meta = parts.join(" | ");
    format!("- [{}]({}) \u{2014} {meta}", entry.title, entry.url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::entry::{Quadrant, SourceEntry, SourceType};
    use crate::sources::parser::parse_sources;
    use crate::sources::registry::ModuleMapping;

    fn make_entry(
        url: &str,
        title: &str,
        source_type: SourceType,
        quadrant: Quadrant,
        subject: &str,
        added_by: &str,
        added_date: &str,
        last_checked: Option<&str>,
    ) -> SourceEntry {
        SourceEntry {
            url: url.to_owned(),
            title: title.to_owned(),
            source_type,
            quadrant,
            subject: subject.to_owned(),
            added_by: added_by.to_owned(),
            added_date: added_date.to_owned(),
            last_checked: last_checked.map(str::to_owned),
            deprecation_reason: None,
            stale: false,
        }
    }

    #[test]
    fn serialize_canonical() {
        let registry = SourcesRegistry {
            inbox: vec![make_entry(
                "https://example.com/inbox",
                "Inbox Entry",
                SourceType::Repo,
                Quadrant::Assess,
                "testing",
                "human",
                "2026-03-29",
                None,
            )],
            entries: vec![
                make_entry(
                    "https://example.com/adopt-rust",
                    "Adopt Rust",
                    SourceType::Repo,
                    Quadrant::Adopt,
                    "rust-patterns",
                    "agent:spec",
                    "2026-03-01",
                    Some("2026-03-15"),
                ),
                make_entry(
                    "https://example.com/adopt-testing",
                    "Adopt Testing",
                    SourceType::Doc,
                    Quadrant::Adopt,
                    "testing",
                    "human",
                    "2026-03-01",
                    Some("2026-03-15"),
                ),
            ],
            module_mappings: vec![ModuleMapping {
                module_path: "crates/ecc-domain/".to_owned(),
                subjects: vec!["domain-modeling".to_owned(), "rust-patterns".to_owned()],
            }],
        };

        let output = serialize_sources(&registry);

        // Canonical header
        assert!(
            output.starts_with("# Knowledge Sources\n"),
            "must start with canonical header"
        );

        // Inbox section present
        assert!(output.contains("## Inbox\n"), "Inbox section must be present");
        assert!(
            output.contains("- [Inbox Entry](https://example.com/inbox)"),
            "inbox entry must appear"
        );

        // Adopt section with subject subsections
        assert!(output.contains("## Adopt\n"), "Adopt section must be present");
        assert!(
            output.contains("### rust-patterns\n"),
            "rust-patterns subject must be a subsection"
        );
        assert!(
            output.contains("### testing\n"),
            "testing subject must be a subsection"
        );

        // Subjects sorted alphabetically: rust-patterns before testing
        let rust_pos = output
            .find("### rust-patterns")
            .expect("rust-patterns section must exist");
        let testing_pos = output
            .find("### testing")
            .expect("testing section must exist");
        assert!(
            rust_pos < testing_pos,
            "rust-patterns must appear before testing (alphabetical)"
        );

        // Module mapping table
        assert!(
            output.contains("## Module Mapping\n"),
            "Module Mapping section must be present"
        );
        assert!(
            output.contains("| Module | Subjects |"),
            "module mapping table header must be present"
        );
        assert!(
            output.contains("| crates/ecc-domain/ |"),
            "domain module must be in table"
        );
    }

    #[test]
    fn round_trip() {
        // Input document in canonical form
        let input = concat!(
            "# Knowledge Sources\n\n",
            "## Inbox\n\n",
            "- [Inbox Entry](https://example.com/inbox) \u{2014} type: repo | quadrant: assess | subject: testing | added: 2026-03-29 | by: human\n\n",
            "## Adopt\n\n",
            "### rust-patterns\n\n",
            "- [Adopt Rust](https://example.com/adopt-rust) \u{2014} type: repo | subject: rust-patterns | added: 2026-03-01 | by: agent:spec | checked: 2026-03-15\n\n",
            "### testing\n\n",
            "- [Adopt Testing](https://example.com/adopt-testing) \u{2014} type: doc | subject: testing | added: 2026-03-01 | by: human | checked: 2026-03-15\n\n",
            "## Trial\n\n",
            "## Assess\n\n",
            "## Hold\n\n",
            "## Module Mapping\n\n",
            "| Module | Subjects |\n",
            "|--------|----------|\n",
            "| crates/ecc-domain/ | domain-modeling, rust-patterns |\n",
        );

        // Parse → serialize → parse: both registries should be semantically equal.
        let registry1 = parse_sources(input).expect("initial parse must succeed");
        let serialized = serialize_sources(&registry1);
        let registry2 = parse_sources(&serialized).expect("re-parse of serialized output must succeed");

        // Compare field by field (SourcesRegistry doesn't derive PartialEq)
        assert_eq!(
            registry1.inbox.len(),
            registry2.inbox.len(),
            "inbox length must match after round-trip"
        );
        assert_eq!(
            registry1.entries.len(),
            registry2.entries.len(),
            "entries length must match after round-trip"
        );
        assert_eq!(
            registry1.module_mappings.len(),
            registry2.module_mappings.len(),
            "module_mappings length must match after round-trip"
        );

        // Deep compare inbox entries
        for (a, b) in registry1.inbox.iter().zip(registry2.inbox.iter()) {
            assert_eq!(a, b, "inbox entry mismatch after round-trip");
        }

        // Deep compare entries (same order)
        for (a, b) in registry1.entries.iter().zip(registry2.entries.iter()) {
            assert_eq!(a, b, "entry mismatch after round-trip");
        }

        // Deep compare module mappings
        for (a, b) in registry1
            .module_mappings
            .iter()
            .zip(registry2.module_mappings.iter())
        {
            assert_eq!(a.module_path, b.module_path);
            assert_eq!(a.subjects, b.subjects);
        }
    }
}
