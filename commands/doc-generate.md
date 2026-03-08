---
description: Generate missing doc comments, module summaries, glossary, changelog, and usage examples.
---

# Documentation Generation

Generate documentation artifacts from analysis data. Writes doc comments directly into source files and produces documentation in `docs/`.

## What This Command Does

1. Write missing JSDoc/TSDoc/docstring comments into source files
2. Generate per-module summaries (purpose, exports, dependencies)
3. Finalize the glossary with definitions and cross-references
4. Generate a CHANGELOG from git conventional commits
5. Extract usage examples from test files

## Arguments

- `--module=<name>` — generate for a specific module only
- `--comments-only` — only write doc comments, skip other artifacts
- `--changelog` — only generate the changelog
- `--dry-run` — report what would be written without writing

## Prerequisites

Requires `docs/ARCHITECTURE.md` from a previous `/doc-analyze` run. If missing, suggests running `/doc-analyze` first.

## When to Use

- After `/doc-analyze` to flesh out documentation
- To add doc comments to undocumented exports
- To generate a changelog before a release
- As part of the full `/doc-suite`

## Related

- Full suite: `/doc-suite`
- Prerequisite: `/doc-analyze`
- Agent: `agents/doc-generator.md`
