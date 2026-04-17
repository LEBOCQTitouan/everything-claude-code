---
id: BL-096
title: "Cost and token tracking — observability prerequisite for optimization"
scope: MEDIUM
target: "/spec dev"
status: implemented
tags: [cost, observability, tokens, optimization, prerequisite]
created: 2026-03-28
related: [BL-094, BL-095, BL-092]
---

# BL-096: Cost and Token Tracking

## Problem

ECC has zero visibility into token consumption per agent, per session, or per command. Without measurement, model routing optimizations (BL-094) and thinking budget tuning (BL-095) cannot be validated. You can't optimize what you can't measure.

Currently:
- No agent tracks its own token usage
- No orchestrator aggregates child agent costs
- No post-session report shows where tokens went
- No way to compare cost before/after an optimization

## Proposed Solution

### Tier 1 — Minor impact (session-level tracking via hooks)

Add a `PostToolUse` hook that captures token metadata from Claude Code's internal reporting:

- **Input tokens** (prompt + cached)
- **Output tokens** (response + thinking)
- **Cache hit ratio**
- **Model used**
- **Agent name** (from context)

Persist to a lightweight append-only log: `~/.ecc/logs/token-usage.jsonl`

Format per line:
```json
{"ts":"2026-03-28T14:30:00Z","session":"abc123","agent":"rust-reviewer","model":"opus","input_tokens":12500,"output_tokens":3200,"thinking_tokens":8000,"cache_hit_pct":0.72}
```

### Tier 2 — Medium impact (per-command cost aggregation)

Orchestrator agents (`audit-orchestrator`, `doc-orchestrator`) aggregate token usage from all child agents and report a summary:

```
Audit session cost breakdown:
  arch-reviewer (opus):     45K in / 12K out / 8K think  = $0.38
  convention-auditor (son): 22K in / 6K out / 4K think   = $0.08
  test-auditor (son):       18K in / 5K out / 3K think   = $0.06
  Total:                    85K in / 23K out / 15K think  = $0.52
```

### Tier 3 — High impact (`ecc cost` CLI subcommand)

Add `ecc cost` subcommands leveraging the JSONL log:

```
ecc cost summary              # Last 7 days aggregate
ecc cost breakdown --by agent # Per-agent breakdown
ecc cost breakdown --by model # Per-model breakdown
ecc cost compare --before <date> --after <date>  # Before/after comparison
```

This builds on BL-092's logging infrastructure (shared `~/.ecc/logs/` directory).

## Evidence

- Anthropic cost page: prompt caching saves up to 90% but requires measurement ([platform.claude.com/docs/en/about-claude/pricing](https://platform.claude.com/docs/en/about-claude/pricing))
- `cost-aware-llm-pipeline` skill already defines budget tracking patterns but no ECC agent implements them
- Community: "If you're spending >$50/mo and not tracking, you're flying blind"

## Estimated Impact

- **Tier 1:** Enables data-driven decisions for BL-094 and BL-095
- **Tier 2:** Identifies which commands/agents are cost hotspots
- **Tier 3:** Ongoing monitoring and regression detection

## Dependencies

- Shares `~/.ecc/logs/` infrastructure with BL-092 (structured log management)

## Ready-to-Paste Prompt

```
/spec dev Add cost and token tracking to ECC. Three tiers:
(1) PostToolUse hook captures token metadata (input, output, thinking,
cache hit, model, agent) to ~/.ecc/logs/token-usage.jsonl.
(2) Orchestrator agents aggregate child agent costs and report summary.
(3) `ecc cost` CLI subcommands for summary, per-agent/model breakdown,
and before/after comparison. Shares log infrastructure with BL-092.
```
