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

If no state.json exists or the phase is `done`, report that no active workflow is in progress.
