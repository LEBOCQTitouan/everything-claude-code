# Spec Preview: Comms Pipeline Slash Commands

## How This Spec Was Understood

The user wants two slash commands wrapping the existing comms-generator agent: `/comms` for infrastructure management (init, strategy editing, draft lifecycle, calendar) and `/comms-generate` for content generation with optional channel filtering. Both are Markdown command files — no Rust code. Follows the `/backlog` pattern with subcommand tables.

## Spec Draft

# Spec: Comms Pipeline Slash Commands

## Problem Statement

BL-109 created the comms-generator agent and 3 supporting skills, but users must invoke the agent directly with natural language. There are no structured slash commands for common comms operations (initialize, edit strategies, manage drafts, generate content). This friction reduces adoption — developers need quick, discoverable commands like `/comms init` and `/comms-generate social` to integrate comms into their workflow.

## Research Summary

- **Subcommand-based architecture**: modern CLI tools use discrete subcommands (like `/backlog add|list|promote`) — maps cleanly to `/comms` subcommands
- **Orchestration as pauseable pipelines**: design commands as composable stages with persistent state between invocations
- **Anti-pattern**: tight coupling between generation and dispatch — externalize command definitions from agent logic
- **Pattern**: bare command with no args shows status overview (like `git status`)

## Decisions Made

| # | Decision | Rationale | ADR? |
|---|----------|-----------|------|
| 1 | Single comms.md with subcommand table | Follows /backlog pattern | No |
| 2 | /comms-generate as separate command | Generation is distinct from management | No |
| 3 | "finalize" instead of "publish" | No auto-publish in MVP — status change only | No |
| 4 | /comms init delegates to agent scaffold | Single implementation, no drift | No |
| 5 | Commits go to comms repo only | Main repo gitignores comms/ | No |
| 6 | Bare /comms shows status overview | Discoverable entry point | No |
| 7 | No Plan Mode for /comms-generate | Quick generation, not a pipeline | No |

## User Stories

### US-001: /comms-generate Command

**As a** developer, **I want** `/comms-generate [channels...]` to generate content across specified channels, **so that** I can produce DevRel drafts with a single command.

#### Acceptance Criteria

- AC-001.1: Given a comms repo exists, when `/comms-generate` runs with no args, then all 4 channels generate drafts
- AC-001.2: Given channel args (e.g., `/comms-generate social blog`), when run, then only specified channels generate
- AC-001.3: Given no comms repo exists, when run, then scaffolding happens first (delegates to agent Phase 1)
- AC-001.4: Given redaction finds CRITICAL patterns, when generation completes, then output is blocked
- AC-001.5: Given the command file, when inspected, then it has proper frontmatter (description, allowed-tools)

#### Dependencies
- None

### US-002: /comms Command — Status Overview

**As a** developer, **I want** bare `/comms` to show a status overview, **so that** I can see comms repo state at a glance.

#### Acceptance Criteria

- AC-002.1: Given a comms repo exists, when `/comms` runs with no subcommand, then it shows: repo path, active channels, draft counts by status, last generation date
- AC-002.2: Given no comms repo exists, when `/comms` runs, then it suggests `/comms init`

#### Dependencies
- None

### US-003: /comms init Subcommand

**As a** developer, **I want** `/comms init` to scaffold the comms infrastructure, **so that** I'm ready to generate content.

#### Acceptance Criteria

- AC-003.1: Given no comms directory, when `/comms init` runs, then full structure is created (strategies/, drafts/{channel}/, CALENDAR.md)
- AC-003.2: Given comms directory already exists, when `/comms init` runs, then user is warned and asked to overwrite or skip
- AC-003.3: Given init completes, when checking strategies/, then default strategy files exist for all 4 channels

#### Dependencies
- None

### US-004: /comms strategy Subcommand

**As a** developer, **I want** `/comms strategy <channel>` to view and edit a channel's strategy, **so that** I can customize content generation parameters.

#### Acceptance Criteria

- AC-004.1: Given a valid channel, when `/comms strategy social` runs, then the strategy file is displayed and the user can edit interactively
- AC-004.2: Given an invalid channel, when run, then an error lists valid channels
- AC-004.3: Given edits are made, when completed, then changes are committed to the comms repo

#### Dependencies
- Depends on: US-003

### US-005: /comms drafts Subcommand

**As a** developer, **I want** `/comms drafts [list|approve|finalize]` to manage generated content, **so that** I have a review workflow for drafts.

#### Acceptance Criteria

- AC-005.1: Given drafts exist, when `/comms drafts` or `/comms drafts list` runs, then a table shows date, channel, title, status
- AC-005.2: Given a draft in "draft" status, when `/comms drafts approve <file>` runs, then status updates to "approved"
- AC-005.3: Given an approved draft, when `/comms drafts finalize <file>` runs, then status updates to "published" and CALENDAR.md updates
- AC-005.4: Given a non-approved draft, when `/comms drafts finalize <file>` runs, then it blocks with "Must approve first"
- AC-005.5: Given no drafts exist, when listing, then "No drafts found. Run /comms-generate first."

#### Dependencies
- Depends on: US-001

### US-006: /comms calendar Subcommand

**As a** developer, **I want** `/comms calendar` to view the content calendar, **so that** I can track what's been generated and published.

#### Acceptance Criteria

- AC-006.1: Given CALENDAR.md has entries, when `/comms calendar` runs, then entries are displayed grouped by date
- AC-006.2: Given no CALENDAR.md, when run, then suggests `/comms init`

#### Dependencies
- Depends on: US-003

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| `commands/comms.md` | Command | Create |
| `commands/comms-generate.md` | Command | Create |
| `docs/commands-reference.md` | Docs | Modify |

## Constraints

- No Rust code — Markdown commands only
- Delegates to existing comms-generator agent
- No Plan Mode for generation
- Commits to comms repo, not main repo
- Follows /backlog command pattern

## Non-Requirements

- External API adapters (Buffer, Typefully, etc.)
- Scheduled/automated generation
- Rust CLI subcommands
- Plan Mode for /comms-generate

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | No Rust changes | No E2E tests |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New commands | docs/commands-reference.md | Update | Add /comms and /comms-generate entries |
| Changelog | CHANGELOG.md | Append | Add comms commands entry |

## Open Questions

None.

## Doc Preview

### CLAUDE.md changes
No changes needed — commands are auto-discovered.

### Other doc changes
- commands-reference.md: add /comms and /comms-generate to side commands table
- CHANGELOG: minor entry under v5.1.0
