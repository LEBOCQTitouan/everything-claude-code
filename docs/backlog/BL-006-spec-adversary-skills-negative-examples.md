---
id: BL-006
title: "spec-adversary: skills preload + negative examples"
tier: 2
scope: LOW
target: direct edit
status: "implemented"
created: 2026-03-20
file: agents/spec-adversary.md
---

## Action

Add `skills: ["clean-craft"]` to frontmatter so the agent doesn't need to discover it at runtime. Add two negative examples in the body:

- "DO NOT accept vague acceptance criteria — 'should handle errors appropriately' is not testable, therefore FAIL"
- "DO NOT soften your verdict to avoid conflict — your job is to be hostile to weak specs"
