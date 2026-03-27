# Design: BL-052 Replace Shell Hooks with Compiled Rust Binaries

## Overview

Port 13 shell scripts under `.claude/hooks/` to a single compiled `ecc-workflow` Rust binary with subcommand dispatch. Domain types (WorkflowState, Phase) live in `ecc-domain`; the binary crate `ecc-workflow` depends on `ecc-domain` only. All command/skill references are updated, and shell scripts are deleted.

## 1. File Changes (dependency order)

### Phase 1: Audit and Delete Dead Scripts (US-001)

| # | File | Action | Spec Ref | Layer |
|---|------|--------|----------|-------|
| 1 | `.claude/hooks/workflow-init.sh` | KEEP (active) | AC-001.1 | - |
| 2 | `.claude/hooks/phase-transition.sh` | KEEP (active) | AC-001.1 | - |
| 3 | `.claude/hooks/toolchain-persist.sh` | KEEP (active) | AC-001.1 | - |
| 4 | `.claude/hooks/memory-writer.sh` | KEEP (active) | AC-001.1 | - |
| 5 | `.claude/hooks/phase-gate.sh` | KEEP (active) | AC-001.1 | - |
| 6 | `.claude/hooks/stop-gate.sh` | KEEP (active) | AC-001.1 | - |
| 7 | `.claude/hooks/grill-me-gate.sh` | KEEP (active) | AC-001.1 | - |
| 8 | `.claude/hooks/tdd-enforcement.sh` | KEEP (active) | AC-001.1 | - |
| 9 | `.claude/hooks/scope-check.sh` | KEEP (active) | AC-001.1 | - |
| 10 | `.claude/hooks/doc-enforcement.sh` | KEEP (active) | AC-001.1 | - |
| 11 | `.claude/hooks/doc-level-check.sh` | KEEP (active) | AC-001.1 | - |
| 12 | `.claude/hooks/pass-condition-check.sh` | KEEP (active) | AC-001.1 | - |
| 13 | `.claude/hooks/e2e-boundary-check.sh` | KEEP (active) | AC-001.1 | - |

**Audit Result**: All 13 scripts are actively referenced. `workflow-init.sh`, `phase-transition.sh`, `toolchain-persist.sh`, `memory-writer.sh` are called from `commands/*.md` and `skills/*.md` (CLI invocation). The remaining 9 (`phase-gate.sh`, `stop-gate.sh`, `grill-me-gate.sh`, `tdd-enforcement.sh`, `scope-check.sh`, `doc-enforcement.sh`, `doc-level-check.sh`, `pass-condition-check.sh`, `e2e-boundary-check.sh`) are called from `.claude/settings.json` (hooks.json runtime invocation). Zero dead scripts found.

Since no scripts are dead, AC-001.2 is satisfied trivially (no deletions needed). AC-001.3 is satisfied by this inventory.

Layers: [Entity]

### Phase 2: WorkflowState Domain Aggregate (US-002)

| # | File | Action | Spec Ref | Layer |
|---|------|--------|----------|-------|
| 1 | `crates/ecc-domain/src/workflow/mod.rs` | CREATE | AC-002.1-002.6 | Entity |
| 2 | `crates/ecc-domain/src/workflow/phase.rs` | CREATE | AC-002.2, AC-002.5 | Entity |
| 3 | `crates/ecc-domain/src/workflow/state.rs` | CREATE | AC-002.1, AC-002.4, AC-002.6 | Entity |
| 4 | `crates/ecc-domain/src/workflow/transition.rs` | CREATE | AC-002.2, AC-002.3, AC-002.5 | Entity |
| 5 | `crates/ecc-domain/src/workflow/error.rs` | CREATE | AC-002.3, AC-002.6 | Entity |
| 6 | `crates/ecc-domain/src/lib.rs` | MODIFY (add `pub mod workflow;`) | AC-002.1 | Entity |

Layers: [Entity]

### Phase 3: ecc-workflow Crate Skeleton + init/transition Subcommands (US-003)

| # | File | Action | Spec Ref | Layer |
|---|------|--------|----------|-------|
| 1 | `crates/ecc-workflow/Cargo.toml` | CREATE | AC-003.1, AC-003.7 | Framework |
| 2 | `crates/ecc-workflow/src/main.rs` | CREATE | AC-003.1, AC-003.6, AC-003.8 | Adapter |
| 3 | `crates/ecc-workflow/src/output.rs` | CREATE | AC-003.6 | Adapter |
| 4 | `crates/ecc-workflow/src/io.rs` | CREATE | AC-003.5, AC-003.8 | Adapter |
| 5 | `crates/ecc-workflow/src/commands/mod.rs` | CREATE | AC-003.1 | Adapter |
| 6 | `crates/ecc-workflow/src/commands/init.rs` | CREATE | AC-003.2, AC-003.5 | Adapter |
| 7 | `crates/ecc-workflow/src/commands/transition.rs` | CREATE | AC-003.3, AC-003.4, AC-003.5 | Adapter |
| 8 | `Cargo.toml` (workspace root) | MODIFY (add `crates/ecc-workflow`) | AC-003.1 | Framework |

Layers: [Adapter, Framework]

### Phase 4: Port Remaining Active Scripts (US-004)

| # | File | Action | Spec Ref | Layer |
|---|------|--------|----------|-------|
| 1 | `crates/ecc-workflow/src/commands/toolchain_persist.rs` | CREATE | AC-004.3 | Adapter |
| 2 | `crates/ecc-workflow/src/commands/memory_write.rs` | CREATE | AC-004.4 | Adapter |
| 3 | `crates/ecc-workflow/src/commands/phase_gate.rs` | CREATE | AC-004.5 | Adapter |
| 4 | `crates/ecc-workflow/src/commands/stop_gate.rs` | CREATE | AC-004.5 | Adapter |
| 5 | `crates/ecc-workflow/src/commands/grill_me_gate.rs` | CREATE | AC-004.5 | Adapter |
| 6 | `crates/ecc-workflow/src/commands/tdd_enforcement.rs` | CREATE | AC-004.5 | Adapter |
| 7 | `crates/ecc-workflow/src/commands/scope_check.rs` | CREATE | AC-004.5 | Adapter |
| 8 | `crates/ecc-workflow/src/commands/doc_enforcement.rs` | CREATE | AC-004.5 | Adapter |
| 9 | `crates/ecc-workflow/src/commands/doc_level_check.rs` | CREATE | AC-004.5 | Adapter |
| 10 | `crates/ecc-workflow/src/commands/pass_condition_check.rs` | CREATE | AC-004.5 | Adapter |
| 11 | `crates/ecc-workflow/src/commands/e2e_boundary_check.rs` | CREATE | AC-004.5 | Adapter |
| 12 | `crates/ecc-workflow/src/commands/mod.rs` | MODIFY (add all subcommands) | AC-004.5 | Adapter |
| 13 | `crates/ecc-workflow/src/main.rs` | MODIFY (register all subcommands) | AC-004.5 | Adapter |
| 14 | `crates/ecc-workflow/src/slug.rs` | CREATE (utility for memory-writer slug generation) | AC-004.4 | Adapter |

Layers: [Adapter]

### Phase 5: Update Commands and Skills (US-005)

| # | File | Action | Spec Ref | Layer |
|---|------|--------|----------|-------|
| 1 | `commands/spec-dev.md` | MODIFY | AC-005.1, AC-005.3 | - |
| 2 | `commands/spec-fix.md` | MODIFY | AC-005.1 | - |
| 3 | `commands/spec-refactor.md` | MODIFY | AC-005.1 | - |
| 4 | `commands/design.md` | MODIFY | AC-005.1 | - |
| 5 | `commands/implement.md` | MODIFY | AC-005.1 | - |
| 6 | `skills/tasks-generation/SKILL.md` | MODIFY | AC-005.2 | - |
| 7 | `skills/spec-pipeline-shared/SKILL.md` | MODIFY | AC-005.2 | - |
| 8 | `.claude/settings.json` | MODIFY | AC-005.4 | - |
| 9 | `hooks/hooks.json` | MODIFY | AC-005.4 | - |

**Reference update mapping** (all references replace `bash .claude/hooks/<script>.sh` with `ecc-workflow <subcommand>`):

| Old | New |
|-----|-----|
| `bash .claude/hooks/workflow-init.sh <concern> "<feature>"` | `ecc-workflow init <concern> "<feature>"` |
| `bash .claude/hooks/phase-transition.sh <target> <artifact> [path]` | `ecc-workflow transition <target> [--artifact <name>] [--path <path>]` |
| `bash .claude/hooks/toolchain-persist.sh "<test>" "<lint>" "<build>"` | `ecc-workflow toolchain-persist "<test>" "<lint>" "<build>"` |
| `bash .claude/hooks/phase-gate.sh` | `ecc-workflow phase-gate` |
| `bash .claude/hooks/stop-gate.sh` | `ecc-workflow stop-gate` |
| `bash .claude/hooks/grill-me-gate.sh` | `ecc-workflow grill-me-gate` |
| `bash .claude/hooks/tdd-enforcement.sh` | `ecc-workflow tdd-enforcement` |
| `bash .claude/hooks/scope-check.sh` | `ecc-workflow scope-check` |
| `bash .claude/hooks/doc-enforcement.sh` | `ecc-workflow doc-enforcement` |
| `bash .claude/hooks/doc-level-check.sh` | `ecc-workflow doc-level-check` |
| `bash .claude/hooks/pass-condition-check.sh` | `ecc-workflow pass-condition-check` |
| `bash .claude/hooks/e2e-boundary-check.sh` | `ecc-workflow e2e-boundary-check` |

Layers: [Adapter]

### Phase 6: Delete Shell Scripts and Update Docs (US-006)

| # | File | Action | Spec Ref | Layer |
|---|------|--------|----------|-------|
| 1 | `.claude/hooks/workflow-init.sh` | DELETE | AC-006.1 | - |
| 2 | `.claude/hooks/phase-transition.sh` | DELETE | AC-006.1 | - |
| 3 | `.claude/hooks/toolchain-persist.sh` | DELETE | AC-006.1 | - |
| 4 | `.claude/hooks/memory-writer.sh` | DELETE | AC-006.1 | - |
| 5 | `.claude/hooks/phase-gate.sh` | DELETE | AC-006.1 | - |
| 6 | `.claude/hooks/stop-gate.sh` | DELETE | AC-006.1 | - |
| 7 | `.claude/hooks/grill-me-gate.sh` | DELETE | AC-006.1 | - |
| 8 | `.claude/hooks/tdd-enforcement.sh` | DELETE | AC-006.1 | - |
| 9 | `.claude/hooks/scope-check.sh` | DELETE | AC-006.1 | - |
| 10 | `.claude/hooks/doc-enforcement.sh` | DELETE | AC-006.1 | - |
| 11 | `.claude/hooks/doc-level-check.sh` | DELETE | AC-006.1 | - |
| 12 | `.claude/hooks/pass-condition-check.sh` | DELETE | AC-006.1 | - |
| 13 | `.claude/hooks/e2e-boundary-check.sh` | DELETE | AC-006.1 | - |
| 14 | `CLAUDE.md` | MODIFY | AC-006.2, AC-006.4 | - |
| 15 | `docs/domain/glossary.md` | MODIFY | AC-006.3 | - |
| 16 | `docs/adr/NNN-separate-workflow-crate.md` | CREATE | spec decision #1 | - |
| 17 | `docs/ARCHITECTURE.md` | MODIFY | AC-006.2 | - |

Layers: [Adapter]

---

## 2. Pass Conditions

### Phase 2: WorkflowState Domain Aggregate (US-002)

#### PC-001: Phase value object has four variants
- **ID**: PC-001
- **Type**: unit
- **Description**: Phase enum has Plan, Solution, Implement, Done variants with Display and serde support
- **Verifies**: AC-002.2
- **Command**: `cargo test -p ecc-domain --lib workflow::phase`
- **Expected**: All tests pass. Phase::Plan displays as "plan", serializes/deserializes correctly.

#### PC-002: WorkflowState aggregate fields
- **ID**: PC-002
- **Type**: unit
- **Description**: WorkflowState struct contains phase, concern, feature, started_at, toolchain, artifacts, completed fields
- **Verifies**: AC-002.1
- **Command**: `cargo test -p ecc-domain --lib workflow::state::tests::creates_workflow_state_with_all_fields`
- **Expected**: Test passes. All fields are accessible and correctly typed.

#### PC-003: Legal transitions allowed
- **ID**: PC-003
- **Type**: unit
- **Description**: plan->solution, solution->implement, implement->done transitions succeed
- **Verifies**: AC-002.2
- **Command**: `cargo test -p ecc-domain --lib workflow::transition::tests::legal_transitions`
- **Expected**: All three legal transitions return Ok.

#### PC-004: Phase aliases accepted
- **ID**: PC-004
- **Type**: unit
- **Description**: spec->design maps to plan->solution, design->implement maps to solution->implement
- **Verifies**: AC-002.2
- **Command**: `cargo test -p ecc-domain --lib workflow::transition::tests::alias_transitions`
- **Expected**: Alias transitions resolve and succeed.

#### PC-005: Illegal transition returns domain error
- **ID**: PC-005
- **Type**: unit
- **Description**: plan->implement, solution->done, done->plan return Err(WorkflowError::IllegalTransition)
- **Verifies**: AC-002.3
- **Command**: `cargo test -p ecc-domain --lib workflow::transition::tests::illegal_transitions`
- **Expected**: Each returns Err with meaningful error message, no panic.

#### PC-006: JSON round-trip matches state.json format
- **ID**: PC-006
- **Type**: unit
- **Description**: WorkflowState serializes to JSON matching existing state.json schema, and deserializes back identically
- **Verifies**: AC-002.4
- **Command**: `cargo test -p ecc-domain --lib workflow::state::tests::json_round_trip`
- **Expected**: Serialized JSON has keys: concern, phase, feature, started_at, toolchain, artifacts, completed. Deserialize round-trip produces equal struct.

#### PC-007: Idempotent re-entry
- **ID**: PC-007
- **Type**: unit
- **Description**: Transition from plan->plan, solution->solution, implement->implement succeeds silently
- **Verifies**: AC-002.5
- **Command**: `cargo test -p ecc-domain --lib workflow::transition::tests::reentry_transitions`
- **Expected**: Re-entry transitions return Ok without modifying the phase.

#### PC-008: Corrupted JSON returns domain error
- **ID**: PC-008
- **Type**: unit
- **Description**: Deserializing invalid JSON, missing fields, or wrong types returns WorkflowError::InvalidState with reason
- **Verifies**: AC-002.6
- **Command**: `cargo test -p ecc-domain --lib workflow::state::tests::corrupted_json`
- **Expected**: Each invalid input returns Err with descriptive message, not a panic.

#### PC-009: Domain crate has zero I/O imports
- **ID**: PC-009
- **Type**: lint
- **Description**: ecc-domain must not import std::fs, std::process, std::net, or tokio
- **Verifies**: AC-002.1 (domain purity constraint)
- **Command**: `! grep -rn 'use std::fs\|use std::process\|use std::net\|use tokio' crates/ecc-domain/src/`
- **Expected**: Zero matches (exit 0 from `!` negation).

### Phase 3: ecc-workflow Crate Skeleton (US-003)

#### PC-010: Binary compiles
- **ID**: PC-010
- **Type**: build
- **Description**: `ecc-workflow` binary is produced by `cargo build`
- **Verifies**: AC-003.1
- **Command**: `cargo build -p ecc-workflow && test -f target/debug/ecc-workflow`
- **Expected**: Build succeeds and binary exists.

#### PC-011: init subcommand creates state.json
- **ID**: PC-011
- **Type**: integration
- **Description**: `ecc-workflow init dev "test feature"` creates state.json with correct initial state
- **Verifies**: AC-003.2
- **Command**: `cargo test -p ecc-workflow --test integration init_creates_state_json`
- **Expected**: state.json contains `{"concern":"dev","phase":"plan","feature":"test feature",...}` with toolchain nulls, empty completed, and ISO 8601 started_at.

#### PC-012: transition subcommand updates state.json
- **ID**: PC-012
- **Type**: integration
- **Description**: After init, `ecc-workflow transition solution --artifact plan` updates phase to "solution" and stamps artifact
- **Verifies**: AC-003.3
- **Command**: `cargo test -p ecc-workflow --test integration transition_updates_state`
- **Expected**: state.json phase is "solution", artifacts.plan has ISO 8601 timestamp.

#### PC-013: Illegal transition exits non-zero with JSON error
- **ID**: PC-013
- **Type**: integration
- **Description**: `ecc-workflow transition done` from plan phase exits non-zero with `{"status":"block","message":"..."}`
- **Verifies**: AC-003.4
- **Command**: `cargo test -p ecc-workflow --test integration transition_illegal_exits_nonzero`
- **Expected**: Exit code != 0, stdout contains JSON with status "block".

#### PC-014: Missing state.json exits 0 with warning
- **ID**: PC-014
- **Type**: integration
- **Description**: Running any subcommand without state.json exits 0 with `{"status":"warn","message":"..."}`
- **Verifies**: AC-003.5
- **Command**: `cargo test -p ecc-workflow --test integration missing_state_exits_zero_with_warning`
- **Expected**: Exit code 0, stderr contains JSON with status "warn".

#### PC-015: Output is structured JSON
- **ID**: PC-015
- **Type**: integration
- **Description**: All subcommands produce valid JSON output with status field
- **Verifies**: AC-003.6
- **Command**: `cargo test -p ecc-workflow --test integration output_is_structured_json`
- **Expected**: Every subcommand's stdout/stderr parses as valid JSON with `status` in `["pass","block","warn"]`.

#### PC-016: Cross-platform compilation
- **ID**: PC-016
- **Type**: build
- **Description**: Binary compiles without POSIX shell dependency (no std::process::Command("bash") in production code)
- **Verifies**: AC-003.7
- **Command**: `! grep -rn 'Command::new("bash")\|Command::new("sh")' crates/ecc-workflow/src/`
- **Expected**: Zero matches.

#### PC-017: Dual invocation mode (stdin JSON + CLI args)
- **ID**: PC-017
- **Type**: integration
- **Description**: Subcommands auto-detect stdin JSON vs CLI args
- **Verifies**: AC-003.8
- **Command**: `cargo test -p ecc-workflow --test integration dual_invocation`
- **Expected**: Both modes produce equivalent results.

### Phase 4: Port Active Scripts (US-004)

#### PC-018: init produces identical output to workflow-init.sh
- **ID**: PC-018
- **Type**: integration
- **Description**: `ecc-workflow init` produces semantically equivalent state.json to shell version
- **Verifies**: AC-004.1
- **Command**: `cargo test -p ecc-workflow --test integration init_matches_shell`
- **Expected**: Same keys, same value types, same default values. Stale workflow archiving behavior preserved.

#### PC-019: transition handles all phase transitions and artifact stamping
- **ID**: PC-019
- **Type**: integration
- **Description**: Full transition sequence plan->solution->implement->done with artifact stamping and path storage
- **Verifies**: AC-004.2
- **Command**: `cargo test -p ecc-workflow --test integration transition_full_sequence`
- **Expected**: Each transition updates phase, stamps artifact timestamp, stores artifact path (spec_path, design_path, tasks_path), and done transition appends to completed array.

#### PC-020: toolchain-persist writes toolchain to state.json
- **ID**: PC-020
- **Type**: integration
- **Description**: `ecc-workflow toolchain-persist "cargo test" "cargo clippy" "cargo build"` writes toolchain fields
- **Verifies**: AC-004.3
- **Command**: `cargo test -p ecc-workflow --test integration toolchain_persist`
- **Expected**: state.json toolchain.test, .lint, .build contain the provided commands.

#### PC-021: memory-write produces identical memory files
- **ID**: PC-021
- **Type**: integration
- **Description**: `ecc-workflow memory-write action/work-item/daily/memory-index` produces same file structure as shell
- **Verifies**: AC-004.4
- **Command**: `cargo test -p ecc-workflow --test integration memory_write_subcommands`
- **Expected**: action-log.json has correct entry schema, work-item files have correct headings, daily file has Activity/Insights sections.

#### PC-022: phase-gate blocks writes during spec/design phases
- **ID**: PC-022
- **Type**: integration
- **Description**: `ecc-workflow phase-gate` reads stdin JSON, blocks Write/Edit to non-allowed paths during plan/solution
- **Verifies**: AC-004.5
- **Command**: `cargo test -p ecc-workflow --test integration phase_gate`
- **Expected**: Exit 2 for blocked paths, exit 0 for allowed paths and during implement/done phases.

#### PC-023: stop-gate warns when workflow incomplete
- **ID**: PC-023
- **Type**: integration
- **Description**: `ecc-workflow stop-gate` warns when phase != done
- **Verifies**: AC-004.5
- **Command**: `cargo test -p ecc-workflow --test integration stop_gate`
- **Expected**: Warning on stderr when phase is plan/solution/implement, silent when done or no state.json.

#### PC-024: grill-me-gate checks for interview markers
- **ID**: PC-024
- **Type**: integration
- **Description**: `ecc-workflow grill-me-gate` warns when spec/campaign lacks grill-me section
- **Verifies**: AC-004.5
- **Command**: `cargo test -p ecc-workflow --test integration grill_me_gate`
- **Expected**: Warning when markers absent, silent when present.

#### PC-025: tdd-enforcement tracks TDD cycle
- **ID**: PC-025
- **Type**: integration
- **Description**: `ecc-workflow tdd-enforcement` reads stdin, tracks RED/GREEN/REFACTOR state
- **Verifies**: AC-004.5
- **Command**: `cargo test -p ecc-workflow --test integration tdd_enforcement`
- **Expected**: TDD state file updated correctly. Always exits 0 (informational).

#### PC-026: scope-check detects unexpected file changes
- **ID**: PC-026
- **Type**: integration
- **Description**: `ecc-workflow scope-check` compares git diff against solution.md expected files
- **Verifies**: AC-004.5
- **Command**: `cargo test -p ecc-workflow --test integration scope_check`
- **Expected**: Warning when unexpected files found, silent otherwise. Always exits 0.

#### PC-027: doc-enforcement checks implement-done.md
- **ID**: PC-027
- **Type**: integration
- **Description**: `ecc-workflow doc-enforcement` checks for required sections in implement-done.md
- **Verifies**: AC-004.5
- **Command**: `cargo test -p ecc-workflow --test integration doc_enforcement`
- **Expected**: Warning when sections missing, silent when present.

#### PC-028: doc-level-check validates document sizes
- **ID**: PC-028
- **Type**: integration
- **Description**: `ecc-workflow doc-level-check` warns about oversized CLAUDE.md, README.md, ARCHITECTURE.md code blocks
- **Verifies**: AC-004.5
- **Command**: `cargo test -p ecc-workflow --test integration doc_level_check`
- **Expected**: Warning for oversized files, silent otherwise.

#### PC-029: pass-condition-check validates pass conditions
- **ID**: PC-029
- **Type**: integration
- **Description**: `ecc-workflow pass-condition-check` checks implement-done.md for pass condition results
- **Verifies**: AC-004.5
- **Command**: `cargo test -p ecc-workflow --test integration pass_condition_check`
- **Expected**: Warning when section missing or failures found, silent when all pass.

#### PC-030: e2e-boundary-check validates E2E section
- **ID**: PC-030
- **Type**: integration
- **Description**: `ecc-workflow e2e-boundary-check` checks implement-done.md for E2E Tests section
- **Verifies**: AC-004.5
- **Command**: `cargo test -p ecc-workflow --test integration e2e_boundary_check`
- **Expected**: Warning when section missing, silent when present.

#### PC-031: ECC_WORKFLOW_BYPASS=1 skips all subcommands
- **ID**: PC-031
- **Type**: integration
- **Description**: All subcommands exit 0 immediately when ECC_WORKFLOW_BYPASS=1
- **Verifies**: AC-004.5 (constraint from spec)
- **Command**: `cargo test -p ecc-workflow --test integration bypass_env_var`
- **Expected**: Exit 0, no output, no state.json modification.

#### PC-032: phase-transition calls memory-write internally
- **ID**: PC-032
- **Type**: integration
- **Description**: Transition subcommand calls memory-write logic as internal function (not subprocess)
- **Verifies**: AC-004.2, AC-004.4 (cross-script coupling constraint)
- **Command**: `cargo test -p ecc-workflow --test integration transition_writes_memory`
- **Expected**: After transition, action-log.json updated, work-item file created, daily file updated.

### Phase 5: Update Commands and Skills (US-005)

#### PC-033: No bash .claude/hooks/ in commands
- **ID**: PC-033
- **Type**: lint
- **Description**: Zero matches for `bash .claude/hooks/` in commands/
- **Verifies**: AC-005.1
- **Command**: `! grep -rn 'bash \.claude/hooks/' commands/`
- **Expected**: Zero matches.

#### PC-034: No bash .claude/hooks/ in skills
- **ID**: PC-034
- **Type**: lint
- **Description**: Zero matches for `bash .claude/hooks/` in skills/
- **Verifies**: AC-005.2
- **Command**: `! grep -rn 'bash \.claude/hooks/' skills/`
- **Expected**: Zero matches.

#### PC-035: hooks.json references ecc-workflow
- **ID**: PC-035
- **Type**: lint
- **Description**: hooks.json and .claude/settings.json reference ecc-workflow, not shell scripts
- **Verifies**: AC-005.4
- **Command**: `! grep -n 'bash \.claude/hooks/' hooks/hooks.json .claude/settings.json`
- **Expected**: Zero matches.

#### PC-036: ecc-workflow subcommands used in commands
- **ID**: PC-036
- **Type**: lint
- **Description**: Commands reference `ecc-workflow init`, `ecc-workflow transition`, `ecc-workflow toolchain-persist`
- **Verifies**: AC-005.3
- **Command**: `grep -c 'ecc-workflow' commands/spec-dev.md commands/spec-fix.md commands/spec-refactor.md commands/design.md commands/implement.md`
- **Expected**: Each file has at least one match.

### Phase 6: Delete Shell Scripts and Update Docs (US-006)

#### PC-037: No .sh files in .claude/hooks/
- **ID**: PC-037
- **Type**: lint
- **Description**: `.claude/hooks/` contains no `.sh` files
- **Verifies**: AC-006.1
- **Command**: `test "$(find .claude/hooks/ -name '*.sh' | wc -l | tr -d ' ')" = "0"`
- **Expected**: Exit 0 (zero .sh files).

#### PC-038: CLAUDE.md references ecc-workflow
- **ID**: PC-038
- **Type**: lint
- **Description**: CLAUDE.md mentions ecc-workflow binary
- **Verifies**: AC-006.2
- **Command**: `grep -q 'ecc-workflow' CLAUDE.md`
- **Expected**: At least one match.

#### PC-039: Glossary defines WorkflowState and Phase
- **ID**: PC-039
- **Type**: lint
- **Description**: Glossary has entries for WorkflowState and Phase
- **Verifies**: AC-006.3
- **Command**: `grep -q 'WorkflowState' docs/domain/glossary.md && grep -q 'Phase' docs/domain/glossary.md`
- **Expected**: Both terms present.

#### PC-040: CLAUDE.md test count updated
- **ID**: PC-040
- **Type**: lint
- **Description**: Test count in CLAUDE.md reflects new test additions
- **Verifies**: AC-006.4
- **Command**: `grep -oE '[0-9]+ tests' CLAUDE.md`
- **Expected**: Number matches actual `cargo test 2>&1 | grep 'test result'` count.

### Cross-Cutting Pass Conditions

#### PC-041: clippy clean
- **ID**: PC-041
- **Type**: lint
- **Description**: Full workspace passes clippy with no warnings
- **Verifies**: All ACs (quality gate)
- **Command**: `cargo clippy -- -D warnings`
- **Expected**: Exit 0.

#### PC-042: cargo build succeeds
- **ID**: PC-042
- **Type**: build
- **Description**: Full workspace builds
- **Verifies**: All ACs (quality gate)
- **Command**: `cargo build`
- **Expected**: Exit 0.

#### PC-043: All existing tests pass
- **ID**: PC-043
- **Type**: unit
- **Description**: No regressions in existing test suite
- **Verifies**: All ACs (quality gate)
- **Command**: `cargo test`
- **Expected**: All tests pass.

#### PC-044: ARCHITECTURE.md mentions ecc-workflow with correct crate count
- **ID**: PC-044
- **Type**: lint
- **Description**: ARCHITECTURE.md contains ecc-workflow in crate list and has correct crate count
- **Verifies**: AC-006.2
- **Command**: `grep -c 'ecc-workflow' docs/ARCHITECTURE.md && grep -q '8 crates\|eight crates' docs/ARCHITECTURE.md`
- **Expected**: grep finds ecc-workflow mention and correct crate count.

#### PC-045: Stale workflow archiving works
- **ID**: PC-045
- **Type**: integration
- **Description**: When ecc-workflow init is called with an existing state.json, the old state is archived before creating new state
- **Verifies**: AC-003.2
- **Command**: `cargo test -p ecc-workflow --test stale_archive`
- **Expected**: PASS

#### PC-046: Behavioral equivalence tests use fixture data
- **ID**: PC-046
- **Type**: integration
- **Description**: All behavioral equivalence PCs (init, transition, toolchain-persist, memory-write) test against expected fixture JSON, not by comparing with shell script output
- **Verifies**: AC-004.1, AC-004.2, AC-004.3, AC-004.4
- **Command**: `grep -rL 'bash\|\.sh' crates/ecc-workflow/tests/ || echo "clean"`
- **Expected**: No test file references shell scripts. Tests use inline fixture JSON.

---

## 3. TDD Order

The pass conditions should be implemented in this order, following dependency chains:

### Round 1: Domain Types (Phase 2)
1. **PC-001** (Phase VO) -- foundation type
2. **PC-005** (Illegal transitions) -- error type needed for PC-003
3. **PC-003** (Legal transitions) -- core domain rule
4. **PC-004** (Alias transitions) -- extends transition rules
5. **PC-007** (Re-entry transitions) -- extends transition rules
6. **PC-002** (WorkflowState fields) -- aggregate depends on Phase
7. **PC-006** (JSON round-trip) -- serde on WorkflowState
8. **PC-008** (Corrupted JSON) -- error path of deserialization
9. **PC-009** (Domain purity lint) -- constraint verification

**Gate**: `cargo test -p ecc-domain --lib` + `cargo clippy -- -D warnings`

### Round 2: ecc-workflow Crate Skeleton (Phase 3)
10. **PC-010** (Binary compiles) -- crate exists
11. **PC-016** (No bash/sh in production code) -- constraint
12. **PC-015** (Structured JSON output) -- output module
13. **PC-014** (Missing state.json exits 0) -- io module
14. **PC-011** (init creates state.json) -- first subcommand
15. **PC-012** (transition updates state.json) -- second subcommand
16. **PC-013** (Illegal transition non-zero exit) -- error path
17. **PC-017** (Dual invocation mode) -- stdin vs CLI detection

**Gate**: `cargo test -p ecc-workflow` + `cargo clippy -- -D warnings` + `cargo build`

### Round 3: Port Remaining Scripts (Phase 4)
18. **PC-031** (ECC_WORKFLOW_BYPASS) -- bypass check used by all subcommands
19. **PC-020** (toolchain-persist) -- simple port
20. **PC-018** (init matches shell) -- behavioral equivalence
21. **PC-019** (transition full sequence) -- behavioral equivalence
22. **PC-032** (transition writes memory) -- cross-script coupling
23. **PC-021** (memory-write) -- complex port
24. **PC-022** (phase-gate) -- stdin-based hook
25. **PC-023** (stop-gate)
26. **PC-024** (grill-me-gate)
27. **PC-025** (tdd-enforcement)
28. **PC-026** (scope-check)
29. **PC-027** (doc-enforcement)
30. **PC-028** (doc-level-check)
31. **PC-029** (pass-condition-check)
32. **PC-030** (e2e-boundary-check)

**Gate**: `cargo test -p ecc-workflow` + `cargo clippy -- -D warnings`

### Round 4: Update References (Phase 5)
33. **PC-033** (No bash hooks in commands)
34. **PC-034** (No bash hooks in skills)
35. **PC-035** (hooks.json updated)
36. **PC-036** (ecc-workflow in commands)

**Gate**: Grep-based verifications all pass

### Round 5: Cleanup and Docs (Phase 6)
37. **PC-037** (No .sh files)
38. **PC-038** (CLAUDE.md updated)
39. **PC-039** (Glossary updated)
40. **PC-040** (Test count updated)

### Final Gate
41. **PC-041** (clippy clean)
42. **PC-042** (cargo build)
43. **PC-043** (All tests pass)

---

## Architecture Notes

### WorkflowState Aggregate (ecc-domain)

```
workflow/
  mod.rs          -- re-exports
  phase.rs        -- Phase enum { Plan, Solution, Implement, Done }
                     with Display, FromStr, Serialize, Deserialize
                     FromStr accepts aliases: "spec" -> Plan, "design" -> Solution
  state.rs        -- WorkflowState { phase, concern, feature, started_at,
                     toolchain: Toolchain, artifacts: Artifacts, completed: Vec<Completion> }
                     Toolchain { test, lint, build }: Option<String> fields
                     Artifacts { plan, solution, implement, campaign_path,
                                 spec_path, design_path, tasks_path }: Option<String> fields
                     Completion { phase, file, at }: String fields
  transition.rs   -- fn transition(state: &WorkflowState, target: Phase) -> Result<Phase, WorkflowError>
                     Pure function, no mutation. Returns the resolved target phase.
  error.rs        -- WorkflowError { IllegalTransition { from, to }, InvalidState(String) }
```

### ecc-workflow Binary

```
main.rs           -- clap CLI, subcommand dispatch, dual-mode detection (stdin vs args)
output.rs         -- WorkflowOutput { status: Status, message: String }
                     Status { Pass, Block, Warn }
                     impl Display for JSON serialization
io.rs             -- read_state(), write_state_atomic(), ensure_workflow_dir()
                     Handles CLAUDE_PROJECT_DIR, mktemp+mv pattern
slug.rs           -- make_slug() ported from shell
commands/
  mod.rs
  init.rs         -- creates state.json, archives stale state
  transition.rs   -- validates+applies transition, stamps artifacts, calls memory_write
  toolchain_persist.rs
  memory_write.rs -- 4 subcommands: action, work-item, daily, memory-index
  phase_gate.rs   -- reads stdin JSON, checks path against allowed list
  stop_gate.rs
  grill_me_gate.rs
  tdd_enforcement.rs
  scope_check.rs
  doc_enforcement.rs
  doc_level_check.rs
  pass_condition_check.rs
  e2e_boundary_check.rs
```

### Dependency Graph

```
ecc-workflow --> ecc-domain (WorkflowState, Phase, transition logic)
             --> serde, serde_json (JSON I/O)
             --> clap (CLI arg parsing)
             --> anyhow (error handling in binary)
```

`ecc-workflow` does NOT depend on `ecc-app`, `ecc-ports`, `ecc-infra`, or `ecc-cli`.

### Dual Invocation Mode

The binary auto-detects invocation mode:
1. **CLI args mode**: When called from `!bash` in commands (e.g., `ecc-workflow init dev "feature"`). Arguments come from clap.
2. **Stdin JSON mode**: When called from hooks.json runtime. The hook runtime pipes JSON on stdin containing `tool_name`, `tool_input`, etc. Detected by checking if stdin is a TTY.

For hooks that need stdin (phase-gate, tdd-enforcement), stdin is always read. For CLI-invoked commands (init, transition, toolchain-persist), stdin is ignored.

### Atomic Writes

All state.json mutations use the pattern:
1. Write to `tempfile` in the same directory (using `tempfile` crate or manual mktemp)
2. `std::fs::rename(tempfile, state_file)` -- atomic on same filesystem

### Exit Code Contract

| Status | Exit Code | When |
|--------|-----------|------|
| pass | 0 | Action succeeded |
| warn | 0 | Warning (informational, no block) |
| block | 2 | Action blocked (hook protocol) |
| error | 1 | Unexpected error (bad args, I/O failure) |

---

## AC Coverage Matrix

| AC | PC(s) |
|----|-------|
| AC-001.1 | PC-033..037 (audit is implicit -- all 13 are active, verified by port) |
| AC-001.2 | N/A (no dead scripts found) |
| AC-001.3 | Design document Phase 1 table |
| AC-002.1 | PC-002, PC-009 |
| AC-002.2 | PC-001, PC-003, PC-004 |
| AC-002.3 | PC-005 |
| AC-002.4 | PC-006 |
| AC-002.5 | PC-007 |
| AC-002.6 | PC-008 |
| AC-003.1 | PC-010 |
| AC-003.2 | PC-011 |
| AC-003.3 | PC-012 |
| AC-003.4 | PC-013 |
| AC-003.5 | PC-014 |
| AC-003.6 | PC-015 |
| AC-003.7 | PC-016 |
| AC-003.8 | PC-017 |
| AC-004.1 | PC-018 |
| AC-004.2 | PC-019, PC-032 |
| AC-004.3 | PC-020 |
| AC-004.4 | PC-021, PC-032 |
| AC-004.5 | PC-022..031 |
| AC-004.6 | PC-033, PC-034, PC-037 |
| AC-005.1 | PC-033 |
| AC-005.2 | PC-034 |
| AC-005.3 | PC-036 |
| AC-005.4 | PC-035 |
| AC-006.1 | PC-037 |
| AC-006.2 | PC-038, PC-044 |
| AC-006.3 | PC-039 |
| AC-006.4 | PC-040 |

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | NEEDS WORK (4 prescriptions) | 1 HIGH, 3 MEDIUM |
| Robert | CLEAN | 0 |
| Security | CLEAR | 2 LOW |

SOLID prescriptions (to apply during implementation):
1. [HIGH] Split memory_write.rs into 4 files (action, work_item, daily, memory_index)
2. [MED] Split io.rs into state_reader.rs + state_writer.rs
3. [MED] Group commands/ into workflow/, gates/, checks/, memory/ subdirs
4. [MED] Rename transition to resolve_transition

### Adversary Findings

| Dimension | Verdict | Key Rationale |
|-----------|---------|---------------|
| Coverage | PASS | 46 PCs, all 31 ACs covered |
| Order | PASS | Dependency chains respected, 5 rounds with gates |
| Fragility | PASS (round 2) | Fixture-based equivalence, no shell comparison |
| Rollback | PASS | Phased deletion, coexistence during transition |
| Architecture | PASS | Hexagonal boundaries respected, domain purity enforced |
| Blast Radius | PASS (round 2) | ARCHITECTURE.md added to Phase 6 |
| Missing PCs | PASS (round 2) | Stale archive PC-045, fixture lint PC-046 added |
| Doc Plan | PASS | ADR, ARCHITECTURE.md, glossary, CLAUDE.md, CHANGELOG |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1-6 | crates/ecc-domain/src/workflow/*.rs | Create | US-002 |
| 7-22 | crates/ecc-workflow/**/*.rs | Create | US-003, US-004 |
| 23 | Cargo.toml (workspace) | Modify | US-003 |
| 24-32 | commands/*.md, skills/*.md | Modify | US-005 |
| 33-45 | .claude/hooks/*.sh | Delete | US-006 |
| 46-50 | docs/ARCHITECTURE.md, glossary, CLAUDE.md, ADR, CHANGELOG | Modify/Create | US-006 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-26-replace-hooks-with-rust/spec.md | Full spec |
| docs/specs/2026-03-26-replace-hooks-with-rust/design.md | Full design |
