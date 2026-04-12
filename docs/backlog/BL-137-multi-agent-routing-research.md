---
id: BL-137
title: "Apply difficulty-aware model routing from multi-agent research"
scope: HIGH
target: "/spec-dev"
status: archived
created: "2026-04-09"
archived: "2026-04-12"
source: "docs/audits/web-radar-2026-04-09.md"
ring: assess
tags: [research, model-routing, agents]
---

## Archival Rationale (2026-04-12)

Archived as **not feasible** with the current Claude Code harness, and **superseded** by BL-094 for the realistic win.

**Why not feasible:**
- The Task tool's input schema is `{subagent_type, description, prompt}` — `model` is not a hookable field.
- PreToolUse hooks (v2.0.10+) can mutate Task input but cannot override the `model` resolved from the agent definition file.
- SubagentStart fires after dispatch — observational only, no input mutation.
- The only workaround is `subagent_type` swap to variant agents (`*-sonnet` / `*-opus`), which doubles or triples the agent count for marginal gain.

**Why superseded:**
- BL-094 already captured the ~80% cost win via a one-time static re-tier of 14 agent frontmatter fields per Anthropic guidance.
- Dynamic routing's remaining upside (runtime fallback on context overflow, cost-weighted decisions) does not justify the variant-explosion maintenance cost.

**Revisit trigger:** if Anthropic ships a runtime model-override API (track via WebSearch quarterly), reopen as MEDIUM.

## Context

ACM TOSEM literature review (Dec 2025) and arxiv papers establish multi-agent LLM systems with difficulty-aware routing (MaAS) as the dominant SE paradigm. 80x improvement in action specificity vs single-agent. Directly applicable to ECC's model-routing rules (opus/sonnet/haiku).

## Prompt

Research and apply difficulty-aware routing patterns from LLM multi-agent literature to ECC's agent orchestration. Evaluate: dynamic model selection based on task complexity (instead of static frontmatter `model` field), runtime fallback on context overflow, cost-weighted routing. Prototype with 2-3 agents that currently use opus but could use sonnet for simple inputs.

## Acceptance Criteria

- [ ] Literature review of MaAS and difficulty-aware routing patterns
- [ ] Design proposal for dynamic model selection in ECC
- [ ] Prototype with measurable cost reduction
- [ ] No quality regression on agent outputs
