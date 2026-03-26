---
id: BL-073
title: Deterministic diagram trigger heuristics — auto-detect when diagrams need updating
status: open
scope: LOW
target: /spec dev
created: 2026-03-26
tags: [deterministic, diagrams, heuristics, rust-cli]
related: []
---

# BL-073: Deterministic Diagram Trigger Heuristics

## Problem

The diagram-updater agent manually evaluates trigger heuristics:
- Do changed files span 2+ crates?
- Do any enums have 3+ variants added?
- Is a new crate directory under `crates/`?
- Are there new port traits or adapter implementations?

These are file-system and AST-level checks — deterministic by nature.

## Proposed Solution

### `ecc analyze diagram-triggers [--changed-files <list>]`

Accepts a list of changed files (from git diff or CI) and evaluates:

1. **Cross-crate boundary**: files from 2+ workspace members → suggest dependency diagram
2. **New crate detection**: new directory under `crates/` with Cargo.toml → suggest component diagram
3. **State machine changes**: new enum variants in domain crate → suggest state diagram
4. **Port/adapter changes**: new trait in ports crate or new impl in infra → suggest sequence diagram

Output:
```json
{
  "triggers": [
    {"type": "cross_crate", "reason": "changes span ecc-domain and ecc-app", "diagram": "dependency"},
    {"type": "new_crate", "reason": "crates/ecc-mcp/ added", "diagram": "component"}
  ]
}
```

### Integration
- `/implement` Phase 7.5 calls `ecc analyze diagram-triggers` before spawning diagram-updater
- If no triggers fire, skip diagram update entirely (saves agent spawn cost)

## Impact

- **Speed**: < 50ms heuristic evaluation vs 5-10s LLM analysis
- **Cost**: Avoid spawning diagram-updater agent when no diagrams needed
- **Reliability**: Binary trigger evaluation, no LLM judgment needed
