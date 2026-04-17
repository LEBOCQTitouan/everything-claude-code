---
id: BL-149
title: "Add agentic self-evaluation step between /implement TDD iterations"
scope: MEDIUM
target: "/spec-dev"
status: implemented
created: "2026-04-12"
source: "docs/research/competitor-claw-goose.md"
ring: assess
tags: [implement, tdd, agentic-loop]
---

## Context

Goose's core agent loop explicitly runs plan → select tools → execute → **evaluate** → loop-or-exit. The evaluation step asks "did this iteration move us toward the goal?" and exits early on blockers. ECC's `/implement` TDD loop has implicit gates (tests green, no clippy warnings) but no explicit self-evaluation of whether the iteration actually advanced the spec.

## Prompt

Add an explicit self-evaluation step after each Pass Condition completes in `/implement`. The check asks: (a) does the current PC output actually satisfy its AC, (b) did it introduce regressions not caught by tests, (c) is the spec still achievable or does it need revision. On "spec needs revision" the loop should escalate to the user rather than continuing blindly. Integrate with existing fix-round budget.

## Acceptance Criteria

- [ ] Self-evaluation rubric defined (3-5 dimensions)
- [ ] Evaluation runs automatically after each PC
- [ ] "Spec needs revision" triggers user escalation via AskUserQuestion
- [ ] Integration with fix-round budget is documented
- [ ] Measurable: does it reduce rework ratio vs current pipeline?
