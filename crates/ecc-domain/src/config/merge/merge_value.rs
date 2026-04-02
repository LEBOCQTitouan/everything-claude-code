use super::MergeHooksPureResult;
use super::legacy::remove_legacy_hooks;

/// Merge source hook entries into existing entries for one event type.
///
/// Returns `(added, already_present)` counts.
fn merge_event_entries(
    source_arr: &[serde_json::Value],
    existing_entries: &mut Vec<serde_json::Value>,
) -> (usize, usize) {
    let mut added = 0usize;
    let mut already_present = 0usize;

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

    (added, already_present)
}

/// Merge hooks from source into existing hooks.
///
/// Steps:
/// 1. Remove legacy hooks from existing
/// 2. Add new hooks from source that are not already present (by serialized hooks key)
pub fn merge_hooks_pure(
    source_hooks: &serde_json::Value,
    existing_hooks: &serde_json::Value,
) -> MergeHooksPureResult {
    let (cleaned, legacy_removed) = remove_legacy_hooks(existing_hooks);

    let mut merged = match cleaned.as_object() {
        Some(o) => o.clone(),
        None => serde_json::Map::new(),
    };

    let source_obj = match source_hooks.as_object() {
        Some(o) => o,
        None => {
            return MergeHooksPureResult {
                merged: serde_json::Value::Object(merged),
                added: 0,
                existing: 0,
                legacy_removed,
            };
        }
    };

    let mut total_added = 0usize;
    let mut total_present = 0usize;

    for (event, entries) in source_obj {
        let Some(source_arr) = entries.as_array() else {
            continue;
        };
        let existing_arr = merged
            .entry(event.clone())
            .or_insert_with(|| serde_json::Value::Array(Vec::new()));
        let Some(existing_entries) = existing_arr.as_array_mut() else {
            continue;
        };
        let (added, present) = merge_event_entries(source_arr, existing_entries);
        total_added += added;
        total_present += present;
    }

    MergeHooksPureResult {
        merged: serde_json::Value::Object(merged),
        added: total_added,
        existing: total_present,
        legacy_removed,
    }
}
