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
/// 3. If match: add that FileChange's file (backtick-stripped, trimmed) to the PC's list.
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

/// Compute a wave plan from a list of PCs and file changes.
///
/// Algorithm: Non-adjacent greedy bin-packing.
/// For each PC in input order, place it in the first existing wave that:
/// - has fewer than `max_per_wave` PCs
/// - shares no files with the PC (empty file set never overlaps)
///
/// If no eligible wave exists, create a new wave.
///
/// Special case: `max_per_wave == 0` is treated as `max_per_wave == 1`.
pub fn compute_wave_plan(
    pcs: &[PassCondition],
    file_changes: &[FileChange],
    max_per_wave: usize,
) -> WavePlan {
    let effective_max = if max_per_wave == 0 { 1 } else { max_per_wave };
    let file_map = build_pc_file_map(pcs, file_changes);
    let mut waves: Vec<Wave> = Vec::new();

    for pc in pcs {
        let pc_files: Vec<String> = file_map.get(&pc.id).cloned().unwrap_or_default();

        // Find the first eligible wave
        let eligible = waves.iter_mut().find(|wave| {
            wave.pc_ids.len() < effective_max && !pc_files.iter().any(|f| wave.files.contains(f))
        });

        if let Some(wave) = eligible {
            wave.pc_ids.push(pc.id.clone());
            for f in &pc_files {
                if !wave.files.contains(f) {
                    wave.files.push(f.clone());
                }
            }
        } else {
            let id = u16::try_from(waves.len() + 1).unwrap_or(u16::MAX);
            waves.push(Wave {
                id,
                pc_ids: vec![pc.id.clone()],
                files: pc_files,
            });
        }
    }

    WavePlan {
        waves,
        total_pcs: pcs.len(),
        max_per_wave: effective_max,
    }
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
        assert!(
            !map.values()
                .any(|files| files.contains(&"src/foo.rs".to_owned()))
        );
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

    // ---------- compute_wave_plan tests ----------

    // PC-011: Empty PC list returns empty wave plan
    #[test]
    fn wave_empty_pcs() {
        let plan = compute_wave_plan(&[], &[], 4);
        assert_eq!(plan.waves, vec![]);
        assert_eq!(plan.total_pcs, 0);
        assert_eq!(plan.max_per_wave, 4);
    }

    // PC-010: Single PC produces one wave
    #[test]
    fn wave_single_pc() {
        let pcs = vec![make_pc(1, vec![ac(1, 1)])];
        let fcs = vec![make_file_change(1, "a.rs", "AC-001.1")];
        let plan = compute_wave_plan(&pcs, &fcs, 4);
        assert_eq!(plan.waves.len(), 1);
        assert_eq!(plan.waves[0].pc_ids, vec![PcId(1)]);
        assert_eq!(plan.total_pcs, 1);
    }

    // PC-008: No-overlap PCs grouped in waves of max 4 (non-adjacent)
    #[test]
    fn wave_no_overlap_max_four() {
        // 4 independent PCs (each touches a unique file) -> all in one wave
        let pcs = vec![
            make_pc(1, vec![ac(1, 1)]),
            make_pc(2, vec![ac(2, 1)]),
            make_pc(3, vec![ac(3, 1)]),
            make_pc(4, vec![ac(4, 1)]),
        ];
        let fcs = vec![
            make_file_change(1, "a.rs", "AC-001.1"),
            make_file_change(2, "b.rs", "AC-002.1"),
            make_file_change(3, "c.rs", "AC-003.1"),
            make_file_change(4, "d.rs", "AC-004.1"),
        ];
        let plan = compute_wave_plan(&pcs, &fcs, 4);
        assert_eq!(
            plan.waves.len(),
            1,
            "4 independent PCs should fit in one wave"
        );
        assert_eq!(
            plan.waves[0].pc_ids,
            vec![PcId(1), PcId(2), PcId(3), PcId(4)]
        );
    }

    // PC-013: >4 independent PCs split into multiple waves
    #[test]
    fn wave_split_over_four() {
        // 5 independent PCs -> wave1 has 4, wave2 has 1
        let pcs = vec![
            make_pc(1, vec![ac(1, 1)]),
            make_pc(2, vec![ac(2, 1)]),
            make_pc(3, vec![ac(3, 1)]),
            make_pc(4, vec![ac(4, 1)]),
            make_pc(5, vec![ac(5, 1)]),
        ];
        let fcs = vec![
            make_file_change(1, "a.rs", "AC-001.1"),
            make_file_change(2, "b.rs", "AC-002.1"),
            make_file_change(3, "c.rs", "AC-003.1"),
            make_file_change(4, "d.rs", "AC-004.1"),
            make_file_change(5, "e.rs", "AC-005.1"),
        ];
        let plan = compute_wave_plan(&pcs, &fcs, 4);
        assert_eq!(plan.waves.len(), 2);
        assert_eq!(plan.waves[0].pc_ids.len(), 4);
        assert_eq!(plan.waves[1].pc_ids.len(), 1);
    }

    // PC-014: max_per_wave parameter respected
    #[test]
    fn wave_custom_max() {
        // 6 independent PCs with max_per_wave=2 -> 3 waves of 2
        let pcs: Vec<PassCondition> = (1u16..=6).map(|i| make_pc(i, vec![ac(i, 1)])).collect();
        let fcs: Vec<FileChange> = (1u16..=6)
            .map(|i| make_file_change(i, &format!("f{i}.rs"), &format!("AC-{i:03}.1")))
            .collect();
        let plan = compute_wave_plan(&pcs, &fcs, 2);
        assert_eq!(plan.waves.len(), 3);
        assert!(plan.waves.iter().all(|w| w.pc_ids.len() <= 2));
        assert_eq!(plan.max_per_wave, 2);
    }

    // PC-015: Input order preserved: 5 independent PCs produce [A,B,C,D] then [E]
    #[test]
    fn wave_preserves_input_order() {
        let pcs = vec![
            make_pc(1, vec![ac(1, 1)]),
            make_pc(2, vec![ac(2, 1)]),
            make_pc(3, vec![ac(3, 1)]),
            make_pc(4, vec![ac(4, 1)]),
            make_pc(5, vec![ac(5, 1)]),
        ];
        let fcs = vec![
            make_file_change(1, "a.rs", "AC-001.1"),
            make_file_change(2, "b.rs", "AC-002.1"),
            make_file_change(3, "c.rs", "AC-003.1"),
            make_file_change(4, "d.rs", "AC-004.1"),
            make_file_change(5, "e.rs", "AC-005.1"),
        ];
        let plan = compute_wave_plan(&pcs, &fcs, 4);
        assert_eq!(plan.waves.len(), 2);
        assert_eq!(
            plan.waves[0].pc_ids,
            vec![PcId(1), PcId(2), PcId(3), PcId(4)]
        );
        assert_eq!(plan.waves[1].pc_ids, vec![PcId(5)]);
    }

    // PC-009: All-overlap PCs fully sequential (one per wave)
    #[test]
    fn wave_all_overlap_sequential() {
        // 3 PCs all touching the same file -> must each be in their own wave
        let pcs = vec![
            make_pc(1, vec![ac(1, 1)]),
            make_pc(2, vec![ac(1, 1)]),
            make_pc(3, vec![ac(1, 1)]),
        ];
        let fcs = vec![make_file_change(1, "shared.rs", "AC-001.1")];
        let plan = compute_wave_plan(&pcs, &fcs, 4);
        assert_eq!(plan.waves.len(), 3, "each overlapping PC gets its own wave");
        for (i, wave) in plan.waves.iter().enumerate() {
            assert_eq!(wave.pc_ids.len(), 1, "wave {i} should have exactly 1 PC");
        }
    }

    // PC-012: Non-adjacent grouping: A(file1) and D(file3) share wave despite B(file1,file2) between
    #[test]
    fn wave_non_adjacent_grouping() {
        // A: file1, B: file1+file2, C: file2, D: file3
        // A and D share no files -> they can be in the same wave (non-adjacent)
        let pc_a = make_pc(1, vec![ac(1, 1)]);
        let pc_b = make_pc(2, vec![ac(1, 1), ac(2, 1)]);
        let pc_c = make_pc(3, vec![ac(2, 1)]);
        let pc_d = make_pc(4, vec![ac(3, 1)]);
        let pcs = vec![pc_a, pc_b, pc_c, pc_d];
        let fcs = vec![
            make_file_change(1, "file1.rs", "AC-001.1"),
            make_file_change(2, "file2.rs", "AC-002.1"),
            make_file_change(3, "file3.rs", "AC-003.1"),
        ];
        let plan = compute_wave_plan(&pcs, &fcs, 4);

        // A is in wave 1. B overlaps A -> wave 2. C overlaps B -> wave 3.
        // D has no overlap with wave 1 (file1 only), so D joins wave 1.
        let wave1 = &plan.waves[0];
        assert!(
            wave1.pc_ids.contains(&PcId(1)) && wave1.pc_ids.contains(&PcId(4)),
            "A and D should be in the same wave (non-adjacent grouping)"
        );
    }

    // PC-016: Identical file sets assigned to different waves
    #[test]
    fn wave_identical_files_different_waves() {
        // 3 PCs all touching the same unique file -> each in a separate wave
        // AC-002.9: identical file sets -> different waves, earlier-indexed first
        let pcs = vec![
            make_pc(1, vec![ac(1, 1)]),
            make_pc(2, vec![ac(1, 1)]),
            make_pc(3, vec![ac(1, 1)]),
        ];
        let fcs = vec![make_file_change(1, "same.rs", "AC-001.1")];
        let plan = compute_wave_plan(&pcs, &fcs, 4);
        assert_eq!(plan.waves.len(), 3);
        assert_eq!(plan.waves[0].pc_ids, vec![PcId(1)]);
        assert_eq!(plan.waves[1].pc_ids, vec![PcId(2)]);
        assert_eq!(plan.waves[2].pc_ids, vec![PcId(3)]);
    }

    // PC-022: max_per_wave=1 produces fully sequential waves (one PC per wave)
    #[test]
    fn wave_max_one_sequential() {
        let pcs = vec![
            make_pc(1, vec![ac(1, 1)]),
            make_pc(2, vec![ac(2, 1)]),
            make_pc(3, vec![ac(3, 1)]),
        ];
        let fcs = vec![
            make_file_change(1, "a.rs", "AC-001.1"),
            make_file_change(2, "b.rs", "AC-002.1"),
            make_file_change(3, "c.rs", "AC-003.1"),
        ];
        let plan = compute_wave_plan(&pcs, &fcs, 1);
        assert_eq!(plan.waves.len(), 3, "max_per_wave=1 means one PC per wave");
        for wave in &plan.waves {
            assert_eq!(wave.pc_ids.len(), 1);
        }
        assert_eq!(plan.max_per_wave, 1);
    }

    // PC-023: max_per_wave=0 treated as max_per_wave=1 (no panic)
    #[test]
    fn wave_max_zero_safe() {
        let pcs = vec![make_pc(1, vec![ac(1, 1)]), make_pc(2, vec![ac(2, 1)])];
        let fcs = vec![
            make_file_change(1, "a.rs", "AC-001.1"),
            make_file_change(2, "b.rs", "AC-002.1"),
        ];
        // max_per_wave=0 must not panic and must behave like max_per_wave=1
        let plan = compute_wave_plan(&pcs, &fcs, 0);
        assert_eq!(
            plan.waves.len(),
            2,
            "max_per_wave=0 treated as 1 -> one PC per wave"
        );
        assert_eq!(plan.max_per_wave, 1);
    }
}
