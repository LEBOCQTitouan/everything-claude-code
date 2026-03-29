# Tasks: Deterministic Wave Grouping Algorithm (BL-070)

## Pass Conditions

- [ ] PC-001: build_pc_file_map basic mapping | `cargo test -p ecc-domain spec::wave::tests::pc_file_map_basic_mapping` | pending@2026-03-29T21:42:43Z
- [ ] PC-002: Multi-AC spec_ref maps to multiple PCs | `cargo test -p ecc-domain spec::wave::tests::pc_file_map_multi_ac_ref` | pending@2026-03-29T21:42:43Z
- [ ] PC-003: Backtick stripping on file paths | `cargo test -p ecc-domain spec::wave::tests::pc_file_map_backtick_stripping` | pending@2026-03-29T21:42:43Z
- [ ] PC-004: No AC match maps to empty Vec | `cargo test -p ecc-domain spec::wave::tests::pc_file_map_no_match_empty` | pending@2026-03-29T21:42:43Z
- [ ] PC-005: Duplicate files deduplicated | `cargo test -p ecc-domain spec::wave::tests::pc_file_map_dedup` | pending@2026-03-29T21:42:43Z
- [ ] PC-006: Duplicate file paths across FileChanges | `cargo test -p ecc-domain spec::wave::tests::pc_file_map_duplicate_file_paths` | pending@2026-03-29T21:42:43Z
- [ ] PC-007: Non-parseable spec_ref skipped | `cargo test -p ecc-domain spec::wave::tests::pc_file_map_non_parseable_ref` | pending@2026-03-29T21:42:43Z
- [ ] PC-008: No-overlap PCs grouped max 4 | `cargo test -p ecc-domain spec::wave::tests::wave_no_overlap_max_four` | pending@2026-03-29T21:42:43Z
- [ ] PC-009: All-overlap PCs fully sequential | `cargo test -p ecc-domain spec::wave::tests::wave_all_overlap_sequential` | pending@2026-03-29T21:42:43Z
- [ ] PC-010: Single PC one wave | `cargo test -p ecc-domain spec::wave::tests::wave_single_pc` | pending@2026-03-29T21:42:43Z
- [ ] PC-011: Empty PC list empty plan | `cargo test -p ecc-domain spec::wave::tests::wave_empty_pcs` | pending@2026-03-29T21:42:43Z
- [ ] PC-012: Non-adjacent grouping | `cargo test -p ecc-domain spec::wave::tests::wave_non_adjacent_grouping` | pending@2026-03-29T21:42:43Z
- [ ] PC-013: >4 independent split | `cargo test -p ecc-domain spec::wave::tests::wave_split_over_four` | pending@2026-03-29T21:42:43Z
- [ ] PC-014: max_per_wave respected | `cargo test -p ecc-domain spec::wave::tests::wave_custom_max` | pending@2026-03-29T21:42:43Z
- [ ] PC-015: Input order preserved | `cargo test -p ecc-domain spec::wave::tests::wave_preserves_input_order` | pending@2026-03-29T21:42:43Z
- [ ] PC-016: Identical file sets different waves | `cargo test -p ecc-domain spec::wave::tests::wave_identical_files_different_waves` | pending@2026-03-29T21:42:43Z
- [ ] PC-017: wave-plan valid JSON output | `cargo test -p ecc-workflow commands::wave_plan::tests::valid_design_json_output` | pending@2026-03-29T21:42:43Z
- [ ] PC-018: wave-plan no PC table blocks | `cargo test -p ecc-workflow commands::wave_plan::tests::no_pc_table_blocks` | pending@2026-03-29T21:42:43Z
- [ ] PC-019: wave-plan nonexistent path | `cargo test -p ecc-workflow commands::wave_plan::tests::nonexistent_path_blocks` | pending@2026-03-29T21:42:43Z
- [ ] PC-020: wave-plan path traversal rejected | `cargo test -p ecc-workflow commands::wave_plan::tests::path_traversal_rejected` | pending@2026-03-29T21:42:43Z
- [ ] PC-021: wave-plan no File Changes warns | `cargo test -p ecc-workflow commands::wave_plan::tests::no_file_changes_warns` | pending@2026-03-29T21:42:43Z
- [ ] PC-022: max_per_wave=1 sequential | `cargo test -p ecc-domain spec::wave::tests::wave_max_one_sequential` | pending@2026-03-29T21:42:43Z
- [ ] PC-023: max_per_wave=0 safe | `cargo test -p ecc-domain spec::wave::tests::wave_max_zero_safe` | pending@2026-03-29T21:42:43Z
- [ ] PC-024: Clippy zero warnings | `cargo clippy -p ecc-domain -p ecc-workflow -- -D warnings` | pending@2026-03-29T21:42:43Z
- [ ] PC-025: Workspace build | `cargo build --workspace` | pending@2026-03-29T21:42:43Z

## Post-TDD

- [ ] E2E tests | pending@2026-03-29T21:42:43Z
- [ ] Code review | pending@2026-03-29T21:42:43Z
- [ ] Doc updates | pending@2026-03-29T21:42:43Z
- [ ] Supplemental docs | pending@2026-03-29T21:42:43Z
- [ ] Write implement-done.md | pending@2026-03-29T21:42:43Z
