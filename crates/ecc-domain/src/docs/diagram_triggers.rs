//! Diagram trigger heuristic evaluation.

use serde::Serialize;

/// Diagram type that can be triggered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagramTrigger {
    /// Sequence diagram (multi-crate interactions).
    Sequence,
    /// Flowchart diagram (enum state machines).
    Flowchart,
    /// C4 diagram (new crate boundaries).
    C4,
}

/// Result of trigger evaluation.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TriggerResult {
    /// Diagrams that should be generated based on detected changes.
    pub triggers: Vec<DiagramTrigger>,
}

/// Evaluate all diagram triggers from changed files and optional content.
pub fn evaluate_triggers(
    changed_files: &[String],
    file_contents: &[(String, String)], // (path, content) for enum analysis
) -> TriggerResult {
    let mut triggers = Vec::new();

    if check_sequence_trigger(changed_files) {
        triggers.push(DiagramTrigger::Sequence);
    }
    if check_flowchart_trigger(file_contents) {
        triggers.push(DiagramTrigger::Flowchart);
    }
    if check_c4_trigger(changed_files) {
        triggers.push(DiagramTrigger::C4);
    }

    TriggerResult { triggers }
}

/// Files span 2+ crate directories → sequence diagram.
fn check_sequence_trigger(changed_files: &[String]) -> bool {
    let crate_dirs: std::collections::HashSet<&str> = changed_files
        .iter()
        .filter_map(|f| {
            f.strip_prefix("crates/")
                .and_then(|rest| rest.split('/').next())
        })
        .collect();
    crate_dirs.len() >= 2
}

/// Any file contains an enum with >=3 variants → flowchart.
fn check_flowchart_trigger(file_contents: &[(String, String)]) -> bool {
    for (_, content) in file_contents {
        if count_max_enum_variants(content) >= 3 {
            return true;
        }
    }
    false
}

/// Count max variants in any enum in the content.
fn count_max_enum_variants(content: &str) -> usize {
    let mut max_variants = 0;
    let mut in_enum = false;
    let mut brace_depth = 0;
    let mut variant_count = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub enum ") || trimmed.starts_with("enum ") {
            in_enum = true;
            brace_depth = 0;
            variant_count = 0;
        }
        if in_enum {
            brace_depth += trimmed.matches('{').count();
            brace_depth -= trimmed.matches('}').count();
            if brace_depth == 1
                && !trimmed.is_empty()
                && !trimmed.starts_with("//")
                && !trimmed.starts_with('#')
                && !trimmed.starts_with('{')
                && !trimmed.contains("enum ")
            {
                variant_count += 1;
            }
            if brace_depth == 0 && in_enum {
                max_variants = max_variants.max(variant_count);
                in_enum = false;
            }
        }
    }
    max_variants
}

/// New crate directory (Cargo.toml in a crates/ subdir) → C4 diagram.
fn check_c4_trigger(changed_files: &[String]) -> bool {
    changed_files
        .iter()
        .any(|f| f.starts_with("crates/") && f.ends_with("Cargo.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_trigger_fires() {
        let files = vec![
            "crates/ecc-domain/src/lib.rs".to_string(),
            "crates/ecc-app/src/lib.rs".to_string(),
        ];
        assert!(check_sequence_trigger(&files));
    }

    #[test]
    fn sequence_trigger_no_fire_same_crate() {
        let files = vec![
            "crates/ecc-domain/src/a.rs".to_string(),
            "crates/ecc-domain/src/b.rs".to_string(),
        ];
        assert!(!check_sequence_trigger(&files));
    }

    #[test]
    fn flowchart_trigger_fires() {
        let content = "pub enum State {\n    A,\n    B,\n    C,\n}";
        assert!(check_flowchart_trigger(&[(
            "f.rs".to_string(),
            content.to_string()
        )]));
    }

    #[test]
    fn c4_trigger_fires() {
        let files = vec!["crates/new-crate/Cargo.toml".to_string()];
        assert!(check_c4_trigger(&files));
    }

    #[test]
    fn no_triggers() {
        let result = evaluate_triggers(&[], &[]);
        assert!(result.triggers.is_empty());
    }
}
