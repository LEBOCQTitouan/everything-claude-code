---
description: Evolution audit — git hotspots, co-change coupling, churn analysis, bus factor, and complexity trends from repository history.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Evolution Audit

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.

Focused evolutionary analysis of the codebase using git history. Produces a dated report in `docs/audits/` with actionable findings.

Scope: $ARGUMENTS (or full codebase if none provided)

## Arguments

- `--scope=<path>` — limit to subdirectory (default: entire repo)
- `--window=<days>` — git history window in days (default: 180)

## 1. Analysis

Invoke the `evolution-analyst` agent with full codebase and git history access.

Pass the `--window` argument to control the git history depth.

The agent evaluates:
- **Change frequency** — files ranked by commit count within the window
- **Complexity approximation** — branching keyword count as complexity proxy
- **Hotspot scoring** — complexity × change frequency to identify risk areas
- **Co-change coupling** — file pairs that change together (>60% co-change rate)
- **Bus factor** — contributor count per file and module
- **Complexity trends** — complexity trajectory for top hotspots (increasing, stable, decreasing)
- **Temporal coupling** — files that co-change without compile-time dependency

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

To act on these findings, run `/plan` referencing this report.
```

## 3. Present

Display a console summary:
- Health grade
- Top 5 hotspots (compact)
- Bus factor risks
- Finding counts by severity
- Report file path

**STOP. DO NOT modify source code.**

Say: "To act on findings, run `/plan` referencing this report."

## When to Use

- Before planning refactoring efforts (target hotspots first)
- To identify knowledge silos and bus factor risks
- When investigating why certain areas keep breaking

## Related Agents

- `agents/evolution-analyst.md` — primary agent for this audit
