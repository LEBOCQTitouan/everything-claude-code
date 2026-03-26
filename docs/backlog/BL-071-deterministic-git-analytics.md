---
id: BL-071
title: Deterministic git analytics CLI — changelog generation, hotspot analysis, evolution metrics
status: open
scope: MEDIUM
target: /spec dev
created: 2026-03-26
tags: [deterministic, git, changelog, evolution, rust-cli]
related: []
---

# BL-071: Deterministic Git Analytics CLI

## Problem

Three LLM-driven agents perform git history analysis that is fully mechanical:

1. **Changelog generation** (changelog-gen skill) — parses conventional commits, groups by type/version
2. **Hotspot analysis** (evolution-analyst agent) — counts file change frequency from git log
3. **Co-change coupling** (evolution-analyst) — identifies files that always change together

These are git log parsing + counting + sorting — zero LLM judgment needed.

## Proposed Solution

### `ecc analyze changelog [--since <tag|date>]`
- Parse `git log --format` output
- Group commits by conventional commit type (feat, fix, refactor, etc.)
- Format as markdown changelog
- LLM can optionally rewrite for human-friendliness (but structure is deterministic)

### `ecc analyze hotspots [--top N] [--since <date>]`
- `git log --name-only` → count file appearances
- Sort by frequency descending
- Output top N hotspots with change count

### `ecc analyze coupling [--threshold 0.7]`
- For each commit, record which files changed together
- Compute co-change frequency for file pairs
- Filter by threshold (default: 70%+ co-change rate)
- Output file pairs with coupling score

### Output
```
Hotspots (top 10, last 90 days):
  1. agents/backlog-curator.md  — 23 changes
  2. commands/implement.md      — 19 changes
  3. crates/ecc-app/src/lib.rs  — 17 changes

Coupling (>70%):
  agents/spec-adversary.md ↔ agents/solution-adversary.md  — 85%
  commands/spec-dev.md ↔ commands/spec-fix.md              — 78%
```

## Impact

- **Speed**: Full git analysis in < 2s vs 30-60s LLM agent
- **Reproducibility**: Same repo = same results, always
- **CI-ready**: Can run in CI for automated health reports

## Research Context

Praetorian: evolution analysis as deterministic tooling, not LLM task.
OpenHands: event sourcing enables deterministic replay and metrics.
