---
description: "Session resumption summary"
allowed-tools: [Bash, Read, Grep, Glob, LS, AskUserQuestion, TodoWrite]
---

# Catchup

> Narrate per `skills/narrative-conventions/SKILL.md`.

Resume a session by gathering context from project state.

## Workflow State

Read `.claude/workflow/state.json`. Display: phase, feature, concern, started_at, artifact timestamps. Show spec_path/design_path if set. Handle: malformed → warn, absent → "No active workflow", done → "Workflow complete: <feature>".

## Stale Workflow Detection

If active workflow: compare `git log -1 --format=%ct HEAD` vs `date +%s`. If >3600s (1hr): flag STALE. Explain reset consequences. AskUserQuestion: Resume/Reset (archive to `.claude/workflow/archive/`).

## Tasks Progress

Only when phase=`implement`. Read `artifacts.tasks_path`. Show: total PCs, completed, in-progress, failed (with error summary), pending.

## Git Status

`git status --short`: modified, untracked, staged counts. Clean state → single line. Check stashes (`git stash list`), worktrees (`git worktree list`), recent commits (`git log --oneline -10`).

## Recent Activity

Check `memory/daily/YYYY-MM-DD.md` (today or most recent). Display Activity section. If none: "No session history available."

## Sources Summary

If `docs/sources.md` exists: show entries added/moved/flagged since last commit. Skip if absent.
