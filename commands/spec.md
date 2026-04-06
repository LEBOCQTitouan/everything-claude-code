---
description: Spec any change — auto-classifies as dev/fix/refactor and delegates to the appropriate spec command.
allowed-tools: [Read, Grep, Glob, Bash, Skill, AskUserQuestion]
---

# Spec Router Command

> Thin router: classifies intent, delegates to `/spec-dev`, `/spec-fix`, or `/spec-refactor`. No Plan Mode. No state.json edits. Narrate per `skills/narrative-conventions/SKILL.md`.

## Phase 0 — Validate Input

### `--show-all` flag
Strip flag, skip in-work filtering.

### Active Session Detection

1. **Worktree scan**: `.claude/worktrees/`, extract BL-NNN IDs via `(?i)bl-?(\d{3})`.
2. **Lock file scan**: `docs/backlog/.locks/BL-NNN.lock`. Remove if >24h stale or orphaned.
3. **Merge**: `claimed_items` = worktree ∪ lock claims.
4. **Display**: Active sessions + hidden count.

### Backlog Picker

If `$ARGUMENTS` empty: read `docs/backlog/BACKLOG.md`, filter open entries (remove claimed unless `--show-all`). Present via AskUserQuestion (BL-NNN options). Selected: read optimized prompt, write lock file. "Other": use custom text. Fallback: ask user to describe.

## Phase 1 — Intent Classification

### Explicit flags
`--dev`→dev, `--fix`→fix, `--refactor`→refactor. Strip from args.

### Signal-word classification

| Intent | Signal Words |
|--------|-------------|
| **dev** | add, implement, create, new, feature, support, enable, introduce, build |
| **fix** | fix, bug, broken, error, crash, fails, wrong, regression |
| **refactor** | refactor, clean, restructure, extract, rename, move, simplify, decouple, split, merge |

Default: **dev**.

## Phase 2 — Confirm

> **Classified as: `<type>`** — delegating to `/spec-<type>`.

Low confidence → AskUserQuestion [dev, fix, refactor].

## Phase 3 — Delegate

`skill: "spec-<type>", args: "<$ARGUMENTS>"`. Fallback: `"plan-<type>"`. Both fail → instruct user to run directly.
