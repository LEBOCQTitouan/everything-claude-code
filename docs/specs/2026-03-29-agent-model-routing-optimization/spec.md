# Spec: Agent model routing optimization (BL-094)

## Problem Statement

32 of 51 ECC agents (62.7%) use Opus, but Anthropic's official guidance recommends Sonnet as the default for most work, reserving Opus for complex multi-step reasoning. Many ECC agents perform checklist-based audits, pattern matching, or code review that Sonnet handles equally well. This misalignment inflates cost by ~75% per agent invocation for tasks that don't benefit from Opus-level reasoning. The current `performance.md` routing rules assign all code review to Opus, contradicting Anthropic's 2026 recommendation.

## Research Summary

- Anthropic official: "Start with Sonnet, route only the most demanding to Opus" (claude.com/resources/tutorials)
- Anthropic official: "Sonnet balances capability and speed for analyzing code patterns" (code.claude.com/docs/en/sub-agents)
- Sonnet 4.6 matches Opus 4.5 in coding benchmarks (Feb 2026)
- Cost ratio: Haiku $1/$5, Sonnet $3/$15, Opus $5/$25 per MTok
- Conservative estimate: 15-25% overall cost reduction; 30-40% on review-heavy sessions

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Downgrade 10 language reviewers to Sonnet | Anthropic confirms Sonnet handles coding tasks equally well | Yes |
| 2 | Downgrade drift-checker to Haiku | Simple diff/staleness detection, no reasoning needed | Yes |
| 3 | Downgrade doc-validator, web-scout to Sonnet | Checklist/dispatch work, not deep analysis | Yes |
| 4 | Downgrade doc-orchestrator to Sonnet | Dispatch/aggregation agent, not deep reasoning | Yes |
| 5 | Keep 18 agents at Opus | code-reviewer, security-reviewer, architect, architect-module, uncle-bob, arch-reviewer, robert, spec-adversary, solution-adversary, planner, requirements-analyst, interviewer, interface-designer, audit-orchestrator + 4 deferred (doc-analyzer, harness-optimizer, evolution-analyst, component-auditor) | Yes |
| 6 | One commit per tier (3 commits) | Easier to revert per tier if quality regresses | No |
| 7 | Defer doc-analyzer, harness-optimizer, evolution-analyst, component-auditor | User chose BL-094 scope only | No |

## User Stories

### US-001: Optimize agent model routing

**As a** developer using ECC, **I want** agents routed to the appropriate model tier, **so that** cost is reduced without quality regression.

#### Acceptance Criteria

- AC-001.1: Given drift-checker agent, when frontmatter is read, then model is `haiku`
- AC-001.2: Given doc-validator agent, when frontmatter is read, then model is `sonnet`
- AC-001.3: Given web-scout agent, when frontmatter is read, then model is `sonnet`
- AC-001.4: Given doc-orchestrator agent, when frontmatter is read, then model is `sonnet` (was opus)
- AC-001.5: Given each of the 10 language-specific reviewers (python, go, rust, typescript, java, kotlin, cpp, csharp, shell, database), when frontmatter is read, then model is `sonnet`
- AC-001.6: Given `rules/common/performance.md`, when model routing section is read, then Haiku lists "diff-based detection, simple extraction, diagram generation", Sonnet lists "code review, language-specific review, audit checks, orchestration, TDD", and Opus lists "architecture decisions, security review, adversarial reasoning, planning"
- AC-001.7: Given the ADR directory, when listing ADRs, then a model routing policy ADR exists with Status: Accepted
- AC-001.8: Given all agent files, when `ecc validate agents` is run, then it passes with zero errors
- AC-001.9: Given the 18 agents that should remain on Opus (14 justified + 4 deferred), when frontmatter is read, then model is still `opus`
- AC-001.10: Given the 4 deferred agents (doc-analyzer, harness-optimizer, evolution-analyst, component-auditor), when frontmatter is read, then model is still `opus` (explicitly deferred, not accidentally changed)

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `agents/drift-checker.md` | Config | model: opus → haiku |
| `agents/doc-validator.md` | Config | model: opus → sonnet |
| `agents/web-scout.md` | Config | model: opus → sonnet |
| `agents/doc-orchestrator.md` | Config | model: opus → sonnet |
| `agents/python-reviewer.md` | Config | model: opus → sonnet |
| `agents/go-reviewer.md` | Config | model: opus → sonnet |
| `agents/rust-reviewer.md` | Config | model: opus → sonnet |
| `agents/typescript-reviewer.md` | Config | model: opus → sonnet |
| `agents/java-reviewer.md` | Config | model: opus → sonnet |
| `agents/kotlin-reviewer.md` | Config | model: opus → sonnet |
| `agents/cpp-reviewer.md` | Config | model: opus → sonnet |
| `agents/csharp-reviewer.md` | Config | model: opus → sonnet |
| `agents/shell-reviewer.md` | Config | model: opus → sonnet |
| `agents/database-reviewer.md` | Config | model: opus → sonnet |
| `rules/common/performance.md` | Rules | Update routing table |
| `docs/adr/NNNN-model-routing-policy.md` | Docs | New ADR |
| `CHANGELOG.md` | Docs | Entry |

## Constraints

- All changes are behavior-preserving in terms of functionality — only quality/cost trade-off shifts
- `ecc validate agents` must pass after every tier commit
- No Rust code changes required — frontmatter only
- Changes shipped in 3 atomic commits (one per tier) for easy per-tier revert

## Non-Requirements

- Changing doc-analyzer, harness-optimizer, evolution-analyst, component-auditor (deferred)
- Changing any currently-sonnet or currently-haiku agents (already correct)
- Adding runtime cost tracking (BL-096)
- Runtime model selection logic (static frontmatter only)
- Measuring before/after cost impact (requires BL-096)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Agent frontmatter | Config change | Agent quality may shift — monitored via manual spot-check |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Routing policy | Rules | rules/common/performance.md | Update Model Selection Strategy section |
| Routing ADR | ADR | docs/adr/NNNN-model-routing-policy.md | Create new ADR |
| Changelog | Project | CHANGELOG.md | Add entry under ### Changed |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Scope — include 4 extra agents? | BL-094's 12 only, defer extras | User |
| 2 | Commit strategy | One commit per tier (3 commits) | User |
| 3 | Testing approach | ecc validate agents + manual spot-check | User |
| 4 | ADR needed | Yes — ADR + CHANGELOG + performance.md | User |

**Smells addressed**: #1 (language reviewers), #2 (drift-checker), #3 (doc-validator), #4 (web-scout), #5 (doc-orchestrator), #10 (performance.md)
**Smells deferred**: #6 (doc-analyzer), #7 (harness-optimizer), #8 (evolution-analyst), #9 (component-auditor)

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Optimize agent model routing | 10 | none |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | drift-checker → haiku | US-001 |
| AC-001.2 | doc-validator → sonnet | US-001 |
| AC-001.3 | web-scout → sonnet | US-001 |
| AC-001.4 | doc-orchestrator → sonnet (was opus) | US-001 |
| AC-001.5 | 10 language reviewers → sonnet | US-001 |
| AC-001.6 | performance.md updated with three-tier routing | US-001 |
| AC-001.7 | ADR created | US-001 |
| AC-001.8 | ecc validate agents passes | US-001 |
| AC-001.9 | 18 agents remain on opus | US-001 |
| AC-001.10 | 4 deferred agents verified unchanged | US-001 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Ambiguity | 70 | PASS | AC-001.6 made concrete after feedback |
| Edge Cases | 75 | PASS | Deferred agents protected by AC-001.10 |
| Scope | 85 | PASS | Clear tier boundaries |
| Dependencies | 85 | PASS | No code deps, config-only |
| Testability | 80 | PASS | ecc validate agents + frontmatter checks |
| Decisions | 80 | PASS | All 7 decisions documented |
| Rollback | 80 | PASS | Per-tier commits enable per-tier revert |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-29-agent-model-routing-optimization/spec.md | Full spec + Phase Summary |
