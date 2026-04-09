# ADR 0060: Absorb Blueprint Community Skill into PRD-to-Plan

## Status

Accepted

## Context

BL-016 required a skill to decompose PRDs into multi-phase implementation plans using tracer-bullet vertical slices. The existing `blueprint` skill (community origin) provided similar functionality — step-by-step construction plans with dependency graphs, parallel detection, and cold-start briefs — but only accepted one-line objectives, not structured PRD input.

Creating `prd-to-plan` alongside `blueprint` would introduce duplicate planning semantics with no clear differentiation for users.

## Decision

Absorb `blueprint` into `prd-to-plan`:
- Delete `skills/blueprint/` directory
- Create `skills/prd-to-plan/SKILL.md` with dual input modes (PRD file + one-liner)
- Change origin from `community` to `ECC`
- Carry forward: cold-start briefs, dependency graph, parallel step detection, vertical-slice enforcement
- Drop: adversarial review gate (handled by `/design`), plan mutation protocol, branch/PR/CI workflow generation (handled by `/implement`), git/gh detection

## Consequences

- Users who reference `/blueprint` by name will need to use `prd-to-plan` triggers instead
- The `plans/` output path (blueprint) changes to `docs/plans/` (prd-to-plan, already in phase-gate allowlist)
- The skill is positioned as pre-pipeline exploration, not a pipeline replacement — plans end with "run `/spec` to formalize"
