//! Wave grouping — pure domain algorithm for deterministic PC-to-wave assignment.
//!
//! Zero I/O. All functions operate on borrowed slices and return owned types.

use crate::spec::ac::AcId;
use crate::spec::ordering::FileChange;
use crate::spec::pc::{PassCondition, PcId};
use serde::Serialize;
use std::collections::HashMap;

/// A single execution wave containing non-overlapping PCs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Wave {
    pub id: u16,
    pub pc_ids: Vec<PcId>,
    pub files: Vec<String>,
}

/// Complete wave plan: all waves plus metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WavePlan {
    pub waves: Vec<Wave>,
    pub total_pcs: usize,
    pub max_per_wave: usize,
}

/// Strips one leading and one trailing backtick from a string, then trims whitespace.
pub fn strip_backticks(s: &str) -> String {
    let trimmed = s.trim();
    let stripped = trimmed.strip_prefix('`').unwrap_or(trimmed);
    let stripped = stripped.strip_suffix('`').unwrap_or(stripped);
    stripped.trim().to_owned()
}

/// Build a `PcId -> Vec<String>` mapping from PCs and FileChanges via AC cross-reference.
///
/// Algorithm:
/// 1. For each FileChange, parse its `spec_ref` into AcId list (skip unparseable silently).
/// 2. For each PC, check if any `verifies_acs` matches any FileChange's parsed ACs.
/// 3. If match: add that FileChange's file (raw-trimmed) to the PC's list.
/// 4. Deduplicate files per PC.
/// 5. PCs with no matches get an empty Vec.
pub fn build_pc_file_map(
    pcs: &[PassCondition],
    file_changes: &[FileChange],
) -> HashMap<PcId, Vec<String>> {
    // Parse each FileChange's spec_ref into a list of AcIds (skip unparseable silently)
    let fc_acs: Vec<Vec<AcId>> = file_changes
        .iter()
        .map(|fc| parse_spec_ref(&fc.spec_ref))
        .collect();

    let mut map: HashMap<PcId, Vec<String>> = HashMap::new();

    for pc in pcs {
        let mut files: Vec<String> = Vec::new();
        for (fc, acs) in file_changes.iter().zip(fc_acs.iter()) {
            // Check if any of the PC's verifies_acs matches any of this FileChange's ACs
            let matches = pc.verifies_acs.iter().any(|pa| acs.contains(pa));
            if matches {
                let file = strip_backticks(&fc.file);
                if !files.contains(&file) {
                    files.push(file);
                }
            }
        }
        // Always insert — PCs with no matches get empty Vec (AC-001.4)
        map.insert(pc.id.clone(), files);
    }

    map
}

/// Parse a spec_ref string into a Vec<AcId>, silently skipping non-parseable parts.
fn parse_spec_ref(spec_ref: &str) -> Vec<AcId> {
    spec_ref
        .split([',', ' '])
        .filter_map(|part| AcId::parse(part.trim()).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::ac::AcId;
    use crate::spec::ordering::FileChange;
    use crate::spec::pc::{PassCondition, PcId};

    fn make_pc(id: u16, acs: Vec<AcId>) -> PassCondition {
        PassCondition {
            id: PcId(id),
            pc_type: String::new(),
            description: String::new(),
            verifies_acs: acs,
            command: String::new(),
            expected: String::new(),
        }
    }

    fn make_file_change(number: u16, file: &str, spec_ref: &str) -> FileChange {
        FileChange {
            number,
            file: file.to_owned(),
            action: "CREATE".to_owned(),
            spec_ref: spec_ref.to_owned(),
        }
    }

    fn ac(us: u16, sub: u16) -> AcId {
        AcId {
            us_number: us,
            sub_number: sub,
        }
    }

    // PC-001: build_pc_file_map returns correct mappings via AC cross-reference
    #[test]
    fn pc_file_map_basic_mapping() {
        let pcs = vec![make_pc(1, vec![ac(1, 1)])];
        let file_changes = vec![make_file_change(1, "src/foo.rs", "AC-001.1")];

        let map = build_pc_file_map(&pcs, &file_changes);

        assert_eq!(map.get(&PcId(1)), Some(&vec!["src/foo.rs".to_owned()]));
    }

    // PC-004: PC with no AC matches maps to empty Vec
    #[test]
    fn pc_file_map_no_match_empty() {
        let pcs = vec![make_pc(1, vec![ac(1, 1)])];
        let file_changes = vec![make_file_change(1, "src/foo.rs", "AC-002.1")];

        let map = build_pc_file_map(&pcs, &file_changes);

        assert_eq!(map.get(&PcId(1)), Some(&vec![]));
    }

    // PC-007: Non-parseable spec_ref silently skipped
    #[test]
    fn pc_file_map_non_parseable_ref() {
        let pcs = vec![make_pc(1, vec![ac(1, 1)])];
        // "US-001" is not a valid AC ref — should be silently skipped
        let file_changes = vec![make_file_change(1, "src/foo.rs", "US-001")];

        let map = build_pc_file_map(&pcs, &file_changes);

        // PC-001 has no matching AC, so maps to empty vec
        assert_eq!(map.get(&PcId(1)), Some(&vec![]));
        // The file change with unparseable ref contributes no files
        assert!(!map.values().any(|files| files.contains(&"src/foo.rs".to_owned())));
    }

    // PC-003: Backtick stripping on file paths
    #[test]
    fn pc_file_map_backtick_stripping() {
        let pcs = vec![make_pc(1, vec![ac(1, 1)])];
        // File path is wrapped in backticks as it would appear in a markdown table
        let file_changes = vec![make_file_change(1, "`src/foo.rs`", "AC-001.1")];

        let map = build_pc_file_map(&pcs, &file_changes);

        // Backticks should be stripped
        assert_eq!(map.get(&PcId(1)), Some(&vec!["src/foo.rs".to_owned()]));
    }

    // PC-002: Multi-AC spec_ref maps file to multiple PCs
    #[test]
    fn pc_file_map_multi_ac_ref() {
        let pc1 = make_pc(1, vec![ac(1, 1)]);
        let pc2 = make_pc(2, vec![ac(2, 1)]);
        let pcs = vec![pc1, pc2];
        // One file change references both AC-001.1 and AC-002.1
        let file_changes = vec![make_file_change(1, "src/shared.rs", "AC-001.1, AC-002.1")];

        let map = build_pc_file_map(&pcs, &file_changes);

        assert_eq!(map.get(&PcId(1)), Some(&vec!["src/shared.rs".to_owned()]));
        assert_eq!(map.get(&PcId(2)), Some(&vec!["src/shared.rs".to_owned()]));
    }

    // PC-005: Duplicate files deduplicated per PC
    #[test]
    fn pc_file_map_dedup() {
        let pcs = vec![make_pc(1, vec![ac(1, 1), ac(1, 2)])];
        // Two file changes for the same file, each referencing a different AC that PC-1 verifies
        let file_changes = vec![
            make_file_change(1, "src/foo.rs", "AC-001.1"),
            make_file_change(2, "src/foo.rs", "AC-001.2"),
        ];

        let map = build_pc_file_map(&pcs, &file_changes);

        let files = map.get(&PcId(1)).expect("PC-1 should be in map");
        assert_eq!(files.len(), 1, "duplicate file should be deduplicated");
        assert!(files.contains(&"src/foo.rs".to_owned()));
    }

    // PC-006: Duplicate file paths across FileChanges contribute both spec_refs
    #[test]
    fn pc_file_map_duplicate_file_paths() {
        // PC-1 verifies AC-001.1, PC-2 verifies AC-002.1
        let pc1 = make_pc(1, vec![ac(1, 1)]);
        let pc2 = make_pc(2, vec![ac(2, 1)]);
        let pcs = vec![pc1, pc2];
        // Same file path appears twice, with different spec_refs
        let file_changes = vec![
            make_file_change(1, "src/shared.rs", "AC-001.1"),
            make_file_change(2, "src/shared.rs", "AC-002.1"),
        ];

        let map = build_pc_file_map(&pcs, &file_changes);

        // Both PCs should include the shared file
        assert_eq!(map.get(&PcId(1)), Some(&vec!["src/shared.rs".to_owned()]));
        assert_eq!(map.get(&PcId(2)), Some(&vec!["src/shared.rs".to_owned()]));
    }
}
