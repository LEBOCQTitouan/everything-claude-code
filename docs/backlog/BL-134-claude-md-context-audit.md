---
id: BL-134
title: "Audit CLAUDE.md for LLM-generated content"
scope: LOW
target: "direct edit"
status: implemented
created: "2026-04-09"
source: "docs/audits/web-radar-2026-04-09.md"
ring: adopt
tags: [context-engineering, docs]
---

## Context

ETH Zurich research (2026) warns that LLM-generated context files hinder agents — human-written, non-inferable instructions only. Anthropic recommends CLAUDE.md contain project-specific rules that cannot be derived from the codebase.

## Prompt

Audit CLAUDE.md for content that is inferable from the codebase (file structure, test counts, command lists that could be derived by reading code). Remove inferable content. Keep only non-obvious rules, gotchas, and constraints that an agent cannot derive by reading source.

## Acceptance Criteria

- [ ] CLAUDE.md contains only non-inferable instructions
- [ ] Removed sections documented in commit message
- [ ] All workflows still function correctly
