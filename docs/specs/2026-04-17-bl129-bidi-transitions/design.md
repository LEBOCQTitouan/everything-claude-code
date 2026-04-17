# Solution: BL-129 Bidirectional Pipeline Transitions

## Spec Reference
Concern: refactor, Feature: BL-129 bidirectional pipeline transitions

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/workflow/transition.rs` | Modify | Add Direction, TransitionPair, TransitionPolicy (default=forward-only, with_backward=full), TransitionResult, TransitionResolver trait. Rewrite resolve_transition to delegate to default policy. Add resolve_transition_with_justification | US-001: AC-001.1-4, US-002: AC-002.2, AC-002.7 |
| 2 | `crates/ecc-domain/src/workflow/error.rs` | Modify | Add MissingJustification variant to WorkflowError | US-002: AC-002.2, AC-002.7 |
| 3 | `crates/ecc-domain/src/workflow/state.rs` | Modify | Add TransitionRecord struct, `#[serde(default)] history: Vec<TransitionRecord>` to WorkflowState, clear_artifacts_for_rollback() on Artifacts | US-002: AC-002.3-4,6,8, US-003: AC-003.1-2 |
| 4 | `crates/ecc-domain/src/workflow/mod.rs` | Modify | Re-export Direction, TransitionPolicy, TransitionResult, TransitionResolver, TransitionRecord | All |
| 5 | `crates/ecc-domain/src/workflow/phase_verify.rs` | Modify | Add backward-direction hints to phase_hint() | US-002: AC-002.2 |
| 6 | `crates/ecc-workflow/src/commands/transition.rs` | Modify | Accept --justify arg, use with_backward() policy, build TransitionRecord, append to history, call clear_artifacts_for_rollback on backward | US-002: AC-002.1,3-6,8, US-003: AC-003.1-2 |
| 7 | `crates/ecc-workflow/src/commands/history.rs` | Create | Display transition history (text table + JSON output) | US-003: AC-003.3-4 |
| 8 | `crates/ecc-workflow/src/commands/mod.rs` | Modify | Add `pub mod history` | US-003 |
| 9 | `crates/ecc-workflow/src/commands/reset.rs` | Modify | Preserve history in archive before reset | US-003: AC-003.5 |
| 10 | `crates/ecc-workflow/src/main.rs` | Modify | Wire History subcommand with --json flag, add --justify to Transition | US-002, US-003 |
| 11 | `crates/ecc-cli/src/commands/workflow.rs` | Modify | CLI delegation: --justify on transition, history subcommand | US-002, US-003 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | integration | Full forward lifecycle: idle→plan→solution→implement→done→idle | AC-004.1 | `cargo test -p ecc-domain -- lifecycle::forward_lifecycle_completes` | PASS |
| PC-002 | unit | All 30 existing transition tests pass unchanged | AC-001.3, AC-004.2 | `cargo test -p ecc-domain -- workflow::transition::tests` | 30 tests PASS |
| PC-003 | unit | TransitionPolicy::default() has 6 forward pairs only | AC-001.1 | `cargo test -p ecc-domain -- policy::default_policy_has_forward_pairs` | PASS |
| PC-004 | unit | Forward via TransitionResolver trait returns Ok with direction=Forward | AC-001.2 | `cargo test -p ecc-domain -- resolver::forward_via_trait` | PASS |
| PC-005 | unit | Backward with justification via with_backward() policy returns Ok | AC-001.4 | `cargo test -p ecc-domain -- resolver::backward_with_justification` | PASS |
| PC-006 | unit | Backward with None justification returns Err(MissingJustification) | AC-002.2 | `cargo test -p ecc-domain -- resolver::backward_missing_justification` | PASS |
| PC-007 | unit | Backward with empty string justification returns Err | AC-002.7 | `cargo test -p ecc-domain -- resolver::backward_empty_justification` | PASS |
| PC-008 | unit | Backward with whitespace-only justification returns Err | AC-002.7 | `cargo test -p ecc-domain -- resolver::backward_whitespace_justification` | PASS |
| PC-009 | unit | clear_artifacts_for_rollback(Implement, Solution) clears solution+implement timestamps, design_path+tasks_path | AC-002.3, AC-002.8 | `cargo test -p ecc-domain -- artifacts::clear_impl_to_solution` | PASS |
| PC-010 | unit | clear_artifacts_for_rollback(Solution, Plan) clears plan+solution timestamps, spec_path+design_path | AC-002.4, AC-002.8 | `cargo test -p ecc-domain -- artifacts::clear_solution_to_plan` | PASS |
| PC-011 | unit | clear_artifacts_for_rollback(Implement, Plan) clears all three timestamps + all three paths | AC-002.6, AC-002.8 | `cargo test -p ecc-domain -- artifacts::clear_impl_to_plan` | PASS |
| PC-012 | unit | Forward re-entry after rollback re-stamps timestamp (not original) | AC-002.5 | `cargo test -p ecc-workflow -- transition::tests::forward_reentry_restamps` | PASS |
| PC-013 | unit | TransitionRecord serde roundtrip (JSON serialize+deserialize) | AC-003.2 | `cargo test -p ecc-domain -- state::transition_record_serde` | PASS |
| PC-014 | unit | WorkflowState with no history field deserializes with history=[] | AC-003.1, AC-003.2 | `cargo test -p ecc-domain -- state::history_default_empty` | PASS |
| PC-015 | integration | Binary backward impl→solution: phase changes, artifacts cleared, history record appended | AC-002.1, AC-002.3, AC-003.1 | `cargo test -p ecc-workflow -- transition::tests::backward_impl_to_solution` | PASS |
| PC-016 | integration | Binary backward without --justify returns block | AC-002.2 | `cargo test -p ecc-workflow -- transition::tests::backward_no_justify_blocks` | PASS |
| PC-017 | integration | ecc-workflow history displays records chronologically | AC-003.3 | `cargo test -p ecc-workflow -- history::tests::history_displays_chronologically` | PASS |
| PC-018 | integration | ecc-workflow history --json outputs valid JSON array | AC-003.4 | `cargo test -p ecc-workflow -- history::tests::history_json_output` | PASS |
| PC-019 | integration | After ecc-workflow reset --force, history preserved in archive | AC-003.5 | `cargo test -p ecc-workflow -- reset::tests::reset_preserves_history_in_archive` | PASS |
| PC-020 | unit | Direction enum serde roundtrip (forward/backward as lowercase) | AC-003.2 | `cargo test -p ecc-domain -- direction::serde_roundtrip` | PASS |
| PC-021 | unit | Done→Plan, Done→Solution, Done→Implement remain illegal | AC-001.1 | `cargo test -p ecc-domain -- resolver::done_backward_still_illegal` | PASS |
| PC-022 | unit | Forward transitions accept None justification (backward compat) | AC-001.2 | `cargo test -p ecc-domain -- resolver::forward_accepts_none_justification` | PASS |
| PC-023 | unit | MissingJustification Display contains "justification must be non-empty" | AC-002.7 | `cargo test -p ecc-domain -- error::missing_justification_message` | PASS |
| PC-024 | lint | cargo clippy -p ecc-domain zero warnings | Build | `cargo clippy -p ecc-domain -- -D warnings` | exit 0 |
| PC-025 | lint | cargo clippy -p ecc-workflow zero warnings | Build | `cargo clippy -p ecc-workflow -- -D warnings` | exit 0 |
| PC-026 | lint | cargo clippy -p ecc-cli zero warnings | Build | `cargo clippy -p ecc-cli -- -D warnings` | exit 0 |
| PC-027 | build | Full workspace tests pass | AC-001.3, AC-004.2 | `cargo test` | exit 0 |
| PC-028 | lint | cargo fmt check passes | Build | `cargo fmt --check` | exit 0 |
| PC-029 | build | cargo build succeeds | Build | `cargo build` | exit 0 |

### Coverage Check

All 19 ACs covered:

| AC | Covering PCs |
|----|-------------|
| AC-001.1 | PC-003, PC-021 |
| AC-001.2 | PC-004, PC-022 |
| AC-001.3 | PC-002, PC-027 |
| AC-001.4 | PC-005 |
| AC-002.1 | PC-015 |
| AC-002.2 | PC-006, PC-016 |
| AC-002.3 | PC-009, PC-015 |
| AC-002.4 | PC-010 |
| AC-002.5 | PC-012 |
| AC-002.6 | PC-011 |
| AC-002.7 | PC-007, PC-008, PC-023 |
| AC-002.8 | PC-009, PC-010, PC-011 |
| AC-003.1 | PC-014, PC-015 |
| AC-003.2 | PC-013, PC-014, PC-020 |
| AC-003.3 | PC-017 |
| AC-003.4 | PC-018 |
| AC-003.5 | PC-019 |
| AC-004.1 | PC-001 |
| AC-004.2 | PC-002, PC-027 |

**Zero uncovered ACs.**

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | ecc-workflow transition | Binary | TransitionResolver | Backward transition with --justify | ignored | transition.rs modified |
| 2 | ecc-workflow history | Binary | WorkflowState | History text + JSON output | ignored | history.rs created |

### E2E Activation Rules

Both boundaries activated — PC-015/016 cover transition, PC-017/018 cover history.

## Test Strategy

TDD order (dependency-driven):

1. **Phase 0**: PC-001 (lifecycle backfill — safety net, GREEN immediately)
2. **Phase 1**: PC-003, PC-020, PC-023 (Direction, TransitionPolicy, MissingJustification — new types)
3. **Phase 2**: PC-002, PC-004, PC-005, PC-006, PC-007, PC-008, PC-021, PC-022 (TransitionResolver trait + backward transitions)
4. **Phase 3**: PC-009, PC-010, PC-011, PC-013, PC-014 (artifact clearing + TransitionRecord)
5. **Phase 4**: PC-012, PC-015, PC-016 (binary backward transitions + re-entry)
6. **Phase 5**: PC-017, PC-018 (history subcommand)
7. **Phase 6**: PC-019 (reset history preservation)
8. **Phase 7**: PC-024-029 (lint + fmt + build verification)

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/adr/0064-bidirectional-transitions.md` | Docs | Create | TransitionPolicy, backward pairs, justification, artifact clearing | Decision 1 |
| 2 | `docs/adr/0065-workflow-sap-trait-extraction.md` | Docs | Create | TransitionResolver trait, D=1.00→lower | Decision 6 |
| 3 | `docs/domain/bounded-contexts.md` | Domain | Modify | Add TransitionPolicy, TransitionRecord, Direction to workflow context | US-001, US-003 |
| 4 | `CLAUDE.md` | Project | Modify | Add ecc workflow history, document backward transitions, update test count | US-002, US-003 |
| 5 | `CHANGELOG.md` | Project | Modify | Add BL-129 entry | All US |

## SOLID Assessment

**PASS** with prescriptions:
- **LSP (MEDIUM)**: TransitionPolicy::default() must contain ONLY forward pairs. with_backward() adds backward. resolve_transition wraps default — can never silently permit backward.
- **CCP (LOW)**: Consider splitting transition.rs into policy.rs + record.rs if it exceeds 400 lines.

## Robert's Oath Check

**CLEAN** with warnings:
- Oath 2: LSP prescription must be followed (default forward-only)
- Oath 6: Extract handle_backward_transition() helper in binary transition.rs to keep run_with_store under 50 lines

## Security Notes

**CLEAR** (2 LOW):
- Justification string: no length cap (informational — local CLI tool)
- Actor field: free-text, no validation (only "ecc-workflow" today)

## Rollback Plan

Reverse dependency order:
1. Revert CLI delegation (`ecc-cli/src/commands/workflow.rs`)
2. Revert binary: main.rs (History), history.rs (delete), transition.rs (remove --justify, backward logic, history append), reset.rs (history preservation), mod.rs
3. Revert domain: state.rs (remove TransitionRecord, history field, clear_artifacts), transition.rs (restore matches! macro), error.rs (remove MissingJustification), phase_verify.rs, mod.rs
4. Revert docs: delete ADRs, revert CLAUDE.md/CHANGELOG/bounded-contexts

Git revert per commit is safe since each TDD phase is an atomic commit.

## Bounded Contexts Affected

| Context | Role | Files Modified |
|---------|------|----------------|
| workflow | Entity + Service | transition.rs, state.rs, error.rs, phase_verify.rs, mod.rs |

Other domain modules: none

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS | 2 (LSP MEDIUM, CCP LOW) |
| Robert | CLEAN | 2 warnings (Oath 2, Oath 6) |
| Security | CLEAR | 2 LOW |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| AC Coverage | 90 | PASS | All 19 ACs covered by 29 PCs |
| Execution Order | 80 | PASS | Dependency-driven 8-phase TDD |
| Fragility | 85 | PASS | LSP prescription, serde(default) explicit |
| Rollback | 88 | PASS | Reverse dependency order documented |
| Architecture | 92 | PASS | TransitionResolver in domain, no I/O bleed |
| Blast Radius | 75 | PASS | 11 files, contained to workflow context |
| Missing PCs | 85 | PASS | fmt + build PCs added (round 2 fix) |
| Doc Plan | 80 | PASS | 2 ADRs, CHANGELOG, bounded-contexts, CLAUDE.md |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | crates/ecc-domain/src/workflow/transition.rs | modify | US-001, US-002 |
| 2 | crates/ecc-domain/src/workflow/error.rs | modify | US-002 |
| 3 | crates/ecc-domain/src/workflow/state.rs | modify | US-002, US-003 |
| 4 | crates/ecc-domain/src/workflow/mod.rs | modify | All |
| 5 | crates/ecc-domain/src/workflow/phase_verify.rs | modify | US-002 |
| 6 | crates/ecc-workflow/src/commands/transition.rs | modify | US-002, US-003 |
| 7 | crates/ecc-workflow/src/commands/history.rs | create | US-003 |
| 8 | crates/ecc-workflow/src/commands/mod.rs | modify | US-003 |
| 9 | crates/ecc-workflow/src/commands/reset.rs | modify | US-003 |
| 10 | crates/ecc-workflow/src/main.rs | modify | US-002, US-003 |
| 11 | crates/ecc-cli/src/commands/workflow.rs | modify | US-002, US-003 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-17-bl129-bidi-transitions/spec.md | Full spec |
| docs/specs/2026-04-17-bl129-bidi-transitions/design.md | Full design |
