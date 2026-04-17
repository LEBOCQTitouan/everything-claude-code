---
id: BL-094
title: "Agent model routing optimization — downgrade misaligned agents from Opus to Sonnet/Haiku"
scope: HIGH
target: "/spec refactor"
status: implemented
tags: [cost, model-routing, agents, optimization]
created: 2026-03-28
related: [BL-095, BL-096]
---

# BL-094: Agent Model Routing Optimization

## Problem

32 of 51 agents (62.7%) use Opus, but official Anthropic guidance recommends Sonnet as the default for most work, reserving Opus for complex multi-step reasoning. Many ECC agents perform checklist-based audits, pattern matching, or simple orchestration that Sonnet handles equally well. This misalignment inflates cost by ~5x per agent invocation for tasks that don't benefit from Opus-level reasoning.

Current `performance.md` rules assign all code review and language-specific review to Opus, contradicting Anthropic's 2026 recommendation: "Start with Sonnet as default, only route the most demanding tasks up to Opus."

## Proposed Solution

### Tier 1 — Minor impact (quick wins, no quality risk)

| Agent | Current | Target | Rationale | Est. savings |
|-------|---------|--------|-----------|-------------|
| `drift-checker` | Opus | Haiku | Simple diff/staleness detection, no reasoning needed | ~95% |
| `module-summary-updater` | Haiku | Haiku | Already correct | — |
| `doc-reporter` | Haiku | Haiku | Already correct | — |

### Tier 2 — Medium impact (audit agents)

| Agent | Current | Target | Rationale | Est. savings |
|-------|---------|--------|-----------|-------------|
| `error-handling-auditor` | Opus | Sonnet | Checklist-based pattern matching | ~75% |
| `convention-auditor` | Sonnet | Sonnet | Already correct | — |
| `observability-auditor` | Sonnet | Sonnet | Already correct | — |
| `test-auditor` | Sonnet | Sonnet | Already correct | — |
| `doc-validator` | Opus | Sonnet | Validation against standards, not deep analysis | ~75% |
| `web-scout` | Opus | Sonnet | Mostly queues tasks and deduplicates results | ~75% |

### Tier 3 — High impact (language reviewers + orchestrators)

| Agent | Current | Target | Rationale | Est. savings |
|-------|---------|--------|-----------|-------------|
| `python-reviewer` | Opus | Sonnet | Code-only review; Anthropic confirms Sonnet handles coding tasks | ~75% |
| `go-reviewer` | Opus | Sonnet | Same as above | ~75% |
| `rust-reviewer` | Opus | Sonnet | Same as above | ~75% |
| `typescript-reviewer` | Opus | Sonnet | Same as above | ~75% |
| `java-reviewer` | Opus | Sonnet | Same as above | ~75% |
| `kotlin-reviewer` | Opus | Sonnet | Same as above | ~75% |
| `cpp-reviewer` | Opus | Sonnet | Same as above | ~75% |
| `csharp-reviewer` | Opus | Sonnet | Same as above | ~75% |
| `shell-reviewer` | Opus | Sonnet | Same as above | ~75% |
| `database-reviewer` | Opus | Sonnet | Same as above | ~75% |
| `doc-orchestrator` | Opus | Sonnet | Dispatch/aggregation, not deep reasoning | ~75% |

### Keep at Opus (justified)

- `code-reviewer` — orchestrates security + architecture checks across multiple agents
- `security-reviewer` — deep vulnerability analysis
- `architect`, `architect-module` — system-level design decisions
- `uncle-bob` — opinionated design critique requiring deep pattern knowledge
- `arch-reviewer` — architectural boundary validation
- `planner`, `requirements-analyst` — complex multi-phase planning
- `solution-adversary`, `spec-adversary` — adversarial reasoning
- `interviewer`, `interface-designer` — design exploration requiring deep reasoning

### Update `performance.md`

Revise model routing rules to match Anthropic's official guidance:

```
Haiku 4.5: Diff-based detection, simple extraction, diagram generation
Sonnet 4.6: Code review, language-specific review, audit checks, orchestration, TDD
Opus 4.6: Architecture decisions, security review, adversarial reasoning, planning
```

## Evidence

- Anthropic official: "Start with Sonnet, route only the most demanding to Opus" ([claude.com/resources/tutorials](https://claude.com/resources/tutorials/choosing-the-right-claude-model))
- Anthropic official: "Sonnet balances capability and speed for analyzing code patterns" ([code.claude.com/docs/en/sub-agents](https://code.claude.com/docs/en/sub-agents))
- Sonnet 4.6 matches Opus 4.5 in coding benchmarks (Feb 2026)
- Cost ratio: Haiku $1/$5, Sonnet $3/$15, Opus $5/$25 per MTok

## Estimated Impact

- **Conservative:** 15-25% overall cost reduction
- **With language reviewers:** 30-40% cost reduction on review-heavy sessions
- **Zero quality regression** for code-only review (Anthropic-confirmed)

## Prerequisite

BL-096 (cost/token tracking) should ship first to measure before/after impact.

## Ready-to-Paste Prompt

```
/spec refactor Optimize agent model routing across ECC. Downgrade ~12 agents from
Opus to Sonnet/Haiku per Anthropic's official guidance. Update performance.md rules.
Agents to change: drift-checker (→Haiku), error-handling-auditor, doc-validator,
web-scout, doc-orchestrator (→Sonnet), all 10 language-specific reviewers (→Sonnet).
Keep Opus for: code-reviewer, security-reviewer, architect*, uncle-bob, adversaries,
planner, requirements-analyst. Prerequisite: BL-096 cost tracking for before/after
measurement.
```
