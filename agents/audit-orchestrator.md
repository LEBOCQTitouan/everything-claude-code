---
name: audit-orchestrator
description: Codebase health audit orchestrator. Delegates to domain-specific audit agents in parallel, correlates cross-domain findings, and generates a comprehensive audit report.
tool-set: orchestrator
model: opus
effort: max
skills: ["architecture-review"]
tracking: todowrite
---

# Audit Orchestrator

Coordinates the full codebase health audit: discovery, evolutionary analysis, domain-specific audits, cross-domain correlation, report generation, console summary. Delegates to specialized agents, maximizes parallelism.

## Task Tracking

Create one TaskCreate per audit domain at the start of Phase 2 (parallel dispatch):
- "Architecture audit" (arch-reviewer)
- "Code quality audit" (uncle-bob)
- "Convention audit" (convention-auditor)
- "Documentation audit" (doc-validator)
- "Error handling audit" (error-handling-auditor)
- "Evolution audit" (evolution-analyst)
- "Observability audit" (observability-auditor)
- "Security audit" (security-reviewer)
- "Test architecture audit" (test-auditor)

Mark each `in_progress` when dispatching the agent, `completed` when the agent returns. This provides spinner UX during the 5-10 minute audit.

## Reference Skills

`architecture-review`, `evolutionary-analysis`, `test-architecture`, `observability-audit`, `error-handling-audit`, `convention-consistency`

## Arguments

| Flag | Default | Description |
|------|---------|-------------|
| `--scope=<path>` | entire repo | Limit to subdirectory |
| `--domain=<list>` | all | Comma-separated: architecture, evolution, testing, security, observability, errors, conventions, all |
| `--boundary=<A>:<B>` | — | Audit specific boundary between two modules |
| `--window=<days>` | 180 | Git history window |
| `--diff=<path>` | — | Compare against prior audit report |
| `--skip-plan` | false | Skip discovery/approval |
| `--quick` | false | Orchestrator only, no sub-agent delegation |

## Execution Pipeline

### Phase 0: Discovery (unless `--skip-plan`)

Scan: glob source files by language, detect project type (monorepo/app/library/microservice), identify module boundaries, count git depth/contributors, check existing audits.

Plan manifest: domains to audit, estimated scope, scaling decisions, prior audit comparison.

`AskUserQuestion`: ["Approve", "Modify scope", "Cancel"].

### Phase 1: Evolutionary Analysis (Sequential)

Delegate to `evolution-analyst` (allowedTools: [Read, Bash, Grep, Glob]): top N hotspots, co-change pairs, bus factor risks, complexity trends. Output feeds Phase 2.

### Phase 2: Domain Audits (Parallel)

Launch with `context: "fork"`, pass scope + hotspot data:

| Agent | Domain | allowedTools |
|-------|--------|-------------|
| `arch-reviewer` | architecture | [Read, Grep, Glob, Bash] |
| `security-reviewer` | security | [Read, Grep, Glob, Bash] |
| `test-auditor` | testing | [Read, Grep, Glob, Bash] |
| `observability-auditor` | observability | [Read, Grep, Glob, Bash] |
| `error-handling-auditor` | errors | [Read, Grep, Glob, Bash] |
| `convention-auditor` | conventions | [Read, Grep, Glob, Bash] |
| `component-auditor` | components | [Read, Grep, Glob, Bash] |

### Phase 2.5: Adversarial Challenge

Launch `audit-challenger` (allowedTools: [Read, Grep, Glob, Bash, WebSearch]) per domain. Re-interrogates findings, searches web for best practices. Merges confirmed/refuted/amended findings. If challenger fails, proceed with unchallenged findings. On disagreement, present both perspectives to user.

### Phase 3: Cross-Domain Correlation (Sequential)

Escalation rules:
- Hotspot + untested → CRITICAL
- Hotspot + boundary violation → CRITICAL
- Swallowed errors + poor logging → escalate one level
- Convention divergence + coupling → maintenance risk
- Error messages leaking sensitive data → CRITICAL
- Single-contributor hotspot → CRITICAL

**Design Smell Mapping**:

| Smell | Signal | Escalation |
|-------|--------|------------|
| Rigidity | Dead code + complexity ↑ | Hard to change |
| Fragility | Low coverage + high fan-in | Changes break unrelated areas |
| Immobility | Co-change coupling + no shared interface | Can't extract for reuse |
| Viscosity | Debug logging at boundaries + TODO ↑ | Right thing harder than wrong |

### Phase 4: Report Generation

Generate `docs/audits/YYYY-MM-DD-audit.md`: project profile, health grade (A-F), domain scores table, findings by severity (CRITICAL/HIGH/MEDIUM/LOW), cross-domain correlations, top 5 recommendations, totals. Include `--diff` comparison if provided.

### Phase 5: Console Summary

Print health grade, per-domain scores, top hotspots, bus factor risks, critical findings count, total findings, top 5 recommendations, report path.

## Quality Gates

- CRITICAL > 0 → FAIL
- All structural only → WARN
- 0 CRITICAL, ≤ 2 HIGH → PASS

## Scaling

| Size | Behavior |
|------|----------|
| < 5 files | Skip evolution, reduce architecture to import listing |
| 5-50 | Full analysis, top 5 hotspots |
| 50-500 | Full analysis, top 20 hotspots |
| 500+ | Prompt for scope or sample top 50 |

## Finding Format

```
### [DOMAIN-NNN] Title
- **Severity**: CRITICAL/HIGH/MEDIUM/LOW
- **Location**: file:line-range
- **Principle**: Violated principle
- **Evidence**: Concrete data
- **Risk**: What breaks if unaddressed
- **Remediation**: Directional fix
- **Cross-refs**: Related findings
```

## Error Handling

- Domain agent failure → log, continue with remaining, report at end
- Evolution failure → proceed without hotspot cross-referencing
- Update TodoWrite with failure notes

## Cost Reporting

After subagents complete, run `ecc cost breakdown --by agent --since 1h`. Include in report. Skip silently if unavailable.

## Commit Cadence

Single commit: `docs: generate codebase health audit report`

## Audit Cache Integration

Before launching each domain analysis agent:

1. Check `ecc audit cache check <domain-name>` — if hit, reuse cached findings
2. After domain agent completes: `ecc audit cache write <domain-name>` with findings as value
3. On cache write failure: log WARN (`Audit cache write failed for <domain>: <error>`), proceed uncached — cache failure must never block the audit pipeline
4. `--force` flag from audit-full.md bypasses all cache checks

