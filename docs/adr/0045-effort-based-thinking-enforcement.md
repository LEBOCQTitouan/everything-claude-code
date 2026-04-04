# ADR 0045: Effort-Based Thinking Enforcement

## Status
Accepted

## Date
2026-04-04

## Context
Claude Code supports extended thinking with configurable `MAX_THINKING_TOKENS`. Different agents have vastly different reasoning needs: a diff-checker (haiku) needs minimal thinking, while an adversarial reviewer (opus) benefits from deep reasoning. Without per-agent control, all subagents inherit the parent session's thinking budget, wasting tokens on simple tasks or starving complex ones.

Anthropic's adaptive thinking (default for Opus/Sonnet 4.6) lets the model self-determine reasoning depth, but a budget cap is still needed to bound cost and latency. The deprecated `budget_tokens` API field was a raw number with no semantic meaning, making it hard to maintain across dozens of agents.

## Decision
Use a `SubagentStart` hook to enforce per-agent thinking budgets:

1. Each agent declares an `effort` field in its YAML frontmatter: one of `low`, `medium`, `high`, or `max`.
2. The `SubagentStart` hook reads the spawned agent's `effort` field and maps it to a `MAX_THINKING_TOKENS` value via a compiled effort-to-tokens table (see ADR 0046).
3. The hook sets the token budget by writing `MAX_THINKING_TOKENS=<value>` to stdout, which Claude Code applies to the subagent session.
4. `ecc validate agents` cross-validates model/effort combinations (e.g., haiku agents must use `low`, opus agents must use `high` or `max`) and rejects the deprecated `budget_tokens` field.
5. The entire mechanism is bypassed when `ECC_EFFORT_BYPASS=1` is set, for debugging or benchmarking.

The convention is advisory in agent files (frontmatter metadata) but enforced at runtime by the compiled hook binary.

## Consequences
- Every agent file gains a required `effort` field, making thinking cost explicit and reviewable
- The SubagentStart hook adds minimal latency (frontmatter parse + table lookup)
- Model/effort cross-validation catches misconfigured agents at `ecc validate` time, not at runtime
- `budget_tokens` is formally deprecated; validation rejects it with a migration hint
- Bypass via `ECC_EFFORT_BYPASS=1` preserves escape hatch for experimentation
- Future tuning (BL-096) can adjust the mapping table without changing agent files
