---
id: BL-017
title: Create /catchup command
tier: 4
scope: MEDIUM
target: /plan dev
status: open
created: 2026-03-20
file: commands/catchup.md
---

## Action

Session resumption command. Reads: `.claude/workflow/state.json` (current pipeline phase and timestamps), `.claude/workflow/plan.md` and `solution.md` (if present), last session log from ECC tracking, `git log --oneline -10`, `git status`, `git stash list`, `git worktree list`. Produces a structured summary: current phase, what's done, what's pending, uncommitted work, stashed work, orphaned worktrees. Stale state detection: if `state.json` shows `implement` phase but `implement` timestamp is null and last commit is >1h old, offer to reset state or resume. If state shows a completed workflow with no active work, offer to start fresh. Trigger: "catch up", "where was I", "resume session", "what's the current state". Does NOT modify workflow state — only reads and reports.
