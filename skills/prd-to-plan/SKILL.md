---
name: prd-to-plan
description: >-
  Decompose a PRD or one-line objective into a multi-phase implementation
  plan using tracer-bullet vertical slices. Each phase is end-to-end thin,
  with cold-start context briefs, dependency graph, and parallel step detection.
  Output to docs/plans/. TRIGGER when: user says "turn this prd into a plan",
  "implementation plan", "break this down into phases", "blueprint", "roadmap",
  or provides a PRD file path for decomposition.
  DO NOT TRIGGER when: task is completable in a single PR or user says "just do it".
origin: ECC
---

# PRD to Plan

Decompose a PRD file or one-line objective into a phased implementation plan using tracer-bullet vertical slices.

## When to Activate

- User says "turn this prd into a plan", "implementation plan", "break this down into phases"
- User says "blueprint" or "roadmap" for a multi-session task
- User provides a PRD file path for decomposition

**Do not activate** for tasks completable in a single PR, fewer than 3 steps, or when user says "just do it".

## Input Modes

### Mode 1: PRD File

User provides a file path (e.g., `docs/prds/feature-prd.md`). Read the PRD using the Read tool. Validate expected sections: Problem Statement, Target Users, User Stories, Non-Goals, Risks & Mitigations, Module Sketch, Success Metrics, Open Questions. If sections are missing, flag gaps and ask whether to proceed or fix the PRD first. If the file doesn't exist, report and ask for a valid path. PRDs from the `write-a-prd` skill follow this template; hand-written PRDs may differ — degrade gracefully.

### Mode 2: One-Liner Objective

User provides a brief objective (e.g., "migrate database to PostgreSQL"). Skip PRD validation; proceed directly to decomposition using the objective as the problem statement.

## Decomposition Process

1. **Explore codebase** — Use Grep and Glob to identify affected modules, existing patterns, and test infrastructure relevant to the objective.

2. **Decompose into vertical slices** — Each phase must be a complete tracer bullet: a thin end-to-end slice through all affected layers (domain, ports, infra, app, CLI, tests). **DO NOT create horizontal slices** — "write all tests" then "write all implementation" is forbidden.

3. **Per-phase structure:**
   - Description (what this phase delivers)
   - Affected modules (from codebase exploration)
   - Acceptance criteria (verifiable by a single command where possible; manual check as fallback)
   - Dependencies on prior phases (no forward references)
   - Complexity: LOW / MEDIUM / HIGH
   - Rollback strategy (how to undo if this phase fails)
   - Self-contained context brief (enough for a fresh agent to execute cold)

4. **Dependency graph** — Order phases by dependency. Identify parallel steps (phases with no shared files or output dependencies). Present the graph visually.

## Output

Write the plan to `docs/plans/{feature}-plan.md` (kebab-case slug, max 40 chars). Create the directory if missing. Include:

- Source reference (PRD path or one-liner objective)
- Phase table with all fields above
- Dependency graph with parallel step markers
- Footer: "This is a draft exploration plan. To execute: run `/spec` to formalize requirements, then `/design` for pass conditions."

## Related Skills

- `write-a-prd` — generates the PRD this skill consumes
- `request-refactor-plan` — similar decomposition scoped to refactoring

## Not Included

Features from the absorbed `blueprint` skill not carried forward: adversarial review gate (use `/design`), plan mutation protocol, branch/PR/CI workflow generation (use `/implement`), git/gh detection.
