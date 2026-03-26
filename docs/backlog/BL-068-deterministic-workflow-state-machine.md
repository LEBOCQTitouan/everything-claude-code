---
id: BL-068
title: Deterministic workflow state machine — typed state.json, phase transitions, artifact resolution
status: open
scope: HIGH
target: /spec dev
created: 2026-03-26
tags: [deterministic, workflow, state-machine, rust-cli]
related: [BL-046, BL-052]
---

# BL-068: Deterministic Workflow State Machine

## Problem

The workflow state (spec -> design -> implement) is currently managed by:
- Shell scripts reading/writing `.claude/workflow/state.json` (partially deterministic)
- LLM-interpreted phase instructions in command prompts (non-deterministic)
- Hook-based gates (partially deterministic, but shell-fragile)

Multiple commands manually parse state.json, check phase fields, and resolve artifact paths — duplicating logic across LLM prompts.

## Proposed Solution

### Typed State in Rust
```rust
pub struct WorkflowState {
    pub phase: Phase,          // enum: Idle, Spec, Design, Implement, Done
    pub concern: String,
    pub feature: String,
    pub started_at: DateTime,
    pub artifacts: Artifacts,
}

pub struct Artifacts {
    pub spec_path: Option<PathBuf>,
    pub design_path: Option<PathBuf>,
    pub tasks_path: Option<PathBuf>,
}
```

### CLI Commands
- `ecc workflow status` — show current phase, feature, artifact paths
- `ecc workflow transition <target-phase>` — validate transition is allowed, update state.json atomically
- `ecc workflow artifact <spec|design|tasks>` — resolve and validate artifact path exists
- `ecc workflow reset` — reset to Idle (with confirmation)

### Phase Transition Rules (Compile-Time Enforced)
```
Idle → Spec
Spec → Design
Design → Implement
Implement → Done | Implement (re-entry)
Done → Idle
```

Invalid transitions return error with explanation.

### Hook Integration
Replace current shell-based phase-gate hook with `ecc workflow check-phase <expected>` — same semantics, Rust reliability, zero shell parsing.

## Impact

- **Reliability**: Impossible state transitions prevented at code level (current shell scripts can be bypassed)
- **Speed**: State access in < 1ms (no shell fork + jq overhead)
- **Agent simplification**: Commands lose ~15-20 lines of state-reading boilerplate each
- **Debugging**: `ecc workflow status` gives instant visibility

## Research Context

Praetorian: "State lives in files that deterministic code manages."
OpenHands: "Event sourcing with typed events — all transitions validated."
