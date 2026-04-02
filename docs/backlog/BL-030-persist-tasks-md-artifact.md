---
id: BL-030
title: Persist tasks.md as trackable artifact
status: "implemented"
created: 2026-03-21
scope: HIGH
target_command: /implement
tags: [kiro, tasks, persistence, cross-session, catchup]
---

## Optimized Prompt

During /implement Phase 2, write `docs/specs/YYYY-MM-DD-<slug>/tasks.md` from the PC table. Format as a Markdown checklist with PC ID, description, command, and status (pending/red/green/done). Update the file as each PC completes its RED→GREEN→REFACTOR cycle. This creates a standalone, session-independent task tracker that /catchup (BL-017) can read to show progress. Include timestamps for each status transition.

## Framework Source

- **Kiro**: tasks.md as a first-class artifact, independently trackable, updated in real-time

## Related Backlog Items

- Depends on: BL-029 (spec file paths in state.json)
- Enables: BL-017 (/catchup command)
