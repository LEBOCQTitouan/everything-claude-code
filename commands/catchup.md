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

If `spec_path` or `design_path` are set in state.json, reference them so the user can review the persisted spec and design documents.

If no state.json exists, report "No active workflow" and continue to the Git Status and Recent Activity sections.

If the phase is `done`, report "No active workflow" as well, noting that the previous workflow is complete, then continue to Git Status.

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

Run `git status` and `git log --oneline -10` to show the current branch, uncommitted changes, and recent commit history. This provides context on where the codebase stands regardless of workflow state.
