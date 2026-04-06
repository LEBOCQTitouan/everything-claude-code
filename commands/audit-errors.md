---
description: Error handling audit — swallowed errors, error taxonomy, boundary translation, and partial failure risk analysis.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Error Handling Audit

> **MANDATORY**: Follow every phase exactly. Narrate per `skills/narrative-conventions/SKILL.md`.

Error handling analysis. Report in `docs/audits/`.

Scope: $ARGUMENTS (or full codebase). Args: `--scope=<path>`.

## 1. Analysis

Invoke `error-handling-auditor` (allowedTools: [Read, Grep, Glob, Bash]): swallowed errors, error taxonomy, boundary translation, partial failure handling, propagation patterns, user-facing errors, retry/recovery.

## Adversarial Challenge

Launch `audit-challenger` with findings. Per-finding verdicts. Quality check, disagreement handling, graceful degradation.

## 2. Report

Write to `docs/audits/errors-YYYY-MM-DD.md`.

Structure: Project Profile, Health Grade (A-F), Error Taxonomy Assessment (layer x consistency/translation/partial-failure), Findings ([ERR-NNN]), Summary, Top 5, Next Steps.

## 3. Present

Console: grade, taxonomy assessment, findings by severity, top 3, report path.

**STOP. DO NOT modify source code.** Say: "To act on findings, run `/spec`."

## When to Use

After complex multi-step operations, when debugging reveals swallowed errors, before releases.

## Related Agents

- `error-handling-auditor`
