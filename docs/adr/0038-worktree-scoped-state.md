# ADR-0038: Scope workflow state to git-dir for worktree isolation

## Status
Accepted (2026-04-02)

## Context
Workflow state (`state.json`) was stored at `<project-dir>/.claude/workflow/`. When Claude entered a git worktree, both the main repo and worktree shared the same state file (via `CLAUDE_PROJECT_DIR`), causing state corruption.

## Decision
Store workflow state at `<git-dir>/ecc-workflow/state.json`:
- For normal repos: `<repo>/.git/ecc-workflow/`
- For worktrees: `<repo>/.git/worktrees/<name>/ecc-workflow/`
- For non-git dirs: fall back to `<project-dir>/.claude/workflow/` with warning
- For existing state at old location: read from old location with migration warning

Resolution uses a new `GitInfo` port trait with `OsGitInfo` adapter.

## Consequences
- Automatic worktree isolation without special logic
- Backward-compatible fallback for non-git directories
- Existing state migrates transparently on first access
- State location is no longer human-predictable (varies by git-dir)
