# Spec: Auto-Commit Backlog Edits at End of /backlog Command (BL-059)

## Problem Statement

The `/backlog` command creates and modifies files under `docs/backlog/` but never commits them. After running `/backlog add`, the new entry file and updated `BACKLOG.md` are left uncommitted, requiring manual `git add` and `git commit`. This breaks the atomic commit convention and leaves backlog state in a dirty working tree. BL-059 adds auto-commit instructions to the three mutating subcommands (`add`, `promote`, `archive`).

## Research Summary

Web research skipped — this is a trivial command file edit with clear scope.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Add commit instructions to 3 mutating subcommands only | `list` and `match` are read-only, must not trigger commits | No |
| 2 | Use `git add docs/backlog/` (directory-scoped) | Prevents accidentally staging unrelated files | No |
| 3 | Commit message format: `docs(backlog): <action> BL-NNN [slug]` | Follows conventional commits, matches existing ECC patterns | No |

## User Stories

### US-001: Auto-commit backlog mutations

**As a** developer using `/backlog`, **I want** `add`, `promote`, and `archive` to auto-commit their changes, **so that** backlog edits are immediately tracked in git.

#### Acceptance Criteria

- AC-001.1: Given `/backlog add <idea>` completes, when git log is checked, then a commit exists with message `docs(backlog): add BL-NNN <slug>` containing only `docs/backlog/` files
- AC-001.2: Given `/backlog promote <id>` completes, when git log is checked, then a commit exists with message `docs(backlog): promote BL-NNN`
- AC-001.3: Given `/backlog archive <id>` completes, when git log is checked, then a commit exists with message `docs(backlog): archive BL-NNN`
- AC-001.4: Given `/backlog list` runs, when git log is checked, then no new commits are created
- AC-001.5: Given `/backlog match <text>` runs, when git log is checked, then no new commits are created
- AC-001.6: Given `commands/backlog.md`, when the `add` section is checked, then it contains a commit instruction block with `git add docs/backlog/` and the commit message format
- AC-001.7: Given `commands/backlog.md`, when the `promote` section is checked, then it contains a commit instruction block
- AC-001.8: Given `commands/backlog.md`, when the `archive` section is checked, then it contains a commit instruction block
- AC-001.9: Given the commit instruction, when read, then it uses "You MUST commit immediately" language (ECC convention)

#### Dependencies

- Depends on: none

### US-002: Documentation

**As a** maintainer, **I want** the CHANGELOG updated, **so that** the change is tracked.

#### Acceptance Criteria

- AC-002.1: Given `CHANGELOG.md`, when checked, then BL-059 entry exists

#### Dependencies

- Depends on: US-001

## Affected Modules

| Module | Layer | Nature of Change |
|--------|-------|-----------------|
| `commands/backlog.md` | Command | Modify: add commit instructions to 3 subcommands |
| `CHANGELOG.md` | Docs | Modify: add BL-059 entry |

## Constraints

- Only `commands/backlog.md` is modified (no Rust code, no hooks, no new files)
- `list` and `match` must NOT get commit instructions
- Commit message format follows conventional commits
- Uses "You MUST commit immediately" language per ECC convention

## Non-Requirements

- Modifying entry format, ID generation, or backlog logic
- Creating new commands or hooks
- Any Rust code changes
- Testing (markdown-only change, verified by grep PCs)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | No port/adapter changes | Pure command markdown modification |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Feature entry | Project | `CHANGELOG.md` | Add BL-059 entry |

## Open Questions

None.
