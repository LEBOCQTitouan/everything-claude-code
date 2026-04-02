//! Pure parser for `docs/sources.md` format.
//!
//! No I/O: operates on `&str` and returns domain types.

use super::entry::{Quadrant, SourceEntry, SourceError, SourceType, SourceUrl};
use super::registry::{ModuleMapping, SourcesRegistry};
use std::str::FromStr;

/// Parse a `docs/sources.md` document into a `SourcesRegistry`.
///
/// - Empty content or missing sections → empty registry (no error).
/// - Malformed entries accumulate per-entry errors without aborting.
/// - If any errors are collected, returns `Err(errors)`.
/// - On clean parse, returns `Ok(registry)`.
pub fn parse_sources(content: &str) -> Result<SourcesRegistry, Vec<SourceError>> {
    if content.trim().is_empty() {
        return Ok(SourcesRegistry::default());
    }

    let mut inbox: Vec<SourceEntry> = vec![];
    let mut entries: Vec<SourceEntry> = vec![];
    let mut module_mappings: Vec<ModuleMapping> = vec![];
    let mut errors: Vec<SourceError> = vec![];

    // Split into sections by `## ` headers (level-2).
    // We collect section name + body pairs.
    let sections = split_sections(content);

    for (section_name, section_body) in &sections {
        let name = section_name.trim();
        match name {
            "Inbox" => {
                parse_inbox_section(section_body, &mut inbox, &mut errors);
            }
            "Adopt" | "Trial" | "Assess" | "Hold" => {
                let quadrant =
                    Quadrant::from_str(name).expect("quadrant name already matched, must be valid");
                parse_quadrant_section(section_body, quadrant, &mut entries, &mut errors);
            }
            "Module Mapping" => {
                parse_module_mapping_section(section_body, &mut module_mappings);
            }
            _ => {
                // Unknown sections are silently ignored.
            }
        }
    }

    if errors.is_empty() {
        Ok(SourcesRegistry {
            inbox,
            entries,
            module_mappings,
        })
    } else {
        Err(errors)
    }
}

/// Split content into `(section_name, section_body)` pairs by `## ` headings.
fn split_sections(content: &str) -> Vec<(String, String)> {
    let mut sections: Vec<(String, String)> = vec![];
    let mut current_name: Option<String> = None;
    let mut current_body_lines: Vec<&str> = vec![];

    for line in content.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            // Flush previous section.
            if let Some(name) = current_name.take() {
                sections.push((name, current_body_lines.join("\n")));
            }
            current_name = Some(heading.trim().to_owned());
            current_body_lines = vec![];
        } else if current_name.is_some() {
            current_body_lines.push(line);
        }
        // Lines before the first `## ` heading (e.g. `# Knowledge Sources`) are discarded.
    }

    // Flush last section.
    if let Some(name) = current_name {
        sections.push((name, current_body_lines.join("\n")));
    }

    sections
}

/// Parse an Inbox section body.
/// Each entry line must include a `quadrant:` key in the metadata.
fn parse_inbox_section(body: &str, inbox: &mut Vec<SourceEntry>, errors: &mut Vec<SourceError>) {
    for line in body.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("- [") {
            continue;
        }
        match parse_entry_line(trimmed, None) {
            Ok(entry) => inbox.push(entry),
            Err(e) => errors.push(e),
        }
    }
}

/// Parse a quadrant section body (may contain `### subject` subsections).
fn parse_quadrant_section(
    body: &str,
    quadrant: Quadrant,
    entries: &mut Vec<SourceEntry>,
    errors: &mut Vec<SourceError>,
) {
    let mut current_subject: Option<String> = None;

    for line in body.lines() {
        if let Some(subj) = line.trim().strip_prefix("### ") {
            current_subject = Some(subj.trim().to_owned());
        } else if line.trim().starts_with("- [") {
            let subject = current_subject.clone();
            match parse_entry_line(line.trim(), Some((&quadrant, subject.as_deref()))) {
                Ok(entry) => entries.push(entry),
                Err(e) => errors.push(e),
            }
        }
    }
}

/// Parse the Module Mapping table section.
fn parse_module_mapping_section(body: &str, mappings: &mut Vec<ModuleMapping>) {
    for line in body.lines() {
        let trimmed = line.trim();
        // Skip header rows and separator rows.
        if !trimmed.starts_with('|')
            || trimmed.starts_with("| Module")
            || trimmed.starts_with("|---")
            || trimmed.starts_with("|-----")
        {
            continue;
        }
        // Parse `| module_path | subjects |`
        let cols: Vec<&str> = trimmed
            .split('|')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if cols.len() < 2 {
            continue;
        }
        let module_path = cols[0].to_owned();
        let subjects: Vec<String> = cols[1]
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect();
        if !module_path.is_empty() && !subjects.is_empty() {
            mappings.push(ModuleMapping {
                module_path,
                subjects,
            });
        }
    }
}

/// Parse a single entry line of the form:
/// `- [Title](url) — key: value | key: value ...`
///
/// `context` provides the quadrant and subject from the surrounding section/subsection.
/// When `context` is `None` (Inbox), both must appear in the metadata.
fn parse_entry_line(
    line: &str,
    context: Option<(&Quadrant, Option<&str>)>,
) -> Result<SourceEntry, SourceError> {
    // Strip the leading `- `.
    let rest = line
        .strip_prefix("- ")
        .ok_or_else(|| SourceError::ParseError {
            line: 0,
            message: format!("entry line must start with '- ': {line}"),
        })?;

    // Extract [Title](url) part.
    let bracket_start = rest.find('[').ok_or_else(|| SourceError::ParseError {
        line: 0,
        message: format!("missing '[' in entry line: {rest}"),
    })?;
    let bracket_end = rest.find("](").ok_or_else(|| SourceError::ParseError {
        line: 0,
        message: format!("missing '](' in entry line: {rest}"),
    })?;
    let paren_end = rest.find(')').ok_or_else(|| SourceError::ParseError {
        line: 0,
        message: format!("missing ')' in entry line: {rest}"),
    })?;

    let title = rest[bracket_start + 1..bracket_end].to_owned();
    let url = rest[bracket_end + 2..paren_end].to_owned();

    // Find the em-dash separator (U+2014).
    let after_link = &rest[paren_end + 1..];
    let meta_str = after_link
        .split_once('\u{2014}')
        .map(|x| x.1)
        .unwrap_or("")
        .trim();

    // Parse key:value pairs separated by `|`.
    let mut kv: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
    let mut flags: Vec<&str> = vec![];
    for part in meta_str.split('|') {
        let part = part.trim();
        if let Some((k, v)) = part.split_once(':') {
            kv.insert(k.trim(), v.trim());
        } else if !part.is_empty() {
            flags.push(part);
        }
    }

    // source_type
    let type_str = kv.get("type").copied().unwrap_or("");
    let source_type = SourceType::from_str(type_str).map_err(|_| SourceError::ParseError {
        line: 0,
        message: format!("unknown source type: '{type_str}'"),
    })?;

    // quadrant: from context (quadrant sections) or metadata (Inbox)
    let quadrant = if let Some((ctx_quadrant, _)) = context {
        ctx_quadrant.clone()
    } else {
        let q_str = kv.get("quadrant").copied().unwrap_or("");
        Quadrant::from_str(q_str).map_err(|_| SourceError::ParseError {
            line: 0,
            message: format!("unknown quadrant: '{q_str}'"),
        })?
    };

    // subject: from context (subsection heading) or metadata
    let subject = if let Some((_, Some(ctx_subject))) = context {
        ctx_subject.to_owned()
    } else {
        kv.get("subject").copied().unwrap_or("").to_owned()
    };

    let added_date = kv.get("added").copied().unwrap_or("").to_owned();
    let added_by = kv.get("by").copied().unwrap_or("").to_owned();
    let last_checked = kv.get("checked").copied().map(str::to_owned);

    let deprecation_reason = kv.get("deprecated").copied().map(str::to_owned);
    let stale = flags.contains(&"stale");

    // Validate
    let source_url = SourceUrl::parse(&url)?;
    super::entry::validate_title(&title)?;

    Ok(SourceEntry {
        url: source_url,
        title,
        source_type,
        quadrant,
        subject,
        added_by,
        added_date,
        last_checked,
        deprecation_reason,
        stale,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::entry::{Quadrant, SourceType};

    // Full document with all sections populated.
    const FULL_DOC: &str = "# Knowledge Sources\n\n## Inbox\n\n- [Inbox Entry](https://example.com/inbox) \u{2014} type: repo | quadrant: assess | subject: testing | added: 2026-03-29 | by: human\n\n## Adopt\n\n### testing\n- [Adopt Testing](https://example.com/adopt-testing) \u{2014} type: doc | subject: testing | added: 2026-03-01 | by: human | checked: 2026-03-15\n\n### rust-patterns\n- [Adopt Rust](https://example.com/adopt-rust) \u{2014} type: repo | subject: rust-patterns | added: 2026-03-01 | by: agent:spec | checked: 2026-03-15\n\n## Trial\n\n### cli\n- [Trial CLI](https://example.com/trial-cli) \u{2014} type: blog | subject: cli | added: 2026-02-01 | by: human\n\n## Assess\n\n### security\n- [Assess Security](https://example.com/assess-security) \u{2014} type: paper | subject: security | added: 2026-01-15 | by: human\n\n## Hold\n\n### legacy\n- [Hold Legacy](https://example.com/hold-legacy) \u{2014} type: package | subject: legacy | added: 2025-12-01 | by: human\n\n## Module Mapping\n\n| Module | Subjects |\n|--------|----------|\n| crates/ecc-domain/ | domain-modeling, rust-patterns |\n| crates/ecc-app/ | app-patterns, testing |\n";

    #[test]
    fn parse_full_document() {
        let registry = parse_sources(FULL_DOC).expect("full document should parse without errors");

        // Inbox entry
        assert_eq!(registry.inbox.len(), 1);
        let inbox_entry = &registry.inbox[0];
        assert_eq!(inbox_entry.title, "Inbox Entry");
        assert_eq!(inbox_entry.url.as_str(), "https://example.com/inbox");
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
            .find(|e| e.url.as_str() == "https://example.com/adopt-testing")
            .expect("adopt-testing entry must exist");
        assert_eq!(adopt_testing.quadrant, Quadrant::Adopt);
        assert_eq!(adopt_testing.subject, "testing");
        assert_eq!(adopt_testing.source_type, SourceType::Doc);
        assert_eq!(adopt_testing.last_checked, Some("2026-03-15".to_owned()));

        let adopt_rust = registry
            .entries
            .iter()
            .find(|e| e.url.as_str() == "https://example.com/adopt-rust")
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
        let registry = parse_sources("# Knowledge Sources\n")
            .expect("no sections should return empty registry");
        assert!(registry.inbox.is_empty());
        assert!(registry.entries.is_empty());
        assert!(registry.module_mappings.is_empty());

        // Sections present but empty
        let minimal = "# Knowledge Sources\n\n## Inbox\n\n## Adopt\n\n## Module Mapping\n";
        let registry = parse_sources(minimal).expect("empty sections should return empty registry");
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
        assert!(
            result.is_err(),
            "errors should be returned for malformed entries"
        );

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

    // --- PC-010: Stale bare flag parses correctly ---
    #[test]
    fn parse_stale_bare_flag() {
        let doc = "# Knowledge Sources\n\n## Inbox\n\n\n## Adopt\n\n### testing\n- [Stale Entry](https://example.com/stale) \u{2014} type: doc | subject: testing | added: 2026-01-01 | by: human | stale\n\n## Trial\n\n## Assess\n\n## Hold\n\n## Module Mapping\n\n| Module | Subjects |\n|--------|----------|\n";
        let registry = parse_sources(doc).expect("should parse stale entry");
        let entry = registry
            .entries
            .iter()
            .find(|e| e.url.as_str() == "https://example.com/stale")
            .expect("stale entry must exist");
        assert!(entry.stale, "bare 'stale' flag must parse as stale=true");
    }
}
