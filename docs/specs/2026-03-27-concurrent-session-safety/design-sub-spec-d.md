# Design: BL-065 Sub-Spec D — Serialized Merge-to-Main

## Spec Reference
Concern: dev, Feature: BL-065 Sub-Spec D: Serialized merge-to-main

## File Changes

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-workflow/src/commands/merge.rs` | CREATE | Merge subcommand with MergeError, orchestration | US-008 |
| 2 | `crates/ecc-workflow/src/commands/mod.rs` | MODIFY | Add module | US-008 |
| 3 | `crates/ecc-workflow/src/main.rs` | MODIFY | Add Merge variant | US-008 |
| 4 | `commands/implement.md` | MODIFY | Phase 7 merge step | AC-008.4-8 |
| 5 | `docs/adr/0024-concurrent-session-safety.md` | CREATE | ADR | AC-009.1 |
| 6 | CHANGELOG.md | MODIFY | Entry | AC-009.2 |
| 7 | `docs/domain/glossary.md` | MODIFY | 3 terms | AC-009.4 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | MergeError maps to WorkflowOutput | AC-008.5-8 | `cargo test -p ecc-workflow -- merge::tests::error_mapping` | PASS |
| PC-002 | unit | acquire uses 60s timeout | AC-008.1 | `cargo test -p ecc-workflow -- merge::tests::acquires_lock` | PASS |
| PC-003 | unit | Timeout returns block | AC-008.7 | `cargo test -p ecc-workflow -- merge::tests::timeout_blocks` | PASS |
| PC-004 | unit | Rebase runs git rebase main | AC-008.2 | `cargo test -p ecc-workflow -- merge::tests::runs_rebase` | PASS |
| PC-005 | unit | Conflict aborts + notifies | AC-008.5, AC-008.8 | `cargo test -p ecc-workflow -- merge::tests::conflict_aborts` | PASS |
| PC-006 | unit | Verify runs build+test+clippy | AC-008.3 | `cargo test -p ecc-workflow -- merge::tests::runs_verify` | PASS |
| PC-007 | unit | Verify failure notifies | AC-008.6 | `cargo test -p ecc-workflow -- merge::tests::verify_failure` | PASS |
| PC-008 | unit | ff-only merge | AC-008.4 | `cargo test -p ecc-workflow -- merge::tests::ff_merge` | PASS |
| PC-009 | unit | Cleanup removes worktree+branch | AC-008.4 | `cargo test -p ecc-workflow -- merge::tests::cleanup` | PASS |
| PC-010 | unit | Rejects main/master branch | Security | `cargo test -p ecc-workflow -- merge::tests::rejects_main` | PASS |
| PC-011 | lint | implement.md references merge | AC-008.4 | `grep -q 'ecc-workflow merge' commands/implement.md` | exit 0 |
| PC-012 | lint | ADR exists | AC-009.1 | `test -f docs/adr/0024-concurrent-session-safety.md` | exit 0 |
| PC-013 | lint | Glossary 3 terms | AC-009.4 | `grep -c 'session worktree\|merge lock\|fast verify' docs/domain/glossary.md` | 3 |
| PC-014 | lint | clippy clean | — | `cargo clippy -- -D warnings` | exit 0 |
| PC-015 | build | cargo build | — | `cargo build` | exit 0 |
| PC-016 | unit | All tests pass | — | `cargo test` | all pass |

## Coverage Check
All 12 ACs covered. AC-009.3 already done in Sub-Spec C.

## Test Strategy
Error-paths first: PC-001→003→005→007→010 (errors), then PC-002→004→006→008→009 (happy path), PC-011-016 (lint/gates).

## SOLID Assessment
PASS (after prescriptions): MergeError enum, focused functions, named constants.

## Robert's Oath Check
CLEAN.

## Security Notes
Session branch guard, `--` separator, rebase abort, no push.

## Rollback Plan
Reverse: revert docs → revert implement.md → remove merge.rs + wiring.
