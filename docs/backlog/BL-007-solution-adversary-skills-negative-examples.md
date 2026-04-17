---
id: BL-007
title: "solution-adversary: skills preload + negative examples"
tier: 2
scope: LOW
target: direct edit
status: implemented
created: 2026-03-20
file: agents/solution-adversary.md
---

## Action

Add `skills: ["clean-craft", "component-principles"]` to frontmatter. Add two negative examples:

- "DO NOT approve solutions that skip rollback planning — every change must be revertible"
- "DO NOT accept pass conditions that can't be verified by a single shell command or test run"
