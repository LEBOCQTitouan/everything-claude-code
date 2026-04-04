---
description: Full codebase audit — all domains in parallel with cross-domain correlation, executive summary, per-domain scorecard, and prioritized remediation roadmap.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Full Codebase Audit

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Narrative**: See `skills/narrative-conventions/SKILL.md` conventions. Before each agent delegation, gate check, and phase transition, tell the user what is happening and why.

> Before dispatching, tell the user which domain agents will run and in what order (evolution first, then all domains in parallel).

Comprehensive audit across all domains with cross-domain correlation. Delegates to the `audit-orchestrator` agent which coordinates all domain-specific agents. Produces a dated report in `docs/audits/` with correlated findings and a prioritized remediation roadmap.

Scope: $ARGUMENTS (or full codebase if none provided)

> **Tracking**: Create a TodoWrite checklist for this command's phases. If TodoWrite is unavailable, proceed without tracking — the workflow executes identically.

TodoWrite items:
- "Parse arguments"
- "Invoke audit-orchestrator"
- "Phase 1: Evolution analysis"
- "Phase 2: Architecture audit"
- "Phase 2: Code quality audit"
- "Phase 2: Security audit"
- "Phase 2: Test audit"
- "Phase 2: Convention audit"
- "Phase 2: Error handling audit"
- "Phase 2: Observability audit"
- "Phase 2: Documentation audit"
- "Phase 3: Cross-domain correlation"
- "Write report"
- "Present summary"

Mark each item complete as the phase finishes.

## Arguments

- `--scope=<path>` — limit to subdirectory (default: entire repo)
- `--window=<days>` — git history window for evolution analysis (default: 180)

## 1. Analysis

Invoke the `audit-orchestrator` agent, which executes the following sequence:

### Phase 1: Evolution Analysis (Sequential)

Run `evolution-analyst` first — hotspot data is required by downstream agents for cross-correlation.

### Phase 2: Domain Audits (Parallel)

Launch all domain agents in parallel:

| Agent | Domain | Prefix |
|-------|--------|--------|
| `arch-reviewer` + `architect` + `component-auditor` | Architecture | ARCH |
| `code-reviewer` + `uncle-bob` | Code Quality | CODE |
| `security-reviewer` | Security | SEC |
| `test-auditor` | Testing | TEST |
| `convention-auditor` | Conventions | CONV |
| `error-handling-auditor` | Error Handling | ERR |
| `observability-auditor` | Observability | OBS |
| `doc-analyzer` + `doc-validator` | Documentation | DOC |
| `/audit-backlog` (inline) | Backlog Conformance | CONF |

### Phase 3: Cross-Domain Correlation

The orchestrator correlates findings across domains and escalates compound risks:

| Correlation | Escalation |
|-------------|------------|
| Hotspot + boundary violation | → CRITICAL |
| Swallowed error + untested code | → CRITICAL |
| Security issue + error leakage | → CRITICAL |
| Bus factor 1 + hotspot | → CRITICAL |
| Low coverage + high fan-in | → Fragility (escalate one level) |
| Convention drift + high churn | → MODERATE |
| Co-change coupling + no shared interface | → Immobility (escalate one level) |

Cross-correlated findings are reported as `[CORR-NNN]` with references to the original domain findings.

### Sources Re-interrogation

If `docs/sources.md` exists, consult it during the analysis phase:

1. Read `docs/sources.md` and identify entries whose subjects overlap with the sub-audit domains being run
2. For each relevant source, check for new releases, deprecation notices, or security advisories since the `last_checked` date
3. Include actionable findings in the cross-domain correlation output
4. List matched sources as "Consulted sources:" in the executive summary

If `docs/sources.md` does not exist, skip this step silently.

## 2. Report

Write to `docs/audits/full-YYYY-MM-DD.md` using today's date.

Report structure:

```markdown
# Full Codebase Audit — YYYY-MM-DD

## Project Profile
- **Repository**: <repo name>
- **Scope**: <audited path or "full codebase">
- **Date**: YYYY-MM-DD
- **Window**: N days
- **Agents**: audit-orchestrator (coordinating all domain agents)

## Executive Summary

<2-3 paragraph overview of codebase health, key strengths, and primary risks>

## Overall Health Grade

| Grade | Criteria |
|-------|----------|
| **A** | 0 CRITICAL, 0 HIGH, ≤3 MEDIUM |
| **B** | 0 CRITICAL, ≤2 HIGH |
| **C** | 0 CRITICAL, >2 HIGH |
| **D** | 1+ CRITICAL or >5 HIGH |
| **F** | 3+ CRITICAL |

**Grade: X**

## Per-Domain Scorecard

| Domain | Grade | CRITICAL | HIGH | MEDIUM | LOW |
|--------|-------|----------|------|--------|-----|
| Architecture | X | N | N | N | N |
| Code Quality | X | N | N | N | N |
| Security | X | N | N | N | N |
| Testing | X | N | N | N | N |
| Conventions | X | N | N | N | N |
| Error Handling | X | N | N | N | N |
| Observability | X | N | N | N | N |
| Documentation | X | N | N | N | N |
| Evolution | X | N | N | N | N |
| **Cross-Correlation** | — | N | N | N | N |

## Cross-Domain Correlations

### [CORR-NNN] Correlation Title
- **Severity**: CRITICAL | HIGH | MEDIUM
- **Sources**: [DOMAIN-NNN] + [DOMAIN-NNN]
- **Correlation**: Description of the compound risk
- **Evidence**: Data from both domains
- **Risk**: Escalated risk assessment
- **Remediation**: Unified fix addressing both concerns

## Domain Findings

### Architecture
<ARCH findings>

### Code Quality
<CODE findings>

### Security
<SEC findings>

### Testing
<TEST findings>

### Conventions
<CONV findings>

### Error Handling
<ERR findings>

### Observability
<OBS findings>

### Documentation
<DOC findings>

### Evolution
<EVL findings>

## Summary

| Severity | Domain | Cross-Corr | Total |
|----------|--------|------------|-------|
| CRITICAL | N | N | N |
| HIGH     | N | N | N |
| MEDIUM   | N | N | N |
| LOW      | N | N | N |

## Prioritized Remediation Roadmap

### Immediate (CRITICAL)
1. ...

### Short-term (HIGH)
1. ...

### Medium-term (MEDIUM)
1. ...

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

> As each domain audit completes, report its completion status to the user conversationally before moving to the next.

Display a console summary:

```
Full Codebase Audit Complete
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Health Grade:        X (GRADE_LABEL)

  Architecture:  X    Code Quality:  X    Security:     X
  Testing:       X    Conventions:   X    Errors:       X
  Observability: X    Documentation: X    Evolution:    X

  Cross-Correlations:  N findings
  Critical Findings:   N
  Total Findings:      N

  Top 3 Risks:
  1. ...
  2. ...
  3. ...

  Report: docs/audits/full-YYYY-MM-DD.md
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**STOP. DO NOT modify source code.**

Say: "To act on findings, run `/spec` referencing this report."

## When to Use

- Periodic comprehensive health checks (monthly/quarterly)
- Before major releases
- After significant refactoring
- Onboarding to an unfamiliar codebase
- Planning technical debt reduction

## Related Agents

- `agents/audit-orchestrator.md` — orchestrates all domain agents
- `agents/evolution-analyst.md` — git history analysis
- `agents/arch-reviewer.md` — architecture quality
- `agents/architect.md` — strategic architecture
- `agents/component-auditor.md` — component principles
- `agents/code-reviewer.md` — code quality
- `agents/uncle-bob.md` — SOLID and clean code
- `agents/security-reviewer.md` — security analysis
- `agents/test-auditor.md` — test architecture
- `agents/convention-auditor.md` — convention consistency
- `agents/error-handling-auditor.md` — error handling
- `agents/observability-auditor.md` — observability
- `agents/doc-analyzer.md` — documentation analysis
- `agents/doc-validator.md` — documentation validation
