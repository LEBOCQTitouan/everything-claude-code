---
id: BL-075
title: Deterministic task synchronization — single source of truth for TodoWrite and TaskCreate
status: implemented
scope: HIGH
target: /spec dev
created: 2026-03-26
tags: [deterministic, tasks, synchronization, rust-cli]
related: [BL-030, BL-041]
---

# BL-075: Deterministic Task Synchronization

## Problem

The /implement command creates BOTH TodoWrite checklist items AND TaskCreate entries manually, duplicating effort:
- LLM reads tasks.md, creates TodoWrite items for each PC
- LLM also creates TaskCreate entries with matching descriptions
- On re-entry, LLM must reconcile both systems (which items are done?)

This dual-maintenance is error-prone — tasks can drift between the two systems.

## Proposed Solution

### Single Source of Truth: tasks.md

### `ecc tasks sync <tasks-path>`
- Parse tasks.md table (PC ID, description, status trail)
- Generate TodoWrite-compatible checklist from task states
- Generate TaskCreate-compatible entries from pending tasks
- Output as JSON that the LLM can consume directly:

```json
{
  "pending": [
    {"id": "PC-003", "description": "Implement wave analyzer", "status": "pending"},
    {"id": "PC-004", "description": "Add CLI subcommand", "status": "pending"}
  ],
  "completed": ["PC-001", "PC-002"],
  "total": 4,
  "progress_pct": 50
}
```

### `ecc tasks update <tasks-path> <pc-id> <status>`
- Update single PC status in tasks.md
- Append timestamp to status trail: `| green@2026-03-26T14:30:00`
- Validate status transition (pending → red → green → done)
- Write atomically

### Integration
- `/implement` Phase 3 calls `ecc tasks sync` at start of each wave
- tdd-executor calls `ecc tasks update` after each PC completes
- Eliminates manual TodoWrite/TaskCreate management entirely

## Impact

- **Reliability**: Single source of truth eliminates drift
- **Speed**: Instant sync vs LLM reading + cross-referencing
- **Re-entry**: Perfect resume from last state (no LLM re-reading)

## Research Context

Praetorian MANIFEST.yaml: "Stateless agents, external state managed by deterministic code."
OpenHands: typed event sourcing for all state transitions.
