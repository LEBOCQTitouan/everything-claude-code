---
name: doc-guidelines
description: Documentation guidelines (CAPITALISED rules) and quality gate thresholds for the doc-suite pipeline. Defines what MUST be documented and blocking/warning criteria.
origin: ECC
---

# Documentation Guidelines

Persistent, prominent guidelines for WHAT SHOULD BE DOCUMENTED and quality gate thresholds that enforce documentation standards across the pipeline.

## When to Activate

- Running `/doc-suite` or any doc-related command
- Planning documentation work
- Reviewing documentation completeness
- Setting up documentation standards for a new project

## DOCUMENTATION GUIDELINES

These rules define what every project MUST document. They are intentionally CAPITALISED to signal that they are non-negotiable standards.

- `ALWAYS DOCUMENT PUBLIC API ENDPOINTS AND THEIR REQUEST/RESPONSE SCHEMAS`
- `ALWAYS DOCUMENT ARCHITECTURAL DECISIONS AS ADRS IN docs/ADR/`
- `ALWAYS DOCUMENT ENVIRONMENT VARIABLES WITH REQUIRED VS OPTIONAL STATUS`
- `ALWAYS DOCUMENT BREAKING CHANGES IN CHANGELOG`
- `ALWAYS DOCUMENT SETUP AND ONBOARDING STEPS`
- `ALWAYS DOCUMENT ERROR CODES AND THEIR MEANINGS`
- `ALWAYS DOCUMENT DATA MODELS AND THEIR RELATIONSHIPS`
- `ALWAYS DOCUMENT DEPLOYMENT AND ROLLBACK PROCEDURES`
- `NEVER DOCUMENT IMPLEMENTATION DETAILS IN PUBLIC API DOCS — DOCUMENT BEHAVIOR`
- `NEVER LET DOCUMENTATION DRIFT MORE THAN 1 SPRINT BEHIND CODE`

## Quality Gates

Quality gates determine whether the doc-suite pipeline passes or fails.

### Blocking (pipeline fails)

| Condition | Threshold |
|-----------|-----------|
| Accuracy score | < 4 (out of 10) |
| CLAUDE.md contradictions | Any HIGH severity |

If any blocking condition is met, the pipeline reports failure and lists the blocking issues for immediate resolution.

### Warning (pipeline passes with warnings)

| Condition | Threshold |
|-----------|-----------|
| Quality grade | Below B (< 7.0/10) |
| Staleness | Any doc > 90 days stale relative to code |
| File size violation | Any doc file outside size guidelines |
| Coverage | Below 70% documented exports |

Warnings are displayed in the console summary but do not block the pipeline.

## File Size Guidelines

| Metric | Threshold | Action |
|--------|-----------|--------|
| Minimum | 20 lines | Flag as potentially insufficient — doc may lack meaningful content |
| Recommended maximum | 300 lines | Recommend splitting into focused sub-documents |
| Hard maximum | 500 lines | Flag as too large — must split for readability |
| README.md | Exempt | No maximum — README serves as the project entry point |

## How Agents Use These Guidelines

- **doc-orchestrator**: Displays the CAPITALISED guidelines during Phase 0 (plan) and checks quality gates after all phases complete
- **doc-validator**: Applies file size validation and reports findings with appropriate severity
- **doc-suite command**: References these guidelines in its plan manifest and quality gate checks

## Related

- Scoring rubric: `skills/doc-quality-scoring/SKILL.md`
- Validator agent: `agents/doc-validator.md`
- Orchestrator agent: `agents/doc-orchestrator.md`
- Command: `commands/doc-suite.md`
