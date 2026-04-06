# Tasks: Worktree Auto-Merge and Cleanup Enforcement

## Status Legend
- `pending` — not started
- `red@<ts>` — test written, failing (dispatched)
- `green@<ts>` — test passes, implementation done
- `done@<ts>` — regression verified, committed

## Pass Conditions

| PC | Status | Description | Verifies | Wave |
|----|--------|-------------|----------|------|
| PC-001 | pending | Uncommitted changes → SafetyViolation::UncommittedChanges | AC-001.1 | 1 |
| PC-002 | pending | Untracked files → SafetyViolation::UntrackedFiles | AC-001.2 | 1 |
| PC-003 | pending | Unmerged commits → SafetyViolation::UnmergedCommits | AC-001.3 | 1 |
| PC-004 | pending | Stashed changes → SafetyViolation::StashedChanges | AC-001.4 | 1 |
| PC-005 | pending | Unpushed commits → SafetyViolation::UnpushedCommits | AC-001.5 | 1 |
| PC-006 | pending | All clean → empty vec | AC-001.6 | 1 |
| PC-007 | pending | assess_safety pure: no I/O imports | AC-001.7 | 1 |
| PC-008 | pending | Multiple unsafe conditions all collected | AC-001.8 | 1 |
| PC-009 | pending | WorktreeManager trait compiles (8 methods) | AC-002.1 | 2 |
| PC-010 | pending | OsWorktreeManager detects uncommitted changes | AC-002.2 | 2 |
| PC-011 | pending | OsWorktreeManager detects untracked files | AC-002.2 | 2 |
| PC-012 | pending | OsWorktreeManager counts unmerged commits | AC-002.2 | 2 |
| PC-013 | pending | OsWorktreeManager detects stash | AC-002.2 | 2 |
| PC-014 | pending | OsWorktreeManager checks push status | AC-002.2 | 2 |
| PC-015 | pending | OsWorktreeManager removes worktree | AC-002.2 | 2 |
| PC-016 | pending | OsWorktreeManager deletes branch | AC-002.2 | 2 |
| PC-017 | pending | OsWorktreeManager lists worktrees | AC-002.2 | 2 |
| PC-018 | pending | MockWorktreeManager returns configured values | AC-002.3 | 2 |
| PC-019 | pending | All git commands use -- before user args | AC-002.4 | 2 |
| PC-020 | pending | Safety check runs after ff-merge | AC-003.1 | 3 |
| PC-021 | pending | Safe worktree removed via git worktree remove | AC-003.2 | 3 |
| PC-022 | pending | Safe branch deleted via git branch -d | AC-003.3 | 3 |
| PC-023 | pending | CWD set to repo root before deletion | AC-003.4 | 3 |
| PC-024 | pending | Unsafe worktree preserved + failed checks listed | AC-003.5 | 3 |
| PC-025 | pending | Worktree remove failure → warning | AC-003.6 | 3 |
| PC-026 | pending | Success message "cleaned up successfully" | AC-003.7 | 3 |
| PC-027 | pending | Branch delete failure → warning | AC-003.8 | 3 |
| PC-028 | pending | CWD failure aborts cleanup | AC-003.9 | 3 |
| PC-029 | pending | Safety data via raw commands | AC-003.10 | 3 |
| PC-030 | pending | Safety check inside merge lock | AC-003.11 | 3 |
| PC-031 | pending | Missing dir prunes metadata | AC-003.12 | 3 |
| PC-032 | pending | GC accepts &dyn WorktreeManager | AC-004.1 | 4 |
| PC-033 | pending | list_worktrees replaces porcelain parsing | AC-004.2 | 4 |
| PC-034 | pending | Port methods replace manual commands | AC-004.3 | 4 |
| PC-035 | pending | Existing GC tests pass with MockWorktreeManager | AC-004.4 | 4 |
| PC-036 | pending | GC still uses age + PID staleness | AC-004.5 | 4 |
| PC-037 | pending | Unmerged worktrees skipped | AC-004.6 | 4 |
| PC-038 | pending | --force overrides merge-status safety | AC-004.7 | 4 |
| PC-039 | pending | Status returns all 8 columns | AC-005.1 | 5 |
| PC-040 | pending | Non-session worktrees excluded | AC-005.2 | 5 |
| PC-041 | pending | Human-readable table output | AC-005.3 | 5 |
| PC-042 | pending | Exit code 0 on success | AC-005.4 | 5 |
| PC-043 | pending | Tab-separated snapshot test | AC-005.5 | 5 |
| PC-044 | pending | Hook calls ecc-workflow merge | AC-006.1 | 6 |
| PC-045 | pending | Success "merged and cleaned up" | AC-006.2 | 6 |
| PC-046 | pending | Merge failure unchanged | AC-006.3 | 6 |
| PC-047 | pending | Cleanup failure lists checks | AC-006.4 | 6 |
| PC-048 | pending | Bypass works | AC-006.5 | 6 |
| PC-049 | pending | cargo clippy -- -D warnings | All | 7 |
| PC-050 | pending | cargo build | All | 7 |
| PC-051 | pending | cargo test | All | 7 |

## Post-TDD

| Task | Status |
|------|--------|
| E2E tests | pending |
| Code review | pending |
| Doc updates | pending |
| Supplemental docs | pending |
| Write implement-done.md | pending |
