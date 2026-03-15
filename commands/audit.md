---
description: Codebase health audit — architecture, evolution, testing, security, observability, error handling, and convention consistency. Deep, git-history-aware analysis with cross-domain correlation.
---

# Codebase Health Audit

### Phase 0: Prompt Refinement

Before executing, analyze the user's input using the `prompt-optimizer` skill:
1. Identify intent and match to available ECC skills/commands/agents
2. Check for ambiguity or missing context
3. Rewrite the task description for clarity and specificity
4. Display the refined prompt to the user

If the refined prompt differs significantly, show both original and refined versions.
Proceed with the refined version unless the user objects.

**FIRST ACTION**: Unless `--skip-plan` is passed, call the `EnterPlanMode` tool immediately. This enters Claude Code plan mode which restricts tools to read-only exploration while you scan the codebase and draft an audit plan. After presenting the plan, call `ExitPlanMode` to proceed with execution after user approval.

Comprehensive codebase health analysis across 7 domains: architecture, evolutionary patterns, test architecture, security, observability, error handling, convention consistency, and component structure. Produces a dated audit report in `docs/audits/` with cross-domain correlations and actionable recommendations.

## What This Command Does

0. **Discovery** — scan codebase, detect boundaries/languages, present audit plan, wait for approval
1. **Evolutionary Analysis** — mine git history for hotspots, co-change coupling, bus factor, complexity trends
2. **Domain Audits** — run all domain-specific audits in parallel (architecture, security, testing, observability, errors, conventions, components)
3. **Cross-Domain Correlation** — connect findings across domains, escalate correlated risks
4. **Report Generation** — produce `docs/audits/YYYY-MM-DD-audit.md`
5. **Console Summary** — display scores, top risks, and health grade

## Arguments

- `--scope=<path>` — limit to subdirectory (default: entire repo)
- `--domain=<domains>` — comma-separated: `architecture`, `evolution`, `testing`, `security`, `observability`, `errors`, `conventions`, `components`, `all` (default: all)
- `--boundary=<moduleA>:<moduleB>` — audit a specific boundary between two modules
- `--window=<days>` — git history window (default: 180)
- `--diff=<path>` — compare against prior audit report
- `--skip-plan` — skip Phase 0 discovery/approval
- `--quick` — orchestrator only, no sub-agent delegation

## Phase Details

### Phase 0: Discovery (unless `--skip-plan`)

1. **Codebase scan**:
   - Glob source files, count and classify by language
   - Detect project type (monorepo, single app, library, microservice)
   - Identify module/package boundaries
   - Count git history depth and contributor count
   - Check for existing audit reports in `docs/audits/`

2. **Plan manifest**:
   - Domains to audit (all or filtered by `--domain`)
   - Estimated scope (files, modules, git commits)
   - Scaling decisions (hotspot count based on codebase size)
   - Prior audit comparison (if `--diff` provided)

3. **Wait for user approval**, then call `ExitPlanMode`

### Phase 1: Evolutionary Analysis (Sequential)

Delegates to `evolution-analyst` agent. Must complete before Phase 2 because domain agents cross-reference hotspot data.

- Change frequency per file
- Complexity approximation via branching keyword count
- Hotspot scoring: complexity × change frequency
- Co-change coupling between file pairs
- Bus factor per file and module
- Complexity trends for top hotspots
- **Temporal coupling detection**: Identify file pairs that change together (>60% co-change rate) but have no compile-time dependency between them. These represent hidden coupling — a change to one file implicitly requires changing the other, but the compiler won't catch missed updates.

### Phase 2: Domain Audits (Parallel)

All domain agents launch in parallel:

| Agent | Domain | Key areas |
|-------|--------|-----------|
| `arch-reviewer` | architecture | Layering, coupling, DDD, dependency metrics, boundary coherence |
| `security-reviewer` | security | OWASP top 10, secrets, auth/authz, input validation |
| `test-auditor` | testing | Test classification, structural coupling, fixture ratios, coverage gaps |
| `observability-auditor` | observability | Log levels, structured logging, correlation IDs, metrics, health checks |
| `error-handling-auditor` | errors | Swallowed errors, error taxonomy, boundary translation, partial failures |
| `convention-auditor` | conventions | Naming patterns, pattern consistency, config access, primitive obsession |
| `component-auditor` | components | REP, CCP, CRP, ADP, SDP, SAP, main sequence distance, zone analysis |

### Phase 3: Cross-Domain Correlation

Orchestrator connects findings across domains:

| Correlation | Result |
|-------------|--------|
| Hotspot + untested | Escalate to CRITICAL |
| Hotspot + boundary violation | Escalate to CRITICAL |
| Swallowed errors + poor logging | Escalate one level |
| Convention divergence + coupling | Flag as maintenance risk |
| Security issue + error leakage | Escalate to CRITICAL |
| Bus factor + hotspot | Escalate to CRITICAL |

### Phase 4: Report Generation

Generates `docs/audits/YYYY-MM-DD-audit.md` containing:
- Project profile
- Overall health grade (A-F)
- Per-domain scores
- All findings grouped by severity
- Cross-domain correlations
- Top 5 recommendations
- Comparison with prior audit (if `--diff` provided)

### Phase 5: Console Summary

Displays health grade, per-domain scores, top hotspots, bus factor risks, critical findings count, and top 5 recommendations.

## Scaling Behavior

| Codebase Size | Behavior |
|---------------|----------|
| < 5 files | Skip evolution, reduce architecture to import listing |
| 5-50 files | Full analysis, top 5 hotspots |
| 50-500 files | Full analysis, top 20 hotspots (sweet spot) |
| 500+ files | Prompt user to select scope, or sample top 50 hotspots |

## Finding Format (Standardized)

```
### [DOMAIN-NNN] Finding Title
- **Severity**: CRITICAL | HIGH | MEDIUM | LOW
- **Location**: file:line-range
- **Principle**: The violated principle
- **Evidence**: Concrete data (metrics, paths, counts)
- **Risk**: What breaks if unaddressed
- **Remediation**: Directional fix (what, not how)
- **Cross-refs**: Related findings in other domains
```

## Quality Gates

| Condition | Result |
|-----------|--------|
| CRITICAL findings > 0 | FAIL |
| All findings are structural only | WARN |
| 0 CRITICAL, ≤ 2 HIGH | PASS |

## Example Usage

```
User: /audit

[Phase 0: Discovery — scans codebase, presents audit plan]

Audit Plan
  Source files:        127 (medium codebase)
  Languages:           TypeScript (89%), Markdown (11%)
  Modules:             8 (src/lib, src/hooks, src/ci, agents, commands, skills, rules, tests)
  Git window:          180 days (342 commits, 4 contributors)
  Domains:             all (7)
  Hotspot count:       top 20

Approve? [y/n]

User: y

[Phases 1-5 execute]

Codebase Health Audit Complete
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Health Grade:        B (GOOD)
  Architecture:        B    Evolution:     A
  Testing:             C    Security:      B
  Observability:       N/A  Errors:        B
  Conventions:         A

  Top Hotspots:        src/lib/utils.ts (score: 0.72)
  Bus Factor Risks:    2 single-contributor modules
  Critical Findings:   0
  Total Findings:      18

  Top 5 Recommendations:
  1. Increase test coverage for src/lib/utils.ts (hotspot)
  2. Add structured logging to hook scripts
  3. Standardize error handling in CI validators
  4. Extract hardcoded paths to config module
  5. Add integration tests for package-manager

  Report: docs/audits/2026-03-14-audit.md
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Domain-specific audit

```
User: /audit --domain=testing,errors --scope=src/

[Runs only test-auditor and error-handling-auditor on src/]
```

### Compare with prior audit

```
User: /audit --diff=docs/audits/2026-02-14-audit.md

[Full audit + comparison section showing new, resolved, and persistent findings]
```

## When to Use

- Periodic health checks (monthly/quarterly)
- Before major releases
- After significant refactoring
- Onboarding to an unfamiliar codebase
- Planning technical debt reduction
- Post-incident analysis (was the risk visible?)

**Distinction from `/verify`**: `/verify` = "Is this ready to ship?" (fast, change-scoped, pass/fail). `/audit` = "What is the long-term health?" (deep, codebase-wide, git-history-aware, report-generating).

## Related

- Orchestrator: `agents/audit-orchestrator.md`
- Evolution: `agents/evolution-analyst.md`
- Testing: `agents/test-auditor.md`
- Observability: `agents/observability-auditor.md`
- Error Handling: `agents/error-handling-auditor.md`
- Conventions: `agents/convention-auditor.md`
- Reused: `agents/arch-reviewer.md`, `agents/security-reviewer.md`
- Skills: `skills/evolutionary-analysis/`, `skills/test-architecture/`, `skills/observability-audit/`, `skills/error-handling-audit/`, `skills/convention-consistency/`, `skills/architecture-review/`
