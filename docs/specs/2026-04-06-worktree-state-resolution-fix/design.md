# Solution: Fix Worktree-Scoped Workflow State Resolution

## Spec Reference
Concern: fix, Feature: Fix worktree-scoped workflow state resolution

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-app/src/workflow/state_resolver.rs` | Modify | Add is_worktree guard to migration fallback | US-001 (AC-001.1-4) |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Worktree ignores main repo state | AC-001.1 | `cargo test -p ecc-app workflow::state_resolver::tests::worktree_ignores_main_repo_state -- --exact` | PASS |
| PC-002 | unit | Worktree with own state uses it | AC-001.2 | `cargo test -p ecc-app workflow::state_resolver::tests::worktree_with_own_state -- --exact` | PASS |
| PC-003 | unit | Main repo migration still works | AC-001.3 | `cargo test -p ecc-app workflow::state_resolver::tests::old_location_fallback -- --exact` | PASS |
| PC-004 | unit | All state_resolver tests pass | AC-001.4 | `cargo test -p ecc-app workflow::state_resolver::tests` | PASS |
| PC-005 | lint | state.json untracked | AC-002.1 | `test -z "$(git ls-files .claude/workflow/state.json)"` | exit 0 |
| PC-006 | lint | implement-done.md untracked | AC-002.2 | `test -z "$(git ls-files .claude/workflow/implement-done.md)"` | exit 0 |
| PC-007 | build | cargo build | All | `cargo build` | exit 0 |
| PC-008 | lint | cargo clippy | All | `cargo clippy -- -D warnings` | exit 0 |

## Test Strategy
TDD order: PC-001 → PC-002 (RED), fix code (GREEN), PC-003-004, PC-005-006, PC-007-008.

## Rollback Plan
1. Revert state_resolver.rs change
2. Re-track workflow files: `git add -f .claude/workflow/state.json .claude/workflow/implement-done.md`
