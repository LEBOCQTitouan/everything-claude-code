# Spec: Wave-based parallel TDD execution (BL-032)

## Problem Statement

The `/implement` command dispatches PCs strictly sequentially — one at a time. For implementations with many independent PCs (e.g., "add TodoWrite to 14 agents"), this wastes time since PCs that touch different files have no dependencies and could run concurrently. A wave-based parallel execution model would group independent PCs into waves and dispatch them concurrently, reducing implementation time proportionally to the parallelism factor.

## Research Summary

- Kahn's algorithm (BFS topological sort) naturally produces wave levels for DAG-based task scheduling
- Git is single-writer (index.lock); parallel branches with post-wave merge is the standard concurrent commit pattern
- Claude Code's `isolation: "worktree"` on Agent calls provides git worktree isolation per subagent — worktree is a valid repo checkout, subagent operates as if at repo root
- Wave-boundary regression verification reduces total regression runs vs per-PC verification
- Max concurrency caps (4 parallel) prevent resource exhaustion in CI-like environments

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Worktree isolation + sequential merge | Each parallel subagent gets `isolation: "worktree"`. Parent merges branches in PC-ID order after wave. Avoids index.lock contention. | Yes |
| 2 | Adjacent + no file overlap for wave grouping | Scan left-to-right through Test Strategy order. Start a wave, add next PC if no file overlap with any PC in the wave. If overlap, close wave and start new one. Respects author's ordering. | No |
| 3 | Max 4 concurrent subagents per wave | Hardcoded cap. Larger waves split into sub-batches of 4. | No |
| 4 | Let wave finish on failure, stop after | Independent PCs' work is valid. Don't waste compute. Re-dispatch only the failed PC on re-entry. | No |
| 5 | Wave-boundary regression verification | Run all prior PC commands after each wave completes, not per-PC. | No |
| 6 | tdd-executor agent unchanged | ADR 0007 confirms. Worktree isolation is transparent — tdd-executor runs in a valid repo checkout. | No |
| 7 | Git tags at wave boundaries for rollback | Create `wave-N-start` tag before each wave for recovery. | No |

## User Stories

### US-001: PC Dependency Graph + Wave Grouping

**As a** developer using `/implement`, **I want** the orchestrator to analyze dependencies between PCs and group independent ones into parallel waves, **so that** I can see the parallelism plan before execution begins.

#### Acceptance Criteria

- AC-001.1: Given parsed PCs with "Files to Modify" lists, when Phase 2 completes, then a dependency graph is computed where two PCs are dependent if they modify any overlapping file.
- AC-001.2: Given the Test Strategy ordered list, when wave grouping runs, then it scans left-to-right: starts a wave, adds the next PC if it shares no files with any PC already in the wave; if overlap, closes the wave and starts a new one.
- AC-001.3: Given PCs that share files, when wave grouping runs, then they are placed in separate sequential waves.
- AC-001.4: Given a wave with more than 4 PCs, when wave grouping runs, then the wave is split into sub-batches of max 4.
- AC-001.5: Given the wave plan, when displayed to the user, then each wave shows its PC IDs, files affected, and parallelism factor (e.g., "Wave 1: PC-003, PC-004, PC-005 [3 parallel]").
- AC-001.6: Given all PCs touch the same file (degenerate case), when wave grouping runs, then each PC gets its own wave (fully sequential — wave machinery skipped).
- AC-001.7: Given all PCs are independent, when wave grouping runs, then they form one wave (split into sub-batches of 4 if > 4).
- AC-001.8: Given a single-PC implementation, when wave grouping runs, then it produces one wave with one PC (no overhead).

#### Dependencies

- Depends on: none

### US-002: Parallel Subagent Dispatch With Worktree Isolation

**As a** developer using `/implement`, **I want** independent PCs within a wave dispatched as concurrent subagents in isolated git worktrees, **so that** implementation time is reduced without git lock contention.

#### Acceptance Criteria

- AC-002.1: Given a wave with N PCs (N <= 4), when the wave is dispatched, then N tdd-executor subagents are launched concurrently with `isolation: "worktree"` on each Agent call.
- AC-002.2: Given parallel subagents in worktrees, when all complete, then the parent merges their worktree branches sequentially in PC-ID order into the main branch.
- AC-002.3: Given a wave with a single PC, when dispatched, then it behaves identically to current sequential dispatch (no worktree, backward compatible).
- AC-002.4: Given parallel subagents, when building context briefs, then "Prior PC Results" includes only PCs from completed prior waves (not same-wave PCs).
- AC-002.5: Given a merge conflict during sequential merge, when detected, then the parent STOPS and reports the conflicting PCs, files, and suggests the user resolve manually.
- AC-002.6: Given worktree creation failure (disk space, git error), when detected, then the parent STOPS and reports the error with remediation guidance.

#### Dependencies

- Depends on: US-001

### US-003: Wave-Level Regression Verification

**As a** developer using `/implement`, **I want** regression verification to run after each wave completes instead of after each PC, **so that** parallel PCs are verified collectively.

#### Acceptance Criteria

- AC-003.1: Given wave W completes and all branches are merged, when regression runs, then ALL PC commands from waves 1..W are executed.
- AC-003.2: Given regression passes after wave W, when progress updates, then all PCs in wave W are marked complete.
- AC-003.3: Given regression fails after wave W, when reported, then the error identifies the failing PC command AND lists all wave-W PCs as potential culprits.
- AC-003.4: Given the first wave, when regression runs, then only the wave's own PCs are verified.

#### Dependencies

- Depends on: US-002

### US-004: Wave-Aware Failure Handling and Re-Entry

**As a** developer using `/implement`, **I want** wave failures to let remaining PCs finish and re-entry to handle partial wave completion, **so that** valid work is preserved.

#### Acceptance Criteria

- AC-004.1: Given one PC in a wave fails, when other PCs in the wave finish successfully, then their worktree branches are merged. The failed PC's branch is discarded.
- AC-004.2: Given a wave with a failed PC, when the wave finishes, then the orchestrator STOPS and reports the failure. Does not proceed to the next wave.
- AC-004.3: Given re-entry after a wave failure, when tasks.md is parsed, then the orchestrator re-derives the wave plan, skips completed PCs (marked `[x]`), and re-dispatches only the failed/incomplete PCs.
- AC-004.4: Given a git tag `wave-N-start` exists, when the user needs to rollback wave N, then `git reset --hard wave-N-start` restores the pre-wave state.
- AC-004.5: Given worktrees from a completed or failed wave, when the wave finishes, then Claude Code's automatic worktree cleanup removes them.

#### Dependencies

- Depends on: US-003

### US-005: Wave-Aware Progress Tracking

**As a** developer using `/implement`, **I want** tasks.md, TodoWrite, and implement-done.md to correctly reflect wave-based execution, **so that** progress reporting and documentation are accurate.

#### Acceptance Criteria

- AC-005.1: Given parallel PCs dispatched in a wave, when the wave launches, then tasks.md is updated with `red@<timestamp>` for all PCs in the wave.
- AC-005.2: Given all PCs in a wave complete and regression passes, when progress updates, then all are marked `done@<timestamp>` with `[x]`. Parent updates sequentially — no concurrent writes.
- AC-005.3: Given implement-done.md, when written after wave-based execution, then the TDD Log and Subagent Execution tables include a Wave column.
- AC-005.4: Given fully sequential execution (all waves have 1 PC), when implement-done.md is written, then output is backward compatible (no Wave column).

#### Dependencies

- Depends on: US-001

### US-006: Documentation

**As a** developer, **I want** the wave-based execution model documented.

#### Acceptance Criteria

- AC-006.1: Given `docs/adr/0012-wave-parallel-tdd.md` exists, when read, then it documents the worktree isolation strategy, wave grouping algorithm, failure handling, and rollback with Status/Context/Decision/Consequences.
- AC-006.2: Given `docs/domain/glossary.md`, when read, then it includes "Wave" and "Wave Plan" definitions.
- AC-006.3: Given `CHANGELOG.md`, when read, then it includes a BL-032 feature entry.

#### Dependencies

- Depends on: US-001, US-002, US-003, US-004, US-005

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| commands/implement.md | Content (command) | Modify — Phase 2 wave analysis, Phase 3 parallel dispatch + merge, regression, progress tracking, Phase 7 implement-done schema |
| docs/adr/0012-wave-parallel-tdd.md | Documentation | New — ADR |
| docs/domain/glossary.md | Documentation | Modify — add Wave + Wave Plan |
| CHANGELOG.md | Documentation | Modify — add BL-032 entry |

No Rust code changes. No tdd-executor agent changes.

## Constraints

- tdd-executor agent unchanged (ADR 0007)
- Max 4 concurrent subagents per wave (hardcoded constant, not magic number)
- Wave grouping: adjacent in Test Strategy + no file overlap
- `isolation: "worktree"` on Agent calls for parallel PCs
- Merge conflicts = STOP, not auto-resolution
- Backward compatible — single-PC waves = current sequential behavior
- Parent owns all progress tracking updates (sequential after wave, no concurrent writes)
- Git tags at wave boundaries (`wave-N-start`) for rollback
- implement.md stays under 800 lines

## Non-Requirements

- No tdd-executor agent changes
- No automatic merge conflict resolution
- No wave-based execution for /design or /spec commands
- No user-configurable concurrency setting
- No explicit sequential annotations in PC format
- No dynamic file generation detection (static file list only)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | — | No E2E boundaries crossed — pure command file content changes |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Feature | architecture | docs/adr/ | ADR 0012 for wave-parallel TDD |
| Term | domain | docs/domain/glossary.md | Add Wave + Wave Plan |
| Feature | project | CHANGELOG.md | Add BL-032 entry |

## Open Questions

None — all resolved during grill-me interview and adversarial review.
