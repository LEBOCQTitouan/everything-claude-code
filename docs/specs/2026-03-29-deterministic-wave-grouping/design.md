# Design: Deterministic Wave Grouping Algorithm (BL-070)

## Architecture Decision

**ADR-032**: Wave grouping moves from an LLM heuristic (5-10s, non-deterministic) to a compiled Rust algorithm in `ecc-domain::spec::wave` with a CLI adapter in `ecc-workflow`. The algorithm uses non-adjacent greedy bin-packing: for each PC in order, try to place it in the first existing wave where (a) no file overlap exists and (b) wave size < max_per_wave. If no wave fits, create a new wave. PCs with no file matches are treated as independent (empty file set, no overlap with anything).

## File Changes

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/spec/wave.rs` | CREATE | Pure domain types (Wave, WavePlan) and functions (build_pc_file_map, compute_wave_plan, strip_backticks). Zero I/O. | AC-001.1, AC-001.2, AC-001.3, AC-001.4, AC-001.5, AC-001.6, AC-001.7, AC-002.1, AC-002.2, AC-002.3, AC-002.4, AC-002.5, AC-002.6, AC-002.7, AC-002.8, AC-002.9 |
| 2 | `crates/ecc-domain/src/spec/mod.rs` | MODIFY | Add `pub mod wave;` and re-export public types. | AC-001.1 |
| 3 | `crates/ecc-workflow/src/commands/wave_plan.rs` | CREATE | CLI adapter: read design file, parse tables, call domain, output JSON. Path validation via canonicalize+starts_with. | AC-003.1, AC-003.2, AC-003.3, AC-003.4, AC-003.5, AC-003.6 |
| 4 | `crates/ecc-workflow/src/commands/mod.rs` | MODIFY | Add `pub mod wave_plan;` | AC-003.1 |
| 5 | `crates/ecc-workflow/src/main.rs` | MODIFY | Add `WavePlan` variant to `Commands` enum and dispatch. | AC-003.1 |
| 6 | `skills/wave-analysis/SKILL.md` | MODIFY | Replace manual algorithm with `ecc-workflow wave-plan` CLI call. | AC-004.3 |
| 7 | `commands/implement.md` | MODIFY | Phase 2 calls `ecc-workflow wave-plan`, Phase 3 uses output. | AC-004.1, AC-004.2, AC-004.4 |

## Domain Types (File 1: `wave.rs`)

```rust
/// A single execution wave containing non-overlapping PCs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Wave {
    pub id: u16,                    // 1-based
    pub pc_ids: Vec<PcId>,          // PCs in this wave, input-order preserved
    pub files: Vec<String>,         // union of files touched by PCs in this wave
}

/// Complete wave plan: all waves plus metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WavePlan {
    pub waves: Vec<Wave>,
    pub total_pcs: usize,
    pub max_per_wave: usize,
}
```

## Algorithm Detail

### `strip_backticks(s: &str) -> String`

Strips leading/trailing backticks from a string. Used to normalize file paths from markdown tables.

### `build_pc_file_map(pcs: &[PassCondition], file_changes: &[FileChange]) -> HashMap<PcId, Vec<String>>`

1. For each `FileChange`, parse its `spec_ref` into a list of `AcId` values (reuse `AcId::parse`, skip unparseable refs silently per AC-001.7).
2. For each PC, check if any of its `verifies_acs` matches any parsed AC from any FileChange.
3. If match: add that FileChange's file (backtick-stripped, trimmed) to the PC's file list.
4. Deduplicate files per PC (AC-001.5).
5. PCs with no matches get an empty Vec (AC-001.4).

### `compute_wave_plan(pcs: &[PassCondition], file_changes: &[FileChange], max_per_wave: usize) -> WavePlan`

1. Call `build_pc_file_map` to get the mapping.
2. Initialize empty `waves: Vec<Wave>`.
3. For each PC in input order:
   a. Collect the PC's files from the map (empty set if no entry).
   b. Scan existing waves in order. A wave is eligible if:
      - `wave.pc_ids.len() < max_per_wave`
      - No file in the PC's file set appears in `wave.files` (empty file set never overlaps)
   c. If an eligible wave is found, add the PC to it (append to `pc_ids`, extend `files`).
   d. If no eligible wave is found, create a new wave with this PC.
4. Return `WavePlan { waves, total_pcs: pcs.len(), max_per_wave }`.

### Determinism guarantee

Same input always produces the same output: PCs are processed in input order, waves are scanned in creation order, and the first eligible wave is chosen. No randomness, no concurrency.

## CLI Adapter (File 3: `wave_plan.rs`)

### Command signature

```
ecc-workflow wave-plan <design-path>
```

### Logic

1. **Path validation**: Resolve `design-path` via `canonicalize`. Check `starts_with(project_dir.canonicalize())`. If traversal detected, exit with `block`.
2. **Read file**: If file doesn't exist, exit with `block` ("design file not found").
3. **Parse PC table**: Call `parse_pcs(content)`. If `Err(NoPassConditions)`, exit with `block` ("no PC table found").
4. **Parse File Changes table**: Call `parse_file_changes(content)`. If empty (warnings only), set `status: "warn"` but continue with empty file changes (AC-003.6).
5. **Compute wave plan**: Call `compute_wave_plan(&pcs, &file_changes, 4)`.
6. **Output JSON**: Serialize `WavePlanOutput` to stdout. Exit 0.

### JSON Output Schema

```json
{
  "status": "pass",
  "waves": [
    {
      "id": 1,
      "pcs": ["PC-001", "PC-002", "PC-003", "PC-004"],
      "files": ["src/foo.rs", "src/bar.rs"]
    },
    {
      "id": 2,
      "pcs": ["PC-005"],
      "files": ["src/foo.rs"]
    }
  ],
  "total_pcs": 5,
  "max_per_wave": 4
}
```

On warn (no File Changes table):
```json
{
  "status": "warn",
  "waves": [...],
  "total_pcs": 5,
  "max_per_wave": 4,
  "warnings": ["no File Changes table found — all PCs treated as independent"]
}
```

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | build_pc_file_map returns correct mappings via AC cross-reference | AC-001.1 | `cargo test -p ecc-domain spec::wave::tests::pc_file_map_basic_mapping` | PASS |
| PC-002 | unit | Multi-AC spec_ref maps file to multiple PCs | AC-001.2 | `cargo test -p ecc-domain spec::wave::tests::pc_file_map_multi_ac_ref` | PASS |
| PC-003 | unit | Backtick stripping on file paths | AC-001.3 | `cargo test -p ecc-domain spec::wave::tests::pc_file_map_backtick_stripping` | PASS |
| PC-004 | unit | PC with no AC matches maps to empty Vec | AC-001.4 | `cargo test -p ecc-domain spec::wave::tests::pc_file_map_no_match_empty` | PASS |
| PC-005 | unit | Duplicate files deduplicated per PC | AC-001.5 | `cargo test -p ecc-domain spec::wave::tests::pc_file_map_dedup` | PASS |
| PC-006 | unit | Duplicate file paths across FileChanges contribute both spec_refs | AC-001.6 | `cargo test -p ecc-domain spec::wave::tests::pc_file_map_duplicate_file_paths` | PASS |
| PC-007 | unit | Non-parseable spec_ref silently skipped | AC-001.7 | `cargo test -p ecc-domain spec::wave::tests::pc_file_map_non_parseable_ref` | PASS |
| PC-008 | unit | No-overlap PCs grouped in waves of max 4 (non-adjacent) | AC-002.1 | `cargo test -p ecc-domain spec::wave::tests::wave_no_overlap_max_four` | PASS |
| PC-009 | unit | All-overlap PCs fully sequential (one per wave) | AC-002.2 | `cargo test -p ecc-domain spec::wave::tests::wave_all_overlap_sequential` | PASS |
| PC-010 | unit | Single PC produces one wave | AC-002.3 | `cargo test -p ecc-domain spec::wave::tests::wave_single_pc` | PASS |
| PC-011 | unit | Empty PC list returns empty wave plan | AC-002.4 | `cargo test -p ecc-domain spec::wave::tests::wave_empty_pcs` | PASS |
| PC-012 | unit | Non-adjacent grouping: A(file1) and D(file3) share wave despite B(file1,file2) between | AC-002.5 | `cargo test -p ecc-domain spec::wave::tests::wave_non_adjacent_grouping` | PASS |
| PC-013 | unit | >4 independent PCs split into multiple waves | AC-002.6 | `cargo test -p ecc-domain spec::wave::tests::wave_split_over_four` | PASS |
| PC-014 | unit | max_per_wave parameter respected | AC-002.7 | `cargo test -p ecc-domain spec::wave::tests::wave_custom_max` | PASS |
| PC-015 | unit | Input order preserved: 5 independent PCs produce [A,B,C,D] then [E] | AC-002.8 | `cargo test -p ecc-domain spec::wave::tests::wave_preserves_input_order` | PASS |
| PC-016 | unit | Identical file sets assigned to different waves | AC-002.9 | `cargo test -p ecc-domain spec::wave::tests::wave_identical_files_different_waves` | PASS |
| PC-017 | integration | wave-plan outputs valid JSON with waves array for valid design | AC-003.1, AC-003.2 | `cargo test -p ecc-workflow commands::wave_plan::tests::valid_design_json_output` | PASS |
| PC-018 | integration | wave-plan exits block when no PC table | AC-003.3 | `cargo test -p ecc-workflow commands::wave_plan::tests::no_pc_table_blocks` | PASS |
| PC-019 | integration | wave-plan exits block for nonexistent path | AC-003.4 | `cargo test -p ecc-workflow commands::wave_plan::tests::nonexistent_path_blocks` | PASS |
| PC-020 | integration | wave-plan rejects path traversal | AC-003.5 | `cargo test -p ecc-workflow commands::wave_plan::tests::path_traversal_rejected` | PASS |
| PC-021 | integration | wave-plan warns with default waves when no File Changes table | AC-003.6 | `cargo test -p ecc-workflow commands::wave_plan::tests::no_file_changes_warns` | PASS |
| PC-022 | unit | max_per_wave=1 produces fully sequential waves (one PC per wave) | AC-002.7 | `cargo test -p ecc-domain spec::wave::tests::wave_max_one_sequential` | PASS |
| PC-023 | unit | max_per_wave=0 treated as max_per_wave=1 (no panic) | AC-002.7 | `cargo test -p ecc-domain spec::wave::tests::wave_max_zero_safe` | PASS |
| PC-024 | lint | All code passes clippy with zero warnings | — | `cargo clippy -p ecc-domain -p ecc-workflow -- -D warnings` | exit 0 |
| PC-025 | build | Full workspace builds successfully | — | `cargo build --workspace` | exit 0 |

## TDD Order

| Order | PC | Rationale |
|-------|-----|-----------|
| 1 | PC-001 | Foundation: basic PC-to-file mapping |
| 2 | PC-004 | Edge case: no-match PC (tests empty Vec path) |
| 3 | PC-007 | Edge case: non-parseable spec_ref (tests skip path) |
| 4 | PC-003 | Backtick stripping (needed before multi-AC tests) |
| 5 | PC-002 | Multi-AC cross-reference (builds on basic mapping) |
| 6 | PC-005 | Deduplication (builds on multi-AC) |
| 7 | PC-006 | Duplicate file paths (builds on dedup) |
| 8 | PC-011 | Wave algorithm: empty input (simplest case) |
| 9 | PC-010 | Wave algorithm: single PC |
| 10 | PC-008 | Wave algorithm: multiple independent PCs |
| 11 | PC-013 | Wave algorithm: >4 independent PCs split |
| 12 | PC-014 | Wave algorithm: custom max_per_wave |
| 13 | PC-015 | Wave algorithm: input order preservation |
| 14 | PC-009 | Wave algorithm: all-overlap sequential |
| 15 | PC-012 | Wave algorithm: non-adjacent grouping (key differentiator) |
| 16 | PC-016 | Wave algorithm: identical file sets |
| 17 | PC-017 | CLI integration: valid design produces JSON |
| 18 | PC-019 | CLI integration: nonexistent path |
| 19 | PC-020 | CLI integration: path traversal |
| 20 | PC-018 | CLI integration: no PC table |
| 21 | PC-021 | CLI integration: no File Changes (warn path) |
| 22 | PC-022 | Wave algorithm: max_per_wave=1 fully sequential |
| 23 | PC-023 | Wave algorithm: max_per_wave=0 safe handling |
| 24 | PC-024 | Lint gate |
| 25 | PC-025 | Build gate |

## Implementation Phases

### Phase 1: Domain — PC-to-Files Mapping (File 1, 2)
Layers: [Entity]

Create `wave.rs` with `strip_backticks`, `build_pc_file_map`, types `Wave` and `WavePlan`. Add `pub mod wave` to `mod.rs`. TDD: PC-001 through PC-007.

Commit cadence:
1. `test: add build_pc_file_map unit tests (PC-001..PC-007)` (RED)
2. `feat: implement build_pc_file_map with backtick stripping` (GREEN)
3. `refactor: improve build_pc_file_map` (REFACTOR, if applicable)

### Phase 2: Domain — Wave Grouping Algorithm (File 1)
Layers: [Entity]

Add `compute_wave_plan` to `wave.rs`. TDD: PC-008 through PC-016.

Commit cadence:
1. `test: add compute_wave_plan unit tests (PC-008..PC-016)` (RED)
2. `feat: implement compute_wave_plan greedy bin-packing` (GREEN)
3. `refactor: improve compute_wave_plan` (REFACTOR, if applicable)

### Phase 3: CLI Adapter — wave-plan Subcommand (Files 3, 4, 5)
Layers: [Adapter]

Create `wave_plan.rs` command handler, register in `mod.rs` and `main.rs`. TDD: PC-017 through PC-021.

Commit cadence:
1. `test: add wave-plan subcommand tests (PC-017..PC-021)` (RED)
2. `feat: implement wave-plan subcommand with path validation` (GREEN)
3. `refactor: improve wave_plan command` (REFACTOR, if applicable)

### Phase 4: Skill and Command Integration (Files 6, 7)
Layers: [Framework]

Update `wave-analysis/SKILL.md` to reference CLI. Update `implement.md` Phase 2/3. No automated tests (markdown content, verified by PC-022/PC-023 gates).

Commit cadence:
1. `docs: update wave-analysis skill to use ecc-workflow wave-plan`
2. `docs: update implement command to use wave-plan output`

### Phase 5: Lint and Build Gates (PC-022, PC-023)

Run lint and build to verify no regressions across the workspace. These are final gates, not TDD phases.

## E2E Assessment

- **Touches user-facing flows?** Yes -- `ecc-workflow wave-plan` is a new CLI subcommand
- **Crosses 3+ modules end-to-end?** No -- only ecc-domain and ecc-workflow
- **New E2E tests needed?** No -- PC-017 through PC-021 cover the CLI integration path via in-process tests. Existing E2E suite will be run as a gate after all phases.

## Risks and Mitigations

- **Risk**: `find_pcs_for_file_change` is private in `ordering.rs` and cannot be reused directly.
  - Mitigation: Re-implement the AC cross-reference logic in `wave.rs` using the same pattern (parse spec_ref, match against PC verifies_acs). The logic is simple enough that duplication is acceptable; a future backlog item can extract a shared helper.

- **Risk**: Backtick stripping may miss edge cases (nested backticks, code spans).
  - Mitigation: `strip_backticks` only strips one leading and one trailing backtick character. This matches the markdown table pattern exactly. Test with various inputs including no backticks, single backticks, and multiple backticks.

- **Risk**: Path validation via `canonicalize` fails on nonexistent files.
  - Mitigation: Check file existence first, then canonicalize. If the file doesn't exist, exit with block immediately (AC-003.4 handles this case before canonicalize is needed).

## Success Criteria

- [ ] `build_pc_file_map` correctly maps PCs to files via AC cross-reference (PC-001..PC-007)
- [ ] `compute_wave_plan` produces deterministic wave groups with non-adjacent bin-packing (PC-008..PC-016)
- [ ] `ecc-workflow wave-plan` outputs valid JSON and handles all error paths (PC-017..PC-021)
- [ ] Same input always produces the same output (determinism)
- [ ] 100% branch coverage on domain logic in `wave.rs`
- [ ] `cargo clippy -- -D warnings` passes (PC-024)
- [ ] `cargo build --workspace` succeeds (PC-025)
- [ ] `wave-analysis` skill references CLI, not manual algorithm (AC-004.3)
- [ ] `/implement` command updated to use `wave-plan` output (AC-004.1, AC-004.2)

## Rollback Plan

Reverse dependency order for safe revert:
1. Revert `commands/implement.md` (restore manual wave analysis in Phase 2)
2. Revert `skills/wave-analysis/SKILL.md` (restore manual algorithm description)
3. Revert `crates/ecc-workflow/src/commands/mod.rs` (remove `pub mod wave_plan`)
4. Revert `crates/ecc-workflow/src/main.rs` (remove WavePlan variant)
5. Delete `crates/ecc-workflow/src/commands/wave_plan.rs`
6. Revert `crates/ecc-domain/src/spec/mod.rs` (remove `pub mod wave`)
7. Delete `crates/ecc-domain/src/spec/wave.rs`

Note: Reverting steps 1-2 alone restores LLM-based wave grouping while keeping the domain code as inert dead code.

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | PASS | 2 LOW |
| Robert | CLEAN | 0 |
| Security | CLEAR | 0 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Coverage | 92 | PASS | All 26 ACs covered by 25 PCs |
| Order | 88 | PASS | TDD dependencies respected |
| Fragility | 78 | PASS | Isolated tests, one concern per PC |
| Rollback | 75 | PASS | Additive changes, rollback procedure documented |
| Architecture | 90 | PASS | Pure domain, zero I/O, adapter does all I/O |
| Blast Radius | 85 | PASS | 2 new files, 3 minor mods, 2 markdown updates |
| Missing PCs | 72 | PASS | max_per_wave=0/1 PCs added in response |
| Doc Plan | 82 | PASS | 5 doc targets across hierarchy levels |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `crates/ecc-domain/src/spec/wave.rs` | create | US-001, US-002 |
| 2 | `crates/ecc-domain/src/spec/mod.rs` | modify | US-001 |
| 3 | `crates/ecc-workflow/src/commands/wave_plan.rs` | create | US-003 |
| 4 | `crates/ecc-workflow/src/commands/mod.rs` | modify | US-003 |
| 5 | `crates/ecc-workflow/src/main.rs` | modify | US-003 |
| 6 | `skills/wave-analysis/SKILL.md` | modify | US-004 |
| 7 | `commands/implement.md` | modify | US-004 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-29-deterministic-wave-grouping/design.md | Full design |
