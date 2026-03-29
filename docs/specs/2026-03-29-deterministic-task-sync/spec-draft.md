# Spec: Deterministic Task Synchronization (BL-075 + BL-072)

## Problem Statement

The `/implement` command maintains task state in three parallel systems: tasks.md (file), TodoWrite (UI checklist), and TaskCreate/TaskUpdate (spinner + progress). The LLM manually constructs all three, leading to drift when context compacts or sessions resume. Status transitions are unvalidated — a PC can jump from `pending` to `done` without going through the TDD cycle. Re-entry requires the LLM to re-read and cross-reference all three systems, wasting tokens and risking inconsistency.

## Research Summary

- **Markdown parsing**: `comrak` (GFM AST) and `pulldown-cmark` (streaming events) are the main Rust crates. For our simple checklist format, hand-rolled parsing is sufficient — the format is constrained and well-defined.
- **Event sourcing for agents**: The ESAA paper establishes append-only event logs as the pattern for deterministic agent state. Our status trail is already an append-only log within each line.
- **Atomic file writes**: Canonical Rust pattern is `tempfile::NamedTempFile` + `.persist()`. Critical: temp file must be on the same filesystem as the target.
- **Deterministic orchestration**: Praetorian's architecture treats agents as ephemeral with curated context, enforcing workflows through deterministic hooks — exactly our pattern.
- **Prior art**: `ccboard` and `conclaude` demonstrate compiled Rust hook binaries for Claude Code. The thin-main pattern (5-15 line main.rs calling library functions) maximizes testability.
- **Pitfall**: Don't use regex for complex markdown tables. For our constrained checklist format, regex is fine since we control the format and it's not arbitrary markdown.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | tasks.md is the single source of truth; TodoWrite/TaskCreate are derived from it | Eliminates drift between three parallel state systems | Yes (ADR-030) |
| 2 | Subcommands live in `ecc-workflow`, not `ecc-cli` | Workflow-internal concern, follows existing pattern (transition, phase-gate, etc.) | Yes (ADR-030) |
| 3 | Domain types and pure parsing/update functions in `ecc-domain` | Zero-I/O domain layer, testable with pure string inputs | Yes (ADR-030) |
| 4 | Status trail separator changes from `\|` to `→` | Avoids ambiguity with markdown table pipes and error messages containing `\|` | No |
| 5 | Atomic writes via tempfile + rename, locked with ecc-flock | Prevents concurrent write corruption during parallel wave execution | Yes (ADR-030) |
| 6 | tasks.md generation from design PCs is in scope (BL-072) | Deterministic scaffolding completes the lifecycle: generate → sync → update | Yes (ADR-031) |
| 7 | Path traversal protection on tasks-path argument | Security hardening for file operations | No |
| 8 | Parser reads both old (`\|`) and new (`→`) separator formats | Backward compatibility with existing spec artifacts | No |
| 9 | Post-TDD entries can transition `pending → done` directly | They don't follow the TDD cycle (red → green) | No |
| 10 | tdd-executor does NOT call tasks update directly — parent orchestrator owns state | Avoids concurrent writes from worktree-isolated subagents | No |

## User Stories

### US-001: tasks.md Parser (Domain Model)

**As a** workflow automation developer, **I want** a Rust parser for the tasks.md format, **so that** downstream commands can operate on structured task data.

#### Acceptance Criteria

- AC-001.1: Given a well-formed tasks.md, when parsed, then returns structured TaskReport with entries, summary counters
- AC-001.2: Given mixed `[x]` and `[ ]` items, when parsed, then correctly identifies completed vs pending
- AC-001.3: Given multi-segment status trails (`pending@... → red@... → green@... → done@...`), when parsed, then extracts all segments in order
- AC-001.4: Given Post-TDD entries (E2E tests, Code review), when parsed, then includes them as distinct entry types
- AC-001.5: Given malformed tasks.md, when parsed, then returns descriptive error, never panics
- AC-001.6: Given old-format tasks.md with `|` separator, when parsed, then reads correctly (backward compat)
- AC-001.7: Given an empty tasks.md (header only, no entries), when parsed, then returns TaskReport with zero entries and zero counters

#### Dependencies

- Depends on: none

### US-002: Status Transition Validation

**As a** workflow state machine, **I want** status transitions validated against a defined FSM, **so that** invalid progressions are caught immediately.

#### Acceptance Criteria

- AC-002.1: Given pending, when transitioning to red, then allowed
- AC-002.2: Given red, when transitioning to green, then allowed
- AC-002.3: Given green, when transitioning to done, then allowed
- AC-002.4: Given pending, when transitioning to green, then rejected (must go through red)
- AC-002.5: Given done, when transitioning to any status, then rejected (terminal)
- AC-002.6: Given red or green, when transitioning to failed, then allowed
- AC-002.7: Given failed, when transitioning to red, then allowed (retry)
- AC-002.8: Given Post-TDD entry with pending, when transitioning to done, then allowed (skip TDD cycle)

#### Dependencies

- Depends on: US-001

### US-003: `ecc-workflow tasks sync` Subcommand

**As a** `/implement` command, **I want** to call `ecc-workflow tasks sync <tasks-path>`, **so that** I can derive TodoWrite and TaskCreate entries deterministically.

#### Acceptance Criteria

- AC-003.1: Given valid tasks.md, when synced, then outputs JSON with `pending`, `completed`, `in_progress`, `failed` arrays and `total`/`progress_pct`
- AC-003.2: Given tasks.md with some completed PCs, then `pending` contains only undone items
- AC-003.3: Given nonexistent path, then exits with block status and error
- AC-003.4: Given malformed tasks.md, then exits with warn status and message
- AC-003.5: Given path traversal attempt (e.g., `../../etc/passwd`), then rejected

#### Dependencies

- Depends on: US-001

### US-004: `ecc-workflow tasks update` Subcommand

**As a** `/implement` parent orchestrator, **I want** to call `ecc-workflow tasks update <tasks-path> <pc-id> <status>`, **so that** task state changes are atomic and validated.

#### Acceptance Criteria

- AC-004.1: Given PC-001 pending, when `update PC-001 red`, then appends `→ red@<ISO>` to status trail
- AC-004.2: Given PC-001 green, when `update PC-001 done`, then appends `→ done@<ISO>` and changes `[ ]` to `[x]`
- AC-004.3: Given PC-001 pending, when `update PC-001 done`, then rejected (invalid transition for TDD entry)
- AC-004.4: Given nonexistent PC-099, then block status with "PC not found"
- AC-004.5: Given concurrent updates, then atomic write via tempfile+rename prevents corruption
- AC-004.6: Given Post-TDD entry "E2E tests", when `update "E2E tests" done`, then updates correctly
- AC-004.7: Given path traversal in tasks-path, then rejected
- AC-004.8: Given PC-001 already in `red`, when `update PC-001 red`, then rejected (same-state transition is a no-op error)

#### Dependencies

- Depends on: US-001, US-002

### US-005: `ecc-workflow tasks init` Subcommand (BL-072)

**As a** `/implement` command during Phase 2, **I want** to call `ecc-workflow tasks init <design-path> --output <tasks-path>`, **so that** tasks.md is generated deterministically from the design's PC table.

#### Acceptance Criteria

- AC-005.1: Given a design.md with PC table, when init called, then generates tasks.md with all PCs in dependency order
- AC-005.2: Given design.md, then generated tasks.md includes Post-TDD section (E2E, Code review, Doc updates, implement-done.md)
- AC-005.3: Given design.md with PC dependencies, then PCs are ordered respecting dependencies
- AC-005.4: Each entry gets `pending@<current ISO timestamp>` in status trail
- AC-005.5: Given output path already exists, then exits with block status (no overwrite without --force)
- AC-005.6: Generated format uses `→` separator for status trail
- AC-005.7: Given a design.md with no PC table, when init called, then exits with block status and "no PC table found" error
- AC-005.8: Given a design.md with duplicate PC IDs, when init called, then exits with block status and "duplicate PC-NNN" error
- AC-005.9: Given `--force` flag and output path exists, when init called, then overwrites existing tasks.md

#### Dependencies

- Depends on: US-001

### US-006: `/implement` Command Integration

**As a** developer using `/implement`, **I want** Phases 2-3 to use `ecc-workflow tasks` subcommands, **so that** task tracking is deterministic and drift-free.

#### Acceptance Criteria

- AC-006.1: Phase 2 calls `ecc-workflow tasks init` to generate tasks.md (replacing LLM-generated tasks)
- AC-006.2: Phase 2 calls `ecc-workflow tasks sync` to derive TodoWrite items (replacing manual construction)
- AC-006.3: Phase 3 calls `ecc-workflow tasks sync` before each wave to get current state
- AC-006.4: Parent orchestrator calls `ecc-workflow tasks update` for each status transition after tdd-executor returns
- AC-006.5: Re-entry calls `ecc-workflow tasks sync` to rebuild state (replacing manual reconciliation)
- AC-006.6: TaskCreate entries are derived from sync output (not manually constructed)

#### Dependencies

- Depends on: US-003, US-004, US-005

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `ecc-domain/src/task/` (new) | Domain | TaskStatus, TaskEntry, TaskReport types + parser + updater + transition FSM |
| `ecc-domain/src/spec/pc.rs` | Domain | Reuse PcId (no changes needed) |
| `ecc-workflow/src/commands/tasks.rs` (new) | Adapter/CLI | sync, update, init subcommand handlers |
| `ecc-workflow/src/main.rs` | Adapter/CLI | Add Tasks { subcmd } variant to Commands enum |
| `commands/implement.md` | Command | Update Phases 2-3 to use ecc-workflow tasks |
| `skills/progress-tracking/SKILL.md` | Skill | Update to derive from sync output |
| `skills/tasks-generation/SKILL.md` | Skill | Update format spec (→ separator) or deprecate in favor of `tasks init` |

## Constraints

- `ecc-domain` must have zero I/O imports — all parsing is pure `&str -> Result`
- Atomic writes must use ecc-flock with lock name `"tasks"`
- Path traversal must be validated before any file operation
- Parser must handle both old (`|`) and new (`→`) separator formats
- <50ms execution target for all subcommands
- 100% branch coverage for all ecc-domain task logic

## Non-Requirements

- Event sourcing / append-only log architecture (overkill for file-based state)
- comrak/pulldown-cmark AST parsing (format is simple enough for hand-rolled parser)
- tdd-executor calling tasks update directly (parent-owned)
- Modifying the Claude Code hooks runtime or hooks.json schema
- TodoWrite/TaskCreate elimination — they are derived, not removed

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| ecc-workflow CLI | New subcommands | Must smoke-test all three: sync, update, init |
| /implement command | Behavioral change | Full implement dry-run needed to verify integration |
| tasks.md format | Format evolution | Old artifacts remain readable, new ones use → separator |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New CLI subcommands | CLAUDE.md | CLI Commands section | Add `ecc-workflow tasks sync/update/init` |
| New domain types | MODULE-SUMMARIES.md | ecc-domain section | Add task module summary |
| Architecture decision | docs/adr/ | New ADR-030 | Task state source of truth |
| Architecture decision | docs/adr/ | New ADR-031 | Deterministic artifact scaffolding |
| Format change | skills/tasks-generation/ | SKILL.md | Update separator from `\|` to `→` |

## Open Questions

None — all resolved during grill-me interview.
