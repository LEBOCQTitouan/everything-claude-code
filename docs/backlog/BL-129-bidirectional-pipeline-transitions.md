---
id: BL-129
title: "Bidirectional pipeline transitions with justification logging"
scope: HIGH
target: "/spec-refactor"
status: open
created: "2026-04-07"
tags: [workflow, pipeline, state-machine, transitions, logging]
---

## Context

The current spec→design→implement→done pipeline is one-way. Users and Claude need the ability to go back to earlier phases (e.g., implement→design when a design flaw is discovered) with mandatory justification. Every backward transition should be logged to a transition history for post-mortem analysis of where problems occurred.

## Prompt

Redesign the workflow state machine to support bidirectional phase transitions:

1. **Backward transitions**: Allow implement→design, design→spec, etc. Each backward transition requires a justification string logged to a transition history.
2. **Transition history**: Append-only log of all phase transitions (forward and backward) with timestamp, from/to phases, justification, and actor (user/claude).
3. **Post-mortem analysis**: `ecc workflow history` command to view the transition log for debugging pipeline friction points.
4. **Guard rails**: Backward transitions must preserve work (no data loss). Forward re-entry must pick up where the phase left off.

## Acceptance Criteria

- [ ] Backward transitions supported with justification
- [ ] Transition history persisted to state.json or companion file
- [ ] `ecc workflow history` command displays transition log
- [ ] Forward re-entry resumes from last checkpoint
