---
id: BL-122
title: Worktree Auto-Merge and Cleanup Enforcement
status: open
scope: workflow
target: ecc-workflow
tags: [worktree, merge, cleanup, workflow]
created: 2026-04-06
---

# BL-122: Worktree Auto-Merge and Cleanup Enforcement

## Problem

After completing work in a worktree (`/implement` → done), the merge to main and worktree cleanup are manual steps that can be forgotten. This leaves stale worktrees accumulating and work stranded on unmerged branches.

## Desired Behavior

1. When workflow reaches `done` phase in a worktree, **automatically trigger merge** (rebase + verify + ff-merge to main)
2. After successful merge, **automatically delete the worktree** directory and branch
3. Before deletion, **safety check**: verify all commits on the worktree branch are reachable from main (no work lost)
4. If merge fails (conflicts, verify failure), preserve worktree and report — never delete unmerged work
5. Enforce this in the `/implement` command workflow so it's not optional

## Context from Session 2026-04-05

- The `ecc-workflow merge` command exists but has bugs with `worktree-` prefix branches (now fixed)
- `session:end:worktree-merge` hook exists but is fragile — Claude's CWD gets orphaned if worktree is deleted
- `ecc worktree gc` cleans stale worktrees at session start but is best-effort
- The `/implement` command's Phase 7 already calls `ecc-workflow merge` but the merge tool rejected our branch
- State resolution bug (now fixed) was causing the phase-gate to block merges

## Optimized Prompt

Enforce automatic worktree merge and cleanup at the end of `/implement` Phase 7. After workflow transitions to `done`: (1) run `ecc-workflow merge` to rebase, verify (build+test+clippy), and ff-merge to main, (2) after successful merge, run safety check confirming all worktree commits are ancestors of main HEAD, (3) if safe, delete the worktree directory and branch via `git worktree remove` + `git branch -d`, (4) if merge fails, preserve worktree and report the failure. Update the `/implement` command to make this mandatory (not optional). Handle the CWD orphaning issue — defer directory deletion to `ecc worktree gc` at next session start if Claude's CWD is inside the worktree.
