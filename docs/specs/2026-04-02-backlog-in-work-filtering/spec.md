# Spec: BL-097 Backlog In-Work Filtering for /spec Picker

## Problem Statement

When multiple Claude Code sessions run concurrently, the `/spec` command's backlog picker shows all open items without awareness of which items are already being worked on in other sessions. This causes accidental duplicate work and user confusion. The picker should detect active sessions, display them as context, and filter claimed items from the selection.

## Research Summary

Web research skipped -- this is an ECC-internal command behavior change with no external dependencies.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Hybrid detection: worktree scan + simple lock file | Worktree names often contain BL-NNN patterns; lock files cover cases where they don't | Yes |
| 2 | Content-only implementation (spec.md + implement.md command edits) | No Rust changes needed -- Claude Code executes the command logic directly | No |
| 3 | Display active sessions before picker, filter claimed items | User needs context about what's happening before choosing | No |

## User Stories

### US-001: Active Session Detection

**As a** developer using ECC, **I want** the /spec picker to detect which backlog items are being worked on in other sessions, **so that** I don't accidentally duplicate work.

#### Acceptance Criteria

- AC-001.1: Given active worktrees in `.claude/worktrees/`, when /spec runs with no arguments, then it scans worktree directory names using regex `(?i)bl-?(\d{3})` to extract all BL-NNN IDs (case-insensitive, handles `bl119`, `bl-119`, `BL-119`; a name like `feature-bl-042-and-bl-055` matches both BL-042 and BL-055)
- AC-001.2: Given matched BL-NNN patterns, when the picker renders, then matched items are excluded from the selectable options
- AC-001.3: Given active worktrees with no BL-NNN pattern (e.g., "dead-code-cleanup"), when the picker renders, then those worktrees are shown in the active sessions info but don't filter any items
- AC-001.4: Given lock files exist in `docs/backlog/.locks/BL-NNN.lock`, when the picker renders, then locked items are also excluded
- AC-001.5: Given `/spec --show-all` is passed, when the picker renders, then ALL open items are shown regardless of worktree or lock status (escape hatch for false-positive filtering)

#### Dependencies

- Depends on: none

### US-002: Active Sessions Display

**As a** developer, **I want** to see which sessions are currently active before choosing a backlog item, **so that** I understand the work landscape.

#### Acceptance Criteria

- AC-002.1: Given active worktrees exist, when /spec runs with no arguments, then it displays "Active sessions:" with worktree names and matched BL-NNN items before the AskUserQuestion picker
- AC-002.2: Given N items are filtered, when the picker renders, then it shows "(N items in progress, hidden)" below the active sessions info

#### Dependencies

- Depends on: none

### US-003: Lock File Claim

**As a** session running /spec, **I want** a lock file written when I claim a backlog item, **so that** other sessions can detect my claim even if my worktree name doesn't contain the BL-NNN.

#### Acceptance Criteria

- AC-003.1: Given the user selects a BL-NNN item from the picker, when the spec command proceeds, then a lock file is written to `docs/backlog/.locks/BL-NNN.lock` containing the worktree name and timestamp
- AC-003.2: Given /implement completes successfully (implement-done.md written), when the workflow finishes, then the lock file is removed
- AC-003.3: Given a lock file exists with timestamp > 24 hours old, when the picker runs, then the stale lock is automatically removed before filtering
- AC-003.4: Given a lock file exists but the corresponding worktree directory no longer exists in `.claude/worktrees/`, when the picker runs, then the orphaned lock is automatically removed (faster than 24h TTL)

#### Dependencies

- Depends on: US-001

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `commands/spec.md` | Content (command) | Add worktree scan + lock check + display to Phase 0 |
| `commands/implement.md` | Content (command) | Add lock release to Phase 7 |
| `docs/backlog/.locks/` | Content (new dir) | Lock files for claimed items |
| `.gitignore` | Config | Add `docs/backlog/.locks/` |

## Constraints

- No Rust code changes
- Lock files are advisory (text files, not POSIX locks)
- Stale lock TTL: 24 hours
- `.locks/` directory gitignored (session-local, not committed)
- Worktree scan uses directory listing only (no git commands)

## Non-Requirements

- No PID-based liveness detection
- No real-time notification when items become available
- No UI for manually releasing locks
- No Rust CLI subcommands for lock management
- No filtering for non-backlog worktrees (only BL-NNN matched ones)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| /spec command | Modified | Phase 0 picker behavior changes |
| /implement command | Modified | Lock release on completion |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New pattern | MEDIUM | docs/adr/ | ADR for session detection pattern |
| Changelog | LOW | CHANGELOG.md | Add entry |

## Open Questions

None -- all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Scope: full lock files vs worktree scan vs hybrid? | Hybrid: worktree scan + simple lock file | User |
| 2 | Edge case: worktrees without BL-NNN pattern? | Show in active sessions, don't filter | Recommended |
| 3 | Test strategy? | Integration test with mock worktrees | User |
| 4 | Performance concerns? | No concerns | Recommended |
| 5 | Breaking changes acceptable? | Yes, better UX | Recommended |
| 6 | ADR needed? | Yes, for session detection pattern | User |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Active Session Detection | 5 | none |
| US-002 | Active Sessions Display | 2 | none |
| US-003 | Lock File Claim | 4 | US-001 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Worktree scan with regex (?i)bl-?(\d{3}) | US-001 |
| AC-001.2 | Matched items excluded from picker | US-001 |
| AC-001.3 | Non-BL worktrees shown as info only | US-001 |
| AC-001.4 | Lock files also exclude items | US-001 |
| AC-001.5 | --show-all escape hatch | US-001 |
| AC-002.1 | Active sessions displayed before picker | US-002 |
| AC-002.2 | N items hidden count shown | US-002 |
| AC-003.1 | Lock file written on claim | US-003 |
| AC-003.2 | Lock released on /implement completion | US-003 |
| AC-003.3 | Stale lock cleanup (24h TTL) | US-003 |
| AC-003.4 | Orphaned lock cleanup (worktree deleted) | US-003 |

### Adversary Findings

| Dimension | R1 Score | R2 Score | Verdict |
|-----------|----------|----------|---------|
| Ambiguity | 72 | 92 | PASS |
| Edge Cases | 58 | 88 | PASS |
| Scope | 82 | 85 | PASS |
| Dependencies | 88 | 90 | PASS |
| Testability | 65 | 85 | PASS |
| Decisions | 78 | 88 | PASS |
| Rollback | 55 | 82 | PASS |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-02-backlog-in-work-filtering/spec.md` | Full spec + Phase Summary |
