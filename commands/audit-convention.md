---
description: Convention audit — naming patterns, style consistency, configuration access scatter, and primitive obsession detection.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Convention Audit

> **MANDATORY**: Follow every phase exactly. Narrate per `skills/narrative-conventions/SKILL.md`.

Convention consistency analysis. Report in `docs/audits/`.

Scope: $ARGUMENTS (or full codebase). Args: `--scope=<path>`.

## 1. Analysis

Invoke `convention-auditor` (allowedTools: [Read, Grep, Glob, Bash]): naming conventions, pattern divergence, config access scatter, primitive obsession, style consistency, file organization, API surface consistency.

## Adversarial Challenge

Launch `audit-challenger` with findings. Per-finding verdicts. Quality check, disagreement handling, graceful degradation.

## 2. Report

Write to `docs/audits/convention-YYYY-MM-DD.md`.

Structure: Project Profile, Health Grade (A-F), Convention Drift Heatmap (module x dimension), Findings ([CONV-NNN]), Summary, Top 5, Next Steps.

## 3. Present

Console: grade, drift heatmap, findings by severity, top 3, report path.

**STOP. DO NOT modify source code.** Say: "To act on findings, run `/spec`."

## When to Use

After onboarding contributors, when codebase feels inconsistent, before style guide.

## Related Agents

- `convention-auditor`
