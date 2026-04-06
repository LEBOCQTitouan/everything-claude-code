---
description: Architecture audit — boundary integrity, dependency direction, component health, DDD compliance, and coupling/cohesion analysis.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Architecture Audit

> **MANDATORY**: Follow every phase exactly. Narrate per `skills/narrative-conventions/SKILL.md`.

Multi-agent architecture analysis. Report in `docs/audits/`.

Scope: $ARGUMENTS (or full codebase). Args: `--scope=<path>`.

## 1. Analysis

Launch **three agents in parallel**:

**1a. `arch-reviewer`** (allowedTools: [Read, Grep, Glob, Bash]): layering violations, dependency direction, coupling/cohesion, circular deps, DDD/hexagonal compliance.

**1b. `architect`** (allowedTools: [Read, Grep, Glob, Bash]): bounded context integrity, port contract completeness, adapter coverage, domain purity.

**1c. `component-auditor`** (allowedTools: [Read, Grep, Glob, Bash]): 6 component principles (REP, CCP, CRP, ADP, SDP, SAP). Computes I, A, D per component.

## Adversarial Challenge

Launch `audit-challenger` (allowedTools: [Read, Grep, Glob, Bash, WebSearch]) with findings. Returns per-finding verdicts: confirmed/refuted/amended.
- **Quality check**: Retry once if unstructured. Warn and proceed if still bad.
- **Disagreement**: Show both side by side; user decides.
- **Degradation**: Skip if agent fails.

## 2. Report

Write to `docs/audits/archi-YYYY-MM-DD.md`.

Structure: Project Profile, Health Grade (A-F), Component Metrics (Ca/Ce/I/A/D/Zone per component), Dependency Direction (violations count), Findings ([ARCH-NNN] with severity/location/principle/evidence/risk/remediation), Summary, Top 5, Next Steps.

## 3. Present

Console: grade, component metrics, dependency violations, findings by severity, top 3, report path.

**STOP. DO NOT modify source code.** Say: "To act on findings, run `/spec`."

## When to Use

Before refactoring, after adding crates/modules, periodic health checks.

## Related Agents

- `arch-reviewer`, `architect`, `component-auditor`
