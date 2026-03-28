---
id: BL-095
title: "Extended thinking and effort tuning — adaptive thinking budgets per agent type"
scope: MEDIUM
target: "/spec dev"
status: open
tags: [cost, thinking, effort, optimization]
created: 2026-03-28
related: [BL-094, BL-096]
---

# BL-095: Extended Thinking and Effort Tuning

## Problem

Extended thinking defaults to 31,999 tokens per request — the single largest cost driver at scale. Every agent invocation, regardless of complexity, burns the same thinking budget. A `drift-checker` comparing two strings uses the same thinking allocation as an `architect` designing bounded contexts. Anthropic's 2026 guidance: "Start at minimum (1,024 tokens) and increase incrementally."

Additionally, Anthropic deprecated fixed `budget_tokens` on Opus/Sonnet 4.6 in favor of **adaptive thinking**, which dynamically allocates reasoning based on complexity. ECC doesn't leverage this yet.

## Proposed Solution

### Tier 1 — Minor impact (environment variable cap)

Set `MAX_THINKING_TOKENS` per agent tier in spawning logic:

| Agent tier | Thinking cap | Rationale |
|-----------|-------------|-----------|
| Haiku agents | N/A | Haiku doesn't use extended thinking |
| Simple Sonnet (auditors, reviewers) | 8,000 | Checklist tasks, pattern matching |
| Complex Sonnet (TDD, build-fix) | 16,000 | Multi-step implementation |
| Opus agents | 32,000 (default) | Full reasoning for architecture/security |

**Est. savings:** ~70% reduction in thinking tokens for lightweight agents.

### Tier 2 — Medium impact (effort levels per agent)

Leverage Claude Code's `/effort` command equivalent in agent spawning:

| Agent category | Effort level | Rationale |
|---------------|-------------|-----------|
| Diagram generators, doc reporters | low | Template-based output |
| Auditors, convention checkers | medium | Structured analysis |
| Code reviewers, TDD executors | high | Nuanced judgment |
| Architects, adversaries, planners | max | Deep multi-step reasoning |

### Tier 3 — High impact (adaptive thinking adoption)

Migrate from fixed `budget_tokens` to adaptive thinking for Opus 4.6 and Sonnet 4.6 agents. Adaptive thinking is Anthropic's recommended approach — the model self-determines how much to think based on query complexity.

**Implementation:** Remove any hardcoded thinking budget overrides and ensure agents use the adaptive default. This is the simplest change with the highest leverage.

## Evidence

- Anthropic: "Adaptive thinking is recommended for Opus 4.6 and Sonnet 4.6" ([platform.claude.com/docs](https://platform.claude.com/docs/en/build-with-claude/adaptive-thinking))
- Anthropic: "budget_tokens deprecated on Opus/Sonnet 4.6" ([platform.claude.com/docs](https://platform.claude.com/docs/en/build-with-claude/extended-thinking))
- Community: Setting effort to medium for daily coding saves significant cost ([code.claude.com/docs](https://code.claude.com/docs/en/costs))
- 70% reduction in thinking cost per request when capping at 10K tokens

## Estimated Impact

- **Tier 1 (caps):** 20-30% reduction in thinking token spend
- **Tier 2 (effort levels):** Additional 10-15% savings
- **Tier 3 (adaptive):** Best quality/cost ratio — model self-optimizes

## Prerequisite

BL-096 (cost/token tracking) for before/after measurement.

## Ready-to-Paste Prompt

```
/spec dev Add thinking budget tuning to ECC agent spawning. Three tiers:
(1) Set MAX_THINKING_TOKENS env var per agent category — 8K for auditors,
16K for TDD/build, 32K for architects. (2) Map agent categories to effort
levels (low/medium/high/max) in agent frontmatter. (3) Ensure all Opus/Sonnet
4.6 agents use adaptive thinking (no fixed budget_tokens). Prerequisite:
BL-096 cost tracking.
```
