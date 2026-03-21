# 6. Doc-first spec-driven pipeline

Date: 2026-03-21

## Status

Accepted

## Context

The ECC pipeline (plan → solution → implement) produced specs and designs in conversation only, without structured user review before execution. The naming ("plan", "solution") didn't accurately describe what the commands produce. Best practices from spec-driven development (BMAD, Kiro, GSD, Spec Kit) emphasize documentation-first workflows where specification artifacts are produced, reviewed, and approved before execution begins.

## Decision

1. Rename the pipeline: `/plan` → `/spec`, `/solution` → `/design` (old names work as aliases)
2. All three pipeline phases (spec, design, implement) enter Plan Mode for user review before execution
3. `/spec` commands draft upper-level doc updates (README, CLAUDE.md) in the plan file
4. `/design` drafts architecture doc updates (ARCHITECTURE.md, diagrams, bounded contexts) in the plan file
5. `/implement` uses native TaskCreate/TaskUpdate alongside TodoWrite for step tracking

## Consequences

**Positive:**
- Users review all artifacts before execution — fewer surprises, less rework
- Doc updates are drafted alongside the spec, not as an afterthought
- Pipeline naming matches output (spec, design) rather than process (plan, solution)
- Plan Mode cleans context between phases, improving output quality
- Backward compatible — old command names still work

**Negative:**
- Additional Plan Mode phase adds one user approval step per command
- Existing muscle memory for /plan and /solution needs retraining (mitigated by aliases)
- Plan file content may duplicate conversation context (mitigated by context cleaning benefit)
