---
name: evolutionary-analysis
description: Git-derived codebase health methodology — hotspot detection, co-change coupling, bus factor analysis, and complexity trends.
origin: ECC
---

# Evolutionary Analysis

Git history mining methodology for identifying structural health risks that are invisible in static analysis. Combines change frequency, complexity, contributor distribution, and co-change patterns.

## When to Activate

- Codebase audit (via `/audit --domain=evolution`)
- Identifying refactoring priorities
- Assessing knowledge distribution risk
- Before major rewrites (find what changes most)
- Team onboarding (understand change hotspots)

## Methodology

### 1. Hotspot Detection

Hotspots are files where high complexity meets high change frequency. They represent the riskiest code.

**Complexity approximation** (language-agnostic):
- Count branching keywords: `if`, `else`, `switch`, `case`, `for`, `while`, `catch`, `&&`, `||`, `?`
- Normalize by file length: `complexity_density = branching_keywords / total_lines`

**Change frequency**:
- `git log --since=<window> --format='%H' -- <file> | wc -l`
- Normalize across all files: `change_ratio = file_changes / max_changes`

**Hotspot score**:
- `hotspot = complexity_density × change_ratio` (both normalized 0-1)
- Rank files by hotspot score, report top N

### 2. Co-Change Coupling

Files that always change together indicate hidden coupling — even without import relationships.

**Extraction**:
- `git log --name-only --format='COMMIT:%H'` to get commit→files mapping
- For each file pair, compute: `co_change_ratio = commits_together / max(commits_A, commits_B)`

**Thresholds**:
- `> 0.7` — CRITICAL: near-lockstep coupling, likely should be merged or have explicit interface
- `> 0.5` — HIGH: frequent co-change, investigate shared dependency
- `> 0.3` — MEDIUM: moderate coupling, may be acceptable

**Cross-module pairs** are higher severity than within-module pairs.

### 3. Bus Factor

Files with too few active contributors represent knowledge concentration risk.

**Extraction**:
- `git shortlog -sn --since=<window> -- <file>` for contributor count per file
- `git shortlog -sn --since=<window> -- <directory>` for contributor count per module

**Thresholds**:
- `1 contributor` — HIGH: single point of failure
- `2 contributors` — MEDIUM: limited knowledge distribution
- `3+ contributors` — OK

**Prioritization**: Bus factor risks on hotspot files are escalated one severity level.

### 4. Complexity Trends

Is a hotspot getting better or worse over time?

**Extraction**:
- For top N hotspots, check out last 10+ commits touching the file
- Compute complexity at each point
- Trend: `growing` (complexity increasing), `stable`, `shrinking`

**Reporting**:
- Growing complexity on a hotspot = CRITICAL
- Stable complexity on a hotspot = HIGH (still risky, not improving)
- Shrinking = MEDIUM (improving but still a hotspot)

### 5. Scaling Rules

| Codebase Size | Behavior |
|---------------|----------|
| < 5 source files | Skip evolutionary analysis entirely |
| 5-50 files | Full analysis, top 5 hotspots, all co-change pairs |
| 50-500 files | Full analysis, top 20 hotspots, top 20 co-change pairs |
| 500+ files | Prompt user to scope, or sample top 50 hotspots |

### 6. Git History Window

- Default: 180 days (configurable via `--window=<days>`)
- Short window (30-90 days): recent activity focus
- Long window (365+ days): structural patterns, seasonal changes

## Finding Format

```
### [EVOLUTION-NNN] Finding Title
- **Severity**: CRITICAL | HIGH | MEDIUM | LOW
- **Location**: file:line-range
- **Principle**: The violated principle (e.g., "Knowledge distribution", "Coupling minimization")
- **Evidence**: Concrete data (change count, co-change ratio, contributor count)
- **Risk**: What breaks if unaddressed
- **Remediation**: Directional fix (what, not how)
```

## Related

- Agent: `agents/evolution-analyst.md`
- Command: `commands/audit.md`
- Complementary: `skills/architecture-review/SKILL.md` (static structure + evolutionary dynamics)
