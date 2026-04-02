use super::super::hook_types;
use super::MergeHooksTypedResult;
use super::legacy::remove_legacy_hooks_typed;

/// Merge hooks from source into existing hooks (typed version).
pub fn merge_hooks_typed(
    source_hooks: &hook_types::HooksMap,
    existing_hooks: &hook_types::HooksMap,
) -> MergeHooksTypedResult {
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

    MergeHooksTypedResult {
        merged,
        added,
        existing: already_present,
        legacy_removed,
    }
}
