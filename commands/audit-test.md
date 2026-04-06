---
description: Test architecture audit — coverage analysis, test classification, fixture ratios, structural coupling, and E2E boundary coverage matrix.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Test Architecture Audit

> **MANDATORY**: Follow every phase exactly. Narrate per `skills/narrative-conventions/SKILL.md`.

Test architecture analysis + E2E boundary coverage. Report in `docs/audits/`.

Scope: $ARGUMENTS (or full codebase). Args: `--scope=<path>`.

## 1. Analysis

### 1a. Standard Analysis
Invoke `test-auditor` (allowedTools: [Read, Grep, Glob, Bash]): test classification, structural coupling, fixture ratios, coverage gaps, test isolation, naming, mock usage.

### 1b. E2E Boundary Coverage

1. Scan port traits in `crates/ecc-ports/src/`
2. Map adapters in `crates/ecc-infra/`
3. Check E2E coverage in `crates/ecc-integration-tests/`
4. Verify `// e2e-boundary:` markers
5. Adapters without E2E tests → HIGH severity

## Adversarial Challenge

Launch `audit-challenger` with findings. Per-finding verdicts. Quality check, disagreement handling, graceful degradation.

## 2. Report

Write to `docs/audits/test-YYYY-MM-DD.md`.

Structure: Project Profile, Health Grade (A-F), Test Distribution table, E2E Boundary Coverage Matrix (port/adapter/coverage/marker/status), Findings ([TEST-NNN]), Summary, Top 5, Next Steps.

## 3. Present

Console: grade, test distribution, E2E coverage (covered/total ports), findings by severity, top 3, report path.

**STOP. DO NOT modify source code.** Say: "To act on findings, run `/spec`."

## When to Use

Before releases, after adding ports/adapters, when tests feel brittle.

## Related Agents

- `test-auditor`
