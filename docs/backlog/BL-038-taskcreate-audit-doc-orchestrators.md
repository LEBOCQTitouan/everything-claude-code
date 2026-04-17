---
id: BL-038
title: Add TaskCreate to audit-full and doc-orchestrator
status: implemented
created: 2026-03-21
scope: LOW
target_command: commands/audit-full.md, agents/audit-orchestrator.md, agents/doc-orchestrator.md
tags: [native, taskcreate, ux, tracking]
---

## Optimized Prompt

Add TaskCreate/TaskUpdate to audit-full command and both orchestrator agents. Create one native task per audit domain (architecture, code, convention, doc, errors, evolution, observability, security, test) or doc pipeline phase. Mark each in_progress when starting and completed when done. This provides spinner UX and cross-session persistence for long-running operations (audits can take 10+ minutes). Add TaskCreate, TaskUpdate, TaskGet, TaskList to allowed-tools in audit-full.md frontmatter.

## Framework Source

- **Native Claude Code**: TaskCreate/TaskUpdate for persistent cross-session tracking with spinner UX

## Related Backlog Items

- None
