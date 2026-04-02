# Spec: Lazy Worktree Isolation with Session-End Merge

## Problem Statement

Currently, only pipeline commands (/spec, /design, /implement) create worktrees for session isolation. Direct editing sessions work on the main branch, risking accidental commits to main and making concurrent sessions unsafe. The `EnterWorktree` tool is a Claude Code agent tool (not a CLI command), so hooks cannot call it directly. A novel "write-guard" approach blocks write operations outside worktrees, forcing Claude to enter a worktree before making any changes, while a SessionEnd hook automatically merges work back to main.

## Research Summary

- Web research skipped: no search tool available.
- `EnterWorktree` is a Claude Code agent tool, not callable from shell hooks
- Existing `ecc-workflow merge` handles rebase + verify + ff-only merge + cleanup
- BL-065 (concurrent session safety) provides the worktree infrastructure foundation
- BL-085 documented that WorktreeCreate hooks break EnterWorktree -- resolved by using PostToolUse matchers
- The `ECC_WORKFLOW_BYPASS=1` env var is the standard escape hatch for all ECC hooks
- PreToolUse hooks can block (exit 2) with a message that Claude sees and acts on

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Lazy worktree via PreToolUse write-guard | Hooks can't call agent tools; blocking writes forces Claude to call EnterWorktree naturally. Avoids overhead for read-only sessions. | Yes |
| 2 | Merge at SessionEnd, not Stop | Stop fires after every response; SessionEnd fires once at close. Merge is heavyweight (build+test+clippy). | No |
| 3 | Full verify gate before merge (always) | Safety over speed. Broken code never reaches main. | No |
| 4 | Write-guard scoped to Write/Edit/MultiEdit only | Blocking Bash would kill read-only commands (cargo test, git log). Scoping to file-write tools is sufficient. | No |
| 5 | ECC_WORKFLOW_BYPASS skips the guard | Standard escape hatch pattern used by all existing hooks. Enables dev/debug sessions without worktree overhead. | No |

## User Stories

### US-001: PreToolUse Write-Guard Hook

**As a** developer, **I want** file-write operations blocked when not in a worktree, **so that** I never accidentally edit files on the main branch.

#### Acceptance Criteria

- AC-001.1: Given a session NOT in a worktree, when Claude attempts Write/Edit/MultiEdit, then the PreToolUse hook exits 2 with message "Not in a worktree. Call EnterWorktree before making changes."
- AC-001.2: Given a session IN a worktree, when Claude attempts Write/Edit/MultiEdit, then the hook exits 0 (allows the operation)
- AC-001.3: Given ECC_WORKFLOW_BYPASS=1, when Claude attempts Write outside a worktree, then the hook exits 0 (bypass)
- AC-001.4: Given a non-git directory, when Claude attempts Write, then the hook exits 0 (graceful degradation -- don't block non-git projects)
- AC-001.5: Given the hook fires, when checking worktree status, then it compares `git rev-parse --show-toplevel` against `git rev-parse --git-common-dir/..` -- if they differ, the session is in a worktree; verified by a test with a fixture worktree
- AC-001.6: Given a session in a worktree created by a pipeline command (/spec, /design, /implement), when Claude attempts Write/Edit/MultiEdit, then the write-guard exits 0 (no double-blocking)
- AC-001.7: Given `EnterWorktree` tool is unavailable or fails, when the write-guard blocks, then stderr includes a fallback message: "If EnterWorktree is unavailable, set ECC_WORKFLOW_BYPASS=1 to proceed on main"

#### Dependencies

- Depends on: none

### US-002: SessionEnd Merge Hook

**As a** developer, **I want** the worktree automatically merged back to main when my session ends, **so that** my work integrates without manual git operations.

#### Acceptance Criteria

- AC-002.1: Given a session in a worktree with commits, when SessionEnd fires, then `ecc-workflow merge` is called (rebase + verify + ff-only merge + cleanup)
- AC-002.2: Given a session NOT in a worktree, when SessionEnd fires, then the merge step is skipped silently
- AC-002.3: Given a merge fails (rebase conflict), when SessionEnd fires, then the worktree is preserved and stderr warns with remediation steps
- AC-002.4: Given the fast-verify fails (build/test/clippy), when SessionEnd fires, then the worktree is preserved and stderr warns
- AC-002.5: Given the merge lock is held by another session, when SessionEnd fires, then stderr warns to retry later
- AC-002.6: Given a worktree with no commits (empty), when SessionEnd fires, then the worktree is cleaned up without attempting merge
- AC-002.7: Given ECC_WORKFLOW_BYPASS=1, when SessionEnd fires, then the merge is skipped
- AC-002.8: Given SessionEnd fires while `ecc-workflow merge` is mid-execution (killed/interrupted), then the worktree is preserved and a `.ecc-merge-recovery` file is written to the worktree root with recovery instructions
- AC-002.9: Given `ecc-workflow merge` exit codes, then: exit 0 = success (AC-002.1), exit 1 = rebase conflict (AC-002.3), exit 2 = verify failure (AC-002.4), exit 3 = lock held (AC-002.5)

#### Dependencies

- Depends on: none (independent of US-001)

### US-003: Hook Configuration in settings.json

**As a** developer, **I want** the write-guard and merge hooks configured in the Claude Code settings, **so that** they activate automatically for all sessions.

#### Acceptance Criteria

- AC-003.1: Given `.claude/settings.json`, when inspected, then a PreToolUse hook entry exists matching Write|Edit|MultiEdit that runs the write-guard script
- AC-003.2: Given `.claude/settings.json`, when inspected, then a Stop hook entry exists that runs the session-end merge script
- AC-003.3: Given the hook scripts, when inspected, then they follow existing ECC hook conventions (set -uo pipefail, ECC_WORKFLOW_BYPASS check, atomic writes)
- AC-003.4: Given `ecc install`, when run, then the new hooks are installed alongside existing hooks
- AC-003.5: Given manual hook removal (deleting hook entries from settings.json), when subsequent sessions run, then writes to main are allowed without being blocked (clean uninstall path)

#### Dependencies

- Depends on: US-001, US-002

### US-004: Documentation and Glossary

**As a** developer, **I want** the write-guard and session merge behavior documented, **so that** I understand why my edits are blocked and how merging works.

#### Acceptance Criteria

- AC-004.1: Given CLAUDE.md, when inspected, then a Gotchas entry explains the write-guard
- AC-004.2: Given CLAUDE.md, when inspected, then a Gotchas entry explains session merge
- AC-004.3: Given the glossary or docs, when inspected, then "write-guard", "lazy worktree", "session merge" are defined
- AC-004.4: Given an ADR, when created, then it documents the lazy-worktree-via-write-guard pattern and the constraint that hooks can't call agent tools

#### Dependencies

- Depends on: US-001, US-002

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| hooks/ (new scripts) | Hook/Shell | New: worktree-write-guard.sh, session-end-merge.sh |
| .claude/settings.json | Configuration | Modify: add PreToolUse + Stop hook entries |
| hooks/hooks.json | Configuration | Modify: add new hook definitions |
| CLAUDE.md | Documentation | Modify: add Gotchas entries |
| docs/adr/ | Documentation | Create: ADR for lazy worktree pattern |

## Constraints

- EnterWorktree is an agent tool -- hooks CANNOT call it
- Write-guard must respect ECC_WORKFLOW_BYPASS
- Hook scripts must use `set -uo pipefail` and atomic writes (existing convention)
- SessionEnd hook timeout must accommodate full cargo build+test+clippy (~3 min)
- Write-guard must NOT block Bash tool (would kill read-only operations)
- Existing pipeline commands already call EnterWorktree in Phase 0 -- no conflict

## Non-Requirements

- Config flag for worktree mode (always/pipeline/off) -- defer to follow-up
- Lightweight merge path for non-code changes -- full verify always
- Handling session interruption (Ctrl+C) -- SessionEnd may not fire; user runs `ecc worktree gc` manually
- Worktree creation for read-only sessions (explicitly avoided by lazy approach)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Hook system / settings.json | New hook entries | Integration tests needed for write-guard and merge behavior |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Gotchas | Project | CLAUDE.md | Add write-guard and session merge entries |
| Glossary | Project | Glossary/docs | Define write-guard, lazy worktree, session merge |
| ADR | Project | docs/adr/ | ADR for lazy worktree pattern |
| CHANGELOG | Project | CHANGELOG.md | Add entry |

## Open Questions

None -- all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Scope | Lazy worktree (write-triggered) + merge hook. No config flag. | User |
| 2 | Guard behavior | Block (exit 2) with message; Claude calls EnterWorktree then retries | Recommended |
| 3 | Test strategy | Rust integration tests | User |
| 4 | Merge performance | Full verify always (build+test+clippy) | Recommended |
| 5 | Security | No implications | Recommended |
| 6 | Pipeline compat | Pipeline enters worktree in Phase 0; guard passes naturally | Recommended |
| 7 | Domain terms | Document in CLAUDE.md Gotchas AND glossary | User |
| 8 | ADR | ADR for lazy worktree write-guard pattern | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | PreToolUse Write-Guard Hook | 7 | none |
| US-002 | SessionEnd Merge Hook | 9 | none |
| US-003 | Hook Configuration in settings.json | 5 | US-001, US-002 |
| US-004 | Documentation and Glossary | 4 | US-001, US-002 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Block writes outside worktree (exit 2) | US-001 |
| AC-001.2 | Allow writes inside worktree (exit 0) | US-001 |
| AC-001.3 | ECC_WORKFLOW_BYPASS skips guard | US-001 |
| AC-001.4 | Non-git directory passes (graceful) | US-001 |
| AC-001.5 | Worktree detection via git rev-parse | US-001 |
| AC-001.6 | Pipeline worktrees pass guard | US-001 |
| AC-001.7 | Fallback message when EnterWorktree unavailable | US-001 |
| AC-002.1 | SessionEnd calls ecc-workflow merge | US-002 |
| AC-002.2 | Skip merge when not in worktree | US-002 |
| AC-002.3 | Rebase conflict preserves worktree | US-002 |
| AC-002.4 | Verify failure preserves worktree | US-002 |
| AC-002.5 | Lock held warns to retry | US-002 |
| AC-002.6 | Empty worktree cleaned up | US-002 |
| AC-002.7 | Bypass skips merge | US-002 |
| AC-002.8 | Mid-merge interruption writes recovery file | US-002 |
| AC-002.9 | Exit code contract (0/1/2/3) | US-002 |
| AC-003.1 | PreToolUse hook in settings.json | US-003 |
| AC-003.2 | Stop hook in settings.json | US-003 |
| AC-003.3 | Hook conventions followed | US-003 |
| AC-003.4 | ecc install installs hooks | US-003 |
| AC-003.5 | Clean uninstall path | US-003 |
| AC-004.1 | CLAUDE.md write-guard gotcha | US-004 |
| AC-004.2 | CLAUDE.md session merge gotcha | US-004 |
| AC-004.3 | Glossary terms defined | US-004 |
| AC-004.4 | ADR for lazy worktree pattern | US-004 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Ambiguity | 82 | PASS | ACs concrete with detection logic |
| Edge Cases | 76 | PASS | Pipeline compat, mid-merge interruption, EnterWorktree unavailable covered |
| Scope | 80 | PASS | Non-requirements explicit |
| Dependencies | 77 | PASS | Exit code contract documented |
| Testability | 75 | PASS | Detection logic testable with fixture worktrees |
| Decisions | 80 | PASS | ADR for novel pattern |
| Rollback | 74 | PASS | Clean uninstall path, recovery file for interruption |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-02-lazy-worktree-isolation/spec.md | Full spec + Phase Summary |
