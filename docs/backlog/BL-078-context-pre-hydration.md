---
id: BL-078
title: Context pre-hydration via hook before command runs
scope: MEDIUM
target: /spec-dev
status: open
created: 2026-03-27
origin: Stripe Minions blog post — pre-hydration pattern
---

# BL-078: Context Pre-Hydration via Hook

## Problem

ECC commands (/spec, /implement, /design) waste early agent tokens on context discovery — reading files, fetching git history, checking project structure. Stripe's Minions solve this by "deterministically running relevant MCP tools over likely-looking links before a minion run even starts."

## Proposal

Create a PreToolUse hook that triggers before slash commands and pre-fetches likely-needed context based on the task description and command type:

- **For /spec-***: Pre-read CLAUDE.md, recent git log, relevant source files matching keywords in the task description, existing backlog entries
- **For /design**: Pre-read the spec artifact, ARCHITECTURE.md, bounded-contexts.md, relevant domain modules
- **For /implement**: Pre-read the design artifact, tasks.md, test files for affected modules, relevant source files from the design's file list

The hook outputs pre-fetched context as structured data that the command can consume immediately, skipping the discovery phase.

## Ready-to-Paste Prompt

```
/spec-dev Context pre-hydration hook for ECC commands

Create a hook-based context pre-hydration system that deterministically pre-fetches
likely-needed context before /spec, /design, and /implement commands run.

Requirements:
- PreToolUse hook that detects when a slash command is about to start
- Per-command-type context fetching strategy (spec reads backlog + git log,
  design reads spec artifact + arch docs, implement reads design + test files)
- Output as structured context block injected into the command's initial prompt
- Must complete in <5 seconds to avoid blocking the user
- Graceful degradation: if pre-fetch fails, command proceeds normally

Inspired by Stripe Minions' pre-hydration pattern where "deterministically running
relevant MCP tools over likely-looking links before a minion run even starts."
```

## Scope Estimate

MEDIUM — new hook + per-command context strategies + structured output format.

## Dependencies

- BL-052 (Rust hook binaries) would make this faster and more reliable
