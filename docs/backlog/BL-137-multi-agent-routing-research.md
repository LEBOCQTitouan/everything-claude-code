---
id: BL-137
title: "Apply difficulty-aware model routing from multi-agent research"
scope: HIGH
target: "/spec-dev"
status: open
created: "2026-04-09"
source: "docs/audits/web-radar-2026-04-09.md"
ring: assess
tags: [research, model-routing, agents]
---

## Context

ACM TOSEM literature review (Dec 2025) and arxiv papers establish multi-agent LLM systems with difficulty-aware routing (MaAS) as the dominant SE paradigm. 80x improvement in action specificity vs single-agent. Directly applicable to ECC's model-routing rules (opus/sonnet/haiku).

## Prompt

Research and apply difficulty-aware routing patterns from LLM multi-agent literature to ECC's agent orchestration. Evaluate: dynamic model selection based on task complexity (instead of static frontmatter `model` field), runtime fallback on context overflow, cost-weighted routing. Prototype with 2-3 agents that currently use opus but could use sonnet for simple inputs.

## Acceptance Criteria

- [ ] Literature review of MaAS and difficulty-aware routing patterns
- [ ] Design proposal for dynamic model selection in ECC
- [ ] Prototype with measurable cost reduction
- [ ] No quality regression on agent outputs
