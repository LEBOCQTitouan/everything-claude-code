---
description: Evolution audit — git hotspots, co-change coupling, churn analysis, bus factor, and complexity trends from repository history.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Evolution Audit

> **MANDATORY**: Follow every phase exactly. Narrate per `skills/narrative-conventions/SKILL.md`.

Evolutionary analysis via git history. Report in `docs/audits/`.

Scope: $ARGUMENTS (or full codebase). Args: `--scope=<path>`, `--window=<days>` (default: 180).

## 1. Analysis

Invoke `evolution-analyst` (allowedTools: [Read, Grep, Glob, Bash]) with `--window`. Evaluates: change frequency, complexity approximation, hotspot scoring, co-change coupling (>60%), bus factor, complexity trends, temporal coupling.

## Sources Re-interrogation

If `docs/sources.md` exists, consult for overlapping subjects, check updates since `last_checked`, include actionable findings. Skip if absent.

## Adversarial Challenge

Launch `audit-challenger` with findings. Per-finding verdicts. Quality check, disagreement handling, graceful degradation.

## 2. Report

Write to `docs/audits/evolution-YYYY-MM-DD.md`.

Structure: Project Profile, Health Grade (A-F), Top 20 Hotspots table, Bus Factor table, Co-Change Coupling table, Findings ([EVL-NNN]), Summary, Top 5, Next Steps.

## 3. Present

Console: grade, top 5 hotspots, bus factor risks, findings by severity, report path.

**STOP. DO NOT modify source code.** Say: "To act on findings, run `/spec`."

## When to Use

Before refactoring, to identify knowledge silos, to investigate recurring breakage.

## Related Agents

- `evolution-analyst`
