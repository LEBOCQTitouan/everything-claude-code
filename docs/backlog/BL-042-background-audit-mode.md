---
id: BL-042
title: Add background mode to /audit-full
status: archived
created: 2026-03-21
scope: LOW
target_command: commands/audit-full.md
tags: [native, background, resume, async]
---

## Optimized Prompt

Add `--background` flag to /audit-full. When set, launch the audit-orchestrator agent with `run_in_background: true`. Return immediately with the agent ID so the user can continue working. The user can check status via TaskGet or resume via SendMessage. The audit report is written to docs/audits/ as usual — the user is notified when it completes. This addresses the fact that full audits can take 10+ minutes and currently block the user's session.

## Framework Source

- **Native Claude Code**: run_in_background parameter for long-running agents with resume by ID

## Related Backlog Items

- None
