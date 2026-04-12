---
id: BL-147
title: "AGENTS.md / AAIF standard alignment audit"
scope: MEDIUM
target: "/spec-dev"
status: implemented
created: "2026-04-12"
source: "docs/research/competitor-claw-goose.md"
ring: assess
tags: [standards, interop, agents]
---

## Context

The Linux Foundation's Agentic AI Foundation (AAIF) brings together MCP (Anthropic), Goose (Block), and Agents.md (OpenAI) under a neutral governance framework. Claw Code and Goose both use `AGENTS.md`-style agent definitions. ECC's agent format is similar but not formally aligned.

## Prompt

Audit ECC's agent and skill definition formats against the emerging AGENTS.md / AAIF standard. Identify gaps, document divergences, and propose alignment where it does not compromise ECC's Claude-specific features. Out of scope: giving up Claude-specific capabilities (memory types, SubagentStart hooks, effort tuning). In scope: frontmatter field naming, discovery paths, metadata conventions.

## Acceptance Criteria

- [ ] Gap analysis document: ECC agent format vs AGENTS.md spec
- [ ] List of adoptable conventions with no loss of ECC capability
- [ ] List of Claude-specific extensions ECC must keep
- [ ] ADR on alignment stance (full-align, partial-align, independent)
