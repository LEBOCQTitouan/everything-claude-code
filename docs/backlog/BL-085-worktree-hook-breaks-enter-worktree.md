---
id: BL-085
title: "WorktreeCreate/WorktreeRemove hooks break EnterWorktree tool"
scope: HIGH
target: "/spec fix"
status: open
tags: [hooks, worktree, bug, EnterWorktree]
created: 2026-03-28
---

# BL-085: WorktreeCreate/WorktreeRemove hooks break EnterWorktree tool

## Problem

The `EnterWorktree` tool fails with `"WorktreeCreate hook failed: no successful output"`. This completely blocks worktree-based isolation, which is a prerequisite for BL-065 Sub-Spec C (worktree isolation for pipeline sessions).

## Root Cause

`~/.claude/settings.json` registers a `WorktreeCreate` hook:

```json
"WorktreeCreate": [{
  "hooks": [{
    "async": true,
    "command": "ecc-hook \"worktree:create:init\" \"standard,strict\"",
    "timeout": 5,
    "type": "command"
  }],
  "matcher": "*"
}]
```

The handler at `crates/ecc-app/src/hook/handlers/tier3_session/worktree.rs` (`worktree_create_init`) is **logging-only** — it writes to the session file and calls `HookResult::passthrough(stdin)`. It does not create a worktree.

Claude Code interprets `WorktreeCreate` as a **delegation hook** — it expects the hook to actually create the worktree and return a success result. When the hook just passthroughs without creating anything, Claude Code reports failure.

## Affected Components

- `~/.claude/settings.json` — `WorktreeCreate` and `WorktreeRemove` hook entries
- `crates/ecc-app/src/hook/handlers/tier3_session/worktree.rs` — handler implementation
- `hooks/hooks.json` — hook definition

## Fix Options

### Option A: Remove delegation hooks, keep logging via different event (Recommended)
1. Remove `WorktreeCreate` and `WorktreeRemove` entries from settings.json
2. Let Claude Code use native `git worktree` creation (the default when no WorktreeCreate hook exists)
3. Move worktree logging to a `PostToolUse` hook matching `EnterWorktree`/`ExitWorktree` tool names
4. Update `hooks/hooks.json` accordingly

### Option B: Make hooks actually create/remove worktrees
1. Implement full worktree creation in the `worktree_create_init` handler
2. Return the expected output format that Claude Code needs
3. Higher complexity, duplicates Claude Code's native git worktree logic

## Impact

- **Blocks**: BL-065 Sub-Spec C (worktree isolation for pipeline sessions)
- **Blocks**: Any session wanting to use `EnterWorktree` for isolated work
- **Severity**: HIGH — core developer workflow feature is broken

## Ready-to-Paste Prompt

```
/spec fix

Fix: WorktreeCreate/WorktreeRemove hooks in settings.json break the EnterWorktree tool.

Root cause: The hooks delegate worktree creation to ecc-hook but the handler only logs — it doesn't create anything. Claude Code expects WorktreeCreate hooks to be creation delegates.

Fix approach: Remove WorktreeCreate/WorktreeRemove from settings.json. Move session logging to PostToolUse matching EnterWorktree/ExitWorktree. Update hooks/hooks.json.

Affected files:
- ~/.claude/settings.json (WorktreeCreate/WorktreeRemove entries)
- crates/ecc-app/src/hook/handlers/tier3_session/worktree.rs
- hooks/hooks.json

See BL-085 for full analysis.
```
