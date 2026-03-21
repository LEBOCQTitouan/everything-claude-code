---
id: BL-005
title: Update commands that call robert to handle his output
tier: 2
scope: MEDIUM
target: direct edit
status: open
created: 2026-03-20
files: commands/review.md, commands/verify.md, commands/audit-full.md
---

## Action

Since robert no longer has Write access (BL-004), each calling command must capture his structured findings and write `docs/audits/robert-notes.md` itself. Add a step after robert's invocation: "Write robert's output to `docs/audits/robert-notes-{date}.md`". This is the separation-of-concerns fix — the conscience evaluates, the workflow acts.

## Dependencies

- BL-004 (robert read-only change)
