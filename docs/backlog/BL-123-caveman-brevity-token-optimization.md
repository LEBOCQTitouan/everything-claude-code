---
id: BL-123
title: "Caveman-style brevity optimization — reduce token consumption across all ECC agents and commands"
scope: HIGH
target: "/spec-refactor"
status: open
tags: [token-optimization, brevity, agents, commands, cost, output-reduction]
created: 2026-04-06
related: [BL-121, BL-094, BL-095]
source: "https://github.com/JuliusBrussee/caveman"
---

# BL-123: Caveman-Style Brevity Token Optimization

## Problem

ECC agents and commands produce verbose output and consume large input context windows. Output verbosity (filler words, pleasantries, hedging language, redundant explanations) wastes ~65% of output tokens per the caveman research. Command instructions (implement.md at 400+ lines, design.md at 250+ lines) inflate input context unnecessarily.

## Proposed Solution

Apply [caveman](https://github.com/JuliusBrussee/caveman) brevity principles across ECC:

1. **Output brevity** — Add a global rule (`rules/common/brevity.md`) enforcing concise agent output: eliminate filler words, pleasantries, hedging language. Preserve code blocks, technical terms, error messages, commit messages.

2. **Input compression** — Audit and compress every command/agent/skill file: remove redundant examples, collapse verbose instructions, use tables instead of prose where possible. Goal: 30-50% instruction size reduction without losing behavioral fidelity.

3. **Per-agent audit** — Review all 57 agents for output verbosity. Agents that produce narrative (planner, requirements-analyst, code-reviewer) get brevity constraints. Agents that produce structured data (tdd-executor, build-error-resolver) are already concise.

4. **Benchmark** — Measure before/after token consumption on a standard task (e.g., `/spec-dev` → `/design` → `/implement` for a LOW scope item) to quantify savings.

Reference: [JuliusBrussee/caveman](https://github.com/JuliusBrussee/caveman) — 65% average output token reduction through linguistic constraint. March 2026 research showed brevity improved accuracy by 26 percentage points on certain benchmarks.

## Ready-to-Paste Prompt

```
/spec-refactor Apply caveman-style brevity optimization across all ECC components:

1. Global brevity rule (rules/common/brevity.md)
   - Eliminate: articles, pleasantries, hedging language, redundant transitions
   - Preserve: code blocks, technical terms, error messages, commit messages
   - Single default intensity (no configuration needed)

2. Per-agent output audit (agents/*.md)
   - Review all 57 agents for output verbosity
   - Add brevity constraints to narrative-producing agents
   - Skip agents that already produce structured/terse output (tdd-executor, etc.)

3. Command instruction compression (commands/*.md)
   - Audit all command files for instruction bloat
   - Compress verbose prose into tables/bullet points
   - Remove redundant examples and repeated boilerplate
   - Target: 30-50% instruction size reduction

4. Skill instruction compression (skills/*.md)
   - Same audit for skill files
   - Collapse multi-paragraph explanations into concise directives

5. Benchmark
   - Measure token consumption on a standard LOW-scope /spec → /design → /implement cycle
   - Before: current verbose instructions
   - After: compressed instructions + brevity rule
   - Report: input tokens saved, output tokens saved, total % reduction

Reference: https://github.com/JuliusBrussee/caveman
Source principle: linguistic constraint reduces output tokens by ~65% while maintaining (or improving) accuracy.
```

## Scope Estimate

HIGH — touches all 57 agents, 30+ commands, 100+ skills. Each individual change is small (add brevity constraints, compress prose) but the audit coverage is broad.

## Dependencies

- None hard. BL-121 (token optimization audit) would provide data-driven prioritization but is not required.
- BL-096 (cost tracking) would enable before/after measurement but benchmark can use manual token counting.
