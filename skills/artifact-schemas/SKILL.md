---
name: artifact-schemas
description: Canonical schema definitions for all pipeline artifacts — spec.md, design.md, tasks.md, state.json, and campaign.md
origin: ECC
---

# Artifact Schemas

Single source of truth for the structure of every pipeline artifact. Commands and hooks reference this skill instead of inlining format definitions.

## spec.md

```markdown
# Spec: <title>

## Problem Statement
## Research Summary
## Decisions Made (table: #, Decision, Rationale, ADR Needed?)
## User Stories (### US-NNN with AC-NNN.N)
## Affected Modules
## Constraints
## Non-Requirements
## E2E Boundaries Affected (table)
## Doc Impact Assessment (table)
## Open Questions
```

## design.md

```markdown
# Solution: <title>

## Spec Reference (Concern, Feature)
## File Changes (table: #, File, Action, Rationale, Spec Ref)
## Pass Conditions (table: ID, Type, Description, Verifies AC, Command, Expected)
### Coverage Check
### E2E Test Plan
### E2E Activation Rules
## Test Strategy (TDD order)
## Doc Update Plan (table)
## SOLID Assessment
## Robert's Oath Check
## Security Notes
## Rollback Plan
```

## tasks.md

```markdown
# Tasks: <title>

## Pass Conditions
- [ ] PC-NNN: <description> | `<command>` | <status trail>

Status trail: pending@<ISO> | red@<ISO> | green@<ISO> | done@<ISO>
Failed: failed@<ISO> ERROR: <summary>

## Post-TDD
- [ ] E2E tests | <status>
- [ ] Code review | <status>
- [ ] Doc updates | <status>
- [ ] Write implement-done.md | <status>
```

## state.json

```json
{
  "concern": "dev|fix|foundation|refactor",
  "phase": "plan|solution|implement|done",
  "feature": "<description>",
  "started_at": "<ISO 8601>",
  "toolchain": {
    "test": "<command or null>",
    "lint": "<command or null>",
    "build": "<command or null>"
  },
  "artifacts": {
    "plan": "<ISO timestamp or null>",
    "solution": "<ISO timestamp or null>",
    "implement": "<ISO timestamp or null>",
    "spec_path": "<path or null>",
    "design_path": "<path or null>",
    "tasks_path": "<path or null>",
    "campaign_path": "<path or null>"
  },
  "completed": []
}
```

## campaign.md

See `skills/campaign-manifest/SKILL.md` for the full campaign manifest schema and lifecycle rules.
