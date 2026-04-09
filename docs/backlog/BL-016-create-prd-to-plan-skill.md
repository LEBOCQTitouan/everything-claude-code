---
id: BL-016
title: Create prd-to-plan skill
tier: 3
scope: MEDIUM
target: /spec dev
status: implemented
created: 2026-03-20
file: skills/prd-to-plan/SKILL.md
---

## Action

Takes a PRD file path as input. Reads the PRD. Decomposes into multi-phase implementation plan using tracer-bullet vertical slices (each phase is end-to-end thin, not horizontal layers). Each phase: description, affected modules, acceptance criteria (must be verifiable by a single command), dependencies on prior phases, estimated complexity, rollback strategy. Output to `docs/plans/{feature}-plan.md`. Trigger: "turn this prd into a plan", "implementation plan", "break this down into phases". Negative example: "DO NOT create horizontal slices — 'write all tests' then 'write all implementation' is forbidden; each phase must be a complete vertical slice".

## Dependencies

- BL-012 (write-a-prd produces the PRD this skill consumes)
