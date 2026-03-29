# Design: B-to-A Grade Push — 5 Remaining HIGHs

## File Changes

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/workflow/state.rs` | Add `impl Transitionable for WorkflowState` | Delegate `transition_to` to `resolve_transition_by_name` | AC-001.1 |
| 2 | `crates/ecc-domain/src/traits.rs` | Fix `make_state()` test helper: `concern` and `started_at` fields use `Concern::Dev` and `Timestamp::new(...)` | Currently passes `String` where `Concern`/`Timestamp` are expected (does not compile). Note: spec says io.rs but the broken helper is in traits.rs; io.rs's `make_state_json` is already correct. | AC-001.2 |
| 3 | `crates/ecc-app/src/session/aliases.rs` | Split into `aliases/mod.rs` + submodules (e.g. `aliases/load.rs`, `aliases/write.rs`, `aliases/tests.rs`) | 814 lines, exceeds 800-line limit | AC-002.1, AC-002.2 |
| 4 | `crates/ecc-domain/src/detection/package_manager.rs` | Add `PackageManagerError` enum; migrate `validate_script_name`, `validate_args`, `get_run_command`, `get_exec_command` from `Result<T, String>` to `Result<T, PackageManagerError>` | Eliminates all `Result<T, String>` from ecc-domain | AC-003.1, AC-003.2, AC-003.3 |
| 5 | `crates/ecc-app/src/detection/package_manager.rs` | Update callers to handle `PackageManagerError` (map to existing error types at boundary) | Callers must compile after error type change | AC-003.1 |
| 6 | `crates/ecc-app/src/validate_design.rs` | Extract `read_design_file`, `parse_and_validate_pcs`, `run_coverage_check`, `build_validation_output` helpers from `run_validate_design` (171 lines) | Function exceeds 50-line limit | AC-004.1 |
| 7 | `crates/ecc-workflow/src/commands/transition.rs` | Extract `resolve_state`, `apply_artifact`, `write_memory_records` helpers from `run` (145 lines) | Function exceeds 50-line limit | AC-004.2 |
| 8 | `crates/ecc-app/src/merge/mod.rs` | Extract `collect_skill_dirs`, `apply_skill_choice`, `report_skill_changes` helpers from `merge_skills` (121 lines) | Function exceeds 50-line limit | AC-004.3 |
| 9 | `crates/ecc-app/src/session/mod.rs` | Extract `collect_sessions`, `paginate_sessions` helpers from `get_all_sessions` (80 lines) | Function exceeds 50-line limit | AC-004.8 |
| 10 | `crates/ecc-app/src/validate_spec.rs` | Extract `read_spec_error_output`, `build_spec_validation_output` helpers from `run_validate_spec` (85 lines) | Function exceeds 50-line limit | AC-004.5 |
| 11 | `crates/ecc-app/src/worktree.rs` | Extract `process_worktree_entry`, `remove_stale_worktree` helpers from `gc` (83 lines) | Function exceeds 50-line limit | AC-004.6 |
| 12 | `crates/ecc-app/src/detection/package_manager.rs` | Extract detection steps from `get_package_manager` (81 lines) into `detect_from_env`, `detect_from_project_config`, `detect_from_global_config` helpers | Function exceeds 50-line limit | AC-004.7 |
| 13 | `crates/ecc-app/src/session/aliases.rs` (post-split) | Extract early-return validation block from `rename_alias` (79 lines) into `validate_rename` helper | Function exceeds 50-line limit | AC-004.9 |
| 14 | `crates/ecc-domain/src/diff/formatter.rs` | Extract `render_same_lines`, `render_diff_lines` helpers from `format_side_by_side_diff` (78 lines) | Function exceeds 50-line limit | AC-004.10 |
| 15 | `crates/ecc-domain/src/session/manager.rs` | No change needed. Spec mentions "SessionManager::default (95 lines)" but `Default for GetAllSessionsOptions` is 8 lines. No 95-line function exists in this file. | AC-004.4 — **SKIP** (already compliant) |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | WorkflowState::transition_to(legal) returns Ok with updated phase | AC-001.1 | `cargo test -p ecc-domain transitionable_impl -- --exact workflow_state_transitions_via_transitionable` | PASS |
| PC-002 | unit | WorkflowState::transition_to(illegal) returns Err(WorkflowError) | AC-001.1 | `cargo test -p ecc-domain transitionable_impl -- --exact workflow_state_rejects_illegal_transition` | PASS |
| PC-003 | unit | WorkflowState::transition_to preserves all fields immutably | AC-001.1 | `cargo test -p ecc-domain transitionable_impl -- --exact workflow_state_transition_returns_new_state_immutably` | PASS |
| PC-004 | unit | traits.rs make_state() uses Concern::Dev and Timestamp::new | AC-001.2 | `cargo test -p ecc-domain transitionable_impl` | PASS (all 3 tests compile and pass) |
| PC-005 | build | All domain tests pass | AC-001.3 | `cargo test -p ecc-domain` | PASS |
| PC-006 | lint | aliases.rs no longer exists as single file; each split file < 400 lines | AC-002.1 | `wc -l crates/ecc-app/src/session/aliases/*.rs` | all < 400 |
| PC-007 | build | Session tests pass after split | AC-002.2 | `cargo test -p ecc-app session` | PASS |
| PC-008 | unit | PackageManagerError enum has variants for empty name, unsafe chars, unsafe args | AC-003.1 | `cargo test -p ecc-domain package_manager` | PASS |
| PC-009 | build | All domain tests pass after error migration | AC-003.2 | `cargo test -p ecc-domain` | PASS |
| PC-010 | grep | No Result<T, String> in package_manager.rs | AC-003.3 | `grep 'Result<.*String>' crates/ecc-domain/src/detection/package_manager.rs` | zero matches |
| PC-011 | lint | run_validate_design main body < 50 lines | AC-004.1 | `grep -c '' crates/ecc-app/src/validate_design.rs` | main fn < 50 lines |
| PC-012 | lint | transition::run main body < 50 lines | AC-004.2 | `grep -c '' crates/ecc-workflow/src/commands/transition.rs` | main fn < 50 lines |
| PC-013 | lint | merge_skills main body < 50 lines | AC-004.3 | `grep -c '' crates/ecc-app/src/merge/mod.rs` | main fn < 50 lines |
| PC-014 | lint | run_validate_spec main body < 50 lines | AC-004.5 | `grep -c '' crates/ecc-app/src/validate_spec.rs` | main fn < 50 lines |
| PC-015 | lint | worktree::gc main body < 50 lines | AC-004.6 | `grep -c '' crates/ecc-app/src/worktree.rs` | main fn < 50 lines |
| PC-016 | lint | get_package_manager main body < 50 lines | AC-004.7 | `grep -c '' crates/ecc-app/src/detection/package_manager.rs` | main fn < 50 lines |
| PC-017 | lint | get_all_sessions main body < 50 lines | AC-004.8 | `grep -c '' crates/ecc-app/src/session/mod.rs` | main fn < 50 lines |
| PC-018 | lint | rename_alias main body < 50 lines | AC-004.9 | `grep -c '' crates/ecc-app/src/session/aliases/mod.rs` | main fn < 50 lines |
| PC-019 | lint | format_side_by_side_diff main body < 50 lines | AC-004.10 | `grep -c '' crates/ecc-domain/src/diff/formatter.rs` | main fn < 50 lines |
| PC-020 | build | All tests pass after all extractions | AC-004.11 | `cargo test` | PASS |
| PC-021 | lint | Zero clippy warnings | all | `cargo clippy -- -D warnings` | PASS |
| PC-022 | build | Release build succeeds | all | `cargo build --release` | PASS |
| PC-023 | build | Full test suite passes | all | `cargo test` | PASS |

## TDD Order

1. **US-001 Phase A: Transitionable impl** (Entity layer)
   - Fix `traits.rs` test helper `make_state()` to use `Concern::Dev` and `Timestamp::new(...)` instead of `String`
   - Add `impl Transitionable for WorkflowState` in `state.rs`, delegating to `resolve_transition_by_name`
   - RED: existing tests in `transitionable_impl` module should now compile and pass
   - GREEN: implement the trait
   - Verify: PC-001 through PC-005
   - Commit: `test: fix transitionable_impl test helper types` then `feat: implement Transitionable for WorkflowState`

2. **US-003: PackageManagerError typed errors** (Entity layer)
   - Add `PackageManagerError` enum with `thiserror` derives in `package_manager.rs`
   - RED: write test asserting `validate_script_name("")` returns `Err(PackageManagerError::EmptyName)`
   - GREEN: migrate 4 functions from `Result<T, String>` to `Result<T, PackageManagerError>`
   - Update callers in `crates/ecc-app/src/detection/package_manager.rs` (map error at boundary)
   - Verify: PC-008 through PC-010
   - Commit: `test: add PackageManagerError type assertions` then `refactor: migrate package_manager to typed errors`

3. **US-002: Split aliases.rs** (Adapter layer)
   - Convert `aliases.rs` (814 lines) to `aliases/mod.rs` directory module
   - Split into `aliases/mod.rs` (re-exports + load/save), `aliases/operations.rs` (set/delete/rename), `aliases/tests.rs`
   - RED: `cargo test -p ecc-app session` must still compile and pass
   - GREEN: all tests pass, each file < 400 lines
   - Verify: PC-006, PC-007
   - Commit: `refactor: split session/aliases.rs into submodules`

4. **US-004 Phase A: Domain function extractions** (Entity layer)
   - Extract helpers from `format_side_by_side_diff` (78 lines) in `diff/formatter.rs`
   - Verify: PC-019, PC-020
   - Commit: `refactor: extract side-by-side diff render helpers`

5. **US-004 Phase B: App function extractions — validation** (UseCase layer)
   - Extract helpers from `run_validate_design` (171 lines)
   - Extract helpers from `run_validate_spec` (85 lines)
   - Verify: PC-011, PC-014, PC-020
   - Commit: `refactor: extract validate_design and validate_spec helpers`

6. **US-004 Phase C: App function extractions — session/merge/worktree** (UseCase layer)
   - Extract helpers from `get_all_sessions` (80 lines)
   - Extract helpers from `rename_alias` (79 lines) — depends on US-002 split
   - Extract helpers from `merge_skills` (121 lines)
   - Extract helpers from `get_package_manager` (81 lines)
   - Extract helpers from `worktree::gc` (83 lines)
   - Verify: PC-013, PC-015, PC-016, PC-017, PC-018, PC-020
   - Commit: `refactor: extract session, merge, worktree, and detection helpers`

7. **US-004 Phase D: Workflow binary function extraction** (Framework layer)
   - Extract helpers from `transition::run` (145 lines)
   - Verify: PC-012, PC-020
   - Commit: `refactor: extract transition command helpers`

8. **Final gate**
   - Verify: PC-021, PC-022, PC-023
   - `cargo clippy -- -D warnings && cargo build --release && cargo test`

## E2E Assessment

- **Touches user-facing flows?** No -- all changes are internal structural refactoring
- **Crosses 3+ modules end-to-end?** No
- **New E2E tests needed?** No
- Existing test suite (`cargo test`) is the gate after all phases

## Notes

- AC-004.4 (SessionManager::default 95 lines) is **skipped**: no such function exists in the codebase. `Default for GetAllSessionsOptions` is 8 lines.
- AC-001.2 references `io.rs` but the broken `make_state()` helper is in `traits.rs` (line 86). The `io.rs` helper `make_state_json` already uses correct types.
- The `PackageManagerError` migration will require updating callers in `ecc-app` since they currently match on `String` errors. The boundary conversion is straightforward (`.map_err(|e| e.to_string())` or similar at the app layer).
