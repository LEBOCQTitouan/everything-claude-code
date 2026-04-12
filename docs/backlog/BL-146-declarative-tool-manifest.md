---
id: BL-146
title: "Declarative tool manifest — centralize allowedTools via Serde-validated YAML/TOML"
scope: HIGH
target: "/spec-dev"
status: open
created: "2026-04-12"
source: "docs/research/competitor-claw-goose.md"
ring: assess
tags: [architecture, agents, tooling]
---

## Context

Claw Code's Tool Manifest Framework defines tool execution pipelines in YAML parsed via Serde for type-safe dispatch. ECC hardcodes `allowedTools` in each agent's frontmatter and repeats the list in slash-command definitions, causing drift and preventing compile-time validation.

## Prompt

Design and implement a declarative tool manifest for ECC. Migrate `allowedTools` lists from agent frontmatter to a central manifest (YAML or TOML) with Serde validation. Requirements: (a) single source of truth for every tool ECC agents may invoke, (b) per-agent scope defined by reference to manifest entries rather than inline repetition, (c) `ecc validate agents` enforces that every referenced tool exists in the manifest, (d) manifest changes are one-edit propagated to all agents. Consider whether to preserve backward compatibility with inline `allowedTools` during migration.

## Acceptance Criteria

- [ ] Central tool manifest file (YAML or TOML) lists every tool with schema
- [ ] Agent frontmatter references manifest entries instead of duplicating tool names
- [ ] `ecc validate agents` fails on references to non-existent manifest entries
- [ ] Migration script converts existing agents to the new format
- [ ] ADR documents the decision
