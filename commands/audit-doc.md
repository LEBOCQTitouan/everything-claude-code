---
description: Documentation audit — coverage analysis, staleness detection, drift identification, and placement violation checks.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Documentation Audit

> **MANDATORY**: Follow every phase exactly. Narrate per `skills/narrative-conventions/SKILL.md`.

Multi-agent documentation analysis + placement checks. Report in `docs/audits/`.

Scope: $ARGUMENTS (or full codebase). Args: `--scope=<path>`.

## 1. Analysis

### 1a. Agent Analysis (Parallel)

**`doc-analyzer`** (allowedTools: [Read, Grep, Glob, Bash]): coverage, module summaries, domain concepts, dependency docs.

**`doc-validator`** (allowedTools: [Read, Grep, Glob, Bash]): accuracy, staleness, code examples, contradictions, duplicates.

### 1b. Placement Violation Checks (Post-Agent)

1. Volatile content in `docs/` (belongs as code comments)
2. Missing architecture decisions in ARCHITECTURE.md
3. Decisions without ADRs
4. Oversized README (>100 content lines)
5. Oversized CLAUDE.md (>120 content lines)
6. Mixed doc types (tutorial + reference)

## Adversarial Challenge

Launch `audit-challenger` with findings. Per-finding verdicts. Quality check, disagreement handling, graceful degradation.

## 2. Report

Write to `docs/audits/doc-YYYY-MM-DD.md`.

Structure: Project Profile, Health Grade (A-F), Coverage Summary table, Placement Violations table, Findings ([DOC-NNN]), Summary, Top 5, Next Steps.

## 3. Present

Console: grade, coverage %, placement violations, findings by severity, top 3, report path.

**STOP. DO NOT modify source code.** Say: "To act on findings, run `/spec`."

## When to Use

Before releases, after major refactoring, periodic checks.

## Related Agents

- `doc-analyzer`, `doc-validator`
