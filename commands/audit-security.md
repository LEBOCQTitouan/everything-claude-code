---
description: Security audit — OWASP top 10, secrets detection, permissions review, hook injection analysis, and attack surface mapping.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Security Audit

> **MANDATORY**: Follow every phase exactly. Narrate per `skills/narrative-conventions/SKILL.md`.

Security analysis. Report in `docs/audits/`.

Scope: $ARGUMENTS (or full codebase). Args: `--scope=<path>`.

## 1. Analysis

Invoke `security-reviewer` (allowedTools: [Read, Grep, Glob, Bash]): OWASP Top 10, secrets detection, permissions review, hook injection, input validation, dependency CVEs, auth/authz patterns, attack surface mapping.

## Adversarial Challenge

Launch `audit-challenger` with findings. Per-finding verdicts. Quality check, disagreement handling, graceful degradation.

## 2. Report

Write to `docs/audits/security-YYYY-MM-DD.md`.

Structure: Project Profile, Health Grade (A-F), Attack Surface Map table, Findings ([SEC-NNN]), Summary, Top 5, Next Steps.

## 3. Present

Console: grade, attack surface count, findings by severity, top 3, report path.

**STOP. DO NOT modify source code.** Say: "To act on findings, run `/spec`."

## When to Use

Before releases/reviews, after adding auth/API/input handling, periodic checks.

## Related Agents

- `security-reviewer`
