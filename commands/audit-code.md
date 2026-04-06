---
description: Code quality audit — SOLID principles, clean code, naming quality, function size, complexity, and craftsmanship assessment.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Code Quality Audit

> **MANDATORY**: Follow every phase exactly. Narrate per `skills/narrative-conventions/SKILL.md`.

Multi-agent code quality analysis. Report in `docs/audits/`.

Scope: $ARGUMENTS (or full codebase). Args: `--scope=<path>`.

## 1. Analysis

Launch **two agents in parallel**:

**1a. `code-reviewer`** (allowedTools: [Read, Grep, Glob, Bash]): readability, function size (>50 lines), file size (>800), nesting (>4), duplication, dead code, security, maintainability.

**1b. `uncle-bob`** (allowedTools: [Read, Grep, Glob]): SOLID (SRP/OCP/LSP/ISP/DIP), Clean Architecture, naming, small functions. Diagnosis only.

## Adversarial Challenge

Launch `audit-challenger` (allowedTools: [Read, Grep, Glob, Bash, WebSearch]) with findings. Per-finding verdicts: confirmed/refuted/amended. Quality check: retry once if unstructured. Disagreement: side by side, user decides. Degradation: skip if fails.

## 2. Report

Write to `docs/audits/code-YYYY-MM-DD.md`.

Structure: Project Profile, Health Grade (A-F), SOLID Compliance table, Code Metrics table, Findings ([CODE-NNN]), Summary, Top 5, Next Steps.

## 3. Present

Console: grade, SOLID compliance, metrics, findings by severity, top 3, report path.

**STOP. DO NOT modify source code.** Say: "To act on findings, run `/spec`."

## When to Use

After feature implementation, before PR/review, periodic checks.

## Related Agents

- `code-reviewer`, `uncle-bob`
