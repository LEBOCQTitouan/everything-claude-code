# Tasks: Phase-Gate Worktree State Resolution Fix

## Pass Conditions

| ID | Status | Wave | Description |
|----|--------|------|-------------|
| PC-001 | pending | 1 | `migrate_if_needed` copies old state to new dir |
| PC-002 | pending | 1 | `migrate_if_needed` returns false when new exists |
| PC-003 | pending | 1 | `migrate_if_needed` returns false same dir |
| PC-004 | pending | 1 | `migrate_if_needed` returns false no state |
| PC-005 | pending | 1 | `lock_dir_for` returns .locks under base |
| PC-006 | pending | 1 | `acquire_for` creates lock under base_dir |
| PC-007 | pending | 1 | Existing lock_dir/acquire backward compat |
| PC-027 | pending | 1 | `migrate_if_needed` logs tracing warn |
| PC-008 | pending | 2 | `io::read_phase(state_dir)` reads from state_dir |
| PC-009 | pending | 2 | `io::write_state_atomic(state_dir)` writes to state_dir |
| PC-010 | pending | 2 | `io::with_state_lock(state_dir)` locks under state_dir |
| PC-028 | pending | 2 | `ensure_state_dir` error contains path |
| PC-026 | pending | 3 | `contains_encoded_traversal` detects patterns |
| PC-011 | pending | 3 | phase_gate allows resolved state dir during spec |
| PC-012 | pending | 3 | phase_gate allows fallback .claude/workflow/ |
| PC-013 | pending | 3 | phase_gate blocks URL-encoded traversal |
| PC-014 | pending | 3 | phase_gate worktree implement allows src/ |
| PC-015 | pending | 3 | phase_gate worktree spec blocks src/ |
| PC-025 | pending | 3 | Existing phase_gate unit tests pass |
| PC-016 | pending | 4 | scope_check uses dynamic state_dir |
| PC-017 | pending | 4 | init creates at state_dir |
| PC-018 | pending | 4 | tdd_enforcement reads from state_dir |
| PC-019 | pending | 4 | doc_enforcement reads from state_dir |
| PC-020 | pending | 4 | pass_condition_check reads from state_dir |
| PC-021 | pending | 4 | e2e_boundary_check reads from state_dir |
| PC-022 | pending | 4 | archive_state uses state_dir |
| PC-023 | pending | 4 | reset uses state_dir |
| PC-024 | pending | 4 | status reads from state_dir |
| PC-029 | pending | 5 | Worktree phase-gate allows implement writes |
| PC-030 | pending | 5 | Worktree init+transition writes to git-dir |
| PC-031 | pending | 5 | Two worktrees independent phases |
| PC-032 | pending | 5 | Non-git dir fallback |
| PC-033 | pending | 5 | Existing phase_gate binary tests pass |
| PC-034 | pending | 5b | Concurrent migration serialized |
| PC-035 | pending | 5b | Shell hook tests pass |
| PC-036 | pending | 6 | cargo clippy -- -D warnings |
| PC-037 | pending | 6 | cargo build |

## Post-TDD

| Task | Status |
|------|--------|
| E2E tests | pending |
| Code review | pending |
| Doc updates | pending |
| Supplemental docs | pending |
| Write implement-done.md | pending |
