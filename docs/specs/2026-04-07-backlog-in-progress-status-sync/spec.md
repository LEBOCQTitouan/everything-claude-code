# Spec: Backlog In-Progress Status Sync

## Problem Statement

Backlog in-progress status tracking is split across three layers with no consistency guarantee: `commands/spec.md` implements a 4-step worktree+lock+filter algorithm as unstructured Claude tool calls, `ecc-workflow` duplicates backlog logic with raw `std::fs` bypassing ports, and `BACKLOG.md` is a manually-maintained index with 98 changes in 180 days (0.00 co-change ratio with Rust code). This produces unreliable filtering, untestable logic, and noisy git history.

## Research Summary

- Kubernetes-style reconciliation loop: derive "in-progress" from worktree existence, don't dual-write
- Idempotent operations: `reconcile()` safe to call at any time without side effects beyond convergence
- File-based state with atomic writes + POSIX flock avoids corruption (ECC already has `ecc-flock`)
- Session lifecycle hooks are natural trigger points for status sync
- Stale status from crashed sessions: need staleness heuristic (timestamp expiry or worktree existence check)
- Rust enum state machines enforce valid transitions at compile time
- Worktree existence as source of truth eliminates dual-write drift

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Create `BacklogRepository` port trait | Typed contract for backlog persistence, matching worktree pattern | Yes |
| 2 | Derive in-progress status from worktree existence | Eliminates dual-write drift; reconciliation over manual sync | No |
| 3 | Make BACKLOG.md a generated artifact | Removes 98-change hotspot, `ecc backlog reindex` becomes authoritative | No |
| 4 | Bundle all 10 smells in single refactoring | User preference; steps are tightly coupled | No |
| 5 | TDD + new integration tests before refactoring | Both unit tests for port trait and integration tests for filtering | No |
| 6 | Fix build error first | ecc-test-support compilation broken (missing `local_llm` field) | No |
| 7 | Add `Clock` port for deterministic staleness tests | 24h lock expiry needs testable time source; inject `&dyn Clock` into `list_available` | No |
| 8 | Add `Serialize` derive to `BacklogEntry` | Required for JSON CLI output; currently only has `Deserialize` | No |
| 9 | `reindex()` stays in app layer, not on port trait | Orchestration function composing BacklogRepository + WorktreeManager; not a data-access concern | No |
| 10 | Lock file schema: line 1 = worktree name, line 2 = ISO 8601 timestamp | Matches existing format in `commands/spec.md`, now codified | No |
| 11 | `reindex` acquires flock for concurrent safety | CLI command may run in parallel sessions | No |

## User Stories

### US-001: Backlog filtering via CLI command

**As a** Claude Code agent, **I want** to call `ecc backlog list --available` to get backlog items not claimed by active worktrees, **so that** the `/spec` picker is deterministic and testable.

#### Acceptance Criteria

- AC-001.1: Given worktrees with BL-NNN patterns exist, when `ecc backlog list --available` runs, then items matching those BL-NNN IDs are excluded
- AC-001.2: Given lock files exist in `docs/backlog/.locks/BL-NNN.lock` (format: line 1 = worktree name, line 2 = ISO 8601 timestamp), when lock is < 24h old with valid worktree, then that BL-NNN is excluded
- AC-001.3: Given a stale lock (> 24h per `Clock` port) or orphaned lock (worktree no longer in `.claude/worktrees/`), when list runs, then lock is auto-removed and item is included
- AC-001.4: Given `--show-all` flag, when list runs, then all open items are returned regardless of claims
- AC-001.5: Output is JSON array of `BacklogEntry` objects for machine consumption (requires `Serialize` derive on `BacklogEntry`)
- AC-001.6: Given zero open entries after filtering, output is empty JSON array `[]` (not an error)

#### Dependencies

- Depends on: US-002

### US-002: BacklogRepository port trait

**As a** developer, **I want** a typed `BacklogRepository` port trait, **so that** backlog operations are testable with in-memory doubles.

#### Acceptance Criteria

- AC-002.1: `BacklogRepository` trait in `ecc-ports` with data-access methods: `load_entries()`, `load_entry()`, `save_entry()`, `next_id()`. `reindex()` stays in `ecc-app` as orchestration (composes `BacklogRepository` + `WorktreeManager`)
- AC-002.2: `FsBacklogRepository` adapter in `ecc-infra` implementing the trait via `FileSystem` port
- AC-002.3: `InMemoryBacklogRepository` in `ecc-test-support` for testing
- AC-002.4: `ecc-app::backlog` functions refactored to accept `&dyn BacklogRepository`

#### Dependencies

- Depends on: none

### US-003: Status reconciliation from worktree state

**As a** developer, **I want** `ecc backlog reindex` to derive in-progress status from active worktrees, **so that** BACKLOG.md reflects reality without manual edits.

#### Acceptance Criteria

- AC-003.1: `reindex` accepts `&dyn WorktreeManager` to list active worktrees
- AC-003.2: Items with BL-NNN matching an active worktree get status `in-progress` in the generated index
- AC-003.3: Items with BL-NNN matching a lock file (valid, non-stale) also get `in-progress`
- AC-003.4: Reconciliation is idempotent — running twice produces same output
- AC-003.6: `reindex` acquires flock before writing BACKLOG.md (concurrent safety)
- AC-003.5: Generated BACKLOG.md preserves the Dependency Graph section

#### Dependencies

- Depends on: US-002

### US-004: Consolidate ecc-workflow backlog operations

**As a** developer, **I want** `ecc-workflow` backlog commands to delegate to `ecc-app` via the port trait, **so that** logic is not duplicated and operations go through the hexagonal boundary.

#### Acceptance Criteria

- AC-004.1: `ecc-workflow::commands::backlog::compute_next_id` removed, delegates to `ecc-app::backlog::next_id`
- AC-004.2: `ecc-workflow::commands::backlog::update_backlog_index` removed, delegates to `ecc-app::backlog::reindex`
- AC-004.3: Raw `std::fs` calls in `ecc-workflow/src/commands/backlog.rs` replaced with `BacklogRepository` port
- AC-004.4: Existing flock-based locking preserved

#### Dependencies

- Depends on: US-002

### US-005: Structural cleanup (file moves and splits)

**As a** developer, **I want** misplaced adapters moved and oversized files split, **so that** the codebase follows its own conventions.

#### Acceptance Criteria

- AC-005.1: `ShellWorktreeManager` moved from `ecc-app/src/worktree.rs` to `ecc-infra/src/shell_worktree.rs` (coexists with `OsWorktreeManager` — `Shell` variant delegates to `ShellExecutor` port for hook contexts, `Os` variant uses `Command::new` directly for CLI contexts)
- AC-005.2: `ecc-workflow/src/commands/merge.rs` (809 lines) split to stay under 800 lines
- AC-005.3: `ecc-app/src/worktree.rs` (773 lines) split into `worktree/gc.rs`, `worktree/status.rs`, `worktree/mod.rs`
- AC-005.4: `BacklogError::IoError(String)` replaced with proper `FsError` variant (audit ERR-009)

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `ecc-domain/src/backlog/` | Domain | Add lock file domain types, update BacklogError |
| `ecc-ports/src/backlog.rs` (new) | Port | New BacklogRepository trait |
| `ecc-app/src/backlog.rs` | App | Refactor to use BacklogRepository, add list_available |
| `ecc-infra/src/fs_backlog.rs` (new) | Infra | FsBacklogRepository adapter |
| `ecc-cli/src/commands/backlog.rs` | CLI | Add `list --available` subcommand |
| `ecc-workflow/src/commands/backlog.rs` | Workflow | Delegate to ecc-app, remove duplicated logic |
| `ecc-app/src/worktree.rs` | App | Split into module directory |
| `ecc-infra/src/shell_worktree.rs` (new) | Infra | Moved from ecc-app |
| `ecc-workflow/src/commands/merge.rs` | Workflow | Split to reduce file size |
| `ecc-test-support/` | Test | InMemoryBacklogRepository, fix local_llm field |

## Constraints

- All refactoring steps must be behavior-preserving (existing tests stay green)
- Test suite must pass after each commit
- BACKLOG.md format must remain backward-compatible (same markdown table structure)
- Flock-based locking must be preserved for concurrent session safety
- Domain crate must have zero I/O imports

## Non-Requirements

- Migrating all 221 `std::fs` calls in ecc-workflow (only backlog module)
- Creating `BacklogStore` trait with higher-level operations (port trait is sufficient)
- Changing the BL-NNN ID format
- Changing worktree naming conventions

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| BacklogRepository (new) | New port | New integration tests needed |
| WorktreeManager | Extended usage | reindex now reads worktree list |
| FileSystem | Unchanged | Existing adapter reused by FsBacklogRepository |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New port trait | Architecture | docs/ARCHITECTURE.md | Add BacklogRepository to port listing |
| ADR | Decision | docs/adr/NNNN-backlog-repository-port.md | New ADR |
| Bounded context | Domain | docs/domain/bounded-contexts.md | Add backlog context (fixes DOC-006) |
| CLI command | Reference | docs/commands-reference.md | Add `ecc backlog list --available` |
| CLAUDE.md | Onboarding | CLAUDE.md | Update test count after adding tests |

## Open Questions

None — all resolved during grill-me interview.
