# ADR 0024: Concurrent Session Safety Architecture

## Status

Accepted

## Context

Multiple concurrent Claude Code sessions in the same project directory corrupt shared state, lose data, and produce broken builds. The codebase audit (2026-03-26) found 9 race conditions across workflow state, memory files, backlog index, and cargo build artifacts. There was zero file locking and no worktree isolation.

## Decision

Implement concurrent session safety in 4 sub-specs:

1. **Lock Infrastructure (Sub-Spec A)**: `FileLock` port trait with POSIX flock adapter. `ecc-flock` shared crate for raw flock mechanics.
2. **Shared State Locking (Sub-Spec B)**: All state.json, memory, and backlog operations serialize via named flock locks (state.lock, action-log.lock, daily.lock, memory-index.lock, work-item.lock, backlog.lock).
3. **Worktree Isolation (Sub-Spec C)**: Pipeline sessions auto-create git worktrees via `EnterWorktree`. Memory/backlog writes resolve to main repo root. `ecc worktree gc` cleans stale worktrees.
4. **Serialized Merge (Sub-Spec D)**: `ecc-workflow merge` acquires exclusive merge.lock (60s timeout), rebases onto main, runs fast verify (build+test+clippy), performs ff-only merge, cleans up worktree+branch.

Key design choices:
- POSIX flock only — no external dependencies (Redis, etcd, SQLite)
- Advisory locking — cooperative processes only
- Worktree isolation via Claude Code's native `EnterWorktree`/`ExitWorktree` tools
- state.json + .tdd-state are worktree-local; memory + backlog target main repo
- Lock paths resolve to main repo root (worktree-safe via `git rev-parse --git-common-dir`)

## Consequences

- Two concurrent `/implement` sessions can work in parallel without data loss or build conflicts
- Merge serialization ensures only passing code reaches main
- Lock contention is bounded (60s timeout for merge, instant for state/memory)
- POSIX flock auto-releases on process crash — no stale lock cleanup needed
- WorktreeName validation prevents command injection in git operations
