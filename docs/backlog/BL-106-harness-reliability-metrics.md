---
id: BL-106
title: "Harness reliability metrics — benchmark ECC against reference patterns"
scope: MEDIUM
target: "/spec-dev"
status: open
tags: [harness, reliability, metrics, benchmarking, quality]
created: 2026-03-29
related: [BL-091, BL-092]
source: "docs/audits/web-radar-2026-03-29-r2.md"
---

# BL-106: Harness Reliability Metrics

## Problem

2026 consensus: the competitive moat for AI agent tools is harness quality (hooks, guardrails, deterministic control), not model selection. ECC has strong harness primitives but no formal reliability metrics to measure and improve harness quality over time.

## Proposed Solution

Formalize harness reliability measurement:
- Define metrics: hook success rate, phase-gate violation rate, state machine consistency, agent failure recovery rate
- Benchmark against Anthropic's reference harness patterns
- Track metrics across sessions via structured logs

## Ready-to-Paste Prompt

```
/spec-dev Define and implement harness reliability metrics for ECC:

1. Metric Definitions
   - Hook success rate: % of hooks that complete without error
   - Phase-gate violation rate: % of state transitions that fail validation
   - Agent failure recovery rate: % of failed agents that are retried successfully
   - Commit atomicity score: % of commits that pass build + test gates

2. Collection Infrastructure
   - Instrument hooks, state machine, and agent spawn with metric counters
   - Store metrics in structured log format (builds on BL-092)
   - Session-level metric summary at /catchup

3. Benchmarking
   - Compare ECC metrics against Anthropic's published harness patterns
   - Identify gaps and improvement opportunities
   - Track trends across sessions

Reference: Anthropic "Effective Harnesses for Long-Running Agents"
Source: docs/audits/web-radar-2026-03-29-r2.md
```
