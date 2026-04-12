# ADR-0061: Grill-Me Foundation-Mode for Project-Level Documents

## Status

Accepted

## Context

ECC has feature-level spec interview (spec-mode) and backlog triage (backlog-mode) in grill-me, but no mode tuned for project-wide foundational documents. The write-a-prd skill covers feature PRDs, not project-level context. BMAD-METHOD research validates multi-persona challenge loops for higher-quality foundational docs. The new `/project-foundation` command needs structured challenge during PRD and architecture creation.

Options considered:
1. **Standalone mode with constraints** — use existing standalone with stage limits in the command. Lightweight but violates the modal pattern.
2. **New foundation-mode** — add a fourth mode. Consistent with existing modal pattern, backward-compatible.
3. **New skill** — create a separate `grill-me-foundation` skill. Duplicates infrastructure.

## Decision

Add foundation-mode to the grill-me skill (option 2). Two sub-invocations:
- **PRD creation**: Clarity + Assumptions stages, maximum 2 questions per stage
- **Architecture creation**: Clarity + Edge Cases stages, maximum 2 questions per stage

Recommended answers enabled (like spec-mode). Depth profile default: standard. Activates only when explicitly requested by `/project-foundation`.

## Consequences

**Positive**: Structured challenge for project-level docs. Consistent with existing modal pattern (standalone/spec/backlog). No new skill files. Backward-compatible — new mode only activates when explicitly requested.

**Negative**: grill-me skill file grows by ~10 lines. Future modes increase cognitive load.

**Risk**: Consumers must explicitly request foundation-mode — no accidental activation. Existing consumers (spec-dev, spec-fix, spec-refactor, backlog) are unaffected.
