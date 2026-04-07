---
description: Evolution audit — git hotspots, co-change coupling, churn analysis, bus factor, and complexity trends from repository history.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Evolution Audit

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Narrative**: See narrative-conventions skill.

Focused evolutionary analysis of the codebase using git history. Produces a dated report in `docs/audits/` with actionable findings.

Scope: $ARGUMENTS (or full codebase if none provided)

## Arguments

- `--scope=<path>` — limit to subdirectory (default: entire repo)
- `--window=<days>` — git history window in days (default: 180)

## 1. Analysis

Invoke the `evolution-analyst` agent with full codebase and git history access (allowedTools: [Read, Grep, Glob, Bash]).

Pass the `--window` argument to control the git history depth.

The agent evaluates:
- **Change frequency** — files ranked by commit count within the window
- **Complexity approximation** — branching keyword count as complexity proxy
- **Hotspot scoring** — complexity × change frequency to identify risk areas
- **Co-change coupling** — file pairs that change together (>60% co-change rate)
- **Bus factor** — contributor count per file and module
- **Complexity trends** — complexity trajectory for top hotspots (increasing, stable, decreasing)
- **Temporal coupling** — files that co-change without compile-time dependency


## Sources Re-interrogation

If `docs/sources.md` exists, consult it for relevant sources:

1. Read `docs/sources.md` and identify entries whose subjects overlap with the evolution audit's focus areas (hotspots, coupling, complexity)
2. List matched sources as "Consulted sources:" in the analysis output
3. Check if any matched sources have new releases, deprecation notices, or security advisories since their `last_checked` date
4. Include actionable findings from source re-interrogation in the analysis output

If `docs/sources.md` does not exist, skip this step silently and proceed to the adversarial challenge.

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

Write findings to `docs/audits/evolution-YYYY-MM-DD.md` using today's date.

Report structure:

```markdown
# Evolution Audit — YYYY-MM-DD

## Project Profile
- **Repository**: <repo name>
- **Scope**: <audited path or "full codebase">
- **Date**: YYYY-MM-DD
- **Window**: N days
- **Commits analyzed**: N
- **Contributors**: N
- **Agent**: evolution-analyst

## Health Grade

| Grade | Criteria |
|-------|----------|
| **A** | 0 CRITICAL, 0 HIGH, ≤3 MEDIUM |
| **B** | 0 CRITICAL, ≤2 HIGH |
| **C** | 0 CRITICAL, >2 HIGH |
| **D** | 1+ CRITICAL or >5 HIGH |
| **F** | 3+ CRITICAL |

**Grade: X**

## Top 20 Hotspots

| Rank | File | Changes | Complexity | Score | Trend |
|------|------|---------|------------|-------|-------|
| 1 | ... | N | N | 0.XX | ↑/→/↓ |
| ... | ... | ... | ... | ... | ... |

## Bus Factor

| Module/File | Contributors | Bus Factor | Risk |
|-------------|-------------|------------|------|
| ... | N | N | HIGH/MEDIUM/LOW |

## Co-Change Coupling

| File A | File B | Co-change Rate | Has Dependency? |
|--------|--------|---------------|-----------------|
| ... | ... | N% | ✅/❌ |

## Findings

### [EVL-NNN] Finding Title
- **Severity**: CRITICAL | HIGH | MEDIUM | LOW
- **Location**: file:line-range
- **Principle**: The violated evolutionary principle
- **Evidence**: Concrete data (commit counts, coupling rates, contributor counts)
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
- Top 5 hotspots (compact)
- Bus factor risks
- Finding counts by severity
- Report file path

**STOP. DO NOT modify source code.**

Say: "To act on findings, run `/spec` referencing this report."

## When to Use

- Before planning refactoring efforts (target hotspots first)
- To identify knowledge silos and bus factor risks
- When investigating why certain areas keep breaking

## Related Agents

- `agents/evolution-analyst.md` — primary agent for this audit
