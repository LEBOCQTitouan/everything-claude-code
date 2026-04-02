---
id: BL-041
title: Add CLAUDE_CODE_TASK_LIST_ID for cross-session persistence
status: "implemented"
created: 2026-03-21
scope: LOW
target_command: .claude/hooks/workflow-init.sh
tags: [native, taskcreate, cross-session, persistence]
---

## Optimized Prompt

In workflow-init.sh, generate and set CLAUDE_CODE_TASK_LIST_ID environment variable (UUID or workflow-based ID). This enables native Claude Code task persistence across sessions for the same workflow. When /implement is interrupted and resumed in a new session, tasks created via TaskCreate in the prior session remain visible and resumable. Export the variable so subagents inherit it.

## Framework Source

- **Native Claude Code**: CLAUDE_CODE_TASK_LIST_ID for cross-session task tracking

## Related Backlog Items

- Enhanced by: BL-030 (tasks.md as file-based backup)
