---
id: BL-081
title: Web-based upgrade audit command with Technology Radar output
scope: EPIC
target: /spec-dev
status: open
created: 2026-03-27
origin: User request — systematic web auditing for codebase upgrades
---

# BL-081: Web-Based Upgrade Audit Command (/audit-web)

## Problem

ECC has no mechanism to systematically scan the web for potential improvements to the current project. Upgrade discovery is ad-hoc — dependent on the user manually finding blog posts, conference talks, or new crates. There's no structured process for evaluating what the ecosystem offers against what the project currently uses.

## Proposal

Create a new `/audit-web` command that performs a comprehensive, multi-agent web research audit of the current project and outputs findings in ThoughtWorks-style Technology Radar format.

### Architecture: 4-Phase Pipeline

**Phase 1 — INVENTORY (automated, no web)**
- Parse Cargo.toml / lock files for current dependencies + versions
- Run `cargo outdated`, `cargo audit`, `cargo clippy` for baseline issues
- Extract architectural patterns from codebase (hexagonal, DDD, TDD, etc.)
- Detect project domain, stack, and toolchain from CLAUDE.md and config files
- Output: dependency list, known issues, architectural patterns, search focus areas
- User can narrow with `--focus=deps|arch|tools|ecosystem|features`

**Phase 2 — LANDSCAPE SCAN (8+ parallel search agents)**
Each agent handles one scope dimension, running source-triangulated searches (3+ sources per finding):

| Agent | Scope | Query Templates |
|-------|-------|----------------|
| dep-scanner | Dependencies | `"[crate] changelog [year]"`, `"[crate] alternatives comparison"`, `"best rust crates for [domain] [year]"` |
| arch-scout | Architecture patterns | `"[pattern] [language] state of the art [year]"`, `"[pattern] improvements [year]"`, `"[pattern] RFC accepted"` |
| tool-scout | Tooling & DX | `"rust dev tools [year]"`, `"[tool] vs alternatives"`, `"CI/CD [language] best practices [year]"` |
| ecosystem-scout | Ecosystem trends | `"this week in rust [topic]"`, `"rustconf [year] highlights"`, `"awesome-[domain] github"` |
| competitor-scout | Competitor features | `"[similar tool] features"`, `"[tool category] comparison [year]"`, `"best [tool type] [year]"` |
| user-request-miner | User demand signals | `"[tool type] feature request"`, `"how I use [tool]"`, `"[tool category] wishlist"`, GitHub issues/discussions |
| blog-miner | Practitioner patterns | `"how I use claude code"`, `"AI coding agent workflow [year]"`, `"[tool type] tips tricks"` |
| research-scout | Academic/research | `"autonomous coding agent architecture"`, `"LLM agent [topic] paper [year]"`, `"agentic software engineering"` |

**Phase 3 — EVALUATE (LLM reasoning)**
For each finding from Phase 2:
- Score on 3 dimensions: strategic fit (0-5), maturity (0-5), migration effort (0-5)
- Classify into radar quadrant: Techniques, Tools, Platforms, Crates/Frameworks, Features
- Classify into radar ring: Adopt, Trial, Assess, Hold
- Flag relevance tags: security, performance, DX, architecture, feature-gap

**Phase 4 — SYNTHESIZE (report generation)**
- Generate Technology Radar report grouped by quadrant and ring
- Include per-finding: title, ring, rationale, source citations (3+), effort estimate
- Rank by impact/effort ratio within each ring
- Persist to `docs/audits/web-radar-YYYY-MM-DD.md`
- Summary table in terminal with top 10 actionable findings

### Search Strategy Design

The search strategy follows research-backed principles for LLM-powered web research:

1. **Short keyword queries** over natural language (better recall with search APIs)
2. **Year-anchored queries** to avoid stale results (`[topic] 2026`)
3. **Pseudo-answer generation**: generate hypothetical answer first, extract search terms
4. **Query variation on retry**: rephrase if first query yields nothing useful
5. **Source triangulation**: cross-reference 3+ independent sources per recommendation
6. **Source weighting**: primary sources (official docs, release notes) > secondary (blog posts, tutorials)
7. **Channel diversity**: newsletters, conferences, awesome-lists, GitHub trending, comparison articles, postmortems

### Technology Radar Output Format

```markdown
# Web Upgrade Radar — 2026-03-27

## Techniques (Adopt)
### Structured Error Handling with thiserror 2.0
- **Ring**: Adopt | **Fit**: 5/5 | **Maturity**: 5/5 | **Effort**: 2/5
- **Rationale**: ...
- **Sources**: [1] ..., [2] ..., [3] ...

## Tools (Trial)
### cargo-mutants for Mutation Testing
- **Ring**: Trial | **Fit**: 4/5 | **Maturity**: 3/5 | **Effort**: 2/5
...
```

## Ready-to-Paste Prompt

```
/spec-dev Web-based upgrade audit command (/audit-web)

Create a new /audit-web command that systematically scans the web for potential
upgrades to the current project and outputs findings in Technology Radar format.

Architecture:
- Phase 1 (INVENTORY): Auto-detect dependencies, patterns, tools from codebase.
  Parse Cargo.toml, run cargo outdated/audit/clippy, extract arch patterns from
  CLAUDE.md. Support --focus=deps|arch|tools|ecosystem|features to narrow scope.
- Phase 2 (LANDSCAPE SCAN): 8+ parallel search agents, one per scope dimension:
  dep-scanner, arch-scout, tool-scout, ecosystem-scout, competitor-scout,
  user-request-miner, blog-miner, research-scout. Each uses source triangulation
  (3+ sources per finding) with keyword queries, year-anchored, varied on retry.
- Phase 3 (EVALUATE): Score each finding on strategic fit, maturity, migration
  effort (0-5 each). Classify into radar quadrant and ring (adopt/trial/assess/hold).
- Phase 4 (SYNTHESIZE): Generate Technology Radar report grouped by quadrant and
  ring. Persist to docs/audits/web-radar-YYYY-MM-DD.md. Display top 10 summary.

Search strategy:
- Short keyword queries > natural language (better search API recall)
- Year-anchored queries to avoid stale results
- Pseudo-answer generation for query formulation
- Query variation on retry
- Source triangulation (3+ independent sources)
- Source weighting (primary > secondary)
- Channel diversity (newsletters, conferences, awesome-lists, GitHub, postmortems)

Scope dimensions with query templates:
- Dependencies: "[crate] changelog [year]", "[crate] alternatives comparison"
- Architecture: "[pattern] state of the art [year]", "[pattern] RFC accepted"
- Tooling: "rust dev tools [year]", "[tool] vs alternatives"
- Ecosystem: "this week in rust", "rustconf [year] highlights"
- Competitors: "[similar tool] features", "[tool category] comparison [year]"
- User requests: GitHub issues, "how I use [tool]", "[category] wishlist"
- Blog patterns: "AI coding agent workflow", "[tool type] tips tricks"
- Research: "autonomous coding agent architecture", "LLM agent paper [year]"

Output: ThoughtWorks-style Technology Radar in docs/audits/web-radar-YYYY-MM-DD.md
with per-finding scores, ring classification, rationale, and 3+ source citations.

This is a new command (commands/audit-web.md) with a new agent (agents/web-scout.md)
that orchestrates the 8 parallel search sub-agents.
```

## Scope Estimate

EPIC — new command + orchestrator agent + 8 search agent definitions + radar output format + search strategy system + persistence.

## Dependencies

- Requires WebSearch and WebFetch tools (already available)
- Benefits from BL-078 (context pre-hydration) for Phase 1 inventory
- Integrates with /backlog for auto-generating BL entries from 'adopt' findings (future enhancement)

## Design Decisions from Grill-Me

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Trigger | On-demand `/audit-web` only | Avoids uncontrolled token burn from scheduled runs |
| Scope dimensions | All 5 + feature upgrades | Maximum discovery value |
| Output format | Technology Radar (quadrants + rings) | Structured, actionable, industry-standard |
| Research depth | Source triangulation (3+ sources) | Reliability over speed |
| Codebase detection | Auto-detect + `--focus` overrides | Discovery value + precision when needed |
| Persistence | `docs/audits/web-radar-YYYY-MM-DD.md` | Track findings over time |
| Feature discovery | Competitor analysis + user mining + blog mining + research papers | All 4 channels for maximum coverage |
| Parallelism | 8+ agents | Maximum speed, user accepted token cost |
