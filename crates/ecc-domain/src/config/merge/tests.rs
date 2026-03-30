use super::*;

// --- empty_report ---

#[test]
fn empty_report_all_fields_empty() {
    let report = empty_report();
    assert!(report.added.is_empty());
    assert!(report.updated.is_empty());
    assert!(report.unchanged.is_empty());
    assert!(report.skipped.is_empty());
    assert!(report.smart_merged.is_empty());
    assert!(report.errors.is_empty());
}

// --- combine_reports ---

#[test]
fn combine_reports_empty() {
    let result = combine_reports(&[]);
    assert!(result.added.is_empty());
}

#[test]
fn combine_reports_merges_correctly() {
    let r1 = MergeReport {
        added: vec!["a.md".into()],
        updated: vec!["b.md".into()],
        unchanged: vec![],
        skipped: vec![],
        smart_merged: vec![],
        errors: vec![],
    };
    let r2 = MergeReport {
        added: vec!["c.md".into()],
        updated: vec![],
        unchanged: vec!["d.md".into()],
        skipped: vec!["e.md".into()],
        smart_merged: vec!["f.md".into()],
        errors: vec!["err".into()],
    };

    let combined = combine_reports(&[r1, r2]);
    assert_eq!(combined.added, vec!["a.md", "c.md"]);
    assert_eq!(combined.updated, vec!["b.md"]);
    assert_eq!(combined.unchanged, vec!["d.md"]);
    assert_eq!(combined.skipped, vec!["e.md"]);
    assert_eq!(combined.smart_merged, vec!["f.md"]);
    assert_eq!(combined.errors, vec!["err"]);
}

// --- merge_hooks_pure ---

#[test]
fn merge_hooks_pure_adds_new() {
    let source = serde_json::json!({
        "PreToolUse": [
            {"hooks": [{"command": "ecc-hook format"}]}
        ]
    });
    let existing = serde_json::json!({});

    let result = merge_hooks_pure(&source, &existing);
    assert_eq!(result.added, 1);
    assert_eq!(result.existing, 0);
    assert_eq!(result.legacy_removed, 0);
    assert_eq!(result.merged["PreToolUse"].as_array().unwrap().len(), 1);
}

#[test]
fn merge_hooks_pure_deduplicates_existing() {
    let source = serde_json::json!({
        "PreToolUse": [
            {"hooks": [{"command": "ecc-hook format"}]}
        ]
    });
    let existing = serde_json::json!({
        "PreToolUse": [
            {"hooks": [{"command": "ecc-hook format"}]}
        ]
    });

    let result = merge_hooks_pure(&source, &existing);
    assert_eq!(result.added, 0);
    assert_eq!(result.existing, 1);
    assert_eq!(result.merged["PreToolUse"].as_array().unwrap().len(), 1);
}

#[test]
fn merge_hooks_pure_removes_legacy() {
    let source = serde_json::json!({
        "PreToolUse": [
            {"hooks": [{"command": "ecc-hook format"}]}
        ]
    });
    let existing = serde_json::json!({
        "PreToolUse": [
            {"hooks": [{"command": "node scripts/hooks/old.js"}]}
        ]
    });

    let result = merge_hooks_pure(&source, &existing);
    assert_eq!(result.added, 1);
    assert_eq!(result.legacy_removed, 1);
    assert_eq!(result.merged["PreToolUse"].as_array().unwrap().len(), 1);
}

#[test]
fn merge_hooks_pure_preserves_user_hooks() {
    let source = serde_json::json!({
        "PreToolUse": [
            {"hooks": [{"command": "ecc-hook format"}]}
        ]
    });
    let existing = serde_json::json!({
        "PreToolUse": [
            {"hooks": [{"command": "my-custom-hook"}]}
        ]
    });

    let result = merge_hooks_pure(&source, &existing);
    assert_eq!(result.added, 1);
    assert_eq!(result.legacy_removed, 0);
    // User hook + new hook
    assert_eq!(result.merged["PreToolUse"].as_array().unwrap().len(), 2);
}

// --- contents_differ ---

#[test]
fn contents_differ_same() {
    assert!(!contents_differ("hello", "hello"));
}

#[test]
fn contents_differ_different() {
    assert!(contents_differ("hello", "world"));
}

#[test]
fn contents_differ_whitespace_only() {
    assert!(!contents_differ("  hello  \n", "hello"));
}

#[test]
fn contents_differ_trailing_newline() {
    assert!(!contents_differ("hello\n", "hello"));
}

// --- format_merge_report ---

#[test]
fn format_merge_report_empty() {
    let report = empty_report();
    let output = format_merge_report("Agents", &report);
    assert_eq!(output, "  Agents: (no changes)");
}

#[test]
fn format_merge_report_with_changes() {
    let report = MergeReport {
        added: vec!["a.md".into()],
        updated: vec!["b.md".into(), "c.md".into()],
        unchanged: vec!["d.md".into()],
        skipped: vec![],
        smart_merged: vec![],
        errors: vec![],
    };
    let output = format_merge_report("Agents", &report);
    assert!(output.contains("1 added"));
    assert!(output.contains("2 updated"));
    assert!(output.contains("1 unchanged"));
    assert!(!output.contains("skipped"));
}

#[test]
fn format_merge_report_with_errors() {
    let report = MergeReport {
        added: vec![],
        updated: vec![],
        unchanged: vec![],
        skipped: vec![],
        smart_merged: vec!["x.md".into()],
        errors: vec!["oops".into()],
    };
    let output = format_merge_report("Skills", &report);
    assert!(output.contains("1 smart-merged"));
    assert!(output.contains("1 errors"));
}

// --- merge_hooks_result_struct ---

#[test]
fn merge_hooks_result_struct_pure_has_fields() {
    let source = serde_json::json!({
        "PreToolUse": [
            {"hooks": [{"command": "ecc-hook format"}]}
        ]
    });
    let existing = serde_json::json!({});

    let result = merge_hooks_pure(&source, &existing);
    // Access named fields (not tuple positions)
    assert_eq!(result.added, 1);
    assert_eq!(result.existing, 0);
    assert_eq!(result.legacy_removed, 0);
    assert_eq!(result.merged["PreToolUse"].as_array().unwrap().len(), 1);
}
