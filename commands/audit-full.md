---
description: Full codebase audit — all domains in parallel with cross-domain correlation, executive summary, per-domain scorecard, and prioritized remediation roadmap.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Full Codebase Audit

> **MANDATORY**: Follow every phase exactly. Narrate per `skills/narrative-conventions/SKILL.md`.

Comprehensive audit across all domains. Delegates to `audit-orchestrator`. Produces dated report in `docs/audits/`.

Scope: $ARGUMENTS (or full codebase)

> **Tracking**: TodoWrite checklist below. If unavailable, proceed without tracking.

TodoWrite: Parse args, invoke orchestrator, Phase 1-3, write report, present summary.

## Arguments

- `--scope=<path>` — subdirectory (default: entire repo)
- `--window=<days>` — git history window (default: 180)

## 1. Analysis

### Phase 1: Evolution Analysis (Sequential)

Run `evolution-analyst` first — hotspot data required by downstream agents.

### Team Manifest

Read `teams/audit-team.md` for agent config. If absent: require `ECC_LEGACY_DISPATCH=1` or fail.

### Phase 2: Domain Audits (Parallel)

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

| Correlation | Escalation |
|-------------|------------|
| Hotspot + boundary violation | CRITICAL |
| Swallowed error + untested code | CRITICAL |
| Security issue + error leakage | CRITICAL |
| Bus factor 1 + hotspot | CRITICAL |
| Low coverage + high fan-in | Fragility (escalate) |
| Convention drift + high churn | MODERATE |
| Co-change coupling + no shared interface | Immobility (escalate) |

Cross-correlated findings: `[CORR-NNN]` with source refs.

### Sources Re-interrogation

If `docs/sources.md` exists, consult for overlapping subjects, check for updates since `last_checked`, include actionable findings. Skip if absent.

## 2. Report

Write to `docs/audits/full-YYYY-MM-DD.md`.

Structure: Project Profile, Executive Summary, Overall Health Grade (A-F), Per-Domain Scorecard (9 domains + cross-correlation), Cross-Domain Correlations ([CORR-NNN] with severity/sources/evidence/remediation), Domain Findings (ARCH/CODE/SEC/TEST/CONV/ERR/OBS/DOC/EVL), Summary table (severity x domain/cross-corr), Prioritized Remediation Roadmap (Immediate/Short/Medium-term), Top 5 Recommendations, Next Steps.

Grade: A (0C/0H/≤3M) | B (0C/≤2H) | C (0C/>2H) | D (1+C or >5H) | F (3+C)

## 3. Present

> Report each domain audit completion.

Console summary: health grade, per-domain grades, cross-correlations count, critical findings, top 3 risks, report path.

**STOP. DO NOT modify source code.** Say: "To act on findings, run `/spec`."

## When to Use

- Periodic health checks (monthly/quarterly)
- Before major releases
- After significant refactoring
- Onboarding to unfamiliar codebase

## Related Agents

- `audit-orchestrator` — coordinates all domain agents
- `evolution-analyst`, `arch-reviewer`, `architect`, `component-auditor`, `code-reviewer`, `uncle-bob`, `security-reviewer`, `test-auditor`, `convention-auditor`, `error-handling-auditor`, `observability-auditor`, `doc-analyzer`, `doc-validator`
