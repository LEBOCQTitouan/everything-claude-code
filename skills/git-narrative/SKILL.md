---
name: git-narrative
description: Extract evolution summary, decision trail, and authorship map from git history for documentation and changelog generation.
origin: ECC
---

# Git Narrative

Atomic extraction skill for deriving documentation-relevant insights from git history. Transforms raw commit logs into structured narratives about how code evolved, who contributed what, and what decisions were made.

## When to Activate

- During changelog generation (doc-generator Step 4)
- When writing architecture decision records (ADRs)
- When analysing module evolution for documentation prioritisation
- When building contribution maps for onboarding docs

## Methodology

### 1. Evolution Summary

Build a timeline of significant changes per module:

```bash
# Get commit history with file change stats
git log --format="%H|%s|%ai|%an" --numstat --no-merges
```

For each module, extract:
- **Birth date**: First commit touching that directory
- **Major milestones**: Commits with `feat:` or `refactor:` that touch 5+ files
- **Stability indicator**: Change frequency over last 90 days (hot/warm/cold)
- **Growth trend**: File count and LOC change over time

Output:
```
Module: src/lib/
  Created: 2025-06-15
  Milestones:
    - 2025-08-20: feat: add merge conflict resolution
    - 2025-11-03: refactor: extract merge strategies
    - 2026-01-15: feat: add dry-run mode
  Stability: warm (12 commits in last 90 days)
  Growth: +3 files, +450 LOC since creation
```

### 2. Decision Trail

Extract architectural decisions from commit messages and PR descriptions:

1. **Conventional commit mining**: Filter for `refactor:`, `feat:`, `fix:` commits
2. **Pattern detection**: Look for decision indicators:
   - "replace X with Y" → technology/approach decision
   - "extract X from Y" → modularity decision
   - "remove/deprecate X" → elimination decision
   - "add X for Y" → capability decision
3. **Context enrichment**: For each decision, record:
   - What changed (files touched)
   - Why it changed (commit message body, if present)
   - What it replaced (if refactor/replace)
   - Who made the decision (author)

### 3. Authorship Map

Build a contributor-to-module mapping:

```bash
git shortlog -sn --no-merges -- <module-path>
```

For each module, record:
- Primary author (most commits)
- Contributors (2+ commits)
- Last active contributor
- Bus factor (number of authors with 20%+ of commits)

### 4. Changelog Extraction

Parse conventional commits into changelog entries:

| Commit Prefix | Changelog Section |
|---------------|-------------------|
| `feat:` | Added |
| `fix:` | Fixed |
| `refactor:` | Changed |
| `perf:` | Performance |
| `docs:` | Documentation |
| `test:` | Testing |
| `chore:` | Maintenance |
| `ci:` | CI/CD |

Group by:
1. Version tag (if using semver tags)
2. Time period (weekly/monthly) if no tags
3. Most recent first

### 5. Scope Control

To keep extraction tractable on large repositories:

- **Default**: Last 6 months or 200 commits (whichever is smaller)
- **Targeted**: Single module via `--module=<path>`
- **Full**: All history via `--full` (use sparingly)
- **Since tag**: From a specific tag via `--since=v1.0.0`

## Output Format

Structured narrative data for downstream consumers:

```
# Git Narrative: project-name
Generated: 2026-03-14
Scope: last 6 months (142 commits)

## Evolution Summary
[per-module timeline as above]

## Decision Trail
| Date | Decision | Rationale | Files | Author |
|------|----------|-----------|-------|--------|
| 2026-01-15 | Add dry-run mode to merge | User request for safe preview | 4 files | alice |
| 2025-11-03 | Extract merge strategies | Single merge function was 400 LOC | 6 files | bob |

## Authorship Map
| Module | Primary | Contributors | Bus Factor |
|--------|---------|-------------|------------|
| src/lib/ | alice (45%) | bob (30%), carol (25%) | 3 |
| src/hooks/ | bob (80%) | alice (20%) | 2 |

## Changelog
[structured changelog entries]
```

## Related

- Changelog generation: `skills/changelog-gen/SKILL.md`
- Architecture generation: `skills/architecture-gen/SKILL.md`
- Evolution analysis agent: `agents/evolution-analyst.md` (agent)
- Doc analysis skill: `skills/doc-analysis/SKILL.md`
