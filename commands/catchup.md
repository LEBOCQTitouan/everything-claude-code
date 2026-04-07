---
description: "Session resumption summary"
allowed-tools: [Bash, Read, Grep, Glob, LS, AskUserQuestion, TodoWrite]
---

# Catchup

> **Narrative**: See narrative-conventions skill.

Resume a development session by gathering context from the current project state.

## Workflow State

Read `.claude/workflow/state.json` to determine the current workflow status.

Display the following fields from state.json:
- **phase**: the current workflow phase (e.g., spec, design, implement, done)
- **feature**: the feature description being worked on
- **concern**: the concern type (dev, fix, refactor)
- **started_at**: the timestamp when the workflow was started

Check which artifacts have non-null timestamps in state.json:
- **plan**: whether a plan artifact has been produced
- **solution**: whether a solution/design artifact has been produced
- **implement**: whether an implement artifact has been produced

If `artifacts.spec_path` or `artifacts.design_path` are set in state.json, display their file paths so the user can review the persisted spec and design documents.

If state.json exists but cannot be parsed (invalid JSON), warn "state.json is malformed" and continue to the Git Status and Recent Activity sections.

If no state.json exists, report "No active workflow" and continue to the Git Status and Recent Activity sections.

If the phase is `done`, report "Workflow complete: <feature>" along with the completion timestamp from state.json, then continue to Git Status.

## Stale Workflow Detection

After reading the workflow state, if the workflow is active (phase is not `done` and state.json exists with valid JSON), check for staleness:

1. Run `git log -1 --format=%ct HEAD` to get the last commit timestamp
2. Get the current time via `date +%s`
3. Calculate the difference in seconds
4. If the difference is greater than 3600 seconds (1 hour), the workflow is **STALE**:
   - Before offering reset, explain the consequences of resetting: archiving the current state, losing in-progress tracking, and requiring a fresh `/spec` run.
   - Flag the output with "**STALE** — last commit is more than 1 hour old"
   - Use `AskUserQuestion` to prompt the user with two options:
     - "Resume current workflow" — continue normally with no state modifications
     - "Reset workflow state" — archive state.json to `.claude/workflow/archive/state-<timestamp>.json` then delete state.json
5. If the difference is less than or equal to 3600 seconds, do NOT show a staleness warning — proceed normally

## Tasks Progress

Only display this section when the current phase is `implement`.

Read `artifacts.tasks_path` from state.json. If `tasks_path` is set but the file does not exist, report "tasks.md not found at <tasks_path>" and skip the progress summary without erroring.

If the path is set and the tasks.md file exists, parse it to display a progress summary:

- **total**: count all Pass Condition (PC) entries in tasks.md
- **completed (done)**: count entries marked with `[x]` (checked checkbox)
- **in-progress**: identify tasks whose latest status trail entry indicates active work (neither completed nor failed)
- **failed**: count entries with a failed status, and include a brief error summary for each
- **pending**: count entries that have not yet been started (unchecked, no status trail)

Format the output as a concise progress table so the user can quickly see where the implementation stands.

## Git Status

Run `git status --short` to detect uncommitted changes. Categorize and count the output lines:
- **modified**: files with `M` status (tracked files with modifications)
- **untracked**: files with `??` status (new files not yet tracked)
- **staged**: files with status in the index column (`A`, `M`, `D`, `R` in first column)

If there are no uncommitted changes, no stashes, and only a single worktree, report: "Clean — no uncommitted work, no stashes, single worktree."

Run `git stash list` to check for stashed work. If stash entries exist, list each stash entry showing its index, branch, and description.

Run `git worktree list` to check for multiple worktrees. If more than one worktree exists, list all worktrees with their paths and branches.

Run `git log --oneline -10` to show recent commit history. If `git log` fails (e.g., a new repository with zero commits), display "No commits yet" instead of erroring.

This provides context on where the codebase stands regardless of workflow state.

## Recent Activity

Check the project memory directory for daily memory files at `memory/daily/YYYY-MM-DD.md` (relative to the project memory path, e.g., `~/.claude/projects/<hash>/memory/daily/`).

1. **Today's file exists**: If `memory/daily/YYYY-MM-DD.md` exists for today's date, read it and display its Activity section entries as a summary of recent work.

2. **Fallback to most recent file**: If today's file does not exist but other daily files exist in `memory/daily/`, find the most recent one by filename sort order. Display its Activity section with a label: "(from YYYY-MM-DD)" indicating the date of the file used.

3. **No daily files**: If no daily memory files exist in the `memory/daily/` directory (or the directory does not exist), state: "No session history available."

## Sources Summary

If `docs/sources.md` exists:
1. Check for entries modified since the last git commit that modified `docs/sources.md` on the current branch
2. Show entries added, moved between quadrants, or flagged stale since last check

If `docs/sources.md` does not exist, skip this step silently.
