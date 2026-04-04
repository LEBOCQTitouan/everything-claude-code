# Spec: Fix worktree merge — add checkout main + working tree verification

## Problem Statement

The `ecc-workflow merge` command's `merge_fast_forward()` function runs `git merge --ff-only` from `repo_root` but never explicitly checks out `main` first. If the main repo's HEAD is on a different branch or in detached HEAD state, the ff-only merge targets the wrong ref. This was masked because the main repo usually has `main` checked out, but concurrent sessions or manual git operations can leave it in a different state. Additionally, no test verifies that working tree files in the main repo reflect the merged content, making this class of bug invisible to the test suite. The April 2nd incident (stale `deploy.rs` after merge) was triggered by manual `git update-ref` -- but the lack of defensive checkout and working tree verification means similar issues could recur.

## Research Summary

- Web research skipped: no search tool available
- `git merge --ff-only` updates both ref AND working tree (unlike `git update-ref` which only updates ref)
- `git checkout main` before merge is the standard defensive pattern for automation scripts
- The existing `ff_merge` test manually checks out main before calling `merge_fast_forward` -- proving the step is needed but not enforced in production code
- No prior audit findings related to this

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Add checkout_main() step in execute_merge | Defensive -- ensures ff-merge always targets main | No |
| 2 | Add integration test for working tree content | Prevents regression -- verifies files match after merge | No |
| 3 | Add CLAUDE.md gotcha for git update-ref | Prevents manual misuse by Claude agents | No |

## User Stories

### US-001: Harden merge with explicit checkout main

**As a** developer, **I want** `ecc-workflow merge` to explicitly checkout main before the ff-only merge, **so that** the merge always targets the correct branch regardless of the main repo's current HEAD state.

#### Acceptance Criteria

- AC-001.1: Given the main repo has `main` checked out, when `ecc-workflow merge` runs, then the merge succeeds and working tree files are updated
- AC-001.2: Given the main repo is in detached HEAD, when `ecc-workflow merge` runs, then it first checks out main, then merges successfully
- AC-001.3: Given the main repo is on a different branch, when `ecc-workflow merge` runs, then it first checks out main, then merges successfully
- AC-001.4: Given `git checkout main` fails (e.g., dirty working tree in main repo), when `ecc-workflow merge` runs, then it reports the error clearly and preserves the worktree

#### Dependencies

- Depends on: none

### US-002: Add working tree content verification test

**As a** developer, **I want** an integration test that verifies working tree files match merged content, **so that** stale-file bugs are caught by the test suite.

#### Acceptance Criteria

- AC-002.1: Given a test that creates a temp repo, adds a worktree, makes commits in the worktree, and merges, when the merge completes, then the file content in the main repo's working tree matches the worktree's committed content
- AC-002.2: Given the test, when the merge succeeds, then `git status` shows clean working tree (no modified files)

#### Dependencies

- Depends on: US-001

### US-003: Document anti-pattern in CLAUDE.md

**As a** developer, **I want** CLAUDE.md to warn against using `git update-ref` for merges, **so that** Claude agents always use `ecc-workflow merge` instead.

#### Acceptance Criteria

- AC-003.1: Given CLAUDE.md Gotchas, when inspected, then an entry warns against `git update-ref` and directs to `ecc-workflow merge`

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| crates/ecc-workflow/src/commands/merge.rs | Standalone binary | Add checkout_main() step in execute_merge |
| crates/ecc-workflow tests | Test | Add integration test for working tree content |
| CLAUDE.md | Documentation | Add gotcha |
| CHANGELOG.md | Documentation | Add fix entry |

## Constraints

- ecc-workflow is a standalone binary -- no hex boundary crossings
- Must not break existing merge tests (7 unit tests)
- checkout_main() must handle dirty working tree gracefully
- Integration test needs a temp git repo with worktree

## Non-Requirements

- Refactoring the existing worktree.rs app module
- Changing how EnterWorktree/ExitWorktree tools work
- Modifying the /implement command's merge instructions

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | N/A | Standalone binary, no hex boundaries |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Gotcha | Project | CLAUDE.md | Add update-ref warning |
| CHANGELOG | Project | CHANGELOG.md | Add fix entry |

## Open Questions

None -- all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Root cause | Both: harden ECC code AND prevent manual misuse | User |
| 2 | Fix approach | Proper structural fix (checkout_main + integration test + CLAUDE.md) | Recommended |
| 3 | Tests | Add integration test verifying file content after merge | Recommended |
| 4 | Regression | Low, contained to ecc-workflow | Recommended |
| 5 | Audit | No prior findings | Recommended |
| 6 | Repro | Checkout different branch in main repo, then merge from worktree | Recommended |
| 7 | Data | No data impact | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Harden merge with checkout main | 4 | none |
| US-002 | Working tree content verification test | 2 | US-001 |
| US-003 | Document anti-pattern | 1 | none |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Main checked out: merge succeeds + working tree updated | US-001 |
| AC-001.2 | Detached HEAD: checkout main first, then merge | US-001 |
| AC-001.3 | Different branch: checkout main first, then merge | US-001 |
| AC-001.4 | Checkout fails: error reported, worktree preserved | US-001 |
| AC-002.1 | File content matches after merge | US-002 |
| AC-002.2 | git status clean after merge | US-002 |
| AC-003.1 | CLAUDE.md warns against git update-ref | US-003 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-04-fix-worktree-merge-checkout-main/spec.md | Full spec + Phase Summary |
