# Spec: Create /catchup session resumption command (BL-017)

## Problem Statement

When a developer returns to a Claude Code session after an interruption (new session, context compaction, or days later), there is no quick way to understand the current state: which workflow phase is active, how many PCs are done, what git work is uncommitted, and what happened in prior sessions. The developer must manually read `state.json`, parse `tasks.md`, run `git status`, and check memory files — a tedious process that delays productive work. A dedicated `/catchup` command would provide a structured summary in seconds.

## Research Summary

- Claude Code supports `--continue`/`--resume` for raw session restoration, but these restore conversation history — not workflow state summaries
- `claude-session-restore` (community tool) restores context from prior sessions via a skill, validating demand for this pattern
- Agent state management is a recognized gap — agents are fundamentally stateless per invocation; file-based state persistence is the standard solution
- BMAD Quick Flow track uses a similar "where am I?" pattern before deciding the next action
- Claude Code session lifecycle hooks (SessionStart) can auto-load context, complementing a manual `/catchup` command

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Read-only command (no state writes except user-initiated stale reset) | Follows /verify and /review observation pattern. Prevents accidental state corruption. | No |
| 2 | Include memory system integration | Memory files provide cross-session context that workflow state alone cannot. | No |
| 3 | Full bash test suite | BL-046 established the pattern. Mocking state.json and tasks.md for assertion-based testing. | No |
| 4 | Stale detection applies to all non-done phases | Not just `implement` — a stuck `plan` phase is equally stale. 1-hour threshold from last git commit on current branch. | No |
| 5 | Single command file, no Rust changes | Pure content-layer addition. No hexagonal boundary crossings. | No |
| 6 | Missing tasks.md reports path, does not attempt recovery | /catchup is read-only. Let /implement handle the 5-level fallback cascade. | No |
| 7 | Stale reset archives then deletes | Matches workflow-init.sh archive pattern. Preserves workflow history. | No |

## User Stories

### US-001: View Current Workflow State

**As a** developer resuming a Claude Code session, **I want** to see the current workflow phase, feature, and artifact progress at a glance, **so that** I know where I left off.

#### Acceptance Criteria

- AC-001.1: Given an active workflow (`state.json` exists with phase not `done`), when I run `/catchup`, then the output shows: current phase, feature description, concern type, started_at timestamp, and which artifacts (plan/solution/implement) have non-null timestamps in state.json.
- AC-001.2: Given an active workflow in `implement` phase with `artifacts.tasks_path` set and the tasks.md file existing, when I run `/catchup`, then the output shows: total Pass Conditions (PCs), completed (`[x]`), in-progress (latest status trail entry), failed (with error summary), and pending.
- AC-001.3: Given `artifacts.tasks_path` is set but the file does not exist, when I run `/catchup`, then the output reports "tasks.md not found at `<path>`" without erroring.
- AC-001.4: Given no active workflow (`state.json` missing), when I run `/catchup`, then the output states "No active workflow" and continues to git/memory sections.
- AC-001.5: Given a completed workflow (`state.json` phase is `done`), when I run `/catchup`, then the output states "Workflow complete: `<feature>`" with completion timestamp.
- AC-001.6: Given `artifacts.spec_path` and/or `artifacts.design_path` are set, when I run `/catchup`, then the output includes their file paths for reference.
- AC-001.7: Given `state.json` exists but contains malformed JSON, when I run `/catchup`, then the output warns "state.json is malformed — cannot read workflow state" and continues to git/memory sections.

#### Dependencies

- Depends on: none

### US-002: View Git State Summary

**As a** developer resuming a session, **I want** to see uncommitted changes, stashes, and worktrees, **so that** I know if there is in-flight work.

#### Acceptance Criteria

- AC-002.1: Given uncommitted changes exist, when I run `/catchup`, then the output shows count of modified, untracked, and staged files from `git status --short`.
- AC-002.2: Given stashed work exists, when I run `/catchup`, then the output lists each stash entry (index, branch, description).
- AC-002.3: Given multiple git worktrees exist, when I run `/catchup`, then the output lists all worktrees with paths and branches, flagging non-main ones.
- AC-002.4: Given clean git state (no changes, no stashes, single worktree), when I run `/catchup`, then the git section reports "Clean — no uncommitted work, no stashes, single worktree."
- AC-002.5: Given a repo with zero commits, when I run `/catchup`, then the recent commits section shows "No commits yet" instead of erroring.

#### Dependencies

- Depends on: none

### US-003: Detect and Offer to Resolve Stale Workflow State

**As a** developer returning after an interruption, **I want** to be warned if the workflow appears stale, **so that** I can decide to resume or reset.

#### Acceptance Criteria

- AC-003.1: Given an active workflow in any non-done phase where the most recent commit on the current branch (`git log -1 --format=%ci HEAD`) is older than 1 hour, when I run `/catchup`, then the output flags "STALE" and offers via `AskUserQuestion`: "Resume current workflow" and "Reset workflow state".
- AC-003.2: Given the user selects "Reset workflow state", when the reset executes, then `state.json` is archived to `.claude/workflow/archive/state-<timestamp>.json` and deleted from the active path.
- AC-003.3: Given the user selects "Resume current workflow", when `/catchup` continues, then no state modification occurs and the summary completes normally.
- AC-003.4: Given the last commit on the current branch is less than 1 hour old, when I run `/catchup`, then no staleness warning is shown.

#### Dependencies

- Depends on: US-001

### US-004: View Recent Activity from Memory System

**As a** developer resuming a session, **I want** to see recent actions from the memory system, **so that** I have cross-session context.

#### Acceptance Criteria

- AC-004.1: Given today's daily memory file exists at the project memory path (`memory/daily/YYYY-MM-DD.md`), when I run `/catchup`, then the output includes its Activity section entries.
- AC-004.2: Given no daily memory file exists for today but a previous day's exists, when I run `/catchup`, then the output includes the most recent daily file's Activity section with a "(from YYYY-MM-DD)" label.
- AC-004.3: Given no daily memory files exist, when I run `/catchup`, then the memory section states "No session history available."

#### Dependencies

- Depends on: none

### US-005: Command Definition and Documentation

**As a** developer, **I want** `/catchup` documented and discoverable, **so that** I can find it when I need it.

#### Acceptance Criteria

- AC-005.1: Given `commands/catchup.md` exists with valid YAML frontmatter, when Claude loads commands, then `/catchup` is available as a slash command.
- AC-005.2: The `allowed-tools` frontmatter is restricted to read-only tools: `[Bash, Read, Grep, Glob, LS, AskUserQuestion, TodoWrite]`. No Write/Edit/Task/Agent tools.
- AC-005.3: The output uses consistent section headers: `## Workflow State`, `## Tasks Progress`, `## Git Status`, `## Recent Activity`.
- AC-005.4: CLAUDE.md is updated to include `/catchup` in the side commands list.
- AC-005.5: `docs/domain/glossary.md` includes a "Catchup" term definition.

#### Dependencies

- Depends on: US-001, US-002, US-003, US-004

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `commands/catchup.md` | Driving adapter (command) | New file — session resumption command |
| `CLAUDE.md` | Documentation | Add /catchup to side commands list |
| `docs/domain/glossary.md` | Documentation | Add Catchup term |
| `docs/commands-reference.md` | Documentation | Add /catchup entry |
| `tests/hooks/test-catchup.sh` | Test (tooling) | New bash test suite |

No Rust crate changes. No hook changes. No state schema changes.

## Constraints

- Command is read-only — no Write/Edit tools in allowed-tools
- Must not call `workflow-init.sh` or `phase-transition.sh`
- Must handle missing/malformed state.json, missing tasks.md, missing memory files gracefully
- Staleness threshold: 1 hour since last git commit on current branch HEAD
- User-initiated archive+delete is the only state mutation (via AskUserQuestion confirmation)

## Non-Requirements

- Archived workflow history display — deferred for minimal scope
- Rust code changes — pure command file
- Per-intent /catchup variants — single command
- Context window usage monitoring (BL-035) — separate concern
- Auto-invocation on session start — manual command only
- tasks.md recovery/regeneration — let /implement handle that

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | — | No E2E boundaries crossed — read-only command file |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New command | project | CLAUDE.md | Add /catchup to side commands |
| New command | reference | docs/commands-reference.md | Add /catchup entry |
| Domain term | domain | docs/domain/glossary.md | Add Catchup definition |
| Feature | project | CHANGELOG.md | Add BL-017 feature entry |

## Open Questions

None — all resolved during grill-me interview and adversarial review.
