---
description: Architecture audit — boundary integrity, dependency direction, component health, DDD compliance, and coupling/cohesion analysis.
allowed-tool-set: audit-command
---

# Architecture Audit

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Narrative**: See narrative-conventions skill.
> After findings are produced, tell the user how to reference this report in a `/spec` command to act on findings.

Multi-agent architecture analysis of the codebase. Produces a dated report in `docs/audits/` with actionable findings.

Scope: $ARGUMENTS (or full codebase if none provided)

## Arguments

- `--scope=<path>` — limit to subdirectory (default: entire repo)

## 1. Analysis

Launch **three agents in parallel**:

### 1a. `arch-reviewer` Agent (allowedTools: [Read, Grep, Glob, Bash])
Evaluates:
- **Layering violations** — imports crossing architectural boundaries
- **Dependency direction** — dependencies pointing inward (domain ← app ← infra)
- **Coupling analysis** — afferent/efferent coupling per module
- **Cohesion analysis** — module focus and responsibility clarity
- **Circular dependencies** — cycles in the dependency graph
- **DDD compliance** — bounded contexts, aggregates, value objects
- **Hexagonal compliance** — ports/adapters pattern adherence

### 1b. `architect` Agent (allowedTools: [Read, Grep, Glob, Bash])
Evaluates:
- **Bounded context integrity** — context boundaries align with domain model
- **Port contract completeness** — all external interactions go through ports
- **Adapter coverage** — every port has at least one adapter
- **Domain purity** — domain crate has zero I/O imports

### 1c. `component-auditor` Agent (allowedTools: [Read, Grep, Glob, Bash])
Evaluates the 6 component principles:
- **REP** (Reuse/Release Equivalence) — reusable units match release units
- **CCP** (Common Closure) — classes that change together belong together
- **CRP** (Common Reuse) — don't force dependents to depend on unused code
- **ADP** (Acyclic Dependencies) — no cycles in component dependency graph
- **SDP** (Stable Dependencies) — depend in direction of stability
- **SAP** (Stable Abstractions) — stable components should be abstract

Computes instability (I), abstractness (A), and main sequence distance (D) per component.


## Adversarial Challenge


### Adversary Gate

If the aggregate finding count from the analysis phase is <3 AND all findings are MEDIUM severity or lower (threshold rationale: low-signal audits provide insufficient material for meaningful adversarial review — see BL-121 finding 4.5), skip the adversary challenge:

> "Adversary challenge skipped: N findings, all ≤MEDIUM severity."

Otherwise, proceed with the challenger launch below.

> After the analysis phase completes, launch an independent adversary to challenge the findings.

Launch a Task with the `audit-challenger` agent (allowedTools: [Read, Grep, Glob, Bash, WebSearch]):

- Pass the findings from the analysis phase as structured input (finding ID, severity, description, evidence)
- The agent independently re-interrogates the codebase and searches web for best practices
- Collect challenged findings: confirmed, refuted, or amended with per-finding rationale

### Quality Check

If the adversary output lacks structured per-finding verdicts (each with finding ID, verdict {confirmed|refuted|amended}, and rationale):
1. Retry once with a stricter prompt demanding the exact output format
2. If second attempt still lacks structure, surface a "Low-quality adversary output" warning alongside the raw content and proceed

### Disagreement Handling

When audit and adversary disagree on a finding:
- Display both the original finding and the challenger's assessment side by side
- Include the challenger's recommendation
- Prompt the user for final decision: accept audit / accept challenger / custom resolution

### Graceful Degradation

If the audit-challenger agent fails to spawn or returns an error:
- Emit: "Adversary challenge skipped: <reason>"
- Proceed with unchallenged findings

## 2. Report

Merge findings from all three agents. Write to `docs/audits/archi-YYYY-MM-DD.md` using today's date.

Report structure:

```markdown
# Architecture Audit — YYYY-MM-DD

## Project Profile
- **Repository**: <repo name>
- **Scope**: <audited path or "full codebase">
- **Date**: YYYY-MM-DD
- **Agents**: arch-reviewer, architect, component-auditor

## Health Grade

| Grade | Criteria |
|-------|----------|
| **A** | 0 CRITICAL, 0 HIGH, ≤3 MEDIUM |
| **B** | 0 CRITICAL, ≤2 HIGH |
| **C** | 0 CRITICAL, >2 HIGH |
| **D** | 1+ CRITICAL or >5 HIGH |
| **F** | 3+ CRITICAL |

**Grade: X**

## Component Metrics

| Component | Ca | Ce | I | A | D | Zone |
|-----------|----|----|---|---|---|------|
| ecc-domain | ... | ... | ... | ... | ... | ... |
| ecc-ports | ... | ... | ... | ... | ... | ... |
| ecc-app | ... | ... | ... | ... | ... | ... |
| ecc-infra | ... | ... | ... | ... | ... | ... |
| ecc-cli | ... | ... | ... | ... | ... | ... |

Ca=afferent coupling, Ce=efferent coupling, I=instability, A=abstractness, D=main sequence distance

## Dependency Direction

```
domain ← ports ← app ← infra ← cli
```

Violations: N

## Findings

### [ARCH-NNN] Finding Title
- **Severity**: CRITICAL | HIGH | MEDIUM | LOW
- **Location**: file:line-range
- **Principle**: The violated architectural principle
- **Evidence**: Concrete data (imports, metrics, dependency paths)
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
- Component metrics table (compact)
- Dependency direction violations count
- Finding counts by severity
- Top 3 most critical findings
- Report file path

**STOP. DO NOT modify source code.**

Say: "To act on findings, run `/spec` referencing this report."

## When to Use

- Before major refactoring to understand current structure
- After adding new crates or modules
- Periodic architecture health checks

## Related Agents

- `agents/arch-reviewer.md` — architecture quality auditor
- `agents/architect.md` — strategic software architect
- `agents/component-auditor.md` — component principle evaluator
