---
description: Error handling audit — swallowed errors, error taxonomy, boundary translation, and partial failure risk analysis.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Error Handling Audit

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.

Focused error handling architecture analysis of the codebase. Produces a dated report in `docs/audits/` with actionable findings.

Scope: $ARGUMENTS (or full codebase if none provided)

## Arguments

- `--scope=<path>` — limit to subdirectory (default: entire repo)

## 1. Analysis

Invoke the `error-handling-auditor` agent with full codebase access (allowedTools: [Read, Grep, Glob, Bash]).

The agent evaluates:
- **Swallowed errors** — caught exceptions with no logging, re-throw, or handling
- **Error taxonomy** — consistency of error types, codes, and categorization
- **Boundary translation** — proper error mapping at layer boundaries (domain → app → infra)
- **Partial failure handling** — transactions, rollbacks, compensating actions for multi-step operations
- **Error propagation** — consistent propagation patterns (Result types, exceptions, error codes)
- **User-facing errors** — clarity, safety (no leaked internals), and actionability
- **Retry and recovery** — appropriate retry strategies, circuit breakers, fallback paths

## 2. Report

Write findings to `docs/audits/errors-YYYY-MM-DD.md` using today's date.

Report structure:

```markdown
# Error Handling Audit — YYYY-MM-DD

## Project Profile
- **Repository**: <repo name>
- **Scope**: <audited path or "full codebase">
- **Date**: YYYY-MM-DD
- **Agent**: error-handling-auditor

## Health Grade

| Grade | Criteria |
|-------|----------|
| **A** | 0 CRITICAL, 0 HIGH, ≤3 MEDIUM |
| **B** | 0 CRITICAL, ≤2 HIGH |
| **C** | 0 CRITICAL, >2 HIGH |
| **D** | 1+ CRITICAL or >5 HIGH |
| **F** | 3+ CRITICAL |

**Grade: X**

## Error Taxonomy Assessment

| Layer | Error Types | Consistent | Boundary Translation | Partial Failure |
|-------|------------|------------|---------------------|-----------------|
| Domain | ... | ✅/❌ | N/A | ... |
| Application | ... | ✅/❌ | ✅/❌ | ... |
| Infrastructure | ... | ✅/❌ | ✅/❌ | ... |
| CLI/API | ... | ✅/❌ | ✅/❌ | ... |

## Findings

### [ERR-NNN] Finding Title
- **Severity**: CRITICAL | HIGH | MEDIUM | LOW
- **Location**: file:line-range
- **Principle**: The violated error handling principle
- **Evidence**: Concrete data (code snippets, counts)
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
- Error taxonomy assessment (compact)
- Finding counts by severity
- Top 3 most critical findings
- Report file path

**STOP. DO NOT modify source code.**

Say: "To act on findings, run `/plan` referencing this report."

## When to Use

- After adding complex multi-step operations
- When debugging reveals swallowed errors
- Before releases to verify error handling completeness

## Related Agents

- `agents/error-handling-auditor.md` — primary agent for this audit
