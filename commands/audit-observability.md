---
description: Observability audit — logging consistency, structured logging, metrics coverage, tracing, correlation IDs, and health endpoint depth.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Observability Audit

> **MANDATORY**: Follow every phase exactly. Narrate per `skills/narrative-conventions/SKILL.md`.

Observability analysis. Report in `docs/audits/`.

Scope: $ARGUMENTS (or full codebase). Args: `--scope=<path>`.

## 1. Analysis

Invoke `observability-auditor` (allowedTools: [Read, Grep, Glob, Bash]): log level consistency, structured logging, correlation IDs, metric coverage, health endpoints, tracing, alert readiness.

## Adversarial Challenge

Launch `audit-challenger` with findings. Per-finding verdicts. Quality check, disagreement handling, graceful degradation.

## 2. Report

Write to `docs/audits/observability-YYYY-MM-DD.md`.

Structure: Project Profile, Health Grade (A-F), Observability Maturity Matrix (Logging/Metrics/Tracing/Health/Alerting x level), Findings ([OBS-NNN]), Summary, Top 5, Next Steps.

## 3. Present

Console: grade, maturity matrix, findings by severity, top 3, report path.

**STOP. DO NOT modify source code.** Say: "To act on findings, run `/spec`."

## When to Use

Before production deploy, after debugging incidents, when adding services.

## Related Agents

- `observability-auditor`
