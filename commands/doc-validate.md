---
description: Validate documentation accuracy, score quality, detect contradictions and duplicates, verify code examples.
---

# Documentation Validation

Validate existing documentation against the actual code. Scores quality across five dimensions, detects contradictions, and verifies examples compile.

## What This Command Does

1. Check doc comments match function signatures (params, types, returns)
2. Score quality using 5-dimension rubric (Presence, Accuracy, Completeness, Clarity, Currency)
3. Detect contradictions between inline docs and project-level docs
4. Find duplicate/conflicting descriptions of the same concept
5. Verify code examples in docs compile and run

## Arguments

- `--module=<name>` — validate a specific module only

## Output

Writes quality report to `docs/DOC-QUALITY.md` (small codebase) or `docs/doc-quality/` folder (large codebase).

Includes:
- Per-module quality scores (0-10) and grades (A-F)
- Issues table with severity (HIGH/MEDIUM/LOW)
- Contradictions list with both locations
- Broken examples list

## Prerequisites

Requires `docs/API-SURFACE.md` from a previous `/doc-analyze` run. If missing, suggests running `/doc-analyze` first.

## When to Use

- To audit documentation quality without generating new docs
- In CI/review workflows to catch doc regressions
- After `/doc-generate` to verify generated docs are accurate
- As part of the full `/doc-suite`

## Related

- Full suite: `/doc-suite`
- Prerequisite: `/doc-analyze`
- Agent: `agents/doc-validator.md`
- Skill: `skills/doc-quality-scoring/SKILL.md`
