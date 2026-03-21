use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Summary of a merge operation across one artifact category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeReport {
    pub added: Vec<String>,
    pub updated: Vec<String>,
    pub unchanged: Vec<String>,
    pub skipped: Vec<String>,
    pub smart_merged: Vec<String>,
    pub errors: Vec<String>,
}

/// A file that differs between source and destination, pending user review.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileToReview {
    pub filename: String,
    pub src_path: PathBuf,
    pub dest_path: PathBuf,
    pub is_new: bool,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

// Re-export from shared location for backwards compatibility.
pub use super::ECC_PACKAGE_IDENTIFIERS;

// ---------------------------------------------------------------------------
// Pure functions
// ---------------------------------------------------------------------------

/// Create an empty merge report with all fields initialized to empty vectors.
pub fn empty_report() -> MergeReport {
    MergeReport {
        added: Vec::new(),
        updated: Vec::new(),
        unchanged: Vec::new(),
        skipped: Vec::new(),
        smart_merged: Vec::new(),
        errors: Vec::new(),
    }
}

/// Combine multiple merge reports into a single report by concatenating all fields.
pub fn combine_reports(reports: &[MergeReport]) -> MergeReport {
    MergeReport {
        added: reports.iter().flat_map(|r| r.added.clone()).collect(),
        updated: reports.iter().flat_map(|r| r.updated.clone()).collect(),
        unchanged: reports.iter().flat_map(|r| r.unchanged.clone()).collect(),
        skipped: reports.iter().flat_map(|r| r.skipped.clone()).collect(),
        smart_merged: reports
            .iter()
            .flat_map(|r| r.smart_merged.clone())
            .collect(),
        errors: reports.iter().flat_map(|r| r.errors.clone()).collect(),
    }
}

/// Check if a hook entry is a legacy ECC hook that should be removed.
///
/// Detects legacy hooks via:
/// 1. Absolute paths containing the ECC package identifier
/// 2. Old-style `scripts/hooks/` paths
/// 3. Unresolved placeholder commands
/// 4. Absolute-path `run-with-flags.js` / `run-with-flags-shell.sh`
/// 5. Inline `node -e` one-liners
///
/// Current `ecc-hook` / `ecc-shell-hook` wrapper commands are NOT flagged.
pub fn is_legacy_ecc_hook(entry: &serde_json::Value) -> bool {
    let hooks = match entry.get("hooks").and_then(|h| h.as_array()) {
        Some(h) => h,
        None => return false,
    };

    for hook in hooks {
        let cmd = match hook.get("command").and_then(|c| c.as_str()) {
            Some(c) => c,
            None => continue,
        };

        // Current wrapper commands are NOT legacy — unless they contain a
        // stale dist/hooks/ JS path from the Node.js era (3-arg format).
        if cmd.starts_with("ecc-hook ") || cmd.starts_with("ecc-shell-hook ") {
            if cmd.contains("dist/hooks/") {
                return true;
            }
            continue;
        }

        // Absolute path containing ECC package identifier
        for identifier in ECC_PACKAGE_IDENTIFIERS {
            if cmd.contains(identifier) {
                return true;
            }
        }

        // Old-style scripts/hooks/ direct paths
        if cmd.contains("scripts/hooks/") && !cmd.contains("run-with-flags-shell.sh") {
            return true;
        }

        // Unresolved placeholder commands
        if cmd.contains("${ECC_ROOT}") || cmd.contains("${CLAUDE_PLUGIN_ROOT}") {
            return true;
        }

        // Resolved absolute-path run-with-flags.js
        if cmd.contains("/dist/hooks/run-with-flags.js") {
            return true;
        }

        // Resolved absolute-path shell hook commands
        if cmd.contains("/scripts/hooks/run-with-flags-shell.sh") {
            return true;
        }

        // Inline node -e one-liners from pre-hook-runner era
        if cmd.contains("node -e")
            && (cmd.contains("dev-server")
                || cmd.contains("tmux")
                || cmd.contains("git push")
                || cmd.contains("console.log")
                || cmd.contains("check-console")
                || cmd.contains("pr-created")
                || cmd.contains("build-complete"))
        {
            return true;
        }
    }

    false
}

/// Remove legacy hooks from a hooks object.
/// Returns a new hooks value with legacy hooks removed, and the count of removed hooks.
pub fn remove_legacy_hooks(hooks: &serde_json::Value) -> (serde_json::Value, usize) {
    let obj = match hooks.as_object() {
        Some(o) => o,
        None => return (hooks.clone(), 0),
    };

    let mut result = serde_json::Map::new();
    let mut removed = 0usize;

    for (event, entries) in obj {
        let arr = match entries.as_array() {
            Some(a) => a,
            None => {
                result.insert(event.clone(), entries.clone());
                continue;
            }
        };

        let original_len = arr.len();
        let filtered: Vec<serde_json::Value> = arr
            .iter()
            .filter(|entry| !is_legacy_ecc_hook(entry))
            .cloned()
            .collect();
        removed += original_len - filtered.len();
        result.insert(event.clone(), serde_json::Value::Array(filtered));
    }

    (serde_json::Value::Object(result), removed)
}

/// Merge hooks from source into existing hooks.
///
/// Returns `(merged_hooks, added_count, existing_count, legacy_removed_count)`.
///
/// Steps:
/// 1. Remove legacy hooks from existing
/// 2. Add new hooks from source that are not already present (by serialized hooks key)
pub fn merge_hooks_pure(
    source_hooks: &serde_json::Value,
    existing_hooks: &serde_json::Value,
) -> (serde_json::Value, usize, usize, usize) {
    let (cleaned, legacy_removed) = remove_legacy_hooks(existing_hooks);

    let mut merged = match cleaned.as_object() {
        Some(o) => o.clone(),
        None => serde_json::Map::new(),
    };

    let source_obj = match source_hooks.as_object() {
        Some(o) => o,
        None => {
            return (serde_json::Value::Object(merged), 0, 0, legacy_removed);
        }
    };

    let mut added = 0usize;
    let mut already_present = 0usize;

    for (event, entries) in source_obj {
        let source_arr = match entries.as_array() {
            Some(a) => a,
            None => continue,
        };

        let existing_arr = merged
            .entry(event.clone())
            .or_insert_with(|| serde_json::Value::Array(Vec::new()));

        let existing_entries = match existing_arr.as_array_mut() {
            Some(a) => a,
            None => continue,
        };

        for entry in source_arr {
            let key = match entry.get("hooks") {
                Some(h) => serde_json::to_string(h).unwrap_or_default(),
                None => serde_json::to_string(entry).unwrap_or_default(),
            };

            let exists = existing_entries.iter().any(|e| {
                let existing_key = match e.get("hooks") {
                    Some(h) => serde_json::to_string(h).unwrap_or_default(),
                    None => serde_json::to_string(e).unwrap_or_default(),
                };
                existing_key == key
            });

            if exists {
                already_present += 1;
            } else {
                existing_entries.push(entry.clone());
                added += 1;
            }
        }
    }

    (
        serde_json::Value::Object(merged),
        added,
        already_present,
        legacy_removed,
    )
}

// ---------------------------------------------------------------------------
// Typed hook functions (using HookEntry / HooksMap)
// ---------------------------------------------------------------------------

/// Check if a typed hook entry is a legacy ECC hook.
pub fn is_legacy_ecc_hook_typed(entry: &super::hook_types::HookEntry) -> bool {
    let hooks = match &entry.hooks {
        Some(h) => h,
        None => return false,
    };

    for hook in hooks {
        let cmd = match &hook.command {
            Some(super::hook_types::HookCommandValue::Single(c)) => c.as_str(),
            _ => continue,
        };

        if cmd.starts_with("ecc-hook ") || cmd.starts_with("ecc-shell-hook ") {
            if cmd.contains("dist/hooks/") {
                return true;
            }
            continue;
        }

        for identifier in ECC_PACKAGE_IDENTIFIERS {
            if cmd.contains(identifier) {
                return true;
            }
        }

        if cmd.contains("scripts/hooks/") && !cmd.contains("run-with-flags-shell.sh") {
            return true;
        }

        if cmd.contains("${ECC_ROOT}") || cmd.contains("${CLAUDE_PLUGIN_ROOT}") {
            return true;
        }

        if cmd.contains("/dist/hooks/run-with-flags.js") {
            return true;
        }

        if cmd.contains("/scripts/hooks/run-with-flags-shell.sh") {
            return true;
        }

        if cmd.contains("node -e")
            && (cmd.contains("dev-server")
                || cmd.contains("tmux")
                || cmd.contains("git push")
                || cmd.contains("console.log")
                || cmd.contains("check-console")
                || cmd.contains("pr-created")
                || cmd.contains("build-complete"))
        {
            return true;
        }
    }

    false
}

/// Remove legacy hooks from a typed hooks map.
/// Returns a new map with legacy hooks removed, and the count of removed hooks.
pub fn remove_legacy_hooks_typed(
    hooks: &super::hook_types::HooksMap,
) -> (super::hook_types::HooksMap, usize) {
    let mut result = super::hook_types::HooksMap::new();
    let mut removed = 0usize;

    for (event, entries) in hooks {
        let original_len = entries.len();
        let filtered: Vec<super::hook_types::HookEntry> = entries
            .iter()
            .filter(|entry| !is_legacy_ecc_hook_typed(entry))
            .cloned()
            .collect();
        removed += original_len - filtered.len();
        result.insert(event.clone(), filtered);
    }

    (result, removed)
}

/// Merge hooks from source into existing hooks (typed version).
///
/// Returns `(merged_hooks, added_count, existing_count, legacy_removed_count)`.
pub fn merge_hooks_typed(
    source_hooks: &super::hook_types::HooksMap,
    existing_hooks: &super::hook_types::HooksMap,
) -> (super::hook_types::HooksMap, usize, usize, usize) {
    let (cleaned, legacy_removed) = remove_legacy_hooks_typed(existing_hooks);

    let mut merged = cleaned;
    let mut added = 0usize;
    let mut already_present = 0usize;

    for (event, source_entries) in source_hooks {
        let existing_entries = merged.entry(event.clone()).or_default();

        for entry in source_entries {
            if existing_entries.contains(entry) {
                already_present += 1;
            } else {
                existing_entries.push(entry.clone());
                added += 1;
            }
        }
    }

    (merged, added, already_present, legacy_removed)
}

/// Check if two strings differ after trimming whitespace.
pub fn contents_differ(a: &str, b: &str) -> bool {
    a.trim() != b.trim()
}

/// Format a merge report as a human-readable string.
pub fn format_merge_report(label: &str, report: &MergeReport) -> String {
    let mut parts = Vec::new();

    if !report.added.is_empty() {
        parts.push(format!("{} added", report.added.len()));
    }
    if !report.updated.is_empty() {
        parts.push(format!("{} updated", report.updated.len()));
    }
    if !report.unchanged.is_empty() {
        parts.push(format!("{} unchanged", report.unchanged.len()));
    }
    if !report.skipped.is_empty() {
        parts.push(format!("{} skipped", report.skipped.len()));
    }
    if !report.smart_merged.is_empty() {
        parts.push(format!("{} smart-merged", report.smart_merged.len()));
    }
    if !report.errors.is_empty() {
        parts.push(format!("{} errors", report.errors.len()));
    }

    if parts.is_empty() {
        format!("  {label}: (no changes)")
    } else {
        format!("  {label}: {}", parts.join(", "))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
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

    // --- is_legacy_ecc_hook ---

    #[test]
    fn is_legacy_ecc_hook_package_identifier_lebocqtitouan() {
        let entry = serde_json::json!({
            "hooks": [{"command": "/usr/lib/node_modules/@lebocqtitouan/ecc/dist/hooks/run.js"}]
        });
        assert!(is_legacy_ecc_hook(&entry));
    }

    #[test]
    fn is_legacy_ecc_hook_package_identifier_everything() {
        let entry = serde_json::json!({
            "hooks": [{"command": "/home/.npm/everything-claude-code/hooks/run.js"}]
        });
        assert!(is_legacy_ecc_hook(&entry));
    }

    #[test]
    fn is_legacy_ecc_hook_scripts_hooks() {
        let entry = serde_json::json!({
            "hooks": [{"command": "node scripts/hooks/check.js"}]
        });
        assert!(is_legacy_ecc_hook(&entry));
    }

    #[test]
    fn is_legacy_ecc_hook_ecc_root_placeholder() {
        let entry = serde_json::json!({
            "hooks": [{"command": "${ECC_ROOT}/hooks/run.js"}]
        });
        assert!(is_legacy_ecc_hook(&entry));
    }

    #[test]
    fn is_legacy_ecc_hook_claude_plugin_root() {
        let entry = serde_json::json!({
            "hooks": [{"command": "${CLAUDE_PLUGIN_ROOT}/hooks/run.js"}]
        });
        assert!(is_legacy_ecc_hook(&entry));
    }

    #[test]
    fn is_legacy_ecc_hook_run_with_flags() {
        let entry = serde_json::json!({
            "hooks": [{"command": "node /abs/path/dist/hooks/run-with-flags.js"}]
        });
        assert!(is_legacy_ecc_hook(&entry));
    }

    #[test]
    fn is_legacy_ecc_hook_shell_hook_path() {
        let entry = serde_json::json!({
            "hooks": [{"command": "bash /abs/path/scripts/hooks/run-with-flags-shell.sh"}]
        });
        assert!(is_legacy_ecc_hook(&entry));
    }

    #[test]
    fn is_legacy_ecc_hook_node_e_dev_server() {
        let entry = serde_json::json!({
            "hooks": [{"command": "node -e 'require(\"dev-server\")'"}]
        });
        assert!(is_legacy_ecc_hook(&entry));
    }

    #[test]
    fn is_legacy_ecc_hook_node_e_tmux() {
        let entry = serde_json::json!({
            "hooks": [{"command": "node -e 'tmux split'"}]
        });
        assert!(is_legacy_ecc_hook(&entry));
    }

    #[test]
    fn is_legacy_ecc_hook_node_e_build_complete() {
        let entry = serde_json::json!({
            "hooks": [{"command": "node -e 'build-complete()'"}]
        });
        assert!(is_legacy_ecc_hook(&entry));
    }

    #[test]
    fn is_legacy_ecc_hook_not_for_ecc_hook_wrapper() {
        let entry = serde_json::json!({
            "hooks": [{"command": "ecc-hook pre-tool-use format"}]
        });
        assert!(!is_legacy_ecc_hook(&entry));
    }

    #[test]
    fn is_legacy_ecc_hook_not_for_ecc_shell_hook_wrapper() {
        let entry = serde_json::json!({
            "hooks": [{"command": "ecc-shell-hook post-tool-use lint"}]
        });
        assert!(!is_legacy_ecc_hook(&entry));
    }

    #[test]
    fn is_legacy_ecc_hook_stale_3arg_ecc_hook_with_dist_path() {
        let entry = serde_json::json!({
            "hooks": [{"command": "ecc-hook \"pre:bash:dev-server-block\" \"dist/hooks/pre-bash-dev-server-block.js\" \"standard,strict\""}]
        });
        assert!(is_legacy_ecc_hook(&entry));
    }

    #[test]
    fn is_legacy_ecc_hook_stale_3arg_ecc_hook_post_tool() {
        let entry = serde_json::json!({
            "hooks": [{"command": "ecc-hook \"post:edit:format\" \"dist/hooks/post-edit-format.js\" \"standard,strict\""}]
        });
        assert!(is_legacy_ecc_hook(&entry));
    }

    #[test]
    fn is_legacy_ecc_hook_not_for_user_hook() {
        let entry = serde_json::json!({
            "hooks": [{"command": "my-custom-hook"}]
        });
        assert!(!is_legacy_ecc_hook(&entry));
    }

    #[test]
    fn is_legacy_ecc_hook_no_hooks_array() {
        let entry = serde_json::json!({"description": "test"});
        assert!(!is_legacy_ecc_hook(&entry));
    }

    // --- remove_legacy_hooks ---

    #[test]
    fn remove_legacy_hooks_removes_legacy() {
        let hooks = serde_json::json!({
            "PreToolUse": [
                {"hooks": [{"command": "ecc-hook format"}]},
                {"hooks": [{"command": "node scripts/hooks/old.js"}]}
            ]
        });

        let (result, removed) = remove_legacy_hooks(&hooks);
        assert_eq!(removed, 1);
        let arr = result["PreToolUse"].as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(
            arr[0]["hooks"][0]["command"].as_str().unwrap(),
            "ecc-hook format"
        );
    }

    #[test]
    fn remove_legacy_hooks_keeps_current() {
        let hooks = serde_json::json!({
            "PreToolUse": [
                {"hooks": [{"command": "ecc-hook format"}]},
                {"hooks": [{"command": "ecc-shell-hook lint"}]}
            ]
        });

        let (result, removed) = remove_legacy_hooks(&hooks);
        assert_eq!(removed, 0);
        let arr = result["PreToolUse"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn remove_legacy_hooks_empty() {
        let hooks = serde_json::json!({});
        let (result, removed) = remove_legacy_hooks(&hooks);
        assert_eq!(removed, 0);
        assert!(result.as_object().unwrap().is_empty());
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

        let (merged, added, existing_count, legacy) = merge_hooks_pure(&source, &existing);
        assert_eq!(added, 1);
        assert_eq!(existing_count, 0);
        assert_eq!(legacy, 0);
        assert_eq!(merged["PreToolUse"].as_array().unwrap().len(), 1);
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

        let (merged, added, existing_count, _) = merge_hooks_pure(&source, &existing);
        assert_eq!(added, 0);
        assert_eq!(existing_count, 1);
        assert_eq!(merged["PreToolUse"].as_array().unwrap().len(), 1);
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

        let (merged, added, _, legacy) = merge_hooks_pure(&source, &existing);
        assert_eq!(added, 1);
        assert_eq!(legacy, 1);
        assert_eq!(merged["PreToolUse"].as_array().unwrap().len(), 1);
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

        let (merged, added, _, legacy) = merge_hooks_pure(&source, &existing);
        assert_eq!(added, 1);
        assert_eq!(legacy, 0);
        // User hook + new hook
        assert_eq!(merged["PreToolUse"].as_array().unwrap().len(), 2);
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
}
