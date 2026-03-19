# ADR-0002: Hook-Based State Machine for Workflows

## Status

Accepted

## Context

ECC's slash commands (`/plan`, `/verify`, `/doc-suite`, etc.) define mandatory multi-phase workflows. Early implementations relied on advisory instructions in command markdown files — Claude was told to follow phases in order, but nothing enforced it. This led to:

- Skipped phases when Claude judged them unnecessary
- Reordered steps that broke prerequisites
- Inconsistent execution across sessions

The workflows needed to be deterministic: every phase executes in order, every gate passes before the next phase starts.

## Decision

Implement workflows as hook-enforced state machines:

1. Each workflow phase is a discrete state
2. Transitions between states are gated by hook checks (build passes, tests pass, lint clean)
3. Phase ordering is enforced by the command definition — Claude cannot skip or reorder
4. Hook profiles (`minimal`, `standard`, `strict`) control which gates are active

The command markdown files declare the workflow as a **MANDATORY WORKFLOW** with explicit phase ordering. Hooks provide the enforcement layer that validates preconditions before each phase transition.

## Consequences

- **Easier**: Workflow execution is deterministic and reproducible
- **Easier**: Quality gates (build, test, lint) catch issues at phase boundaries rather than at the end
- **Harder**: ECC development itself needs a dual-mode approach — strict hooks for users, relaxed hooks during ECC development to avoid circular enforcement
- **Trade-off**: Hooks add overhead to each phase transition; the `minimal` profile exists for speed-sensitive workflows
