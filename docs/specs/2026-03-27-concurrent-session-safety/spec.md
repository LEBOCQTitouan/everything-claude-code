# Spec: Concurrent Session Safety (BL-065)

## Problem Statement

Multiple concurrent Claude Code sessions in the same project directory corrupt shared state, lose data, and produce broken builds. The codebase audit (2026-03-26) found 9 race conditions across workflow state, memory files, backlog index, and cargo build artifacts. There is zero use of file locking anywhere, and no worktree isolation for pipeline sessions. Two `/implement` sessions running simultaneously will race on `target/`, `state.json`, and `action-log.json`.

## Research Summary

- POSIX `flock` is the standard advisory locking mechanism — available on Linux and macOS, no external dependencies
- Git worktrees provide isolated working directories with shared `.git` — ideal for parallel session isolation
- Rust `std::fs::File` + `flock` via `libc` crate or `fs2` crate provides cross-platform file locking
- `git rev-parse --git-common-dir` resolves the main repo root from inside a worktree — critical for lock path resolution
- Claude Code provides `EnterWorktree`/`ExitWorktree` tools for native worktree switching
- Merge serialization via advisory lock is a proven pattern (similar to `git gc --auto` lock)
- Atomic file operations (`mktemp+mv`) prevent partial writes but NOT read-modify-write races — flock is needed for the full cycle

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Split into 4 sub-specs (A/B/C/D) | EPIC scope too large for single pipeline | No |
| 2 | Lock only in Rust, not shell hooks | BL-052 replaces shell hooks; avoid double-implementing | No |
| 3 | Use EnterWorktree/ExitWorktree for session isolation | Native Claude Code tools, cleanest approach | No |
| 4 | Lock paths resolve to main repo root | Worktrees must share locks with main repo | No |
| 5 | state.json + .tdd-state worktree-local; memory + backlog target main repo | Correct isolation boundary | No |
| 6 | 60s merge lock timeout with user feedback | Prevents indefinite blocking | No |
| 7 | Concurrent session architecture | Worktree isolation + serialized merge + flock locking | Yes |
| 8 | ecc-workflow uses flock directly via libc (no port trait refactor) | ecc-workflow is a standalone binary; refactoring it to use ecc-ports is out of scope for BL-065 | No |
| 9 | Pipeline commands = /spec, /design, /implement, /backlog | Enumerated list; all others (/audit, /verify, /review, /commit, /catchup) are non-pipeline | No |
| 10 | One lock per memory file type (action-log, daily, MEMORY.md, work-item) | Fine-grained locking reduces contention; named locks: action-log.lock, daily.lock, memory-index.lock, work-item.lock | No |
| 11 | Non-merge locks have no timeout (block indefinitely; auto-release on crash) | flock auto-releases on process exit; non-merge operations are fast (<1s) so timeout adds unnecessary complexity | No |
| 12 | BL-052 is NOT a prerequisite — BL-065 is safe to implement independently | Shell hooks remain racey until BL-052 lands; Rust paths are safe after BL-065 | No |
| 13 | Lock files are zero-byte and permanent — no cleanup needed | flock advisory locks on empty files are harmless; .locks/ is gitignored | No |

## User Stories

### Sub-Spec A: Lock Infrastructure

#### US-001: FileLock Port Trait and flock Implementation

**As a** Rust crate needing exclusive file access, **I want** a `FileLock` port trait with POSIX flock implementation, **so that** shared file access can be serialized.

##### Acceptance Criteria

- AC-001.1: Given the `ecc-ports` crate, when a module needs exclusive file access, then it can use a `FileLock` trait with `acquire(repo_root: &Path, name: &str) -> Result<LockGuard, LockError>` and `acquire_with_timeout(repo_root: &Path, name: &str, timeout: Duration) -> Result<LockGuard, LockError>`. `LockGuard` implements `Drop` to release the lock automatically (RAII). No manual `release()` method exists on the trait.
- AC-001.2: Given two processes both acquiring the same lock, when they run concurrently, then one blocks until the other releases.
- AC-001.3: Given `.claude/workflow/.locks/` does not exist, when a lock is requested, then the directory is created automatically.
- AC-001.4: Given a process holds a lock via `LockGuard`, when the guard is dropped or the process crashes, then the lock is automatically released (RAII Drop + POSIX flock kernel release).
- AC-001.5: Given `ecc-infra`, then a standalone `resolve_repo_root(path: &Path) -> PathBuf` function exists that uses `git rev-parse --git-common-dir` with fallback to the input path. `FileLock::acquire` takes an absolute `&Path` — callers compose `resolve_repo_root` + `acquire`.
- AC-001.6: Given the `ecc-test-support` crate, when tests need a lock double, then an in-memory `FileLock` implementation is available.
- AC-001.7: Given the test suite, when lock contention is tested, then a multi-process integration test using `std::process::Command` (not fork) proves actual flock contention and ordering.
- AC-001.8: Given the `ecc-ports` crate, then a `LockError` enum exists with variants for I/O errors, directory creation failures, acquisition failures, and timeouts, and all lock operations return `Result<_, LockError>`.

##### Dependencies

- Depends on: none

### Sub-Spec B: Shared State Locking

#### US-002: action-log.json Locking (CRITICAL)

**As a** concurrent session, **I want** writes to action-log.json protected by FileLock, **so that** no entries are lost.

##### Acceptance Criteria

- AC-002.1: Given two sessions writing to action-log.json simultaneously, when both use the memory-write command, then both entries appear.
- AC-002.2: Given the action-log lock is held by session A, when session B writes, then session B blocks until released.

##### Dependencies

- Depends on: US-001

#### US-003: state.json TOCTOU Fix (HIGH)

**As a** pipeline session, **I want** all read-modify-write on state.json protected by FileLock, **so that** phase transitions are never lost.

##### Acceptance Criteria

- AC-003.1: Given two sessions both transitioning phases, when they modify state.json concurrently, then both transitions apply sequentially.
- AC-003.2: Given workflow-init archives and reinitializes state.json, when another session is mid-transition, then no data is lost.
- AC-003.3: Given a phase-gate reads state.json, when another session transitions between read and check, then the gate operates on post-transition state.

##### Dependencies

- Depends on: US-001

#### US-004: BACKLOG.md ID Race Fix (HIGH)

**As a** session running `/backlog add`, **I want** ID generation + file writes protected by FileLock, **so that** no duplicate IDs occur.

##### Acceptance Criteria

- AC-004.1: Given two sessions running `/backlog add` simultaneously, then they receive unique sequential IDs.
- AC-004.2: Given two sessions appending to BACKLOG.md, then both entries appear.

##### Dependencies

- Depends on: US-001

#### US-005: Memory File Locking (MEDIUM)

**As a** concurrent session, **I want** daily memory, MEMORY.md index, and work-item writes protected, **so that** no entries are lost.

##### Acceptance Criteria

- AC-005.1: Given two sessions writing daily memory entries concurrently, then both entries appear.
- AC-005.2: Given two sessions updating MEMORY.md index concurrently, then both updates appear.
- AC-005.3: Given two sessions creating the same work-item file, then the second appends a revision.
- AC-005.4: Given each memory file type, then it has its own dedicated lock file (daily.lock, memory-index.lock, work-item.lock).

##### Dependencies

- Depends on: US-001

### Sub-Spec C: Worktree Isolation + Lifecycle

#### US-006: Automatic Worktree Isolation

**As a** developer running a pipeline command, **I want** my session to auto-create a git worktree, **so that** my changes are isolated from other sessions.

##### Acceptance Criteria

- AC-006.1: Given a session enters a pipeline command, when workflow-init runs, then `EnterWorktree` is called to create and switch to a worktree.
- AC-006.2: Given a worktree is created, when the session performs file operations, then all writes happen in the worktree.
- AC-006.3: Given a non-pipeline command, when it runs, then no worktree is created.
- AC-006.4: Given state.json in the worktree, then it tracks the session's own phase independently.
- AC-006.5: Given memory/backlog writes in a worktree session, then they target the main repo root.
- AC-006.6: Given a worktree is created, then it is named `ecc-session-{timestamp}-{slug}` with PID suffix for collision avoidance.

##### Dependencies

- Depends on: none

#### US-007: Worktree Lifecycle Management

**As a** developer, **I want** worktrees auto-cleaned on success, preserved on failure, and manually cleanable via `ecc worktree gc`.

##### Acceptance Criteria

- AC-007.1: Given a pipeline completes and merges successfully, then the worktree + branch are deleted.
- AC-007.2: Given a session fails or is aborted, then the worktree is preserved with a warning.
- AC-007.3: Given stale worktrees exist, when `ecc worktree gc` runs, then they are identified and removed after confirmation.
- AC-007.4: Given `ecc worktree gc` lists worktrees, when one is still active, then it is skipped.

##### Dependencies

- Depends on: US-006

### Sub-Spec D: Serialized Merge

#### US-008: Serialized Merge-to-Main with Fast Verify

**As a** pipeline session that completed, **I want** my branch rebased, verified, and merged under exclusive lock, **so that** only passing code reaches main.

##### Acceptance Criteria

- AC-008.1: Given a session initiates merge, then it acquires `.locks/merge.lock` via FileLock.
- AC-008.2: Given the lock is held, then `git rebase main` runs in the worktree.
- AC-008.3: Given rebase succeeds, then `cargo build && cargo test && cargo clippy -- -D warnings` runs.
- AC-008.4: Given fast verify passes, then `git merge --ff-only` on main, worktree+branch deleted.
- AC-008.5: Given rebase conflicts, then merge paused, user notified, lock released.
- AC-008.6: Given fast verify fails, then merge paused, user notified, lock released.
- AC-008.7: Given session B tries to merge while A holds the lock, then B blocks (up to 60s timeout with feedback).
- AC-008.8: Given rebase conflicts, then the worktree is left with `git rebase --abort` already executed (clean state), and the user is told to resolve conflicts manually and re-run `/implement` to re-trigger merge.

##### Dependencies

- Depends on: US-001, US-006

#### US-009: Documentation

**As a** future contributor, **I want** an ADR, CHANGELOG entry, and glossary additions.

##### Acceptance Criteria

- AC-009.1: ADR exists at `docs/adr/NNNN-concurrent-session-safety.md`.
- AC-009.2: CHANGELOG includes BL-065 entry.
- AC-009.3: CLAUDE.md references `ecc worktree gc` command.
- AC-009.4: Glossary includes: session worktree, merge lock, fast verify gate.

##### Dependencies

- Depends on: US-006, US-008

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `crates/ecc-ports` | Ports | New `FileLock` trait, `Git` trait (worktree ops) |
| `crates/ecc-infra` | Infrastructure | flock implementation, git worktree implementation |
| `crates/ecc-domain` | Domain | Worktree naming/validation value objects |
| `crates/ecc-app` | Application | Merge use case, worktree lifecycle use case |
| `crates/ecc-cli` | CLI | `ecc worktree gc` subcommand |
| `crates/ecc-workflow` | Workflow | Locked state transitions, locked memory writes |
| `crates/ecc-test-support` | Test | In-memory FileLock + Git doubles |

## Constraints

- POSIX flock only — no external dependencies
- Lock only in Rust — shell hooks transient (BL-052)
- Lock paths resolve to main repo root, not worktree
- state.json + .tdd-state worktree-local; memory + backlog target main repo
- Read-only commands do not use worktrees
- No cross-machine locking
- Audit path resolution in commands with relative paths

## Non-Requirements

- Shell hook locking (BL-052 replaces them)
- Cross-machine locking
- External dependency locking (Redis, etcd, SQLite)
- Session-ID-based .tdd-state
- Read-only command worktrees

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| FileLock port | New trait | Integration tests with real flock on temp dirs |
| Git port | New trait | Integration tests with real git worktree ops |
| FileSystem adapter | Extended | Lock file creation in .locks/ |
| CLI | New subcommand | `ecc worktree gc` E2E test |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Architecture | Major | ADR | Create concurrent session ADR |
| CLI | Minor | CLAUDE.md | Add `ecc worktree gc` |
| Content | Minor | CHANGELOG.md | Add BL-065 entry |
| Domain | Minor | Glossary | session worktree, merge lock, fast verify gate |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Split into 4 sub-specs or monolithic? | Split into 4: locks, state fixes, worktrees, merge | Recommended |
| 2 | Shell + Rust locking or Rust only? | Rust only; shell hooks transient (BL-052) | Recommended |
| 3 | Worktree switching mechanism? | EnterWorktree/ExitWorktree native tools | Recommended |
| 4 | Lock path resolution, file locality, .tdd-state, merge timeout? | Main repo root; split local/main; no session-ID; 60s timeout | Recommended |
| 5 | Test coverage targets? | 100% for locks+merge+state.json; 80% for others; real flock integration tests | Recommended |
| 6 | Security, breaking changes, glossary, ADR? | Clear; audit paths; 3 glossary terms; ADR Yes | Recommended |

### User Stories

| ID | Title | Sub-Spec | AC Count | Dependencies |
|----|-------|----------|----------|--------------|
| US-001 | FileLock Port Trait + flock | A | 7 | none |
| US-002 | action-log.json Locking | B | 2 | US-001 |
| US-003 | state.json TOCTOU Fix | B | 3 | US-001 |
| US-004 | BACKLOG.md ID Race Fix | B | 2 | US-001 |
| US-005 | Memory File Locking | B | 4 | US-001 |
| US-006 | Automatic Worktree Isolation | C | 6 | none |
| US-007 | Worktree Lifecycle Management | C | 4 | US-006 |
| US-008 | Serialized Merge-to-Main | D | 8 | US-001, US-006 |
| US-009 | Documentation | — | 4 | US-006, US-008 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | FileLock trait with acquire/release | US-001 |
| AC-001.2 | Concurrent lock blocks one process | US-001 |
| AC-001.3 | .locks/ auto-created | US-001 |
| AC-001.4 | Crash auto-releases lock | US-001 |
| AC-001.5 | Lock path resolves to main repo root | US-001 |
| AC-001.6 | In-memory test double | US-001 |
| AC-001.7 | Multi-process integration test | US-001 |
| AC-002.1 | Both action-log entries appear | US-002 |
| AC-002.2 | Lock blocks concurrent writes | US-002 |
| AC-003.1 | Sequential phase transitions | US-003 |
| AC-003.2 | No data loss on reinit | US-003 |
| AC-003.3 | Gate reads post-transition state | US-003 |
| AC-004.1 | Unique sequential BL IDs | US-004 |
| AC-004.2 | Both backlog entries appear | US-004 |
| AC-005.1 | Both daily entries appear | US-005 |
| AC-005.2 | Both index updates appear | US-005 |
| AC-005.3 | Work-item revision append | US-005 |
| AC-005.4 | One lock per memory file type | US-005 |
| AC-006.1 | EnterWorktree on pipeline command | US-006 |
| AC-006.2 | Writes in worktree | US-006 |
| AC-006.3 | No worktree for non-pipeline | US-006 |
| AC-006.4 | Worktree-local state.json | US-006 |
| AC-006.5 | Memory/backlog to main repo | US-006 |
| AC-006.6 | Naming: ecc-session-{ts}-{slug}-{pid} | US-006 |
| AC-007.1 | Auto-cleanup on success | US-007 |
| AC-007.2 | Preserved on failure | US-007 |
| AC-007.3 | gc removes stale | US-007 |
| AC-007.4 | gc skips active | US-007 |
| AC-008.1 | Acquire merge.lock | US-008 |
| AC-008.2 | Rebase onto main | US-008 |
| AC-008.3 | Fast verify runs | US-008 |
| AC-008.4 | FF merge + cleanup | US-008 |
| AC-008.5 | Rebase conflict → pause | US-008 |
| AC-008.6 | Verify fail → pause | US-008 |
| AC-008.7 | 60s timeout with feedback | US-008 |
| AC-008.8 | Clean worktree after abort | US-008 |
| AC-009.1 | ADR created | US-009 |
| AC-009.2 | CHANGELOG updated | US-009 |
| AC-009.3 | CLAUDE.md updated | US-009 |
| AC-009.4 | Glossary terms added | US-009 |

### Adversary Findings

| Dimension | Verdict | Key Rationale |
|-----------|---------|---------------|
| Ambiguity | PASS (round 2) | Added decisions 8-10 for ecc-workflow, pipeline enumeration, lock granularity |
| Edge cases | PASS (round 2) | Added ACs for naming collision, rebase conflict state, non-merge timeout |
| Scope | PASS | Clean 4-sub-spec decomposition |
| Dependencies | PASS (round 2) | Clarified BL-052 independence |
| Testability | PASS (round 2) | Added multi-process flock integration test AC |
| Decisions | PASS (round 2) | Added decisions 11-13 for timeouts, prerequisites, lock cleanup |
| Rollback | PASS | Worktree preserved on failure, gc for cleanup |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-27-concurrent-session-safety/spec.md | Full spec + phase summary |
