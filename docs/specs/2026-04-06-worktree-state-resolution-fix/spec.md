# Spec: Fix Worktree-Scoped Workflow State Resolution

## Problem Statement

The `resolve_state_dir()` function in `state_resolver.rs` has a migration fallback (lines 63-73) that returns the main repo's `.claude/workflow/` path when a worktree has no local state yet. When another session sets the main repo's state to `phase: "plan"`, all worktree sessions are blocked from editing source files by the phase-gate hook. Additionally, `state.json` and `implement-done.md` are tracked in git despite `.gitignore`, causing merge conflicts when rebasing worktree branches onto main.

## Research Summary

- Root cause identified through direct session observation — `resolve_state_dir()` migration fallback fires for worktrees when it should only fire for main repos
- `state_resolver.rs` lines 63-73: `if !new_exists && old_exists` returns `old_location` (main repo path) for worktrees that have no local state
- All workflow commands (`phase_gate`, `stop_gate`, `grill_me_gate`, `tdd_enforcement`, etc.) share the same incorrectly resolved `state_dir`
- Bug 2 (merge prefix rejection) was disproven — `WorktreeName::parse` correctly strips `worktree-` prefix
- `state.json` and `implement-done.md` are tracked despite `.gitignore` — files were committed before the ignore rule

## Decisions Made

| # | Decision | Rationale | ADR? |
|---|----------|-----------|------|
| 1 | Skip migration fallback for worktrees — always use git-dir-scoped path | Worktrees must have independent state; migration only for main repo | No |
| 2 | Untrack state.json and implement-done.md from git index | Gitignored but tracked; untracking prevents merge conflicts | No |

## User Stories

### US-001: Worktree State Independence

**As a** developer using multiple concurrent Claude sessions, **I want** each worktree to have its own workflow state, **so that** one session's phase-gate state does not block another session's file edits.

#### Acceptance Criteria

- AC-001.1: Given a worktree with no local state and the main repo has `phase: "plan"`, when `resolve_state_dir()` is called in the worktree, then it returns the git-dir-scoped path (not the main repo path)
- AC-001.2: Given a worktree with its own state.json, when `resolve_state_dir()` is called, then it returns the worktree's git-dir-scoped path
- AC-001.3: Given the main repo (not a worktree), when the old-location state.json exists but the new-location does not, then migration still works (backward compatibility)
- AC-001.4: All existing state_resolver tests pass after the fix

#### Dependencies

- Depends on: none

### US-002: Untrack Workflow Files from Git

**As a** developer, **I want** `state.json` and `implement-done.md` to not be tracked in git, **so that** worktree branch rebases don't produce merge conflicts on ephemeral workflow files.

#### Acceptance Criteria

- AC-002.1: `git ls-files .claude/workflow/state.json` returns empty (file untracked)
- AC-002.2: `git ls-files .claude/workflow/implement-done.md` returns empty (file untracked)
- AC-002.3: `.gitignore` rule `.claude/workflow/` continues to prevent re-staging

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `crates/ecc-app/src/workflow/state_resolver.rs` | App | Add `is_worktree` guard to migration fallback |

## Constraints

- Main repo migration fallback must continue to work (backward compatibility)
- No changes to the public API of `resolve_state_dir()`
- Fix must not require other sessions to restart

## Non-Requirements

- Not changing the phase-gate hook logic (it correctly uses the state_dir it's given)
- Not changing the merge command (Bug 2 was disproven)
- Not restructuring the state resolution architecture

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| `FileSystem` (state resolution) | Logic fix | Same port, corrected path selection |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CHANGELOG | root | CHANGELOG.md | Add fix entry |
| CLAUDE.md | root | Gotchas section | Update worktree state resolution gotcha |

## Open Questions

None — all resolved during investigation and grill-me interview.
