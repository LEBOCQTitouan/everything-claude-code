# ADR 0042: Lazy Worktree via Write-Guard Pattern

## Status

Accepted

## Context

We want every Claude Code session to work in an isolated worktree. However, `EnterWorktree` is a Claude Code agent tool — hooks (which are shell commands) cannot call it. The `SessionStart` hook cannot trigger worktree creation.

Three approaches were considered:
- Option A: CLAUDE.md instruction (advisory, not enforced)
- Option B: SessionStart hint (stderr message, not enforced)
- Option C: PreToolUse write-guard that blocks file writes outside worktrees (enforced)

## Decision

Use Option C: a PreToolUse hook on Write/Edit/MultiEdit that checks if the session is in a worktree. If not, it blocks (exit 2) with an actionable message telling Claude to call `EnterWorktree`. Claude sees the block, calls `EnterWorktree`, and retries. This creates a "lazy worktree" — created on first write, not at session start.

Additionally, a SessionEnd hook calls `ecc-workflow merge` to automatically merge the worktree back to main.

## Consequences

- **Positive**: Read-only sessions have zero overhead (no worktree created for questions)
- **Positive**: Write sessions get enforced isolation — no accidental commits to main
- **Positive**: Merge happens automatically at session end
- **Negative**: First write in a session has a one-time delay (worktree creation)
- **Negative**: If `EnterWorktree` tool is unavailable, the user must set `ECC_WORKFLOW_BYPASS=1`
- **Precedent**: This "tool-forcing-via-hook-block" pattern can be reused for other mandatory tool calls
