# Tasks: Fix Worktree GC PID Bug (BL-150)

Source: `docs/specs/2026-04-17-worktree-gc-pid-bug/design.md`

## TDD Loop

- [ ] PC-001: parent_id in generated name | AC-001.1/2/3 | status: pending
- [ ] PC-002: WorktreeGcError compiles | AC-004.1/2/3 | status: pending
- [ ] PC-003: unwrap_or(u64::MAX) in source | AC-002.1 | status: pending
- [ ] PC-004: GC skips on unmerged query failure | AC-002.2 | status: pending
- [ ] PC-005: GC still removes stale when Ok(0) | AC-002.3 | status: pending
- [ ] PC-006: is_worktree_stale new signature compiles | AC-003.1/5 | status: pending
- [ ] PC-007: Recently modified = not stale | AC-003.2 | status: pending
- [ ] PC-008: Old + dead PID + old mtime = stale | AC-003.3 | status: pending
- [ ] PC-009: Stat failure = existing behavior | AC-003.4 | status: pending
- [ ] PC-009b: Malformed stat output = failure | AC-003.4 | status: pending
- [ ] PC-010: Live PID overrides old mtime | AC-003.6 | status: pending
- [ ] PC-011: CLAUDE.md gotcha present | AC-005.1 | status: pending
- [ ] PC-012: All tests pass | all | status: pending
- [ ] PC-013: Clippy clean | all | status: pending

## Post-TDD

- [ ] E2E tests (none activated)
- [ ] Code review
- [ ] Doc updates
- [ ] Supplemental docs
- [ ] Write implement-done.md
