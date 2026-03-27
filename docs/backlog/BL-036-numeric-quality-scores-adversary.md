---
id: BL-036
title: Add numeric quality scores to adversary agents
status: implemented
created: 2026-03-21
scope: MEDIUM
target_command: agents/spec-adversary.md, agents/solution-adversary.md
tags: [bmad, quality, scoring, adversary, metrics]
---

## Optimized Prompt

Add a scoring rubric to spec-adversary and solution-adversary agents. Each dimension gets a 0-100 score. PASS requires >= 70 average with no dimension below 50. CONDITIONAL: 50-69 average or any dimension below 50. FAIL: < 50 average. The numeric scores are included in the verdict output alongside the existing PASS/FAIL/CONDITIONAL verdict. When specs are persisted (BL-029), scores are recorded in the spec file for quality trend analysis over time. This enables quantified quality gates rather than binary verdicts.

## Framework Source

- **BMAD**: Solutioning-gate-check requires 90% completeness score before proceeding

## Related Backlog Items

- Enhanced by: BL-029 (scores persist in spec files)
