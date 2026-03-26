---
id: BL-066
title: Deterministic backlog management — ID generation, duplicate detection, index auto-generation
status: open
scope: MEDIUM
target: /spec dev
created: 2026-03-26
tags: [deterministic, backlog, rust-cli]
related: [BL-059]
---

# BL-066: Deterministic Backlog Management

## Problem

The backlog-curator agent currently uses the LLM to:
1. **Generate sequential IDs** — reads BACKLOG.md, finds highest BL-NNN, increments (mechanical counting)
2. **Detect duplicates** — reads all BL-*.md files, compares titles/tags manually (fuzzy matching)
3. **Update the BACKLOG.md index** — manually edits the markdown table (derived data)

All three operations are fully deterministic and waste LLM tokens on mechanical work.

## Proposed Solution

Add three `ecc` CLI subcommands:

### `ecc backlog next-id`
- Glob `docs/backlog/BL-*.md`, extract numeric IDs, return max + 1 zero-padded
- Output: `BL-066`

### `ecc backlog check-duplicates <title> [--tags tag1,tag2]`
- Extract title + tags from all open BL-*.md frontmatter
- Compute keyword set intersection + Levenshtein distance
- Return candidates with confidence score (>0.6 = likely duplicate)
- Output: JSON array of `{id, title, score}`

### `ecc backlog reindex`
- Glob all `docs/backlog/BL-*.md` files
- Extract frontmatter (id, title, status, scope, target, created, tier)
- Generate canonical BACKLOG.md table sorted by ID
- Auto-compute Stats section
- Write atomically via tempfile + rename

## Impact

- **Reliability**: 100% accurate ID generation (LLM occasionally miscounts)
- **Speed**: < 50ms vs 5-10s LLM round-trip
- **Agent simplification**: backlog-curator.md loses ~30% of its instructions

## Research Context

Praetorian architecture principle: "Index files are derived data — generate, don't maintain."
CodeRabbit pattern: deterministic file scanning before LLM review.
