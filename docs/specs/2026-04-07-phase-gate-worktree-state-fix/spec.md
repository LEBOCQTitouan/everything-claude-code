# Spec: Fix Phase-Gate Hook State Resolution in Worktrees (BL-131)

## Problem Statement

The phase-gate hook (`ecc-workflow phase-gate`) blocks source file writes during spec/design phases. When running in a git worktree, the hook subprocess can resolve CWD to the **main repo** instead of the worktree, causing `git rev-parse --git-dir` to return `.git` (main) rather than `.git/worktrees/<name>`. This means the hook reads the **main repo's** `.git/ecc-workflow/state.json` — which may have a stale blocking phase — instead of the worktree's state.

The existing `is_worktree` guard (BL-131 wave 1, spec 2026-04-06) only prevents fallback to `.claude/workflow/`. It does NOT prevent the hook from reading the main repo's git-dir-scoped state when the subprocess CWD is wrong.

**Root cause**: Hook subprocess CWD can differ from the interactive session CWD in worktrees (documented in Claude Code issues #22343, #30906, #43779).

## Research Summary

- Claude Code hook subprocesses inherit session CWD, but CWD can be wrong on session resume or worktree cleanup
- `git rev-parse --git-dir` depends on CWD being inside the correct worktree checkout
- Pre-commit and lint-staged have documented analogous bugs (pre-commit#2295, lint-staged#886)
- Industry pattern for worktree tools: anchor files or explicit env vars to force resolution
- `.claude/workflow/` is already gitignored, so anchor files there are automatically excluded

## Decisions Made

| # | Decision | Rationale | ADR? |
|---|----------|-----------|------|
| 1 | Dotfile anchor at `.claude/workflow/.state-dir` | Deterministic, no env var inheritance dependency, discoverable from CLAUDE_PROJECT_DIR or CWD | No |
| 2 | Written by `ecc-workflow init` only | Before init, no workflow active so phase-gate passes (no state.json = exit 0) | No |
| 3 | Deleted by `ecc-workflow reset --force` | Clean slate for next workflow in same worktree | No |
| 4 | Include untrack implement-done.md (US-002) | Both worktree-related cleanup, bundle together | No |
| 5 | `.state-dir` format: single-line UTF-8 absolute path, trimmed on read | Minimal overhead, no serde dependency | No |
| 6 | Corrupt or stale `.state-dir` treated as absent — fall back to git resolution | Fail-open to avoid blocking developer work | No |
| 7 | Write ordering: `state.json` first, `.state-dir` second | If `.state-dir` fails, git resolution still works; if `state.json` fails, no workflow active | No |

## User Stories

### US-001: Dotfile Anchor for State Directory Resolution

**As a** developer using multiple concurrent Claude sessions in worktrees, **I want** `resolve_state_dir()` to read a `.state-dir` anchor file before falling back to git resolution, **so that** the hook subprocess always finds the correct `state.json` regardless of CWD.

#### Acceptance Criteria

- AC-001.1: `ecc-workflow init` writes `.claude/workflow/.state-dir` containing the absolute path to the git-dir-scoped state directory (atomic write via mktemp + mv)
- AC-001.2: `resolve_state_dir()` reads `.state-dir` first; if it exists and points to a valid directory, returns that path (skipping git resolution)
- AC-001.3: If `.state-dir` is missing or unreadable, falls back to existing git-based resolution (backward compatibility)
- AC-001.4: `ecc-workflow reset --force` deletes `.state-dir`
- AC-001.5: All existing state_resolver + init + reset tests still pass
- AC-001.6: `.state-dir` contains exactly one line: the absolute path to the state directory, UTF-8 encoded. `resolve_state_dir()` trims whitespace before use.
- AC-001.7: If `.state-dir` exists but its content (after trimming) does not resolve to an existing directory, `resolve_state_dir()` falls back to git-based resolution and emits a warning.
- AC-001.8: `ecc-workflow init` writes `state.json` first, then `.state-dir`. If `.state-dir` write fails, init still succeeds (best-effort optimization, not correctness requirement). Init must `create_dir_all` for `.claude/workflow/` if needed.
- AC-001.9: `ecc-workflow reset --force` attempts to delete `.state-dir` on best-effort basis. Deletion failure emits a warning but does not block the reset.

#### Dependencies

- Depends on: none

### US-002: Untrack Workflow Files from Git Index

**As a** developer, **I want** `implement-done.md` to not be tracked in git, **so that** worktree branch rebases don't produce merge conflicts on ephemeral workflow files.

#### Acceptance Criteria

- AC-002.1: `git ls-files .claude/workflow/implement-done.md` returns empty (file untracked)
- AC-002.2: `.gitignore` rule `.claude/workflow/` continues to cover `.state-dir`

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `crates/ecc-app/src/workflow/state_resolver.rs` | App | Read `.state-dir` anchor before git resolution |
| `crates/ecc-workflow/src/commands/init.rs` | CLI (workflow) | Write `.state-dir` after state.json creation |
| `crates/ecc-workflow/src/commands/reset.rs` | CLI (workflow) | Delete `.state-dir` on force reset |

## Constraints

- Main repo migration fallback must continue to work (backward compatibility)
- No changes to the public API of `resolve_state_dir()`
- No changes to phase-gate logic (it correctly uses the state_dir it's given)
- `.state-dir` must be written atomically (mktemp + mv)
- Fix must not require other sessions to restart
- No new crate dependencies

## Non-Requirements

- Not changing hook configuration or subprocess environment
- Not restructuring the state resolution architecture
- Not adding ECC_STATE_DIR env var (dotfile approach chosen instead)
- Not changing the merge command

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| `FileSystem` (state resolution) | Read `.state-dir` anchor | Same port, new read path |
| `FileSystem` (init/reset) | Write/delete `.state-dir` | Same port, new write/delete |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CHANGELOG | root | CHANGELOG.md | Add fix entry |
| CLAUDE.md | root | Gotchas section | Update worktree state resolution gotcha |

## Open Questions

None — all resolved during investigation and grill-me interview.
