---
id: BL-023
title: Clean up stale workflow state
tier: 5
scope: LOW
target: direct edit
status: open
created: 2026-03-20
file: .claude/workflow/state.json
---

## Action

The current state shows a previous refactoring stuck at `implement` phase with `implement: null`. Either reset to idle or archive the stale state to `.claude/workflow/archive/`. The `/catchup` command (BL-017) will prevent this from recurring by detecting and offering to clean up stale states.
