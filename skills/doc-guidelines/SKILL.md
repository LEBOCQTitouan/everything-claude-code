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

## Design Principles

These principles guide how documentation is written and structured across the pipeline.

### Audience Layering

Write documentation at multiple levels, each serving a different reader:

| Layer | Audience | Content | Where |
|-------|----------|---------|-------|
| **L1 — Overview** | New team members, non-technical stakeholders | What the system does, high-level architecture | README, ARCHITECTURE.md (System Context) |
| **L2 — Component** | Developers joining the team | Module purposes, dependencies, key interfaces | Module summaries, ARCHITECTURE.md (Component View) |
| **L3 — API** | Developers using the API | Signatures, parameters, return types, examples | Doc comments, API-SURFACE.md |
| **L4 — Operational** | On-call engineers, DevOps | Runbooks, failure modes, config knobs | Runbooks, config docs |

Every documentation decision should consider: "which audience needs this, and at which layer?"

### Intent Inference

When documentation is missing, infer what the developer *intended* to communicate:

1. **Function name** → primary purpose
2. **Parameter names and types** → expected inputs and constraints
3. **Return type** → what the caller gets back
4. **Error handling** → what can go wrong
5. **Test names** → expected behaviour in plain language

Use inferred intent as a starting point, then verify against the implementation. Mark inferred documentation with appropriate confidence levels.

### Living Over Comprehensive

Prefer documentation that stays accurate over documentation that covers everything:

- **A few accurate docs > many stale docs** — better to document 70% well than 100% poorly
- **Generated over hand-written** — auto-generated docs from code stay in sync; hand-written docs drift
- **Manifests over snapshots** — use `docs/.doc-manifest.json` to track freshness, not manual dates
- **Incremental over batch** — update docs when code changes, not in periodic documentation sprints

### Progressive Disclosure

Structure documentation so readers can drill down:

1. **SKILL.md** — complete but concise (under 2000 words)
2. **references/** — detailed patterns, anti-examples, language-specific guides
3. **assets/** — templates, schemas, diagrams

Readers start at the SKILL.md level and only dive into references when they need specifics.

## How Agents Use These Guidelines

- **doc-orchestrator**: Displays the CAPITALISED guidelines during Phase 0 (plan) and checks quality gates after all phases complete
- **doc-validator**: Applies file size validation and reports findings with appropriate severity
- **doc-suite command**: References these guidelines in its plan manifest and quality gate checks

## Related

- Scoring rubric: `skills/doc-quality-scoring/SKILL.md`
- Drift detection: `skills/doc-drift-detector/SKILL.md`
- Gap analysis: `skills/doc-gap-analyser/SKILL.md`
- Validator agent: `agents/doc-validator.md`
- Orchestrator agent: `agents/doc-orchestrator.md`
- Command: `commands/doc-suite.md`
- Manifest schema: `schemas/doc-manifest.schema.json`
