---
description: "Test the ECC hook pipeline in the current repo or an isolated worktree"
---

# ECC Test Mode

> **Narrative**: See narrative-conventions skill.
> Before starting, tell the user what "hooks active" means and what to expect during testing.

Test the full ECC hook pipeline. Hooks are managed by the auditable bypass system (ADR-0055). Use `ecc bypass grant` to selectively bypass individual hooks when needed.

## Quick Test (same repo)

Launch Claude Code — all hooks fire by default. To bypass a specific hook:

```bash
ecc bypass grant --hook <hook_id> --reason "testing"
```

## Isolated Test (worktree)

Create an isolated worktree using Claude Code's native `EnterWorktree` tool:

1. Call `EnterWorktree` to create an isolated copy of the repository
2. If the worktree path or branch already exists, `EnterWorktree` handles this automatically — no manual fallback needed
3. In the worktree, hooks fire normally
4. Run your tests with hooks active

## Cleanup

Call `ExitWorktree` to clean up the isolated worktree. This removes the worktree directory and associated branch automatically.
