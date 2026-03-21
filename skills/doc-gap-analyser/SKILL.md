---
name: doc-gap-analyser
description: Systematic documentation gap analysis — identifies undocumented areas by priority, suggests what to document first based on usage, complexity, and change frequency.
origin: ECC
---

# Documentation Gap Analyser

Quality skill for identifying what's missing from documentation and prioritising what to document first. Goes beyond coverage percentages to consider *impact* — a heavily-used, complex function with no docs is worse than an internal helper with no docs.

## When to Activate

- After doc-reporter calculates coverage (to prioritise next actions)
- When planning documentation sprints
- When deciding where to invest documentation effort
- During doc-validator pipeline for comprehensive gap analysis

## Methodology

### 1. Gap Identification

Scan for documentation gaps across all layers:

| Layer | What to Check | Gap If Missing |
|-------|--------------|----------------|
| **Source comments** | Doc comments on public symbols | Undocumented API |
| **Module summaries** | `docs/module-summaries/*.md` or `MODULE-SUMMARIES.md` | No module overview |
| **Architecture** | `docs/ARCHITECTURE.md` | No system understanding |
| **API reference** | `docs/API-SURFACE.md` or `docs/api-surface/` | No API docs |
| **README** | `README.md` with setup, usage, structure | No entry point |
| **CLAUDE.md** | Project instructions file | No AI-assisted development context |
| **Changelog** | `CHANGELOG.md` or `CHANGELOG.md` | No change history |
| **Runbooks** | `docs/runbooks/` or deployment docs | No operational docs |
| **Error docs** | Error code documentation | Undocumented failures |
| **Config docs** | Env var / config documentation | Setup confusion |

### 2. Priority Scoring

Score each gap on four dimensions:

| Dimension | Weight | Scoring (0-3) |
|-----------|--------|---------------|
| **Usage frequency** | 3x | 0=unused, 1=rare, 2=moderate, 3=hot path |
| **Complexity** | 2x | 0=trivial, 1=simple, 2=moderate, 3=complex |
| **Change frequency** | 2x | 0=stable, 1=occasional, 2=monthly, 3=weekly |
| **Blast radius** | 3x | 0=internal, 1=single consumer, 2=multi-consumer, 3=public API |

**Priority score** = (usage × 3) + (complexity × 2) + (change_freq × 2) + (blast_radius × 3)

| Score Range | Priority | Action |
|-------------|----------|--------|
| 25-30 | **CRITICAL** | Document immediately |
| 18-24 | **HIGH** | Document this sprint |
| 10-17 | **MEDIUM** | Document when touching the code |
| 0-9 | **LOW** | Nice to have |

### 3. Usage Frequency Estimation

Estimate how often a symbol is used:

1. **Import count**: How many files import this symbol?
2. **Entry point proximity**: Is it called from main/index/handler? (hot path)
3. **Test coverage**: More tests = more important code
4. **Git blame frequency**: How often is this file/function touched?

### 4. Complexity Assessment

Estimate documentation need based on complexity:

1. **Cyclomatic complexity**: Functions with many branches need more docs
2. **Parameter count**: 4+ params = needs usage examples
3. **Side effects**: Functions with I/O or state mutation need behaviour docs
4. **Generic types**: Generic/template functions need type constraint docs
5. **Callback patterns**: Async callbacks need lifecycle/ordering docs

### 5. Gap Report Structure

```markdown
# Documentation Gap Analysis
Generated: 2026-03-14

## Summary
Total gaps: 34
Critical: 3 | High: 8 | Medium: 15 | Low: 8

## Coverage by Layer
| Layer | Status | Gaps |
|-------|--------|------|
| Source comments | 73% (104/142) | 38 undocumented exports |
| Architecture doc | Exists | Outdated (3 months) |
| API reference | Missing | No docs/API-SURFACE.md |
| README | Exists | Missing setup section |
| Runbooks | Missing | No operational docs |

## Top 10 Gaps by Priority

| Priority | Symbol/Area | Score | Why |
|----------|-----------|-------|-----|
| CRITICAL | `mergeDirectory()` | 28 | Public API, complex, used in 15 files, no docs |
| CRITICAL | Deployment procedure | 27 | No runbook, 3 incidents last month |
| CRITICAL | Error codes | 25 | 12 custom errors, none documented |
| HIGH | `PackageManager` class | 22 | 8 public methods, 2 documented |
| HIGH | Environment variables | 20 | 8 required vars, only 3 in README |
| ... | ... | ... | ... |

## Quick Wins (low effort, high impact)
- Add `@param` docs to 5 functions that already have descriptions
- Document 3 env vars referenced in .env.example but not in README
- Add return type docs to 8 functions with `@returns` missing

## Recommended Documentation Sprint
1. Document top 3 CRITICAL gaps (estimated: 2 hours)
2. Add missing sections to README (estimated: 30 min)
3. Create basic runbook from existing deployment scripts (estimated: 1 hour)
```

### 6. Incremental Tracking

If a previous gap analysis exists, show progress:

```markdown
## Progress Since Last Analysis (2026-02-28)
| Metric | Previous | Current | Delta |
|--------|----------|---------|-------|
| Total gaps | 42 | 34 | -8 (19% improvement) |
| Critical gaps | 5 | 3 | -2 |
| Coverage | 68% | 73% | +5% |
```

## Related

- Doc reporter agent: `agents/doc-reporter.md`
- Doc drift detector: `skills/doc-drift-detector/SKILL.md`
- Doc coverage reporting: `agents/doc-reporter.md`
- Doc quality scoring: `skills/doc-quality-scoring/SKILL.md`
- Doc guidelines: `skills/doc-guidelines/SKILL.md`
