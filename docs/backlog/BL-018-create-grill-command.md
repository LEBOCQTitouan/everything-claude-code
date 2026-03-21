---
id: BL-018
title: Create /grill command
tier: 4
scope: LOW
target: direct edit
status: open
created: 2026-03-20
file: commands/grill.md
---

## Action

Thin wrapper that invokes the grill-me skill (BL-011) standalone. Takes an optional topic as `$ARGUMENTS`. If no topic provided, asks the user what they want to stress-test. Set `disable-model-invocation: true` — this must be user-initiated, never auto-triggered. Does NOT enter the plan pipeline, does NOT modify `.claude/workflow/state.json`. Output stays in conversation unless user asks to persist (in which case, grill-me skill writes to `docs/interviews/`).

## Dependencies

- BL-011 (grill-me skill)
