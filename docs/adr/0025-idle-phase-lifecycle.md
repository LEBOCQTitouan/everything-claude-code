# ADR 0025: Idle Phase Lifecycle

## Status

Accepted

## Context

The ECC workflow state machine (BL-052) had four phases: Plan, Solution, Implement, Done. Done was a terminal state with no forward transition. Reset deleted state.json entirely, making it impossible to distinguish "workflow never initialized" (no file) from "workflow completed, ready for next task."

This created three problems:
1. No clean cycle closure — Done → (delete) → init is a discontinuity, not a transition
2. Reset destroyed state without archiving (asymmetric with init, which archives)
3. No explicit "resting" state for the state machine

## Decision

Add an `Idle` phase as the fifth variant in the `Phase` enum. The lifecycle becomes:

```
[no file] → init → Plan → Solution → Implement → Done → reset → Idle → init → Plan → ...
```

Key transitions:
- `Done → Idle` (via reset): archives state, writes minimal Idle state
- `Idle → Plan` (via init): begins a new workflow cycle
- `Idle → Idle` (idempotent re-entry)

Reset now archives state.json to `.claude/workflow/archive/` before writing Idle state, symmetric with init's archive behavior. Reset acquires the state lock to prevent TOCTOU races.

`Phase::is_gated()` centralizes the gating logic: Plan and Solution are gated (Write/Edit restricted), while Idle, Implement, and Done are ungated.

## Consequences

### Positive
- Clean state machine cycle with no discontinuities
- "No state.json" unambiguously means "never initialized"
- "phase: idle" unambiguously means "completed, ready for next"
- Reset is recoverable (archived state can be inspected)
- Gating logic lives in the domain layer, not scattered as string comparisons in adapters

### Negative
- Existing state.json files cannot contain "idle" — backward compatible for deserialization but reverting this change would break any Idle-phase files created after deployment
- All exhaustive `match` on `Phase` required updating (scope_check, grill_me_gate, stop_gate, phase_gate) — Rust's compiler enforced this, preventing silent regressions
