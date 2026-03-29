# Design: Deterministic Task Synchronization (BL-075 + BL-072)

## Overview

Add a `task` domain module to `ecc-domain` (pure parsing, FSM validation, rendering) and three `ecc-workflow tasks` subcommands (`sync`, `update`, `init`) that make tasks.md the single source of truth for `/implement` progress tracking. Update `/implement` Phases 2-3 and the two skills to consume the new CLI.

## File Changes

| # | File | Action | Spec Ref | Rationale |
|---|------|--------|----------|-----------|
| 1 | `crates/ecc-domain/src/task/error.rs` | create | US-001, AC-001.5 | Domain error enum for task parsing/validation failures. Follows `spec/error.rs` pattern. |
| 2 | `crates/ecc-domain/src/task/status.rs` | create | US-001, US-002 | `TaskStatus` enum (Pending, Red, Green, Done, Failed) and FSM transition table. Pure functions, zero I/O. |
| 3 | `crates/ecc-domain/src/task/entry.rs` | create | US-001, AC-001.1-AC-001.4 | `TaskEntry` struct: id (PcId or string label), description, command, checkbox state, status trail. `EntryKind` enum (Pc/PostTdd) for AC-001.4. |
| 4 | `crates/ecc-domain/src/task/parser.rs` | create | US-001, AC-001.1-AC-001.7 | `parse_tasks(content: &str) -> Result<TaskReport, TaskError>`. Reads both `\|` and `→` separators (AC-001.6). Returns structured `TaskReport` with entries + summary counters. |
| 5 | `crates/ecc-domain/src/task/updater.rs` | create | US-004, AC-004.1-AC-004.3, AC-004.6, AC-004.8 | `apply_update(content: &str, entry_id: &str, new_status: TaskStatus, timestamp: &str) -> Result<String, TaskError>`. Pure string-in/string-out. Validates FSM transition, appends trail segment, flips checkbox on done. |
| 6 | `crates/ecc-domain/src/task/renderer.rs` | create | US-005, AC-005.1-AC-005.4, AC-005.6 | `render_tasks(pcs: &[PassCondition], feature_title: &str, timestamp: &str) -> String`. Generates tasks.md content from design PCs with `→` separator and Post-TDD section. |
| 7 | `crates/ecc-domain/src/task/mod.rs` | create | US-001 | Module declaration and re-exports. |
| 8 | `crates/ecc-domain/src/lib.rs` | modify | US-001 | Add `pub mod task;` |
| 9 | `crates/ecc-workflow/src/commands/tasks.rs` | create | US-003, US-004, US-005 | Three subcommand handlers: `run_sync`, `run_update`, `run_init`. All I/O here: file read/write, flock, path validation. |
| 10 | `crates/ecc-workflow/src/main.rs` | modify | US-003, US-004, US-005 | Add `Tasks { subcmd: TasksCmd }` variant to `Commands` enum and `TasksCmd` enum with `Sync`, `Update`, `Init` variants. Wire dispatch. |
| 11 | `crates/ecc-workflow/src/commands/mod.rs` | modify | US-003 | Add `pub mod tasks;` |
| 12 | `commands/implement.md` | modify | US-006 | Update Phase 2 (use `tasks init` + `tasks sync`) and Phase 3 (use `tasks update` after tdd-executor, `tasks sync` for re-entry). |
| 13 | `skills/tasks-generation/SKILL.md` | modify | US-006 | Update separator from `\|` to `→`, note that generation is now done by `ecc-workflow tasks init`. |
| 14 | `skills/progress-tracking/SKILL.md` | modify | US-006 | Update to show `ecc-workflow tasks update` calls instead of manual file edits. |

## Architecture

### Domain Layer (`ecc-domain/src/task/`)

```
task/
  mod.rs          -- re-exports
  error.rs        -- TaskError enum
  status.rs       -- TaskStatus enum + FSM transitions
  entry.rs        -- TaskEntry, EntryKind, StatusSegment, TaskReport
  parser.rs       -- parse_tasks(&str) -> Result<TaskReport>
  updater.rs      -- apply_update(&str, &str, TaskStatus, &str) -> Result<String>
  renderer.rs     -- render_tasks(&[PassCondition], &str, &str) -> String
```

All functions are pure `&str -> Result<T>` or `&[T] -> String`. Zero I/O imports. Reuses `PcId` from `ecc-domain::spec::pc`.

### Adapter Layer (`ecc-workflow/src/commands/tasks.rs`)

Handles all I/O: file reads, atomic writes (tempfile + rename), flock locking, path traversal validation. Calls domain functions for parsing, validation, and rendering.

### Type Definitions

```rust
// task/error.rs
#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("parse error at line {line}: {message}")]
    ParseError { line: usize, message: String },
    #[error("invalid status transition: {from} -> {to}")]
    InvalidTransition { from: String, to: String },
    #[error("same-state transition: already in {status}")]
    SameState { status: String },
    #[error("entry not found: {id}")]
    EntryNotFound { id: String },
    #[error("no PC table found in design file")]
    NoPcTable,
    #[error("duplicate PC ID: {id}")]
    DuplicatePcId { id: String },
    #[error("invalid status: {0}")]
    InvalidStatus(String),
}

// task/status.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Red,
    Green,
    Done,
    Failed,
}

// task/entry.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryKind {
    Pc(PcId),
    PostTdd(String),  // "E2E tests", "Code review", etc.
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusSegment {
    pub status: TaskStatus,
    pub timestamp: String,
    pub error_detail: Option<String>,  // only for Failed
}

#[derive(Debug, Clone)]
pub struct TaskEntry {
    pub kind: EntryKind,
    pub description: String,
    pub command: Option<String>,  // None for PostTdd
    pub completed: bool,          // [x] vs [ ]
    pub trail: Vec<StatusSegment>,
}

#[derive(Debug, Serialize)]
pub struct TaskReport {
    pub entries: Vec<TaskEntry>,
    pub total: usize,
    pub completed: usize,
    pub pending: usize,
    pub in_progress: usize,
    pub failed: usize,
    pub progress_pct: f64,
}
```

### FSM Transition Table

```
            Pending   Red     Green   Done    Failed
Pending     --        OK(*)   REJECT  POST(*) REJECT
Red         REJECT    REJECT  OK      REJECT  OK
Green       REJECT    REJECT  REJECT  OK      OK
Done        REJECT    REJECT  REJECT  REJECT  REJECT
Failed      REJECT    OK      REJECT  REJECT  REJECT

(*) pending->red: TDD entries only
(**) pending->done: PostTdd entries only (AC-002.8)
Same-state transitions always rejected (AC-004.8)
```

### Sync Output JSON Schema

```json
{
  "status": "pass",
  "message": "{\"pending\":[...],\"completed\":[...],\"in_progress\":[...],\"failed\":[...],\"total\":N,\"progress_pct\":NN.N}"
}
```

Each array item: `{"id":"PC-001","description":"...","current_status":"pending"}` or `{"id":"E2E tests","description":"E2E tests","current_status":"done"}`.

### Path Traversal Validation

In `tasks.rs` (adapter layer), before any file operation. Uses `canonicalize` + `starts_with` pattern (matching `memory_write.rs`), upgraded from the initial `contains("..")` approach per security review:

```rust
fn validate_path(path: &Path, project_dir: &Path) -> Result<(), anyhow::Error> {
    // For existing files: canonicalize and check prefix
    // For new files (init --output): canonicalize parent directory
    let resolved = if path.exists() {
        std::fs::canonicalize(path)?
    } else {
        let parent = path.parent()
            .ok_or_else(|| anyhow::anyhow!("invalid path: no parent directory"))?;
        std::fs::canonicalize(parent)?.join(path.file_name().unwrap_or_default())
    };
    let project_root = std::fs::canonicalize(project_dir)
        .unwrap_or_else(|_| project_dir.to_path_buf());
    if !resolved.starts_with(&project_root) {
        anyhow::bail!("path escapes project directory: {}", path.display());
    }
    Ok(())
}
```

### Atomic Write Pattern

Following `io.rs` and `backlog.rs` patterns:

```rust
let _guard = ecc_flock::acquire(project_dir, "tasks")?;
let content = std::fs::read_to_string(&tasks_path)?;
let updated = ecc_domain::task::updater::apply_update(&content, id, status, &timestamp)?;
let tmp = tasks_path.with_extension("tmp");
std::fs::write(&tmp, &updated)?;
std::fs::rename(&tmp, &tasks_path)?;
// _guard drops, releasing lock
```

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | TaskStatus FSM allows valid transitions (pending->red, red->green, green->done, red->failed, green->failed, failed->red) | AC-002.1, AC-002.2, AC-002.3, AC-002.6, AC-002.7 | `cargo test --lib -p ecc-domain task::status::tests -- --nocapture` | PASS |
| PC-002 | unit | TaskStatus FSM rejects invalid transitions (pending->green, pending->done for TDD, done->any, same-state) | AC-002.4, AC-002.5, AC-002.8, AC-004.8 | `cargo test --lib -p ecc-domain task::status::tests::rejects -- --nocapture` | PASS |
| PC-003 | unit | Parser extracts well-formed tasks.md into TaskReport with correct counters | AC-001.1, AC-001.2 | `cargo test --lib -p ecc-domain task::parser::tests -- --nocapture` | PASS |
| PC-004 | unit | Parser extracts multi-segment status trails in order | AC-001.3 | `cargo test --lib -p ecc-domain task::parser::tests::multi_segment -- --nocapture` | PASS |
| PC-005 | unit | Parser identifies PostTdd entries as distinct kind | AC-001.4 | `cargo test --lib -p ecc-domain task::parser::tests::post_tdd -- --nocapture` | PASS |
| PC-006 | unit | Parser returns descriptive error on malformed input, never panics | AC-001.5 | `cargo test --lib -p ecc-domain task::parser::tests::malformed -- --nocapture` | PASS |
| PC-007 | unit | Parser reads old-format pipe separator correctly | AC-001.6 | `cargo test --lib -p ecc-domain task::parser::tests::old_format -- --nocapture` | PASS |
| PC-008 | unit | Parser returns zero-entry report for header-only tasks.md | AC-001.7 | `cargo test --lib -p ecc-domain task::parser::tests::empty -- --nocapture` | PASS |
| PC-009 | unit | Updater appends status trail segment with correct separator and timestamp | AC-004.1 | `cargo test --lib -p ecc-domain task::updater::tests::append_trail -- --nocapture` | PASS |
| PC-010 | unit | Updater flips checkbox on done transition | AC-004.2 | `cargo test --lib -p ecc-domain task::updater::tests::done_checkbox -- --nocapture` | PASS |
| PC-011 | unit | Updater rejects invalid transition (pending->done for TDD entry) | AC-004.3 | `cargo test --lib -p ecc-domain task::updater::tests::reject_invalid -- --nocapture` | PASS |
| PC-012 | unit | Updater returns error for nonexistent entry ID | AC-004.4 | `cargo test --lib -p ecc-domain task::updater::tests::not_found -- --nocapture` | PASS |
| PC-013 | unit | Updater handles PostTdd entry update by string identifier | AC-004.6 | `cargo test --lib -p ecc-domain task::updater::tests::post_tdd_update -- --nocapture` | PASS |
| PC-014 | unit | Updater allows PostTdd pending->done (skip TDD cycle) | AC-002.8, AC-004.6 | `cargo test --lib -p ecc-domain task::updater::tests::post_tdd_done -- --nocapture` | PASS |
| PC-015 | unit | Renderer generates tasks.md from PCs with arrow separator and Post-TDD section | AC-005.1, AC-005.2, AC-005.4, AC-005.6 | `cargo test --lib -p ecc-domain task::renderer::tests -- --nocapture` | PASS |
| PC-016 | unit | Renderer orders PCs respecting dependency order (input order preserved) | AC-005.3 | `cargo test --lib -p ecc-domain task::renderer::tests::order -- --nocapture` | PASS |
| PC-017 | integration | `tasks sync` outputs JSON with correct arrays and counters for valid tasks.md | AC-003.1, AC-003.2 | `cargo test -p ecc-workflow tasks::tests::sync -- --nocapture` | PASS |
| PC-018 | integration | `tasks sync` returns block error for nonexistent path | AC-003.3 | `cargo test -p ecc-workflow tasks::tests::sync_missing -- --nocapture` | PASS |
| PC-019 | integration | `tasks sync` returns warn for malformed tasks.md | AC-003.4 | `cargo test -p ecc-workflow tasks::tests::sync_malformed -- --nocapture` | PASS |
| PC-020 | integration | `tasks sync` rejects path traversal | AC-003.5 | `cargo test -p ecc-workflow tasks::tests::sync_traversal -- --nocapture` | PASS |
| PC-021 | integration | `tasks update` performs atomic write with flock, appends trail | AC-004.1, AC-004.5 | `cargo test -p ecc-workflow tasks::tests::update_atomic -- --nocapture` | PASS |
| PC-022 | integration | `tasks update` rejects path traversal | AC-004.7 | `cargo test -p ecc-workflow tasks::tests::update_traversal -- --nocapture` | PASS |
| PC-023 | integration | `tasks init` generates tasks.md from design file's PC table | AC-005.1, AC-005.2, AC-005.3 | `cargo test -p ecc-workflow tasks::tests::init_generate -- --nocapture` | PASS |
| PC-024 | integration | `tasks init` blocks when output exists (no --force) | AC-005.5 | `cargo test -p ecc-workflow tasks::tests::init_exists -- --nocapture` | PASS |
| PC-025 | integration | `tasks init` overwrites with --force | AC-005.9 | `cargo test -p ecc-workflow tasks::tests::init_force -- --nocapture` | PASS |
| PC-026 | integration | `tasks init` blocks when design has no PC table | AC-005.7 | `cargo test -p ecc-workflow tasks::tests::init_no_pcs -- --nocapture` | PASS |
| PC-027 | integration | `tasks init` blocks on duplicate PC IDs | AC-005.8 | `cargo test -p ecc-workflow tasks::tests::init_dup_pcs -- --nocapture` | PASS |
| PC-028 | unit | TaskStatus serde serialization produces lowercase strings ("pending", "red", "green", "done", "failed") | AC-003.1 | `cargo test --lib -p ecc-domain task::status::tests::serde_format -- --nocapture` | PASS |
| PC-029 | unit | Updater rejects same-state transition at updater level (not just FSM) | AC-004.8 | `cargo test --lib -p ecc-domain task::updater::tests::same_state -- --nocapture` | PASS |
| PC-030 | unit | Parser and updater handle error_detail in Failed status segments | AC-002.6 | `cargo test --lib -p ecc-domain task::parser::tests::failed_detail -- --nocapture` | PASS |
| PC-031 | lint | Clippy passes with zero warnings across workspace | — | `cargo clippy --workspace -- -D warnings` | exit 0 |
| PC-032 | build | Release build succeeds | — | `cargo build --release` | exit 0 |

## TDD Order

Dependency-driven ordering:

1. **PC-001** — TaskStatus FSM valid transitions (foundation for everything)
2. **PC-002** — TaskStatus FSM rejection cases (completes FSM coverage)
3. **PC-003** — Parser: well-formed input (depends on TaskStatus, TaskEntry types)
4. **PC-004** — Parser: multi-segment trails
5. **PC-005** — Parser: PostTdd entries
6. **PC-006** — Parser: malformed input errors
7. **PC-007** — Parser: old-format pipe separator backward compat
8. **PC-008** — Parser: empty tasks.md
9. **PC-009** — Updater: append trail segment (depends on parser + FSM)
10. **PC-010** — Updater: done checkbox flip
11. **PC-011** — Updater: reject invalid transition
12. **PC-012** — Updater: entry not found
13. **PC-013** — Updater: PostTdd update by string ID
14. **PC-014** — Updater: PostTdd pending->done
15. **PC-015** — Renderer: generate from PCs (depends on PassCondition from spec::pc)
16. **PC-016** — Renderer: dependency order preserved
17. **PC-017** — Sync subcommand (depends on parser)
18. **PC-018** — Sync: missing path
19. **PC-019** — Sync: malformed input
20. **PC-020** — Sync: path traversal
21. **PC-021** — Update subcommand: atomic write (depends on updater + flock)
22. **PC-022** — Update: path traversal
23. **PC-023** — Init subcommand: generate from design (depends on renderer + pc parser)
24. **PC-024** — Init: existing output blocked
25. **PC-025** — Init: --force overwrite
26. **PC-026** — Init: no PC table
27. **PC-027** — Init: duplicate PCs
28. **PC-028** — TaskStatus serde serialization format
29. **PC-029** — Updater same-state rejection (updater-level, complements FSM PC-002)
30. **PC-030** — Parser/updater error_detail in Failed segments
31. **PC-031** — Clippy lint (after all code written)
32. **PC-032** — Release build (final gate)

## Wave Analysis

| Wave | PCs | Rationale |
|------|-----|-----------|
| 1 | PC-001, PC-002 | FSM — no file overlap with others |
| 2 | PC-003, PC-004, PC-005, PC-006 | Parser — all in parser.rs, sequential within wave |
| 3 | PC-007, PC-008 | Parser edge cases |
| 4 | PC-009, PC-010, PC-011, PC-012 | Updater — all in updater.rs |
| 5 | PC-013, PC-014 | Updater PostTdd cases |
| 6 | PC-015, PC-016 | Renderer |
| 7 | PC-017, PC-018, PC-019, PC-020 | Sync subcommand (tasks.rs) |
| 8 | PC-021, PC-022 | Update subcommand |
| 9 | PC-023, PC-024, PC-025, PC-026, PC-027 | Init subcommand (cap 4 parallel — sequential within) |
| 10 | PC-028, PC-029, PC-030 | Serde format, same-state updater, error_detail |
| 11 | PC-031, PC-032 | Lint + Build gates |

All waves within domain layer (1-6) touch different files so could be parallelized; however waves 3+ depend on types from waves 1-2. Adapter waves 7-9 depend on domain waves being complete. Wave 10 is a final gate.

## E2E Activation Rules

- **Touches user-facing flows?** Yes — new CLI subcommands
- **Crosses 3+ modules?** Yes — ecc-domain -> ecc-workflow -> /implement command
- **New E2E tests needed?** Yes — smoke tests for sync/update/init subcommands

### E2E Scenarios (after all TDD PCs complete)

1. `ecc-workflow tasks init <design> --output <tasks>` produces valid tasks.md from a fixture design file
2. `ecc-workflow tasks sync <tasks>` returns correct JSON for a fixture tasks.md
3. `ecc-workflow tasks update <tasks> PC-001 red` modifies fixture file atomically
4. Round-trip: init -> sync -> update -> sync shows updated state

## Doc Update Plan

| # | Target Doc | Level | Action |
|---|-----------|-------|--------|
| 1 | `CLAUDE.md` | project | Add `ecc-workflow tasks sync\|update\|init` to CLI Commands section |
| 2 | `docs/MODULE-SUMMARIES.md` | reference | Add `ecc-domain::task` module summary |
| 3 | `docs/adr/030-task-state-source-of-truth.md` | decision | ADR: tasks.md is single source of truth |
| 4 | `docs/adr/031-deterministic-artifact-scaffolding.md` | decision | ADR: tasks.md generated from design PCs |
| 5 | `CHANGELOG.md` | project | Add BL-075/BL-072 entry |
| 6 | `docs/domain/bounded-contexts.md` | reference | Add task domain concept to Workflow context |
| 7 | `CLAUDE.md` | project | Update test count (currently 1562) after adding tests |
| 8 | `docs/commands-reference.md` | reference | Update /implement command description if behavior changes |

## Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| Parser regex complexity for dual-separator support | Medium | Simple split on both `\|` and `→`, tested with fixtures for each format |
| Concurrent writes corrupting tasks.md | High | flock locking + atomic tempfile+rename (proven pattern from backlog.rs) |
| Path traversal on tasks-path argument | High | `canonicalize` + `starts_with(project_dir)` validation (upgraded per security review) |
| Breaking existing tasks.md files | Medium | Parser reads both old and new format; new writes use `→` only |
| Domain crate accidentally importing I/O | High | CI lint: grep for `std::fs`, `std::process`, `std::net` in ecc-domain |

## Spec Reference

Concern: dev, Feature: BL-075 Deterministic task synchronization

## Coverage Check

All 43 ACs from the spec are covered by at least one PC. Zero uncovered ACs.

| AC Range | Covering PCs |
|----------|-------------|
| AC-001.1-007 | PC-003 through PC-008 |
| AC-002.1-008 | PC-001, PC-002, PC-014 |
| AC-003.1-005 | PC-017 through PC-020 |
| AC-004.1-008 | PC-002, PC-009 through PC-014, PC-021, PC-022, PC-029 |
| AC-005.1-009 | PC-015, PC-016, PC-023 through PC-027 |
| AC-006.1-006 | Verified by command integration (markdown changes) |

## E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | ecc-workflow CLI | tasks.rs | — | `tasks sync` returns valid JSON for fixture | ignored | tasks.rs modified |
| 2 | ecc-workflow CLI | tasks.rs | — | `tasks update` atomically modifies file | ignored | tasks.rs modified |
| 3 | ecc-workflow CLI | tasks.rs | — | `tasks init` generates from design fixture | ignored | tasks.rs modified |
| 4 | ecc-workflow CLI | tasks.rs | — | Round-trip: init → sync → update → sync | ignored | Any task subcommand modified |

### E2E Activation Rules

All 4 E2E tests un-ignored during this implementation (all subcommands are new).

## Test Strategy

TDD order (10 waves):

1. **Wave 1** (PC-001, PC-002): FSM foundation — valid and invalid transitions
2. **Wave 2** (PC-003-006): Parser core — well-formed, trails, PostTdd, errors
3. **Wave 3** (PC-007, PC-008): Parser edge cases — old format, empty
4. **Wave 4** (PC-009-012): Updater core — trail append, checkbox, reject, not-found
5. **Wave 5** (PC-013, PC-014): Updater PostTdd cases
6. **Wave 6** (PC-015, PC-016): Renderer — generate and ordering
7. **Wave 7** (PC-017-020): Sync subcommand — valid, missing, malformed, traversal
8. **Wave 8** (PC-021, PC-022): Update subcommand — atomic write, traversal
9. **Wave 9** (PC-023-027): Init subcommand — generate, exists, force, no-PCs, duplicates
10. **Wave 10** (PC-028, PC-029, PC-030): Serde format, same-state updater, error_detail
11. **Wave 11** (PC-031, PC-032): Lint + build gates

## SOLID Assessment

**PASS** — 3 LOW-severity observations (non-blocking):
1. LOW-001: `TaskReport` derives `Serialize` but `TaskEntry` may need it for JSON output
2. LOW-002: Path validation upgraded to `canonicalize` pattern (addressed)
3. LOW-003: `TaskError` vs `SpecError` separate types — correct per ISP but adapter must map both

## Robert's Oath Check

**CLEAN** — No warnings. Design demonstrates craftsmanship across all oath dimensions: no harmful code, clean structure, comprehensive proof (29 PCs, 43 ACs), small releases (10 waves), fearless improvement (single source of truth replacing three-way drift).

## Security Notes

1 MEDIUM finding (addressed): Path validation upgraded from `contains("..")` to `canonicalize` + `starts_with(project_dir)`, matching the existing `memory_write.rs` pattern. Handles symlink escapes and eliminates false positives.

1 LOW finding: Confirm no regex in parser (use `str::split`/`str::find`). Consider max file size check (1MB).

## Rollback Plan

Reverse dependency order for safe revert:

1. Revert `skills/progress-tracking/SKILL.md` (restore manual tracking instructions)
2. Revert `skills/tasks-generation/SKILL.md` (restore `|` separator)
3. Revert `commands/implement.md` (restore manual TodoWrite/TaskCreate)
4. Revert `crates/ecc-workflow/src/commands/mod.rs` (remove `pub mod tasks`)
5. Revert `crates/ecc-workflow/src/main.rs` (remove Tasks variant)
6. Delete `crates/ecc-workflow/src/commands/tasks.rs`
7. Revert `crates/ecc-domain/src/lib.rs` (remove `pub mod task`)
8. Delete `crates/ecc-domain/src/task/` directory (all 7 files)

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | PASS | 3 LOW |
| Robert | CLEAN | 0 |
| Security | 1 MEDIUM (addressed), 1 LOW | 2 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Coverage | 88 | PASS | All 43 ACs covered; AC-006.x acknowledged as non-automatable |
| Order | 95 | PASS | TDD dependencies well-respected across 11 waves |
| Fragility | 82 | PASS | Pure domain functions reduce fragility; centralized fixtures recommended |
| Rollback | 90 | PASS | All changes additive, reverse dependency order documented |
| Architecture | 92 | PASS | Clean hexagonal layering, zero I/O in domain |
| Blast Radius | 85 | PASS | Largely additive; implement.md is highest-risk change |
| Missing PCs | 88 | PASS | 3 PCs added in round 2 (serde, same-state, error_detail) |
| Doc Plan | 90 | PASS | 8 doc targets covering all hierarchy levels |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `crates/ecc-domain/src/task/error.rs` | create | US-001, AC-001.5 |
| 2 | `crates/ecc-domain/src/task/status.rs` | create | US-001, US-002 |
| 3 | `crates/ecc-domain/src/task/entry.rs` | create | US-001, AC-001.1-004 |
| 4 | `crates/ecc-domain/src/task/parser.rs` | create | US-001, AC-001.1-007 |
| 5 | `crates/ecc-domain/src/task/updater.rs` | create | US-004, AC-004.1-008 |
| 6 | `crates/ecc-domain/src/task/renderer.rs` | create | US-005, AC-005.1-006 |
| 7 | `crates/ecc-domain/src/task/mod.rs` | create | US-001 |
| 8 | `crates/ecc-domain/src/lib.rs` | modify | US-001 |
| 9 | `crates/ecc-workflow/src/commands/tasks.rs` | create | US-003, US-004, US-005 |
| 10 | `crates/ecc-workflow/src/main.rs` | modify | US-003, US-004, US-005 |
| 11 | `crates/ecc-workflow/src/commands/mod.rs` | modify | US-003 |
| 12 | `commands/implement.md` | modify | US-006 |
| 13 | `skills/tasks-generation/SKILL.md` | modify | US-006 |
| 14 | `skills/progress-tracking/SKILL.md` | modify | US-006 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-29-deterministic-task-sync/design.md | Full design |
