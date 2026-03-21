---
id: BL-031
title: Fresh context per TDD task via subagent isolation
status: open
created: 2026-03-21
scope: HIGH
target_command: /implement
tags: [gsd, context, subagent, tdd, isolation, quality]
---

## Optimized Prompt

In /implement's TDD loop (Phase 3), spawn each PC's RED→GREEN→REFACTOR cycle as an isolated subagent with `context: "fork"`. The parent provides: the PC spec (ID, command, expected, description), the spec file path (from BL-029), the list of files to modify, and the current test state. The subagent implements the PC and commits. The parent then verifies no regressions by running all prior PCs. This prevents context degradation during long implementations (observed: quality drops after PC-010+ in a single context). The parent retains only PC results, not the full implementation reasoning.

## Framework Source

- **GSD**: Fresh 200K context per task. Each task gets clean context with only relevant spec + files.

## Related Backlog Items

- Depends on: BL-029 (spec file for subagent to read)
- Enables: BL-032 (wave-based parallel TDD)
