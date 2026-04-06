---
description: Capture, challenge, optimize, and manage implementation ideas in a persistent backlog.
allowed-tools: [Bash, Read, Write, Edit, Grep, Glob, AskUserQuestion]
---

# Backlog Command

> **MANDATORY**: Follow every phase exactly. Narrate per `skills/narrative-conventions/SKILL.md`.

Capture ideas outside `/spec` sessions. Challenge, optimize into ready-to-execute prompts, store in `docs/backlog/`.

## Subcommands

| Subcommand | Usage | Description |
|------------|-------|-------------|
| `add` | `/backlog add <idea>` | Capture + optimize (default) |
| `list` | `/backlog list` | Show entries with status |
| `promote` | `/backlog promote <id>` | Mark as promoted |
| `archive` | `/backlog archive <id>` | Mark as archived |
| `match` | `/backlog match <prompt>` | Cross-reference against open entries |

No subcommand = `add`.

## Add Workflow

1. **Challenge** via `grill-me` (backlog-mode). LOW/MEDIUM: max 3 stages, 2 q/stage. HIGH/EPIC: all 5 stages.
2. **Determine** target command + scope
3. **Optimize** into self-contained prompt
4. **Check duplicates** against open entries
5. **Persist** to `docs/backlog/BL-NNN-<slug>.md`
6. **Update** index at `docs/backlog/BACKLOG.md`
7. **Commit immediately**: `docs(backlog): add BL-NNN <slug>`

## Commit Rules

| Subcommand | Message |
|------------|---------|
| `add` | `docs(backlog): add BL-NNN <slug>` |
| `promote` | `docs(backlog): promote BL-NNN` |
| `archive` | `docs(backlog): archive BL-NNN` |

Mutating subcommands MUST commit immediately. Read-only (`list`, `match`) MUST NOT.

## Integration

- `/spec` cross-references backlog in Phase 0.25
- `prompt-optimizer` checks backlog in Phase 2.5

## Related Agents

- `backlog-curator` — implements all subcommand flows
