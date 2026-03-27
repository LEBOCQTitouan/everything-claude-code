# Spec: Add Worktree Display Segment to Statusline

## Problem Statement

The ECC statusline provides no visual feedback when Claude Code is running inside a git worktree. During `/implement` TDD waves, agents operate in isolated worktrees, but users cannot tell which worktree is active or what branch it tracks. Additionally, when inside a worktree, the branch name appears redundantly if both a branch segment and a worktree segment are shown.

## Research Summary

- The canonical worktree detection method compares `git rev-parse --git-dir` against `git rev-parse --git-common-dir` — when they differ, the CWD is inside a linked worktree
- For performance, avoid `git status` in statuslines; use `git rev-parse` commands which run in ~2ms vs ~80ms for `git status`
- Common display pattern: `[worktree:name] branch` or `🌳 name (branch)`
- Worktree name should use `basename $(git rev-parse --show-toplevel)` to handle subdirectories correctly
- Edge cases: detached HEAD (no branch), bare repos (`--git-common-dir` returns `.` or repo path), stale worktree entries (deleted directories)
- Cache worktree detection with the same TTL as branch detection to avoid redundant git calls

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Replace branch segment when in worktree | Avoids redundancy — worktree segment already includes branch | Yes (ADR-0018) |
| 2 | Use `basename $(git rev-parse --show-toplevel)` for name | Works correctly when CWD is a subdirectory of the worktree | No |
| 3 | Share existing branch cache file | Single git call, one file read, same 5s TTL — more efficient | No |
| 4 | Show `(detached)` for detached HEAD | Graceful degradation rather than hiding the segment entirely | No |
| 5 | Add Worktree to domain default field_order | Keeps domain model in sync with runtime behavior | No |
| 6 | Reconcile domain field_order with script order | Reduces confusion — domain should reflect actual rendering | No |

## User Stories

### US-001: Worktree Detection and Display

**As a** developer using ECC in a git worktree, **I want** the statusline to show which worktree I'm in, **so that** I can distinguish between worktree sessions and the main working tree.

#### Acceptance Criteria

- AC-001.1: Given the user is inside a git worktree, when the statusline renders, then a segment `🌳 <worktree-name> (<branch>)` appears
- AC-001.2: Given the user is NOT inside a git worktree (main working tree), when the statusline renders, then no worktree segment appears
- AC-001.3: Given the user is inside a git worktree, when the statusline renders, then the standalone branch segment (`⎇ main`) is NOT shown (replaced by worktree segment)
- AC-001.4: Given the user is in a subdirectory of a worktree, when the statusline renders, then the worktree name is the basename of the worktree root (not the subdirectory)

#### Dependencies

- None

### US-002: Edge Case Handling

**As a** developer, **I want** the worktree segment to handle edge cases gracefully, **so that** the statusline never breaks or shows misleading information.

#### Acceptance Criteria

- AC-002.1: Given the worktree is in detached HEAD state (no branch), when the statusline renders, then the segment shows `🌳 <worktree-name> (detached)`
- AC-002.2: Given the user is in a bare repository, when the statusline renders, then no worktree segment appears
- AC-002.3: Given git commands fail (not a git repo), when the statusline renders, then no worktree segment appears and no errors are emitted
- AC-002.4: Given a stale worktree entry (worktree directory was deleted but `.git/worktrees/` still references it), when the statusline renders, then no worktree segment appears

#### Dependencies

- Depends on US-001

### US-003: Caching and Performance

**As a** developer, **I want** worktree detection to be cached, **so that** the statusline remains responsive without repeated git calls.

#### Acceptance Criteria

- AC-003.1: Given worktree info was cached less than 5 seconds ago, when the statusline renders, then the cached value is used (no git calls)
- AC-003.2: Given the cache has expired (>5s), when the statusline renders, then fresh git commands are executed and cache is updated
- AC-003.3: Given worktree detection shares the branch cache file, when inside a worktree, then a single cache read provides both worktree name and branch info. Cache format: newline-delimited (`line1=branch`, `line2=worktree_name`; line2 absent when not in a worktree)
- AC-003.4: Given an existing cache file in the legacy format (single-line branch-only), when the statusline renders, then the script gracefully reads the branch from line1 and treats missing line2 as "not in a worktree" (backward compatible)

#### Dependencies

- Depends on US-001

### US-004: Truncation and Narrow Variant

**As a** developer with a narrow terminal, **I want** the worktree segment to truncate gracefully, **so that** essential statusline information is preserved.

#### Acceptance Criteria

- AC-004.1: Given terminal width is insufficient for all segments, when the worktree segment is dropped by priority-based truncation, then no worktree info appears (graceful degradation). Worktree has priority rank 4 (after Model, ContextBar, GitBranch/Worktree; before RateLimits)
- AC-004.2: Given terminal width is tight but not critically narrow, when the statusline tries narrow variants, then the worktree segment has a narrow form `🌳 <name>` (branch dropped)

#### Dependencies

- Depends on US-001

### US-005: Domain Model Update

**As a** maintainer, **I want** the domain `StatuslineConfig` default to include the Worktree field and match the script's actual rendering order, **so that** the domain model is authoritative.

#### Acceptance Criteria

- AC-005.1: Given `StatuslineConfig::default()` is constructed, then `StatuslineField::Worktree` appears in `field_order` after `GitBranch`
- AC-005.2: Given the domain `field_order` default, then the order matches the script's actual rendering priority: Model, ContextBar, GitBranch, Worktree, RateLimitFiveHour, RateLimitSevenDay, TokenCounts, LinesChanged, Duration, Cost, EccVersion
- AC-005.3: Given existing Rust tests for `StatuslineConfig`, then all tests still pass after the field_order update

#### Dependencies

- None (can be done in parallel with US-001)

### US-006: Bats Test Infrastructure

**As a** maintainer, **I want** Bats tests for the worktree statusline segment, **so that** behavior is verified and regressions are caught.

#### Acceptance Criteria

- AC-006.1: Given a Bats test file exists for the statusline, then worktree detection tests cover: in-worktree, not-in-worktree, subdirectory-of-worktree, detached HEAD, bare repo, git-not-available
- AC-006.2: Given worktree rendering tests exist, then they verify: segment format, branch replacement, narrow variant, truncation priority
- AC-006.3: Given cache tests exist, then they verify: cache hit (within TTL), cache miss (expired), shared cache with branch
- AC-006.4: Given `bats tests/statusline/` runs, then all tests pass and all worktree code paths enumerated in AC-006.1 through AC-006.3 are exercised
- AC-006.5: Given Bats tests run, then each test creates a disposable git repo in `setup()` and cleans it in `teardown()` — tests must not depend on the host repo state

#### Dependencies

- Depends on US-001, US-002, US-003, US-004

### US-007: ADR for Branch Replacement

**As a** maintainer, **I want** an ADR documenting the decision that the worktree segment replaces (not supplements) the branch segment, **so that** the rationale is preserved for future developers.

#### Acceptance Criteria

- AC-007.1: Given `docs/adr/0018-worktree-replaces-branch-segment.md` exists, then it documents the decision, context, and consequences in the standard ADR format

#### Dependencies

- None

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `statusline/statusline-command.sh` | Adapter (bash) | Add worktree detection, caching, segment rendering, branch replacement logic, narrow variant |
| `crates/ecc-domain/src/config/statusline.rs` | Domain | Update default `field_order` to include Worktree and reconcile with script order |
| `tests/statusline/` (new) | Test | Bats test infrastructure for statusline |
| `docs/adr/0018-*.md` (new) | Documentation | ADR for branch replacement decision |

## Constraints

- BL-076 (Unicode byte-counting bug) is out of scope — the worktree segment will be subject to the same bug
- No changes to the Claude Code JSON input contract — worktree detection uses local git commands only
- The `ecc-domain` crate must remain pure (no I/O) — only the default `field_order` vector changes

## Non-Requirements

- Fixing BL-076 (Unicode byte-counting)
- No performance benchmark target (git rev-parse is ~2ms)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Statusline bash script | New segment added | No new port needed — git detection is local to the script |
| Domain StatuslineConfig | Default config change | No E2E impact — config is internal |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New ADR | docs | docs/adr/0018-worktree-replaces-branch-segment.md | Create |
| Backlog update | docs | docs/backlog/BL-082 | Promote after implementation |

## Open Questions

None — all resolved during grill-me.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | What is explicitly out of scope? | BL-076 fix only; Bats tests, domain reconciliation, narrow variant are IN scope | User |
| 2 | Branch duplication handling? | Replace branch segment when in worktree | Recommended |
| 3 | Worktree name extraction method? | `basename $(git rev-parse --show-toplevel)` | Recommended |
| 4 | Cache strategy? | Shared cache with existing branch cache, same 5s TTL | Recommended |
| 5 | Test coverage target? | 100% on all worktree-related code paths | User |
| 6 | Edge case behavior? | Show `(detached)` for detached HEAD; hide for bare repos and stale entries | Recommended |
| 7 | Security / breaking changes? | No security impact, no breaking changes | Recommended |
| 8 | ADR needed? | Yes — ADR-0018 for branch replacement behavior | User |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Worktree Detection and Display | 4 | None |
| US-002 | Edge Case Handling | 4 | US-001 |
| US-003 | Caching and Performance | 4 | US-001 |
| US-004 | Truncation and Narrow Variant | 2 | US-001 |
| US-005 | Domain Model Update | 3 | None |
| US-006 | Bats Test Infrastructure | 5 | US-001, US-002, US-003, US-004 |
| US-007 | ADR for Branch Replacement | 1 | None |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Worktree segment `🌳 name (branch)` appears when in worktree | US-001 |
| AC-001.2 | No worktree segment when not in worktree | US-001 |
| AC-001.3 | Branch segment replaced by worktree segment | US-001 |
| AC-001.4 | Worktree name is basename of worktree root | US-001 |
| AC-002.1 | Detached HEAD shows `(detached)` | US-002 |
| AC-002.2 | Bare repo hides segment | US-002 |
| AC-002.3 | Git failure hides segment silently | US-002 |
| AC-002.4 | Stale worktree entry hides segment | US-002 |
| AC-003.1 | Cache hit within 5s TTL | US-003 |
| AC-003.2 | Cache miss refreshes git data | US-003 |
| AC-003.3 | Shared cache format: newline-delimited | US-003 |
| AC-003.4 | Legacy single-line cache backward compatible | US-003 |
| AC-004.1 | Truncation drops worktree at priority rank 4 | US-004 |
| AC-004.2 | Narrow variant `🌳 name` drops branch | US-004 |
| AC-005.1 | Worktree in domain field_order after GitBranch | US-005 |
| AC-005.2 | Domain order matches script rendering | US-005 |
| AC-005.3 | Existing Rust tests pass | US-005 |
| AC-006.1 | Detection tests: 6 scenarios covered | US-006 |
| AC-006.2 | Rendering tests: format, replacement, narrow, truncation | US-006 |
| AC-006.3 | Cache tests: hit, miss, shared | US-006 |
| AC-006.4 | All enumerated code paths exercised | US-006 |
| AC-006.5 | Test isolation with disposable git repos | US-006 |
| AC-007.1 | ADR-0018 documents branch replacement decision | US-007 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Ambiguity | 85 | PASS | Cache format and truncation priority now specified |
| Edge Cases | 80 | PASS | Stale worktree, detached HEAD, bare repo all covered |
| Scope | 85 | PASS | BL-076 explicitly excluded; field_order reorder justified |
| Dependencies | 88 | PASS | All deps identified; no circular deps |
| Testability | 90 | PASS | Coverage claim reworded; test isolation specified |
| Decisions | 85 | PASS | Cache migration documented; ADR flagged |
| Rollback | 90 | PASS | Additive change; cache self-heals via TTL |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-27-statusline-worktree-segment/spec.md | Full spec |
