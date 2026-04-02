---
id: BL-014
title: Create design-an-interface skill
tier: 3
scope: HIGH
target: /spec dev
status: "implemented"
created: 2026-03-20
file: skills/design-an-interface/SKILL.md
---

## Action

Spawns 3+ parallel sub-agents, each constrained to produce a radically different interface design for a given module or port. Constraints assigned per agent: (1) minimize method count — aim for 1-3 methods max, (2) maximize flexibility — support many use cases, (3) optimize for the most common case, (4) take inspiration from a named paradigm. Each agent outputs: interface signature, usage example, what it hides internally, tradeoffs. After all complete, compare on: interface simplicity, general-purpose vs specialized, implementation efficiency, depth (small interface hiding significant complexity = good), ease of correct use vs ease of misuse. Synthesize by asking user which design fits their primary use case and whether elements from other designs should be incorporated. Output to `docs/designs/{module}-interface-{date}.md`. Trigger: "design an interface", "design it twice", "explore interface options", "compare API shapes", "what should the port look like". Anti-patterns: "DO NOT let sub-agents produce similar designs — enforce radical difference", "DO NOT skip comparison — the value is in the contrast", "DO NOT implement anything — this is purely about interface shape".
