---
description: Convention audit — naming patterns, style consistency, configuration access scatter, and primitive obsession detection.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Convention Audit

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.

Focused convention consistency analysis of the codebase. Produces a dated report in `docs/audits/` with actionable findings.

Scope: $ARGUMENTS (or full codebase if none provided)

## Arguments

- `--scope=<path>` — limit to subdirectory (default: entire repo)

## 1. Analysis

Invoke the `convention-auditor` agent with full codebase access.

The agent evaluates:
- **Naming conventions** — variable, function, type, file, and module naming consistency
- **Pattern divergence** — inconsistent use of established patterns across modules
- **Configuration access** — scattered config access vs centralized configuration
- **Primitive obsession** — raw primitives used where domain types should exist
- **Style consistency** — formatting, import ordering, export patterns
- **File organization** — naming conventions, directory structure consistency
- **API surface consistency** — consistent parameter ordering, return types, error shapes

## 2. Report

Write findings to `docs/audits/convention-YYYY-MM-DD.md` using today's date.

Report structure:

```markdown
# Convention Audit — YYYY-MM-DD

## Project Profile
- **Repository**: <repo name>
- **Scope**: <audited path or "full codebase">
- **Date**: YYYY-MM-DD
- **Agent**: convention-auditor

## Health Grade

| Grade | Criteria |
|-------|----------|
| **A** | 0 CRITICAL, 0 HIGH, ≤3 MEDIUM |
| **B** | 0 CRITICAL, ≤2 HIGH |
| **C** | 0 CRITICAL, >2 HIGH |
| **D** | 1+ CRITICAL or >5 HIGH |
| **F** | 3+ CRITICAL |

**Grade: X**

## Convention Drift Heatmap

| Module/Directory | Naming | Patterns | Config | Primitives | Overall |
|-----------------|--------|----------|--------|------------|---------|
| ... | ✅/⚠️/❌ | ✅/⚠️/❌ | ✅/⚠️/❌ | ✅/⚠️/❌ | ✅/⚠️/❌ |

## Findings

### [CONV-NNN] Finding Title
- **Severity**: CRITICAL | HIGH | MEDIUM | LOW
- **Location**: file:line-range
- **Principle**: The violated convention principle
- **Evidence**: Concrete data (examples, counts, patterns)
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

To act on these findings, run `/plan` referencing this report.
```

## 3. Present

Display a console summary:
- Health grade
- Convention drift heatmap (compact)
- Finding counts by severity
- Top 3 most impactful findings
- Report file path

**STOP. DO NOT modify source code.**

Say: "To act on findings, run `/plan` referencing this report."

## When to Use

- After onboarding new contributors
- When codebase feels inconsistent
- Before establishing or enforcing a style guide

## Related Agents

- `agents/convention-auditor.md` — primary agent for this audit
