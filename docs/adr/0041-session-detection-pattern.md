# ADR 0041: Session Detection Pattern for Backlog Filtering

## Status
Accepted

## Context
Multiple Claude Code sessions can run concurrently on the same repository using worktree isolation. The `/spec` command's backlog picker shows all open items without awareness of which items are being worked on in other sessions, leading to accidental duplicate work.

## Decision
Adopt a hybrid session detection pattern combining two mechanisms:

1. **Worktree name scan**: List `.claude/worktrees/` directories and extract BL-NNN IDs using regex `(?i)bl-?(\d{3})`. This is zero-cost (directory listing) and covers sessions that encode their backlog item in the worktree name.

2. **Advisory lock files**: When `/spec` claims a backlog item, write `docs/backlog/.locks/BL-NNN.lock` containing the worktree name and timestamp. This covers sessions whose worktree names don't contain the BL-NNN pattern. Lock files are gitignored (session-local).

Stale locks (>24h) and orphaned locks (worktree deleted) are automatically cleaned up before filtering. A `--show-all` flag bypasses all filtering as an escape hatch.

## Consequences
- `/spec` picker hides in-progress items by default — reduces duplicate work
- Lock files are advisory (not POSIX locks) — simple, no Rust changes needed
- Stale lock TTL (24h) means abandoned sessions leave temporary false positives
- `--show-all` provides an escape if filtering is too aggressive
- Pattern is reusable by other commands that need session awareness
