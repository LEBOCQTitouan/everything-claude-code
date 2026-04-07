---
description: "Test the ECC hook pipeline in the current repo or an isolated worktree"
---

# ECC Test Mode

> **Narrative**: See narrative-conventions skill.
> Before starting, tell the user what "hooks active" means and what to expect during testing.

Test the full ECC hook pipeline. By default, hooks are bypassed in the ECC repo via `.envrc` (`ECC_WORKFLOW_BYPASS=1`). This command provides two ways to re-enable them for testing.

## Quick Test (same repo)

Launch Claude Code with hooks active in the current repo:

```bash
ECC_WORKFLOW_BYPASS=0 claude
```

All 12 hooks will fire normally. Exit Claude Code to return to bypassed mode.

## Isolated Test (worktree)

Create an isolated worktree using Claude Code's native `EnterWorktree` tool:

1. Call `EnterWorktree` to create an isolated copy of the repository
2. If the worktree path or branch already exists, `EnterWorktree` handles this automatically — no manual fallback needed
3. In the worktree, `ECC_WORKFLOW_BYPASS` is unset (no `.envrc`), so hooks fire normally
4. Run your tests with hooks active

## Cleanup

Call `ExitWorktree` to clean up the isolated worktree. This removes the worktree directory and associated branch automatically.
