---
id: BL-032
title: Wave-based parallel TDD execution
status: implemented
created: 2026-03-21
scope: MEDIUM
target_command: /implement
tags: [gsd, parallel, tdd, performance, waves]
---

## Optimized Prompt

In /implement Phase 2, after parsing PCs, analyze the dependency graph between PCs. Group independent PCs into parallel waves (PCs that touch different files and have no data dependencies). Execute each wave's PCs as concurrent subagents (using BL-031's isolation pattern). Sequential PCs remain sequential. This dramatically speeds up implementations like "add TodoWrite to 14 agents" where all PCs are independent. Include a dependency analysis step that outputs the wave plan before execution.

## Framework Source

- **GSD**: Analyzes task dependencies and runs independent tasks in parallel waves

## Related Backlog Items

- Depends on: BL-031 (subagent isolation per PC)
