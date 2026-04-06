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

## Harness Metrics

Display harness reliability metrics for the current session (if available).

1. Run `ecc metrics summary --session "$CLAUDE_SESSION_ID" --json` via Bash. If `CLAUDE_SESSION_ID` is not set, use `ecc metrics summary --since 1d --json` as a fallback.
2. If the command fails (non-zero exit, no output, or invalid JSON), skip this section silently — do not display an error.
3. Parse the JSON output. The JSON object has fields: `hook_success_rate`, `phase_gate_violation_rate`, `agent_failure_recovery_rate`, `commit_atomicity_score` (each a number or `null`), and `total_events` (integer).
4. If `total_events` is 0 or all rate fields are `null`, display: "No harness metrics recorded for this session."
5. Otherwise, display a table:

```
Harness Metrics:
  Hook success rate:           XX.X% (or N/A)
  Phase-gate violation rate:   XX.X% (or N/A)
  Agent failure recovery rate: XX.X% (or N/A)
  Commit atomicity score:      XX.X% (or N/A)
```

Format each rate as a percentage (multiply by 100). Display "N/A" for `null` values.

## Git Status

`git status --short`: modified, untracked, staged counts. Clean state → single line. Check stashes (`git stash list`), worktrees (`git worktree list`), recent commits (`git log --oneline -10`).

## Recent Activity

Check `memory/daily/YYYY-MM-DD.md` (today or most recent). Display Activity section. If none: "No session history available."

## Sources Summary

If `docs/sources.md` exists: show entries added/moved/flagged since last commit. Skip if absent.
