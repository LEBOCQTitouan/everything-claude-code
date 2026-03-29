//! Pure parser for `docs/sources.md` format.
//!
//! No I/O: operates on `&str` and returns domain types.

use super::entry::{Quadrant, SourceEntry, SourceError, SourceType};
use super::registry::{ModuleMapping, SourcesRegistry};
use std::str::FromStr;

/// Parse a `docs/sources.md` document into a `SourcesRegistry`.
///
/// - Empty content or missing sections → empty registry (no error).
/// - Malformed entries accumulate per-entry errors without aborting.
/// - If any errors are collected, returns `Err(errors)`.
/// - On clean parse, returns `Ok(registry)`.
pub fn parse_sources(content: &str) -> Result<SourcesRegistry, Vec<SourceError>> {
    let _ = content;
    unimplemented!("parse_sources not yet implemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::entry::{Quadrant, SourceType};

    // Full document with all sections populated.
    const FULL_DOC: &str = r#"# Knowledge Sources

## Inbox

- [Inbox Entry](https://example.com/inbox) \u{2014} type: repo | quadrant: assess | subject: testing | added: 2026-03-29 | by: human

## Adopt

### testing
- [Adopt Testing](https://example.com/adopt-testing) \u{2014} type: doc | subject: testing | added: 2026-03-01 | by: human | checked: 2026-03-15

### rust-patterns
- [Adopt Rust](https://example.com/adopt-rust) \u{2014} type: repo | subject: rust-patterns | added: 2026-03-01 | by: agent:spec | checked: 2026-03-15

## Trial

### cli
- [Trial CLI](https://example.com/trial-cli) \u{2014} type: blog | subject: cli | added: 2026-02-01 | by: human

## Assess

### security
- [Assess Security](https://example.com/assess-security) \u{2014} type: paper | subject: security | added: 2026-01-15 | by: human

## Hold

### legacy
- [Hold Legacy](https://example.com/hold-legacy) \u{2014} type: package | subject: legacy | added: 2025-12-01 | by: human

## Module Mapping

| Module | Subjects |
|--------|----------|
| crates/ecc-domain/ | domain-modeling, rust-patterns |
| crates/ecc-app/ | app-patterns, testing |
"#;

    #[test]
    fn parse_full_document() {
        let registry = parse_sources(FULL_DOC).expect("full document should parse without errors");

        // Inbox entry
        assert_eq!(registry.inbox.len(), 1);
        let inbox_entry = &registry.inbox[0];
        assert_eq!(inbox_entry.title, "Inbox Entry");
        assert_eq!(inbox_entry.url, "https://example.com/inbox");
        assert_eq!(inbox_entry.source_type, SourceType::Repo);
        assert_eq!(inbox_entry.quadrant, Quadrant::Assess);
        assert_eq!(inbox_entry.subject, "testing");
        assert_eq!(inbox_entry.added_by, "human");
        assert_eq!(inbox_entry.added_date, "2026-03-29");

        // Non-inbox entries: Adopt×2, Trial×1, Assess×1, Hold×1 = 5
        assert_eq!(registry.entries.len(), 5);

        let adopt_testing = registry
            .entries
            .iter()
            .find(|e| e.url == "https://example.com/adopt-testing")
            .expect("adopt-testing entry must exist");
        assert_eq!(adopt_testing.quadrant, Quadrant::Adopt);
        assert_eq!(adopt_testing.subject, "testing");
        assert_eq!(adopt_testing.source_type, SourceType::Doc);
        assert_eq!(adopt_testing.last_checked, Some("2026-03-15".to_owned()));

        let adopt_rust = registry
            .entries
            .iter()
            .find(|e| e.url == "https://example.com/adopt-rust")
            .expect("adopt-rust entry must exist");
        assert_eq!(adopt_rust.quadrant, Quadrant::Adopt);
        assert_eq!(adopt_rust.subject, "rust-patterns");
        assert_eq!(adopt_rust.added_by, "agent:spec");

        // Module mappings
        assert_eq!(registry.module_mappings.len(), 2);
        let domain_mapping = registry
            .module_mappings
            .iter()
            .find(|m| m.module_path == "crates/ecc-domain/")
            .expect("domain module mapping must exist");
        assert_eq!(
            domain_mapping.subjects,
            vec!["domain-modeling", "rust-patterns"]
        );
    }

    #[test]
    fn parse_empty_file() {
        // Empty string
        let registry = parse_sources("").expect("empty content should return empty registry");
        assert!(registry.inbox.is_empty());
        assert!(registry.entries.is_empty());
        assert!(registry.module_mappings.is_empty());

        // Only a title, no sections
        let registry =
            parse_sources("# Knowledge Sources\n").expect("no sections should return empty registry");
        assert!(registry.inbox.is_empty());
        assert!(registry.entries.is_empty());
        assert!(registry.module_mappings.is_empty());

        // Sections present but empty
        let minimal = "# Knowledge Sources\n\n## Inbox\n\n## Adopt\n\n## Module Mapping\n";
        let registry =
            parse_sources(minimal).expect("empty sections should return empty registry");
        assert!(registry.inbox.is_empty());
        assert!(registry.entries.is_empty());
        assert!(registry.module_mappings.is_empty());
    }

    #[test]
    fn parse_errors_per_entry() {
        // Document with two malformed entries mixed with a valid one
        let doc = concat!(
            "# Knowledge Sources\n\n",
            "## Inbox\n\n",
            "- [Bad Entry](https://example.com/bad1) \u{2014} type: INVALID_TYPE | quadrant: assess | subject: testing | added: 2026-03-29 | by: human\n",
            "- [Good Entry](https://example.com/good) \u{2014} type: repo | quadrant: assess | subject: testing | added: 2026-03-29 | by: human\n",
            "- [Bad Quadrant](https://example.com/bad2) \u{2014} type: repo | quadrant: INVALID_QUADRANT | subject: testing | added: 2026-03-29 | by: human\n",
        );

        let result = parse_sources(doc);
        assert!(result.is_err(), "errors should be returned for malformed entries");

        let errors = result.unwrap_err();
        // Both bad entries should produce errors (not just the first)
        assert!(
            errors.len() >= 2,
            "expected at least 2 errors, got {}: {:?}",
            errors.len(),
            errors
        );
    }

    #[test]
    fn parse_module_mapping() {
        let doc = concat!(
            "# Knowledge Sources\n\n",
            "## Module Mapping\n\n",
            "| Module | Subjects |\n",
            "|--------|----------|\n",
            "| crates/ecc-domain/ | domain-modeling, rust-patterns |\n",
            "| crates/ecc-app/ | app-patterns |\n",
        );

        let registry = parse_sources(doc).expect("module mapping table should parse cleanly");

        assert_eq!(registry.module_mappings.len(), 2);

        let domain = registry
            .module_mappings
            .iter()
            .find(|m| m.module_path == "crates/ecc-domain/")
            .expect("domain mapping must exist");
        assert_eq!(domain.subjects, vec!["domain-modeling", "rust-patterns"]);

        let app = registry
            .module_mappings
            .iter()
            .find(|m| m.module_path == "crates/ecc-app/")
            .expect("app mapping must exist");
        assert_eq!(app.subjects, vec!["app-patterns"]);
    }
}
