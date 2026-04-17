---
description: Manage comms pipeline infrastructure — init repo, edit strategies, manage drafts, view calendar.
allowed-tools: [Read, Write, Edit, Bash, Grep, Glob, AskUserQuestion, TodoWrite]
---

# /comms

Manage comms pipeline. Bare `/comms` shows status. Subcommands for targeted operations.

## Subcommands

| Subcommand | Usage | Description |
|------------|-------|-------------|
| (none) | `/comms` | Status overview |
| `init` | `/comms init` | Scaffold comms directory |
| `strategy` | `/comms strategy <channel>` | View/edit channel strategy |
| `drafts` | `/comms drafts [list\|approve\|finalize]` | Draft lifecycle |
| `calendar` | `/comms calendar` | View calendar |

## `/comms` — Status Overview

Check `comms/` exists. If not: "Run /comms init." If exists: show repo path, active channels, draft counts by status, last generation date.

## `/comms init` — Scaffold

> **Tracking**: TodoWrite checklist. If unavailable, proceed without tracking.

1. Check if `comms/` exists. If yes, ask overwrite/skip.
2. Create: `comms/{strategies/{social,blog,devblog,docs-site}.md, drafts/{social,blog,devblog,docs-site}/, CALENDAR.md}`
3. Write default strategy files (channel name, audience, tone, constraints)
4. Init git repo in `comms/`
5. Commit: `chore: scaffold comms repo`

## `/comms strategy <channel>` — View/Edit

Channels: `social`, `blog`, `devblog`, `docs-site`. Invalid → error. Display strategy, ask to edit, write back, commit: `docs(strategy): update <channel> strategy`.

## `/comms drafts` — Lifecycle

### `list` (default)
Scan `comms/drafts/` recursively. Table: Date, Channel, Title, Status, Path.

### `approve <file>`
Validate path. Update `status: draft` → `approved`. Commit: `docs(drafts): approve <file>`.

### `finalize <file>`
Must be `approved`. Update → `published`. Update CALENDAR.md. Commit: `docs(drafts): finalize <file>`.

## `/comms calendar`

Read and display `comms/CALENDAR.md` grouped by date. If absent: "Run /comms init."

## Commit Rules

All commits to comms repo (`git -C comms/`). Read-only subcommands MUST NOT commit. Mutating subcommands MUST commit immediately.
