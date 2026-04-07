---
description: Error handling audit — swallowed errors, error taxonomy, boundary translation, and partial failure risk analysis.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Error Handling Audit

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Narrative**: See narrative-conventions skill.

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


## Adversarial Challenge

> After the analysis phase completes, launch an independent adversary to challenge the findings.

Launch a Task with the `audit-challenger` agent (allowedTools: [Read, Grep, Glob, Bash, WebSearch]):

- Pass the findings from the analysis phase as structured input (finding ID, severity, description, evidence)
- The agent independently re-interrogates the codebase and searches web for best practices
- Collect challenged findings: confirmed, refuted, or amended with per-finding rationale

### Quality Check

If the adversary output lacks structured per-finding verdicts (each with finding ID, verdict {confirmed|refuted|amended}, and rationale):
1. Retry once with a stricter prompt demanding the exact output format
2. If second attempt still lacks structure, surface a "Low-quality adversary output" warning alongside the raw content and proceed

### Disagreement Handling

When audit and adversary disagree on a finding:
- Display both the original finding and the challenger's assessment side by side
- Include the challenger's recommendation
- Prompt the user for final decision: accept audit / accept challenger / custom resolution

### Graceful Degradation

If the audit-challenger agent fails to spawn or returns an error:
- Emit: "Adversary challenge skipped: <reason>"
- Proceed with unchallenged findings

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

To act on these findings, run `/spec` referencing this report.
```

## 3. Present

Display a console summary:
- Health grade
- Error taxonomy assessment (compact)
- Finding counts by severity
- Top 3 most critical findings
- Report file path

**STOP. DO NOT modify source code.**

Say: "To act on findings, run `/spec` referencing this report."

## When to Use

- After adding complex multi-step operations
- When debugging reveals swallowed errors
- Before releases to verify error handling completeness

## Related Agents

- `agents/error-handling-auditor.md` — primary agent for this audit
