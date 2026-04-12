---
id: BL-148
title: "Formalize session resume/persist/delegate hook lifecycle"
scope: MEDIUM
target: "/spec-dev"
status: open
created: "2026-04-12"
source: "docs/research/competitor-claw-goose.md"
ring: assess
tags: [hooks, session, lifecycle]
---

## Context

Claw Code's plugin lifecycle includes distinct `resume`, `persist`, and `delegate` phases with automatic session-state restoration. ECC has SessionStart / SessionEnd hooks and the `catchup` skill, but session continuity is ad-hoc — there is no formal contract for what state is persisted, what triggers resume, or how work is delegated across sessions.

## Prompt

Promote session continuity patterns currently embedded in `catchup` and the worktree merge flow into first-class lifecycle hooks: `session:persist` (what state to snapshot), `session:resume` (how to restore), `session:delegate` (handoff to a sibling session/worktree). Keep SessionStart/SessionEnd as low-level events; layer the new hooks on top. Consider whether this is a harness-level change (requires Claude Code hook API extension) or can be implemented purely as an ECC convention.

## Acceptance Criteria

- [ ] Formal contract for persist/resume/delegate semantics
- [ ] `catchup` skill refactored to use the new hooks
- [ ] Determine if harness changes required, or pure ECC convention
- [ ] ADR documents the lifecycle model
- [ ] Worktree merge flow integrates with `session:delegate`
