# Journey: Backlog Management

## Overview

**Actor:** Developer
**Goal:** Manage implementation ideas via the `ecc backlog` CLI and `/backlog` slash command
**Trigger:** Developer has an idea to capture, or wants to pick up work from the backlog

## Steps

1. **Capture idea** — `/backlog add <idea>` or `ecc backlog add-entry`
2. **Challenge & optimize** — backlog-curator agent runs grill-me interview, produces optimized prompt
3. **Persist** — Entry written to `docs/backlog/BL-NNN-<slug>.md` with atomic flock locking
4. **Index** — `docs/backlog/BACKLOG.md` updated with new row
5. **Pick up** — `/spec` presents backlog picker, filters in-progress items via worktree scan + lock files
6. **Lock** — Selected item gets `.locks/BL-NNN.lock` with worktree name + timestamp
7. **Execute** — Item flows through `/spec` → `/design` → `/implement` pipeline
8. **Release** — Lock removed at implement-done; status updated to `implemented`

## Key Components

| Layer | Component | Role |
|-------|-----------|------|
| Domain | `ecc-domain/src/backlog/lock.rs` | `LockFile` value object — ISO 8601 parsing, 24h staleness |
| Ports | `ecc-ports/src/backlog.rs` | `BacklogLockStore` trait — read/write/delete/exists |
| Infra | `ecc-infra/src/fs_backlog.rs` | `FsBacklogRepository` — filesystem implementation |
| App | `ecc-app/src/backlog.rs` | Use cases: `reindex`, `list_available`, `collect_claimed_ids` |
| CLI | `ecc-cli/src/commands/backlog.rs` | `next-id`, `check-duplicates`, `reindex`, `list` |
| Workflow | `ecc-workflow/src/commands/backlog.rs` | `add-entry` with POSIX flock atomicity |

## GAP Annotations

- InProgress detection relies on worktree directory naming convention (`bl-NNN` pattern) — fragile if naming changes
- Lock staleness TTL (24h) is hardcoded in domain layer — not configurable
