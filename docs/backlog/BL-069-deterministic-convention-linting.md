---
id: BL-069
title: Deterministic convention linting — naming, placement, frontmatter field values
status: "implemented"
scope: MEDIUM
target: /spec dev
created: 2026-03-26
tags: [deterministic, linting, conventions, rust-cli]
related: []
---

# BL-069: Deterministic Convention Linting

## Problem

Multiple agents and rules enforce naming/placement conventions via LLM instructions:
- Agent filenames must be kebab-case and match frontmatter `name` field
- Skill directory names must match SKILL.md `name` field
- `model` field must be one of {haiku, sonnet, opus}
- `tools` field values must be from the known tool set
- Files must be in canonical directories (agents/, commands/, skills/, hooks/, rules/)

Currently `ecc validate` partially covers this (frontmatter presence), but not value validation.

## Proposed Solution

### Expand `ecc validate` with convention checks

**Naming checks:**
- Filename matches `name` field in frontmatter (case-insensitive, hyphen-normalized)
- All filenames are kebab-case (no underscores, no camelCase)
- Skill directories match SKILL.md `name` field

**Value checks:**
- `model` field in allowed set: `{haiku, sonnet, opus}`
- `tools` field values in known tool registry (Read, Write, Edit, Bash, Grep, Glob, Agent, etc.)
- `type` field in frontmatter matches expected values per content kind

**Placement checks:**
- No agent .md files outside agents/
- No orphaned skill directories (directory exists but no SKILL.md)
- No command files outside commands/

### Output Format
```
WARN  agents/foo_bar.md: filename uses underscores, expected kebab-case
ERROR agents/planner.md: model "gpt-4" not in allowed set {haiku, sonnet, opus}
WARN  skills/my-skill/: missing SKILL.md
OK    67/70 files pass convention checks
```

## Impact

- **Reliability**: Catches convention drift before it reaches LLM prompts
- **Speed**: Full convention scan in < 200ms
- **CI-ready**: Can be added to pre-commit or CI pipeline
- **Agent simplification**: convention-auditor agent can focus on semantic patterns, not mechanical checks

## Research Context

CodeRabbit: "40+ built-in linters for concrete violations; LLM for semantic review."
Continue.dev: deterministic.ts module for structured enforcement.
