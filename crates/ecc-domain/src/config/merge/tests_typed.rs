use super::*;
use super::super::hook_types;

fn typed_entry(cmd: &str) -> hook_types::HookEntry {
    hook_types::HookEntry {
        description: None,
        matcher: None,
        hooks: Some(vec![hook_types::HookCommand {
            hook_type: Some("command".to_string()),
            command: Some(hook_types::HookCommandValue::Single(cmd.to_string())),
            r#async: None,
            timeout: None,
        }]),
    }
}

fn typed_map(event: &str, entries: Vec<hook_types::HookEntry>) -> hook_types::HooksMap {
    let mut map = hook_types::HooksMap::new();
    map.insert(event.to_string(), entries);
    map
}

// --- merge_hooks_typed ---

#[test]
fn merge_hooks_typed_empty_both() {
    let source = hook_types::HooksMap::new();
    let existing = hook_types::HooksMap::new();
    let result = merge_hooks_typed(&source, &existing);
    assert!(result.merged.is_empty());
    assert_eq!(result.added, 0);
    assert_eq!(result.existing, 0);
    assert_eq!(result.legacy_removed, 0);
}

#[test]
fn merge_hooks_typed_adds_new_to_empty() {
    let source = typed_map("PreToolUse", vec![
        typed_entry("ecc-hook format"),
        typed_entry("ecc-hook lint"),
    ]);
    let existing = hook_types::HooksMap::new();
    let result = merge_hooks_typed(&source, &existing);
    assert_eq!(result.added, 2);
    assert_eq!(result.existing, 0);
    assert_eq!(result.legacy_removed, 0);
    assert_eq!(result.merged["PreToolUse"].len(), 2);
}

#[test]
fn merge_hooks_typed_deduplicates() {
    let entry = typed_entry("ecc-hook format");
    let source = typed_map("PreToolUse", vec![entry.clone()]);
    let existing = typed_map("PreToolUse", vec![entry]);
    let result = merge_hooks_typed(&source, &existing);
    assert_eq!(result.added, 0);
    assert_eq!(result.existing, 1);
    assert_eq!(result.merged["PreToolUse"].len(), 1);
}

#[test]
fn merge_hooks_typed_removes_legacy() {
    let source = typed_map("PreToolUse", vec![typed_entry("ecc-hook format")]);
    let existing = typed_map("PreToolUse", vec![
        typed_entry("node scripts/hooks/old.js"),
    ]);
    let result = merge_hooks_typed(&source, &existing);
    assert_eq!(result.added, 1);
    assert_eq!(result.legacy_removed, 1);
    assert_eq!(result.merged["PreToolUse"].len(), 1);
    assert_eq!(
        result.merged["PreToolUse"][0],
        typed_entry("ecc-hook format")
    );
}

// --- remove_legacy_hooks_typed ---

#[test]
fn remove_legacy_hooks_typed() {
    let mut hooks = hook_types::HooksMap::new();
    hooks.insert("PreToolUse".to_string(), vec![
        // current — keep
        typed_entry("ecc-hook format"),
        // legacy: absolute path with ECC package identifier
        typed_entry("/home/.npm/everything-claude-code/hooks/run.js"),
        // legacy: scripts/hooks/ path
        typed_entry("node scripts/hooks/old.js"),
        // legacy: placeholder
        typed_entry("${ECC_ROOT}/hooks/run.js"),
        // legacy: run-with-flags.js
        typed_entry("node /abs/path/dist/hooks/run-with-flags.js"),
        // legacy: node -e one-liner
        typed_entry("node -e 'require(\"dev-server\")'"),
    ]);

    let (result, removed) = super::remove_legacy_hooks_typed(&hooks);
    assert_eq!(removed, 5);
    assert_eq!(result["PreToolUse"].len(), 1);
    assert_eq!(result["PreToolUse"][0], typed_entry("ecc-hook format"));
}

// --- BL-085: typed worktree hook detection ---

#[test]
fn is_legacy_ecc_hook_typed_worktree_create_init() {
    let entry = hook_types::HookEntry {
        description: None,
        matcher: None,
        hooks: Some(vec![hook_types::HookCommand {
            hook_type: Some("command".to_string()),
            command: Some(hook_types::HookCommandValue::Single(
                "ecc-hook \"worktree:create:init\" \"standard,strict\"".to_string(),
            )),
            r#async: None,
            timeout: None,
        }]),
    };
    assert!(is_legacy_ecc_hook_typed(&entry));
}

#[test]
fn is_legacy_ecc_hook_typed_worktree_cleanup_reminder() {
    let entry = hook_types::HookEntry {
        description: None,
        matcher: None,
        hooks: Some(vec![hook_types::HookCommand {
            hook_type: Some("command".to_string()),
            command: Some(hook_types::HookCommandValue::Single(
                "ecc-hook \"stop:worktree-cleanup-reminder\" \"standard,strict\"".to_string(),
            )),
            r#async: None,
            timeout: None,
        }]),
    };
    assert!(is_legacy_ecc_hook_typed(&entry));
}

// --- merge_hooks_result_struct (typed) ---

#[test]
fn merge_hooks_result_struct_typed_has_fields() {
    let source = hook_types::HooksMap::new();
    let existing = hook_types::HooksMap::new();

    let result = merge_hooks_typed(&source, &existing);
    assert_eq!(result.added, 0);
    assert_eq!(result.existing, 0);
    assert_eq!(result.legacy_removed, 0);
    assert!(result.merged.is_empty());
}
