# Spec: Numeric Quality Scores for Adversary Agents (BL-036)

## Problem Statement

The spec-adversary and solution-adversary agents produce binary PASS/FAIL/CONDITIONAL verdicts per dimension, but these lack granularity. A dimension rated "PASS" could be barely passing or excellent — there's no way to distinguish. Adding 0-100 numeric scores per dimension enables quantified quality gates, trend analysis across specs, and clearer communication of quality levels. Scores are persisted in spec/design Phase Summary tables for historical comparison.

## Research Summary

Web research skipped — this is a focused enhancement to existing agent markdown files with clear scoring rubric from BMAD framework.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | 0-100 scale per dimension | Granular enough for meaningful comparison, simple enough for LLM scoring | No |
| 2 | Thresholds: >=70 avg PASS, 50-69 CONDITIONAL, <50 FAIL | Matches typical quality gate patterns (BMAD uses 90% for solutioning) | No |
| 3 | Any single dimension <50 forces CONDITIONAL regardless of average | Prevents weak dimensions from being masked by strong ones | No |
| 4 | Scores included in adversary output alongside existing verdict | Additive, not replacing — backward compatible with existing verdict handling | No |
| 5 | Scores persisted in spec/design Phase Summary "Adversary Findings" table | Enables historical quality trend analysis | No |

## User Stories

### US-001: Add scoring rubric to spec-adversary

**As a** developer reviewing specs, **I want** each adversarial dimension to have a 0-100 score, **so that** I can see granular quality levels beyond binary PASS/FAIL.

#### Acceptance Criteria

- AC-001.1: Given `agents/spec-adversary.md`, when checked, then it contains a scoring rubric defining the 0-100 scale
- AC-001.2: Given the rubric, when scores are computed, then each of the 7 dimensions (ambiguity, edge cases, scope, dependencies, testability, decisions, rollback) gets an independent 0-100 score
- AC-001.3: Given the scores, when the average is >=70 and no dimension is <50, then the verdict is PASS
- AC-001.4: Given the scores, when the average is 50-69 OR any dimension is <50, then the verdict is CONDITIONAL
- AC-001.5: Given the scores, when the average is <50, then the verdict is FAIL
- AC-001.6: Given the adversary output, when it produces a verdict, then it includes both the score table AND the PASS/FAIL/CONDITIONAL verdict
- AC-001.7: Given the score table, when output, then format is: `| Dimension | Score | Verdict | Rationale |` with Score as integer 0-100

#### Dependencies

- Depends on: none

### US-002: Add scoring rubric to solution-adversary

**As a** developer reviewing designs, **I want** the solution adversary to also produce 0-100 scores per dimension, **so that** design quality is equally quantified.

#### Acceptance Criteria

- AC-002.1: Given `agents/solution-adversary.md`, when checked, then it contains the same scoring rubric
- AC-002.2: Given the rubric, when scores are computed, then each of the 8 dimensions (coverage, order, fragility, rollback, architecture, blast radius, missing PCs, doc plan) gets a 0-100 score
- AC-002.3: Given the scores, when thresholds are applied, then the same PASS/CONDITIONAL/FAIL rules apply (>=70 avg PASS, 50-69 CONDITIONAL, <50 FAIL, any dim <50 forces CONDITIONAL)
- AC-002.4: Given the adversary output, when produced, then it includes the score table with the verdict

#### Dependencies

- Depends on: none

### US-003: Persist scores in Phase Summary tables

**As a** maintainer tracking quality trends, **I want** adversary scores persisted in spec and design files, **so that** I can compare quality across features over time.

#### Acceptance Criteria

- AC-003.1: Given `commands/spec-dev.md` Phase Summary "Adversary Findings" table, when updated, then the table includes a "Score" column
- AC-003.2: Given `commands/spec-fix.md`, when checked, then same Score column addition
- AC-003.3: Given `commands/spec-refactor.md`, when checked, then same Score column addition
- AC-003.4: Given `commands/design.md` Phase Summary "Adversary Findings" table, when updated, then the table includes a "Score" column
- AC-003.5: Given the Score column, when populated, then it contains the 0-100 integer score for each dimension

#### Dependencies

- Depends on: US-001, US-002

### US-004: Documentation

**As a** maintainer, **I want** CHANGELOG updated and glossary entry added, **so that** the change is tracked.

#### Acceptance Criteria

- AC-004.1: Given `CHANGELOG.md`, when checked, then BL-036 entry exists
- AC-004.2: Given `docs/domain/glossary.md`, when checked, then "Quality Score" entry exists

#### Dependencies

- Depends on: US-001, US-002, US-003

## Affected Modules

| Module | Layer | Nature of Change |
|--------|-------|-----------------|
| `agents/spec-adversary.md` | Agent | Modify: add scoring rubric, score output format |
| `agents/solution-adversary.md` | Agent | Modify: add scoring rubric, score output format |
| `commands/spec-dev.md` | Command | Modify: Phase Summary table Score column |
| `commands/spec-fix.md` | Command | Modify: Phase Summary table Score column |
| `commands/spec-refactor.md` | Command | Modify: Phase Summary table Score column |
| `commands/design.md` | Command | Modify: Phase Summary table Score column |
| `docs/domain/glossary.md` | Docs | Modify: add Quality Score entry |
| `CHANGELOG.md` | Docs | Modify: add BL-036 entry |

## Constraints

- No Rust code changes
- Existing PASS/FAIL/CONDITIONAL verdict preserved (scores are additive)
- Score table format must be markdown-compatible for persistence in spec/design files
- Both adversaries use identical threshold logic

## Non-Requirements

- Automated trend analysis tooling (future work)
- Score weighting by dimension (all equally weighted for v1)
- Custom threshold configuration per project
- Any Rust code changes

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | No port/adapter changes | Pure agent/command markdown modification |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New concept | Domain | `docs/domain/glossary.md` | Add "Quality Score" entry |
| Feature entry | Project | `CHANGELOG.md` | Add BL-036 entry |

## Open Questions

None.
