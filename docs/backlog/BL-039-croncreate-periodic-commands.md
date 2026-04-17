---
id: BL-039
title: Add CronCreate suggestion to periodic commands
status: archived
created: 2026-03-21
scope: LOW
target_command: commands/audit-full.md, commands/review.md, commands/verify.md
tags: [native, croncreate, scheduling, recurring]
---

## Optimized Prompt

After /audit-full, /review, and /verify complete successfully, add a suggestion: "Schedule recurring? You can use CronCreate to run this periodically, or /loop <interval> /audit-full for a recurring loop." Document the /loop integration pattern. These commands are described as "periodic" in their docs but don't offer scheduling. This is a UX improvement — users who want recurring audits currently must remember to run them manually.

## Framework Source

- **Native Claude Code**: CronCreate for scheduled task execution, /loop for recurring commands

## Related Backlog Items

- None
