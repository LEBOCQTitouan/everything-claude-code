---
description: "Session resumption summary"
allowed-tools: [Bash, Read, Grep, Glob, LS, AskUserQuestion, TodoWrite]
---

# Catchup

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
