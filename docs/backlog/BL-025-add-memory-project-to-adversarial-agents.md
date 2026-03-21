---
id: BL-025
title: "Add memory:project to adversarial agents"
tier: 6
scope: LOW
target: direct edit
status: open
created: 2026-03-20
files: agents/spec-adversary.md, agents/solution-adversary.md, agents/drift-checker.md
---

## Action

Add `memory: project` so these agents accumulate knowledge about recurring weak spots — if the same class of spec weakness keeps appearing, the adversary can escalate rather than rediscovering it each time. Lower priority than robert's memory (BL-004) because adversarial agents are invoked less frequently.
