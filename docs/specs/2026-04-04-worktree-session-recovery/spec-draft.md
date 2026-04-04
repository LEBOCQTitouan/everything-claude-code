# Spec: Worktree Session CWD Orphaning Fix

## Problem Statement

ECC's `session:end:worktree-merge` hook and `ecc-workflow merge` command delete the worktree directory after a successful merge, orphaning Claude Code's session CWD. This is a fundamental POSIX constraint — no child process (hook) can change the parent process's working directory. The bug manifests as complete session paralysis: every shell command fails with ENOENT because the CWD no longer exists on disk. The only recovery is restarting Claude Code from the main repo root. Two code paths trigger this: (1) the zero-commit cleanup in `session_merge.rs` line 41, and (2) `cleanup_worktree()` in `merge.rs` line 73.

## Research Summary

- `git worktree remove` fails or orphans the caller when CWD is inside the worktree being removed (POSIX constraint)
- Claude Code issue #29260: hooks fail with ENOENT when CWD no longer exists — `posix_spawn` inherits invalid CWD
- Established fix pattern: parent process must validate CWD existence before spawning; child processes cannot fix this
- Signaling alternatives (temp file, stdout, OSC 7) are insufficient for cross-process CWD changes
- Worktrees from crashed sessions accumulate without gc — existing gc handles stale removal
- `WorktreeRemove` hooks are informational only — cannot block removal
- Risk: `git worktree remove` can silently delete branches with unmerged commits

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Defer worktree deletion from merge/hook paths | POSIX invariant makes in-hook deletion unsafe for Claude Code's CWD | No |
| 2 | Add session-start gc trigger | Self-healing: stale worktrees from previous sessions cleaned on next startup | No |
| 3 | Update merge success message | Current message says "Worktree cleaned up" which becomes false after deferring deletion | No |

## User Stories

### US-001: Prevent CWD orphaning on session-end merge

**As a** Claude Code user working in an ECC worktree, **I want** the session-end merge to complete without deleting my active worktree directory, **so that** my session remains functional after a successful merge.

#### Acceptance Criteria

- AC-001.1: Given a worktree with commits ahead of main, when session-end merge runs, then the merge completes (rebase + verify + ff-only) without deleting the worktree directory
- AC-001.2: Given an empty worktree (zero commits ahead), when session-end merge runs, then the worktree is NOT deleted (deferred to gc)
- AC-001.3: Given a successful merge, when the hook returns, then the worktree directory still exists on disk
- AC-001.4: Given a merge failure, when the hook returns, then the worktree is preserved with a recovery file (existing behavior, unchanged)
- AC-001.5: Given a successful merge, then the merge success message does not claim the worktree was cleaned up

#### Dependencies

- Depends on: none

### US-002: Self-healing worktree cleanup at session start

**As a** Claude Code user, **I want** stale worktrees from previous sessions to be automatically cleaned up when I start a new session, **so that** worktrees don't accumulate on disk indefinitely.

#### Acceptance Criteria

- AC-002.1: Given stale worktrees exist (24h age OR dead owning PID per existing `is_worktree_stale()` criteria), when a new session starts, then `ecc worktree gc` runs and removes them
- AC-002.2: Given a worktree whose owning PID is still alive (existing `kill -0` check in `is_worktree_stale()`), when gc runs at session start, then that worktree is skipped
- AC-002.3: Given gc fails, when session start continues, then the session is not blocked (gc is best-effort, errors are logged and swallowed)
- AC-002.4: Given gc runs concurrently with an in-progress merge on another worktree, then gc does not interfere with the merge (existing staleness check prevents this since the merge process PID is alive)

#### Dependencies

- Depends on: US-001 (deletion must be removed from merge path before gc becomes the sole cleanup mechanism)

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `crates/ecc-app/src/hook/handlers/tier3_session/session_merge.rs` | Adapter (hook handler) | Remove worktree deletion from zero-commit path; update success message reference |
| `crates/ecc-workflow/src/commands/merge.rs` | Standalone binary (adapter) | Remove `cleanup_worktree()` call from `execute_merge()`; update success message |
| `crates/ecc-app/src/hook/handlers/tier3_session/lifecycle.rs` | Adapter (hook handler) | Add gc trigger in `session_start` |
| `crates/ecc-app/src/worktree.rs` | Application (use case) | No structural change — existing PID-based staleness check is sufficient |

## Constraints

- Must not change public API of `ecc-workflow merge` (other tooling may call it)
- Must not block session start if gc fails
- Must preserve existing merge failure recovery behavior (recovery file, preserved worktree)
- `ecc-domain` must remain I/O-free
- Post-merge worktree will have a merged branch; gc must handle the branch-already-deleted case (existing behavior: `git branch -D` errors are ignored)

## Non-Requirements

- Not addressing Claude Code's inability to handle deleted CWDs (upstream limitation)
- Not implementing `--no-cleanup` flag on `ecc-workflow merge` (deferred cleanup is unconditional)
- Not adding cron-based gc (session-start trigger is sufficient)
- Not addressing worktrees with uncommitted changes — gc may remove them per existing staleness criteria
- Not changing `ecc worktree gc --force` semantics (the `_force` parameter remains unused for now)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| `ShellExecutor` | Usage change (fewer commands) | Merge flow runs fewer git commands — no port change |
| `FileSystem` | No change | N/A |
| Session hooks | Behavioral change | Session end no longer removes worktree; session start adds gc |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Behavior change | CLAUDE.md | Gotchas section | Update worktree cleanup description |
| New behavior | CLAUDE.md | CLI commands | Note session-start gc behavior |
| Bug fix | CHANGELOG | Next release | Add entry |

## Open Questions

None — all resolved during grill-me interview and adversarial review.
