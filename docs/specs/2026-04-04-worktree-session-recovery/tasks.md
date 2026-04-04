# Tasks: Worktree Session CWD Orphaning Fix

## Pass Conditions

- [ ] PC-001: Empty worktree defers to gc (unit) — AC-001.2, AC-001.3, AC-001.5
- [ ] PC-002: Merge success message no cleanup claim (unit) — AC-001.5
- [ ] PC-003: execute_merge preserves worktree directory (unit) — AC-001.1, AC-001.3
- [ ] PC-004: execute_merge success message no cleanup (unit) — AC-001.5
- [ ] PC-005: execute_merge still deletes branch (unit) — AC-001.1
- [ ] PC-006: Merge failure preserves worktree (unit, existing) — AC-001.4
- [ ] PC-007: session_start runs gc (unit) — AC-002.1
- [ ] PC-008: session_start gc skips alive (unit) — AC-002.2, AC-002.4
- [ ] PC-009: session_start gc failure non-blocking (unit) — AC-002.3
- [ ] PC-010: clippy passes (lint)
- [ ] PC-011: workspace builds (build)
- [ ] PC-012: full test suite passes (suite)
- [ ] PC-013: cargo fmt passes (lint)

## Post-TDD

- [ ] E2E tests
- [ ] Code review
- [ ] Doc updates
- [ ] Supplemental docs
- [ ] Write implement-done.md
