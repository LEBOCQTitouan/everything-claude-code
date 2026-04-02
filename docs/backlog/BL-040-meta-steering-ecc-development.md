---
id: BL-040
title: Create meta-steering rules for ECC development
status: "implemented"
created: 2026-03-21
scope: LOW
target_command: rules/ecc/development.md (new)
tags: [kiro, steering, meta, conventions, self-enforcing]
---

## Optimized Prompt

Create `rules/ecc/development.md` — a meta-steering file documenting ECC's own development conventions. This rule file steers how ECC agents, commands, skills, and hooks should be built. Include: (1) All agents must have TodoWrite with graceful degradation, (2) All commands with Plan Mode must have EnterPlanMode in allowed-tools, (3) All subagent spawns must specify allowedTools, (4) All adversary verdicts must include rationale, (5) All new skills must be under 500 words for v1, (6) All hooks must check ECC_WORKFLOW_BYPASS, (7) Frontmatter must include name, description, origin, and tools. This is self-enforcing documentation — when Claude reads this rule while working on ECC, it follows these conventions automatically.

## Framework Source

- **Kiro**: Steering files change AI planning behavior (e.g., "always write tests first")

## Related Backlog Items

- None
