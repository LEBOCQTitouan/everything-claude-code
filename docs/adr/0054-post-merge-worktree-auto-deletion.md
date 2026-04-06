# ADR 0054: Post-Merge Worktree Auto-Deletion with Safety Checks

## Status

Accepted

## Context

Session worktrees accumulate after merge because deletion was deferred to `ecc worktree gc` at next session start. The merge was best-effort (warned on failure, never blocked). This caused disk bloat, branch litter, and risk of stranded work on unmerged branches.

Key constraints:
- Deleting a worktree while Claude Code's CWD is inside it breaks the session
- Concurrent sessions require serialized merge operations
- The `ecc-workflow` binary is a standalone process outside the hexagonal architecture stack

## Decision

1. **Mandatory merge at session end**: Always attempt merge, never silently skip. On failure: preserve worktree, report clearly.

2. **5-point safety check before deletion**: Uncommitted changes, untracked files, unmerged commits, stash, remote push. All conditions collected (not short-circuited). Safety assessment is a pure function in `ecc-domain` (`WorktreeSafetyInput` → `Vec<SafetyViolation>`).

3. **CWD handling**: Git commands in cleanup use `Command::current_dir(repo_root)` (per-command, not process-global `std::env::set_current_dir`).

4. **No backwards-compat flag**: Merge now always includes cleanup. Only caller is the session hook.

5. **New `WorktreeManager` port**: Dedicated port trait in `ecc-ports` with 8 methods (ISP from `GitInfo`). Used by `ecc-app` (GC, status command) but NOT by `ecc-workflow`.

6. **GC refactored to use port**: Eliminates raw shell git calls in app layer. GC now skips unmerged worktrees unless `--force`.

7. **ecc-workflow uses raw commands**: The merge cleanup in `ecc-workflow` gathers safety data via `std::process::Command`, not the `WorktreeManager` port. This keeps the standalone binary decoupled from the hexagonal stack. The canonical port-based implementation lives in `ecc-infra::os_worktree`.

8. **Safety check inside merge lock**: The safety check runs after `merge_fast_forward` but before the lock guard is dropped, preventing concurrent safety-check-vs-write races.

## Consequences

### Positive

- Clean session end: worktrees auto-deleted when safe, no manual gc needed
- Domain rule testable in isolation (pure function, zero I/O)
- New `WorktreeManager` port enables future GC improvements and `ecc worktree status` command
- 5-point safety check with collect-all semantics prevents any data loss

### Negative

- Safety-check logic exists in two places: `ecc-workflow` (raw commands) and `ecc-infra` (port adapter). Domain `assess_safety` is shared, but data gathering is duplicated. This is an intentional trade-off to keep `ecc-workflow` decoupled.
- Merge.rs + merge_cleanup.rs total ~1330 lines (split across two files to stay under 800-line limit)

### Alternatives Considered

1. **Deferred deletion via GC only**: Rejected — doesn't solve the "merge succeeded but worktree lingers" UX.
2. **Full hexagonal refactor of merge**: Deferred to Phase 2 — too large for this scope.
3. **`--no-cleanup` backwards-compat flag**: Rejected — only caller is the session hook, simpler without.
