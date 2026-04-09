---
id: BL-140
title: "Competitor analysis: Claw Code and Goose agent frameworks"
scope: LOW
target: "/spec-dev"
status: open
created: "2026-04-09"
source: "docs/audits/web-radar-2026-04-09.md"
ring: assess
tags: [competitors, agents, research]
---

## Context

Claw Code (72k GitHub stars, Python+Rust) is the fastest-growing open-source Claude Code alternative. Goose (Block/Square) is an extensible terminal agent. Both use "harness engineering" patterns that mirror ECC's architecture.

## Prompt

Analyze Claw Code and Goose for patterns ECC should adopt. Focus on: tool dispatch mechanisms, context management strategies, agent isolation models, hook/plugin systems. Document convergent patterns and divergences. Identify features ECC lacks that users value in these alternatives.

## Acceptance Criteria

- [ ] Feature comparison matrix: ECC vs Claw Code vs Goose
- [ ] Identified patterns worth adopting
- [ ] Backlog entries for actionable improvements
