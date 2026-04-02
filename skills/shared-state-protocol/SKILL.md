---
name: shared-state-protocol
description: Defines read/write contracts for shared state files between team agents
origin: ECC
---

# Shared State Protocol

Defines how agents in a team read and write shared state files. All agents must follow these contracts to avoid conflicts and ensure consistent coordination.

## tasks.md Contract

- **Format**: Markdown checklist with status trail (see `skills/tasks-generation/SKILL.md`)
- **Status values**: `pending`, `red@<ISO>`, `green@<ISO>`, `done@<ISO>`, `failed@<ISO>`
- **Claiming**: Only the parent orchestrator updates task status. Subagents report results; the orchestrator writes the status change.
- **Concurrency**: File-level writes are serialized by the orchestrator. Subagents do not write to tasks.md directly.

## campaign.md Contract

- **Format**: Single orientation file per work item (see `skills/campaign-manifest/SKILL.md`)
- **Sections**: Artifacts table, Grill-Me decisions, Agent outputs, Commit trail
- **Write rules**: Only the parent orchestrator writes to campaign.md. Subagents return results in their output; the orchestrator persists them.
- **Read rules**: Any agent can read campaign.md for orientation context.

## state.json Contract

- **Owner**: `ecc-workflow` binary (or `ecc workflow` unified command)
- **Read**: Any agent can read state.json to determine current phase, artifact paths, and toolchain
- **Write**: Only the workflow binary writes state.json via `ecc-workflow transition` or `ecc workflow transition`. No agent should write directly.
- **Lock**: All reads/writes are serialized via POSIX flock (see `ecc-flock` crate)
