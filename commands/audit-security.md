---
description: Security audit — OWASP top 10, secrets detection, permissions review, hook injection analysis, and attack surface mapping.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Security Audit

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.

Focused security analysis of the codebase. Produces a dated report in `docs/audits/` with actionable findings.

Scope: $ARGUMENTS (or full codebase if none provided)

## Arguments

- `--scope=<path>` — limit to subdirectory (default: entire repo)

## 1. Analysis

Invoke the `security-reviewer` agent with full codebase access (allowedTools: [Read, Grep, Glob, Bash]).

The agent evaluates:
- **OWASP Top 10** — injection, broken auth, sensitive data exposure, XXE, broken access control, misconfig, XSS, insecure deserialization, known vulnerabilities, insufficient logging
- **Secrets detection** — hardcoded API keys, passwords, tokens, connection strings in source and config files
- **Permissions review** — file permissions, directory access, privilege escalation vectors
- **Hook injection** — command injection via hooks, unsafe shell expansion, unvalidated hook inputs
- **Input validation** — missing or insufficient validation at system boundaries
- **Dependency vulnerabilities** — known CVEs in dependencies
- **Auth/authz patterns** — authentication bypass, authorization gaps, session management
- **Attack surface mapping** — entry points, trust boundaries, data flow paths

## 2. Report

Write findings to `docs/audits/security-YYYY-MM-DD.md` using today's date.

Report structure:

```markdown
# Security Audit — YYYY-MM-DD

## Project Profile
- **Repository**: <repo name>
- **Scope**: <audited path or "full codebase">
- **Date**: YYYY-MM-DD
- **Agent**: security-reviewer

## Health Grade

| Grade | Criteria |
|-------|----------|
| **A** | 0 CRITICAL, 0 HIGH, ≤3 MEDIUM |
| **B** | 0 CRITICAL, ≤2 HIGH |
| **C** | 0 CRITICAL, >2 HIGH |
| **D** | 1+ CRITICAL or >5 HIGH |
| **F** | 3+ CRITICAL |

**Grade: X**

## Attack Surface Map

| Entry Point | Trust Boundary | Data Flow | Risk Level |
|-------------|---------------|-----------|------------|
| ... | ... | ... | ... |

## Findings

### [SEC-NNN] Finding Title
- **Severity**: CRITICAL | HIGH | MEDIUM | LOW
- **Location**: file:line-range
- **Principle**: The violated security principle
- **Evidence**: Concrete data (code snippets, paths, counts)
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
- Attack surface entry point count
- Finding counts by severity
- Top 3 most critical findings
- Report file path

**STOP. DO NOT modify source code.**

Say: "To act on findings, run `/spec` referencing this report."

## When to Use

- Before releases or security reviews
- After adding authentication, API endpoints, or user input handling
- Periodic security health checks

## Related Agents

- `agents/security-reviewer.md` — primary agent for this audit
