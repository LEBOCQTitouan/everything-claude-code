---
description: Analyze codebase structure, public API surface, domain concepts, and dependencies for documentation purposes.
---

# Documentation Analysis

Run codebase analysis to understand what needs documenting. Produces structured output in `docs/` that other doc commands consume.

## What This Command Does

1. Detect project profile (language, framework, size)
2. Inventory all modules and their public exports
3. Extract domain concepts and build a draft glossary
4. Map module dependencies with doc status annotations
5. Write `docs/ARCHITECTURE.md`, `docs/API-SURFACE.md`, `docs/DEPENDENCY-GRAPH.md`, `docs/GLOSSARY.md` (draft)

## Arguments

- `--scope=<path>` — limit analysis to a subdirectory (default: project root)

## Output

Writes to `docs/` — single files for small codebases, folders for large ones.

## When to Use

- Before running `/doc-generate`, `/doc-validate`, or `/doc-coverage`
- To understand a new codebase's structure
- As a prerequisite for the full `/doc-suite`

## Related

- Full suite: `/doc-suite`
- Agent: `agents/doc-analyzer.md`
- Skill: `skills/doc-analysis/SKILL.md`
