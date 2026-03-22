---
description: Observability audit — logging consistency, structured logging, metrics coverage, tracing, correlation IDs, and health endpoint depth.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Observability Audit

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Narrative**: See `skills/narrative-conventions/SKILL.md` conventions. Before each agent delegation, tell the user what is happening and why.

Focused observability analysis of the codebase. Produces a dated report in `docs/audits/` with actionable findings.

Scope: $ARGUMENTS (or full codebase if none provided)

## Arguments

- `--scope=<path>` — limit to subdirectory (default: entire repo)

## 1. Analysis

Invoke the `observability-auditor` agent with full codebase access (allowedTools: [Read, Grep, Glob, Bash]).

The agent evaluates:
- **Log level consistency** — appropriate use of debug, info, warn, error across modules
- **Structured logging** — JSON/structured format vs unstructured strings
- **Correlation IDs** — request/trace ID propagation across service boundaries
- **Metric coverage** — key operations instrumented with counters, histograms, gauges
- **Health endpoints** — depth and accuracy of health/readiness/liveness checks
- **Tracing** — distributed tracing spans at critical paths
- **Alert readiness** — whether logs and metrics support meaningful alerting

## 2. Report

Write findings to `docs/audits/observability-YYYY-MM-DD.md` using today's date.

Report structure:

```markdown
# Observability Audit — YYYY-MM-DD

## Project Profile
- **Repository**: <repo name>
- **Scope**: <audited path or "full codebase">
- **Date**: YYYY-MM-DD
- **Agent**: observability-auditor

## Health Grade

| Grade | Criteria |
|-------|----------|
| **A** | 0 CRITICAL, 0 HIGH, ≤3 MEDIUM |
| **B** | 0 CRITICAL, ≤2 HIGH |
| **C** | 0 CRITICAL, >2 HIGH |
| **D** | 1+ CRITICAL or >5 HIGH |
| **F** | 3+ CRITICAL |

**Grade: X**

## Observability Maturity Matrix

| Dimension | Level | Evidence | Gap |
|-----------|-------|----------|-----|
| Logging | Basic/Structured/Correlated | ... | ... |
| Metrics | None/Basic/Comprehensive | ... | ... |
| Tracing | None/Basic/Distributed | ... | ... |
| Health Checks | None/Shallow/Deep | ... | ... |
| Alerting | None/Basic/Actionable | ... | ... |

## Findings

### [OBS-NNN] Finding Title
- **Severity**: CRITICAL | HIGH | MEDIUM | LOW
- **Location**: file:line-range
- **Principle**: The violated observability principle
- **Evidence**: Concrete data (log samples, missing instrumentation)
- **Risk**: What breaks if unaddressed
- **Remediation**: Directional fix (what, not how)

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | N |
| HIGH     | N |
| MEDIUM   | N |
| LOW      | N |

## Top 5 Recommendations

1. ...
2. ...
3. ...
4. ...
5. ...

## Next Steps

To act on these findings, run `/spec` referencing this report.
```

## 3. Present

Display a console summary:
- Health grade
- Observability maturity matrix (compact)
- Finding counts by severity
- Top 3 most impactful findings
- Report file path

**STOP. DO NOT modify source code.**

Say: "To act on findings, run `/spec` referencing this report."

## When to Use

- Before deploying to production
- After incidents where debugging was difficult
- When adding new services or integration points

## Related Agents

- `agents/observability-auditor.md` — primary agent for this audit
