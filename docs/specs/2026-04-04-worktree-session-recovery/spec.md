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

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 0 | Root cause vs symptom | Root cause — ECC deletes worktree while session CWD is bound to it (POSIX invariant) | Recommended |
| 1 | Minimal vs proper fix | Deferred cleanup: remove deletion from merge/hook, add session-start gc | Recommended |
| 2 | Missing tests | All 4 scenarios: deferred merge, deferred empty, session-start gc, gc skip active | Recommended |
| 3 | Regression risk | Accept architect's list: accumulation, startup perf, PID race, message wording | Recommended |
| 4 | Related audit findings | CONV-002 anyhow leak already fixed — removed from scope after adversarial review | User (revised) |
| 5 | Reproducibility | Derive from code: EnterWorktree -> commit -> session end -> hook deletes -> ENOENT | Recommended |
| 6 | Data impact | No data migration needed — purely process CWD issue | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Prevent CWD orphaning on session-end merge | 5 | none |
| US-002 | Self-healing worktree cleanup at session start | 4 | US-001 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Merge completes without deleting worktree directory | US-001 |
| AC-001.2 | Empty worktree NOT deleted (deferred to gc) | US-001 |
| AC-001.3 | Worktree directory still exists on disk after hook returns | US-001 |
| AC-001.4 | Merge failure preserves worktree with recovery file | US-001 |
| AC-001.5 | Merge success message does not claim worktree was cleaned up | US-001 |
| AC-002.1 | Session start runs gc to remove stale worktrees (24h/dead PID) | US-002 |
| AC-002.2 | GC skips worktrees with alive owning PID | US-002 |
| AC-002.3 | GC failure does not block session start | US-002 |
| AC-002.4 | GC does not interfere with concurrent merge | US-002 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Ambiguity | 80 | PASS | ACs now reference explicit staleness criteria and PID checks |
| Edge Cases | 78 | PASS | Concurrent gc/merge covered; uncommitted changes fenced as non-requirement |
| Scope Creep Risk | 90 | PASS | US-003 removed; focused on CWD fix only |
| Dependency Gaps | 85 | PASS | US-002 depends on US-001 declared |
| Testability | 80 | PASS | AC-001.3 rewritten as filesystem assertion |
| Decision Completeness | 78 | PASS | Merge message decision added |
| Rollback & Failure | 85 | PASS | Purely subtractive + additive; safe rollback |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-04-worktree-session-recovery/spec.md | Full spec |
| docs/specs/2026-04-04-worktree-session-recovery/campaign.md | Campaign manifest |
| docs/specs/2026-04-04-worktree-session-recovery/spec-draft.md | Pre-adversary draft |
