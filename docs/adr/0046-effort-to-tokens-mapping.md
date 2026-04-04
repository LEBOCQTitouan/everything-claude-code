# ADR 0046: Effort-to-Tokens Mapping

## Status
Accepted

## Date
2026-04-04

## Context
ADR 0045 introduces effort levels (low/medium/high/max) for per-agent thinking budget control. The mapping from effort to `MAX_THINKING_TOKENS` must balance three concerns: (1) sufficient reasoning depth for the task category, (2) cost and latency control, and (3) staying within Anthropic's 32,768-token thinking ceiling.

## Decision
The following mapping is adopted:

| Effort | MAX_THINKING_TOKENS | Rationale |
|--------|---------------------|-----------|
| low    | 2,048               | Minimal reasoning for extraction, formatting, diff detection |
| medium | 8,192               | Standard reasoning for code review, audit checks, TDD |
| high   | 16,384              | Deep reasoning for architecture, security analysis |
| max    | 32,768              | Full budget for adversarial review, multi-phase planning |

The values follow a geometric progression (4x, 2x, 2x) reflecting the diminishing marginal value of additional thinking tokens for simpler tasks. The `max` tier uses Anthropic's full 32,768-token ceiling.

This mapping is compiled into the SubagentStart hook binary as a constant table. It is not user-configurable at runtime (aside from the full `ECC_EFFORT_BYPASS=1` bypass).

## Consequences
- Predictable per-agent thinking costs: haiku agents use ~6% of the budget, opus agents use 100%
- Geometric progression means medium agents get 4x low, not 16x, keeping costs proportional
- The mapping is tunable in future releases (BL-096 data collection) by updating the constant table
- No runtime configuration complexity; the table is baked into the binary
- If Anthropic raises the thinking ceiling, only the `max` tier needs updating
