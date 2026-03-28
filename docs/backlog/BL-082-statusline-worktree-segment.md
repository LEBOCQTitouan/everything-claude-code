---
id: BL-082
title: Add worktree display segment to statusline
scope: LOW
target: direct edit
status: promoted
created: 2026-03-27
---

# BL-082: Add Worktree Display Segment to Statusline

## Problem

The statusline has no indicator when Claude Code is running inside a git worktree. During `/implement` TDD waves, agents run in isolated worktrees but the user has no visual feedback about which worktree is active or what branch it tracks.

## Proposal

Add a worktree segment to `statusline/statusline-command.sh` that displays the worktree name and its branch when the current directory is inside a worktree. Hidden when not in a worktree.

Format: `🌳 wt-bl-052 (feat/rust-hooks)`

The `StatuslineField::Worktree` variant already exists in the Rust domain enum — just needs wiring in the bash script.

## Ready-to-Paste Prompt

```
Direct edit to statusline/statusline-command.sh.

Add a worktree segment that:
1. Detects if CWD is inside a git worktree (git rev-parse --git-common-dir != .git)
2. Extracts worktree name from the worktree path (basename of CWD)
3. Extracts the branch tracked by this worktree (git branch --show-current)
4. Renders as: 🌳 <worktree-name> (<branch>)
5. Only shown when inside a worktree — hidden otherwise
6. Insert between Git Branch and Rate Limits in render priority
7. Cache worktree detection with the same 5-second TTL as git branch

The StatuslineField::Worktree enum variant already exists in
crates/ecc-domain/src/config/statusline.rs — no Rust changes needed.
```

## Scope Estimate

LOW — single segment addition to existing bash script, enum already exists.
