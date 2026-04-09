# Design: Backlog In-Progress Status Sync

## Overview

Promote the backlog subsystem to a full hexagonal concern (domain -> port -> infra -> app -> cli) by creating 3 focused port traits (ISP-compliant: `BacklogEntryStore`, `BacklogLockStore`, `BacklogIndexStore`), deriving in-progress status from worktree existence (reconciliation pattern), consolidating duplicated logic from `ecc-workflow`, and cleaning up misplaced adapters and oversized files. Fix the ecc-test-support build error first as a prerequisite.

## Spec Reference
Concern: refactor, Feature: backlog in-progress status sync

## Architecture Changes

### New Files

| File | Layer | Purpose |
|------|-------|---------|
| `crates/ecc-domain/src/backlog/lock.rs` | Domain | `LockFile` value object, parse/format lock files, staleness check |
| `crates/ecc-ports/src/backlog.rs` | Port | `BacklogRepository` trait: `load_entries`, `load_entry`, `save_entry`, `next_id`, `load_lock`, `save_lock`, `remove_lock`, `list_locks` |
| `crates/ecc-infra/src/fs_backlog.rs` | Infra | `FsBacklogRepository` adapter via `&dyn FileSystem` |
| `crates/ecc-infra/src/shell_worktree.rs` | Infra | `ShellWorktreeManager` moved from `ecc-app/src/worktree.rs` |
| `crates/ecc-test-support/src/in_memory_backlog.rs` | Test | `InMemoryBacklogRepository` for deterministic testing |
| `crates/ecc-app/src/worktree/mod.rs` | App | Module root after split (re-exports) |
| `crates/ecc-app/src/worktree/gc.rs` | App | GC use case (extracted from monolith) |
| `crates/ecc-app/src/worktree/status.rs` | App | Status use case (extracted from monolith) |

### Modified Files

| File | Change | Spec Ref |
|------|--------|----------|
| `crates/ecc-domain/src/backlog/entry.rs` | Add `Serialize` derive to `BacklogEntry`; replace `BacklogError::IoError(String)` with `FsError(FsError)` variant | AC-001.5, AC-005.4 |
| `crates/ecc-domain/src/backlog/mod.rs` | Add `pub mod lock;` | US-003 |
| `crates/ecc-ports/src/lib.rs` | Add `pub mod backlog;` | US-002 |
| `crates/ecc-ports/Cargo.toml` | No change (already depends on ecc-domain) | -- |
| `crates/ecc-infra/src/lib.rs` | Add `pub mod fs_backlog;`, `pub mod shell_worktree;` | US-002, AC-005.1 |
| `crates/ecc-app/src/backlog.rs` | Refactor to accept `&dyn BacklogEntryStore + &dyn BacklogLockStore + &dyn BacklogIndexStore` + `&dyn WorktreeManager` + `&dyn Clock`; add `list_available()` | AC-002.4, US-001, US-003 |
| `crates/ecc-cli/src/commands/backlog.rs` | Add `List { available: bool, show_all: bool }` subcommand | US-001 |
| `crates/ecc-workflow/src/commands/backlog.rs` | Remove `compute_next_id`, `update_backlog_index`; delegate to `ecc-app` | US-004 |
| `crates/ecc-workflow/src/commands/merge.rs` | Extract helper functions to stay under 800 lines | AC-005.2 |
| `crates/ecc-test-support/src/in_memory_config_store.rs` | Fix `RawEccConfig` struct literals to include `local_llm` field | Decision 6 |
| `crates/ecc-test-support/src/lib.rs` | Add `pub mod in_memory_backlog;` and re-export | US-002 |

## File Changes (Dependency Order)

### FC-001: Fix ecc-test-support build error

| Attribute | Value |
|-----------|-------|
| File | `crates/ecc-test-support/src/in_memory_config_store.rs` |
| Layer | Test Support |
| Spec Ref | Decision 6 |
| Action | Add `local_llm: None` to both `RawEccConfig { .. }` struct literals in tests (lines 75-77, 92-94) |
| Rationale | `RawEccConfig` gained a `local_llm: Option<LocalLlmConfig>` field but test-support tests weren't updated |

### FC-002: Add Serialize to BacklogEntry

| Attribute | Value |
|-----------|-------|
| File | `crates/ecc-domain/src/backlog/entry.rs` |
| Layer | Domain |
| Spec Ref | AC-001.5, Decision 8 |
| Action | Change `#[derive(Debug, Clone, Deserialize)]` to `#[derive(Debug, Clone, Serialize, Deserialize)]` on `BacklogEntry` |
| Rationale | JSON output in `list --available` requires serialization |

### FC-003: Replace BacklogError::IoError with FsError variant

| Attribute | Value |
|-----------|-------|
| File | `crates/ecc-domain/src/backlog/entry.rs` |
| Layer | Domain |
| Spec Ref | AC-005.4 |
| Action | Replace `IoError(String)` with `Io { path: String, message: String }` in `BacklogError` (domain stays pure — no ports dependency). Add `impl From<ecc_ports::fs::FsError> for BacklogError` in `ecc-app/src/backlog.rs` (app layer, where both crates are visible). Update all call sites that construct `BacklogError::IoError(e.to_string())` to use the structured `Io` variant |
| Rationale | Audit ERR-009 flagged lossy string conversion. Domain cannot depend on ports (hexagonal constraint) |
| Risk | MEDIUM -- must update all callers in ecc-app and ecc-workflow |

### FC-004: Add lock file domain types

| Attribute | Value |
|-----------|-------|
| File | `crates/ecc-domain/src/backlog/lock.rs` (new) |
| Layer | Domain |
| Spec Ref | AC-001.2, AC-001.3, Decision 10 |
| Action | Create `LockFile` value object with fields: `worktree_name: String`, `timestamp: String` (ISO 8601). Functions: `parse(content: &str) -> Result<LockFile, BacklogError>`, `format(&self) -> String`, `is_stale(&self, now_epoch_secs: u64) -> bool` (24h threshold). Constants: `LOCK_STALE_SECS: u64 = 24 * 3600` |
| Rationale | Pure domain logic for lock file format; `is_stale` takes epoch secs (no I/O, Clock port injected at app layer) |

### FC-005: Add backlog module to domain mod.rs

| Attribute | Value |
|-----------|-------|
| File | `crates/ecc-domain/src/backlog/mod.rs` |
| Layer | Domain |
| Spec Ref | US-003 |
| Action | Add `pub mod lock;` |

### FC-006: Create BacklogRepository port trait

| Attribute | Value |
|-----------|-------|
| File | `crates/ecc-ports/src/backlog.rs` (new) |
| Layer | Port |
| Spec Ref | AC-002.1, Decision 1 |
| Action | Define trait: |

```rust
use ecc_domain::backlog::entry::{BacklogEntry, BacklogError};
use std::path::Path;

/// Data access for backlog entries (BL-*.md files).
/// ISP-compliant: only entry operations, no lock or index concerns.
pub trait BacklogEntryStore: Send + Sync {
    fn load_entries(&self, backlog_dir: &Path) -> Result<Vec<BacklogEntry>, BacklogError>;
    fn load_entry(&self, backlog_dir: &Path, id: &str) -> Result<BacklogEntry, BacklogError>;
    fn save_entry(&self, backlog_dir: &Path, entry: &BacklogEntry, body: &str) -> Result<(), BacklogError>;
    fn next_id(&self, backlog_dir: &Path) -> Result<String, BacklogError>;
}

/// Lock file lifecycle for session claiming.
pub trait BacklogLockStore: Send + Sync {
    fn load_lock(&self, backlog_dir: &Path, id: &str) -> Result<Option<LockFile>, BacklogError>;
    fn save_lock(&self, backlog_dir: &Path, id: &str, lock: &LockFile) -> Result<(), BacklogError>;
    fn remove_lock(&self, backlog_dir: &Path, id: &str) -> Result<(), BacklogError>;
    fn list_locks(&self, backlog_dir: &Path) -> Result<Vec<(String, LockFile)>, BacklogError>;
}

/// BACKLOG.md index I/O.
pub trait BacklogIndexStore: Send + Sync {
    fn write_index(&self, backlog_dir: &Path, content: &str) -> Result<(), BacklogError>;
    fn read_index(&self, backlog_dir: &Path) -> Result<Option<String>, BacklogError>;
}
```

### FC-007: Register backlog port module

| Attribute | Value |
|-----------|-------|
| File | `crates/ecc-ports/src/lib.rs` |
| Layer | Port |
| Spec Ref | US-002 |
| Action | Add `/// Backlog data access port.` + `pub mod backlog;` after the `bypass_store` line |

### FC-008: Create FsBacklogRepository adapter

| Attribute | Value |
|-----------|-------|
| File | `crates/ecc-infra/src/fs_backlog.rs` (new) |
| Layer | Infra |
| Spec Ref | AC-002.2 |
| Action | Implement `BacklogRepository` trait using `&dyn FileSystem` port. Locks stored at `{backlog_dir}/.locks/{id}.lock`. Uses `ecc_domain::backlog::entry::extract_id_from_filename` and `parse_frontmatter`. Atomic writes via temp file + rename (matching existing pattern in `ecc-app/src/backlog.rs`). |

### FC-009: Register fs_backlog module in infra lib.rs

| Attribute | Value |
|-----------|-------|
| File | `crates/ecc-infra/src/lib.rs` |
| Layer | Infra |
| Spec Ref | US-002 |
| Action | Add `pub mod fs_backlog;` |

### FC-010: Move ShellWorktreeManager to infra

| Attribute | Value |
|-----------|-------|
| File | `crates/ecc-infra/src/shell_worktree.rs` (new), `crates/ecc-app/src/worktree.rs` (modified) |
| Layer | Infra |
| Spec Ref | AC-005.1 |
| Action | Extract `ShellWorktreeManager` struct + impl block (lines 644-773 of `ecc-app/src/worktree.rs`) to `ecc-infra/src/shell_worktree.rs`. Add `pub mod shell_worktree;` to infra `lib.rs`. Update all `use` paths. `ecc-app` Cargo.toml already depends on `ecc-infra` (indirect via test-support), but it must NOT depend on infra directly in prod code. Check: if `ecc-app` tests import `ShellWorktreeManager`, they need it via `ecc-test-support` or `ecc-infra` dev-dependency. |

**Dependency check**: `ecc-app/Cargo.toml` has `ecc-test-support` as dev-dependency. `ShellWorktreeManager` is used in ecc-app *production* code? No -- it's defined there but only consumed via `&dyn WorktreeManager`. Moving it to infra is clean. Any ecc-app test that constructs `ShellWorktreeManager` directly (unlikely -- tests use `MockWorktreeManager`) would need updating.

### FC-011: Split ecc-app/src/worktree.rs into module directory

| Attribute | Value |
|-----------|-------|
| File | `crates/ecc-app/src/worktree/mod.rs`, `crates/ecc-app/src/worktree/gc.rs`, `crates/ecc-app/src/worktree/status.rs` (new files, replacing `crates/ecc-app/src/worktree.rs`) |
| Layer | App |
| Spec Ref | AC-005.3 |
| Action | Split 773-line file into: `gc.rs` (gc function, WorktreeGcResult, helpers), `status.rs` (status function, WorktreeStatusEntry, WorktreeStatus, format_status_table), `mod.rs` (WorktreeError, shared helpers like compact_ts_to_secs/now_secs/is_worktree_stale, re-exports). After removing `ShellWorktreeManager` (FC-010), the file drops to ~637 lines, which splits comfortably into ~200-line files. |

### FC-012: Create InMemoryBacklogRepository

| Attribute | Value |
|-----------|-------|
| File | `crates/ecc-test-support/src/in_memory_backlog.rs` (new) |
| Layer | Test Support |
| Spec Ref | AC-002.3 |
| Action | Implement `BacklogRepository` trait backed by `Mutex<HashMap<String, BacklogEntry>>` and `Mutex<HashMap<String, LockFile>>` for entries and locks. Builder methods: `with_entry()`, `with_lock()`. Include `with_index()` for pre-populating BACKLOG.md content. |

### FC-013: Register InMemoryBacklogRepository

| Attribute | Value |
|-----------|-------|
| File | `crates/ecc-test-support/src/lib.rs` |
| Layer | Test Support |
| Spec Ref | AC-002.3 |
| Action | Add `pub mod in_memory_backlog;` and `pub use in_memory_backlog::InMemoryBacklogRepository;` |

### FC-014: Refactor ecc-app/src/backlog.rs to use BacklogRepository

| Attribute | Value |
|-----------|-------|
| File | `crates/ecc-app/src/backlog.rs` |
| Layer | App |
| Spec Ref | AC-002.4, US-001, US-003 |
| Action | Replace `&dyn FileSystem` parameter with `&dyn BacklogEntryStore + &dyn BacklogLockStore + &dyn BacklogIndexStore` in `next_id`, `check_duplicates`, `reindex`. Remove private `load_entries` function (now on the port trait). Add `list_available()` function accepting `&dyn BacklogEntryStore + &dyn BacklogLockStore + &dyn BacklogIndexStore`, `&dyn WorktreeManager`, `&dyn Clock`, plus `backlog_dir` and `project_dir`. Add `From<FsError> for BacklogError` impl. The `reindex` function gains `&dyn WorktreeManager` and `&dyn Clock` parameters for reconciliation (AC-003.1, AC-003.2, AC-003.3). Flock acquisition happens in the CLI layer before calling `reindex` (AC-003.6). |

Function signatures after refactor:

```rust
pub fn next_id(entries: &dyn BacklogEntryStore, backlog_dir: &Path) -> Result<String, BacklogError>;

pub fn check_duplicates(
    entries: &dyn BacklogEntryStore,
    backlog_dir: &Path,
    query: &str,
    query_tags: &[String],
) -> Result<Vec<DuplicateCandidate>, BacklogError>;

pub fn reindex(
    entries: &dyn BacklogEntryStore,
    locks: &dyn BacklogLockStore,
    index: &dyn BacklogIndexStore,
    worktree_mgr: &dyn WorktreeManager,
    clock: &dyn Clock,
    backlog_dir: &Path,
    project_dir: &Path,
    dry_run: bool,
) -> Result<Option<String>, BacklogError>;

pub fn list_available(
    entries: &dyn BacklogEntryStore,
    locks: &dyn BacklogLockStore,
    worktree_mgr: &dyn WorktreeManager,
    clock: &dyn Clock,
    backlog_dir: &Path,
    project_dir: &Path,
    show_all: bool,
) -> Result<Vec<BacklogEntry>, BacklogError>;
```

### FC-015: Update CLI backlog command

| Attribute | Value |
|-----------|-------|
| File | `crates/ecc-cli/src/commands/backlog.rs` |
| Layer | CLI |
| Spec Ref | US-001 |
| Action | Add `List` variant to `BacklogAction` with `--available` and `--show-all` flags. Wire up `FsBacklogRepository`, `OsWorktreeManager`, `SystemClock` in `run()`. Update existing `NextId`, `CheckDuplicates`, `Reindex` to use `FsBacklogRepository`. Add flock acquisition before `Reindex` (AC-003.6). Output JSON array for `List` (AC-001.5, AC-001.6). |

### FC-016: Consolidate ecc-workflow backlog operations

| Attribute | Value |
|-----------|-------|
| File | `crates/ecc-workflow/src/commands/backlog.rs` |
| Layer | Workflow |
| Spec Ref | US-004 |
| Action | Remove `compute_next_id()` (AC-004.1) and `update_backlog_index()` (AC-004.2). Replace with calls to `ecc_app::backlog::next_id(repo, dir)` and `ecc_app::backlog::reindex(repo, wm, clock, dir, proj, false)` using `FsBacklogRepository` adapter. Replace `std::fs::write` / `std::fs::create_dir_all` / `std::fs::read_dir` with port calls (AC-004.3). Preserve existing flock acquisition (AC-004.4). |

### FC-017: Split ecc-workflow/src/commands/merge.rs

| Attribute | Value |
|-----------|-------|
| File | `crates/ecc-workflow/src/commands/merge.rs`, `crates/ecc-workflow/src/commands/merge_helpers.rs` (new) |
| Layer | Workflow |
| Spec Ref | AC-005.2 |
| Action | Extract git helper functions (`checkout_main`, `merge_fast_forward`, `rebase_onto_main`, etc.) and the `MergeError` enum to `merge_helpers.rs`. Keep `run()` and high-level orchestration in `merge.rs`. Target: both files under 500 lines. Currently 809 lines -- extracting ~300 lines of helpers brings it well under 800. |

**Revised**: The file already has a `merge_cleanup` module (`crate::commands::merge_cleanup`). The split should extract `MergeError` and the helper functions (checkout, rebase, ff-merge, verify steps) to a `merge_steps.rs` or similar. The existing `merge_cleanup.rs` handles post-merge cleanup. This gives three focused files.

## Implementation Phases

### Phase 0: Build Fix (prerequisite)
**Layers**: Test Support
**Files**: FC-001

### Phase 1: Domain Types
**Layers**: Domain
**Files**: FC-002, FC-003, FC-004, FC-005

### Phase 2: Port Trait + Test Double
**Layers**: Port, Test Support
**Files**: FC-006, FC-007, FC-012, FC-013

### Phase 3: Infra Adapter
**Layers**: Infra
**Files**: FC-008, FC-009

### Phase 4: App Layer Refactor
**Layers**: App
**Files**: FC-014

### Phase 5: CLI Integration
**Layers**: CLI
**Files**: FC-015

### Phase 6: Workflow Consolidation
**Layers**: Workflow
**Files**: FC-016

### Phase 7: Structural Cleanup
**Layers**: App, Infra, Workflow
**Files**: FC-010, FC-011, FC-017

## Pass Conditions (TDD Order)

### Phase 0: Build Fix

| PC | Type | Description | AC | Command | Expected |
|----|------|-------------|-----|---------|----------|
| PC-001 | build | ecc-test-support compiles after adding `local_llm: None` | Decision 6 | `cargo build -p ecc-test-support` | success |
| PC-002 | unit | existing InMemoryConfigStore tests pass | Decision 6 | `cargo test -p ecc-test-support --lib in_memory_config_store` | 4 tests pass |

### Phase 1: Domain Types

| PC | Type | Description | AC | Command | Expected |
|----|------|-------------|-----|---------|----------|
| PC-003 | unit | BacklogEntry serializes to JSON with serde_json | AC-001.5 | `cargo test -p ecc-domain --lib backlog::entry::tests::serialize_backlog_entry` | pass |
| PC-004 | unit | BacklogError::Io variant has path and message fields | AC-005.4 | `cargo test -p ecc-domain --lib backlog::entry::tests::backlog_error_io_variant` | pass |
| PC-005 | unit | Existing BacklogError tests updated for new Io variant | AC-005.4 | `cargo test -p ecc-domain --lib backlog::entry::tests::backlog_error_variants` | pass |
| PC-006 | unit | LockFile::parse extracts worktree_name and ISO 8601 timestamp | AC-001.2 | `cargo test -p ecc-domain --lib backlog::lock::tests::parse_valid_lock` | pass |
| PC-007 | unit | LockFile::parse rejects invalid content (missing fields, empty) | AC-001.2 | `cargo test -p ecc-domain --lib backlog::lock::tests::parse_invalid_lock` | pass |
| PC-008 | unit | LockFile::format round-trips through parse | AC-001.2 | `cargo test -p ecc-domain --lib backlog::lock::tests::format_roundtrip` | pass |
| PC-009 | unit | LockFile::is_stale returns true when age > 24h | AC-001.3 | `cargo test -p ecc-domain --lib backlog::lock::tests::stale_after_24h` | pass |
| PC-010 | unit | LockFile::is_stale returns false when age < 24h | AC-001.3 | `cargo test -p ecc-domain --lib backlog::lock::tests::fresh_within_24h` | pass |
| PC-011 | build | ecc-domain compiles with all changes | -- | `cargo build -p ecc-domain` | success |

### Phase 2: Port Trait + Test Double

| PC | Type | Description | AC | Command | Expected |
|----|------|-------------|-----|---------|----------|
| PC-012 | build | BacklogRepository trait compiles in ecc-ports | AC-002.1 | `cargo build -p ecc-ports` | success |
| PC-013 | unit | InMemoryBacklogRepository implements BacklogRepository (compile check) | AC-002.3 | `cargo test -p ecc-test-support --lib in_memory_backlog::tests::implements_trait` | pass |
| PC-014 | unit | InMemoryBacklogRepository load_entries returns seeded entries | AC-002.3 | `cargo test -p ecc-test-support --lib in_memory_backlog::tests::load_entries_returns_seeded` | pass |
| PC-015 | unit | InMemoryBacklogRepository next_id computes max+1 | AC-002.3 | `cargo test -p ecc-test-support --lib in_memory_backlog::tests::next_id_sequential` | pass |
| PC-016 | unit | InMemoryBacklogRepository lock CRUD works | AC-002.3 | `cargo test -p ecc-test-support --lib in_memory_backlog::tests::lock_crud` | pass |
| PC-017 | unit | InMemoryBacklogRepository write_index + read_index round-trip | AC-002.3 | `cargo test -p ecc-test-support --lib in_memory_backlog::tests::index_roundtrip` | pass |

### Phase 3: Infra Adapter

| PC | Type | Description | AC | Command | Expected |
|----|------|-------------|-----|---------|----------|
| PC-018 | unit | FsBacklogRepository::load_entries reads BL-*.md files via InMemoryFileSystem | AC-002.2 | `cargo test -p ecc-infra --lib fs_backlog::tests::load_entries_reads_bl_files` | pass |
| PC-019 | unit | FsBacklogRepository::load_entries skips non-BL and malformed files | AC-002.2 | `cargo test -p ecc-infra --lib fs_backlog::tests::load_entries_skips_invalid` | pass |
| PC-020 | unit | FsBacklogRepository::next_id returns max+1 | AC-002.2 | `cargo test -p ecc-infra --lib fs_backlog::tests::next_id_computes_max_plus_one` | pass |
| PC-021 | unit | FsBacklogRepository::load_lock parses lock file | AC-002.2 | `cargo test -p ecc-infra --lib fs_backlog::tests::load_lock_parses` | pass |
| PC-022 | unit | FsBacklogRepository::save_lock + load_lock round-trip | AC-002.2 | `cargo test -p ecc-infra --lib fs_backlog::tests::lock_roundtrip` | pass |
| PC-023 | unit | FsBacklogRepository::remove_lock deletes file | AC-002.2 | `cargo test -p ecc-infra --lib fs_backlog::tests::remove_lock_deletes` | pass |
| PC-024 | unit | FsBacklogRepository::write_index uses atomic write (temp + rename) | AC-002.2 | `cargo test -p ecc-infra --lib fs_backlog::tests::write_index_atomic` | pass |
| PC-025 | build | ecc-infra compiles with fs_backlog module | -- | `cargo build -p ecc-infra` | success |

### Phase 4: App Layer Refactor

| PC | Type | Description | AC | Command | Expected |
|----|------|-------------|-----|---------|----------|
| PC-026 | unit | next_id(&dyn BacklogEntryStore + &dyn BacklogLockStore + &dyn BacklogIndexStore) works with InMemoryBacklogRepository | AC-002.4 | `cargo test -p ecc-app --lib backlog::tests::next_id_sequential` | pass |
| PC-027 | unit | check_duplicates(&dyn BacklogEntryStore + &dyn BacklogLockStore + &dyn BacklogIndexStore) works | AC-002.4 | `cargo test -p ecc-app --lib backlog::tests::check_duplicates_finds_similar` | pass |
| PC-028 | unit | reindex with WorktreeManager marks matching entries as in-progress | AC-003.2 | `cargo test -p ecc-app --lib backlog::tests::reindex_marks_in_progress_from_worktree` | pass |
| PC-029 | unit | reindex with lock files marks matching entries as in-progress | AC-003.3 | `cargo test -p ecc-app --lib backlog::tests::reindex_marks_in_progress_from_lock` | pass |
| PC-030 | unit | reindex is idempotent (running twice produces same output) | AC-003.4 | `cargo test -p ecc-app --lib backlog::tests::reindex_idempotent` | pass |
| PC-031 | unit | reindex preserves Dependency Graph section | AC-003.5 | `cargo test -p ecc-app --lib backlog::tests::reindex_preserves_dep_graph` | pass |
| PC-032 | unit | list_available excludes items matching active worktree BL-NNN | AC-001.1 | `cargo test -p ecc-app --lib backlog::tests::list_available_excludes_worktree_claims` | pass |
| PC-033 | unit | list_available excludes items with fresh lock files | AC-001.2 | `cargo test -p ecc-app --lib backlog::tests::list_available_excludes_locked` | pass |
| PC-034 | unit | list_available includes items with stale locks (auto-removes lock) | AC-001.3 | `cargo test -p ecc-app --lib backlog::tests::list_available_includes_stale_lock` | pass |
| PC-035 | unit | list_available with show_all=true returns all open items | AC-001.4 | `cargo test -p ecc-app --lib backlog::tests::list_available_show_all` | pass |
| PC-036 | unit | list_available returns empty vec when no open entries | AC-001.6 | `cargo test -p ecc-app --lib backlog::tests::list_available_empty_result` | pass |
| PC-037 | unit | From<FsError> for BacklogError conversion works | AC-005.4 | `cargo test -p ecc-app --lib backlog::tests::fs_error_conversion` | pass |
| PC-038 | unit | All existing ecc-app backlog tests still pass | -- | `cargo test -p ecc-app --lib backlog` | all pass |
| PC-039 | build | ecc-app compiles | -- | `cargo build -p ecc-app` | success |

### Phase 5: CLI Integration

| PC | Type | Description | AC | Command | Expected |
|----|------|-------------|-----|---------|----------|
| PC-040 | build | `ecc backlog list --available` CLI parses | US-001 | `cargo build -p ecc-cli` | success |
| PC-041 | integration | `ecc backlog list --available` outputs JSON array | AC-001.5 | `cargo test -p ecc-cli --lib commands::backlog::tests::list_available_json_output` | pass |
| PC-042 | integration | `ecc backlog list --show-all` returns all open items | AC-001.4 | `cargo test -p ecc-cli --lib commands::backlog::tests::list_show_all` | pass |
| PC-043 | integration | existing CLI tests still pass | -- | `cargo test -p ecc-cli --lib commands::backlog` | all pass |

### Phase 6: Workflow Consolidation

| PC | Type | Description | AC | Command | Expected |
|----|------|-------------|-----|---------|----------|
| PC-044 | unit | compute_next_id function removed from ecc-workflow | AC-004.1 | `cargo build -p ecc-workflow` | success (no compile error for removed fn) |
| PC-045 | unit | update_backlog_index function removed from ecc-workflow | AC-004.2 | grep for `fn update_backlog_index` in `crates/ecc-workflow/src/commands/backlog.rs` returns 0 matches | pass |
| PC-046 | unit | no raw std::fs calls in ecc-workflow backlog module | AC-004.3 | grep for `std::fs::` in `crates/ecc-workflow/src/commands/backlog.rs` returns 0 matches | pass |
| PC-047 | unit | existing ecc-workflow backlog tests pass | AC-004.4 | `cargo test -p ecc-workflow --lib commands::backlog` | all pass |
| PC-048 | build | ecc-workflow compiles | -- | `cargo build -p ecc-workflow` | success |

### Phase 7: Structural Cleanup

| PC | Type | Description | AC | Command | Expected |
|----|------|-------------|-----|---------|----------|
| PC-049 | build | ShellWorktreeManager compiles in ecc-infra | AC-005.1 | `cargo build -p ecc-infra` | success |
| PC-050 | unit | ShellWorktreeManager implements WorktreeManager (existing tests via ecc-app pass) | AC-005.1 | `cargo test -p ecc-app --lib worktree` | all pass |
| PC-051 | lint | ecc-app/src/worktree.rs no longer exists (replaced by module dir) | AC-005.3 | `test ! -f crates/ecc-app/src/worktree.rs` | pass |
| PC-052 | unit | worktree gc tests pass from new location | AC-005.3 | `cargo test -p ecc-app --lib worktree::gc` | all pass |
| PC-053 | unit | worktree status tests pass from new location | AC-005.3 | `cargo test -p ecc-app --lib worktree::status` | all pass |
| PC-054 | lint | merge.rs under 800 lines | AC-005.2 | `wc -l crates/ecc-workflow/src/commands/merge.rs` | < 800 |
| PC-055 | build | full workspace builds | -- | `cargo build` | success |

### Final Verification

| PC | Type | Description | AC | Command | Expected |
|----|------|-------------|-----|---------|----------|
| PC-056 | lint | zero clippy warnings | -- | `cargo clippy -- -D warnings` | success |
| PC-057 | unit | full test suite passes | -- | `cargo test` | all pass |
| PC-058 | build | release build succeeds | -- | `cargo build --release` | success |

## TDD Execution Order

```
PC-001 -> PC-002                          (Phase 0: build fix)
PC-003 -> PC-004 -> PC-005                (Phase 1: Serialize + BacklogError)
PC-006 -> PC-007 -> PC-008 -> PC-009 -> PC-010 -> PC-011  (Phase 1: lock types)
PC-012                                    (Phase 2: port trait compile)
PC-013 -> PC-014 -> PC-015 -> PC-016 -> PC-017  (Phase 2: InMemory test double)
PC-018 -> PC-019 -> PC-020 -> PC-021 -> PC-022 -> PC-023 -> PC-024 -> PC-025  (Phase 3: FsBacklogRepository)
PC-026 -> PC-027 -> PC-028 -> PC-029 -> PC-030 -> PC-031  (Phase 4: app refactor - reindex)
PC-032 -> PC-033 -> PC-034 -> PC-035 -> PC-036 -> PC-037 -> PC-038 -> PC-039  (Phase 4: list_available + compat)
PC-040 -> PC-041 -> PC-042 -> PC-043     (Phase 5: CLI)
PC-044 -> PC-045 -> PC-046 -> PC-047 -> PC-048  (Phase 6: workflow)
PC-049 -> PC-050 -> PC-051 -> PC-052 -> PC-053 -> PC-054 -> PC-055  (Phase 7: structural)
PC-056 -> PC-057 -> PC-058               (Final verification)
```

## Commit Cadence

| Phase | RED commit | GREEN commit | REFACTOR commit |
|-------|-----------|-------------|-----------------|
| 0 | -- | `fix: add missing local_llm field to InMemoryConfigStore tests` | -- |
| 1 | `test: add BacklogEntry serialize and LockFile domain tests` | `feat: add Serialize to BacklogEntry, structured Io error, LockFile domain type` | `refactor: improve BacklogError display format` |
| 2 | `test: add BacklogRepository trait and InMemoryBacklogRepository tests` | `feat: create BacklogRepository port trait and in-memory test double` | -- |
| 3 | `test: add FsBacklogRepository adapter tests` | `feat: implement FsBacklogRepository via FileSystem port` | -- |
| 4 | `test: add list_available and reconciliation tests` | `feat: refactor backlog app layer to use BacklogRepository port` | `refactor: extract From<FsError> impl for BacklogError` |
| 5 | `test: add CLI list --available integration tests` | `feat: add ecc backlog list --available CLI command` | -- |
| 6 | `test: verify workflow backlog delegates to app layer` | `refactor: consolidate ecc-workflow backlog to use BacklogRepository` | -- |
| 7 | -- | `refactor: move ShellWorktreeManager to ecc-infra, split worktree module, split merge.rs` | `chore(scout): remove dead code in worktree module` |

## E2E Assessment

- **Touches user-facing flows?** Yes -- new `ecc backlog list --available` CLI command
- **Crosses 3+ modules?** Yes -- domain, ports, infra, app, cli, workflow (6 crates)
- **New E2E tests needed?** No -- the feature is internal tooling for Claude agents, not end-user-facing. CLI integration tests in Phase 5 are sufficient. The command is consumed by agents, not humans.

## Risks & Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| BacklogError variant change breaks callers across 3 crates | HIGH | Phase 1 updates all callers atomically; `cargo build` after each change confirms no breakage |
| ecc-workflow tests use `std::fs` directly (tempfile-based); refactoring to port may break them | MEDIUM | Keep tempfile-based tests for ecc-workflow (integration), add new unit tests using InMemoryBacklogRepository |
| ShellWorktreeManager move breaks ecc-app compilation | LOW | No production code in ecc-app imports it; only defined there. Move is additive to infra, subtractive from app. |
| ecc-app Cargo.toml may need ecc-infra as dev-dependency for ShellWorktreeManager tests | LOW | Verify test imports; if needed, add `ecc-infra = { workspace = true }` under `[dev-dependencies]` |
| reindex signature change (new parameters) breaks all callers | MEDIUM | Phase 4 updates callers in ecc-app tests; Phase 5+6 update CLI and workflow callers |
| Domain layer accidentally imports ecc-ports for FsError | HIGH | Keep `From<FsError>` impl in ecc-app (not domain). CI `cargo deny` and manual review ensure clean dependency graph |

## Success Criteria

- [ ] `cargo build` succeeds for all crates
- [ ] `cargo test` passes all existing + new tests
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `ecc backlog list --available` returns JSON array of available entries
- [ ] `ecc backlog reindex` derives in-progress from worktree state
- [ ] No `std::fs` calls in `ecc-workflow/src/commands/backlog.rs`
- [ ] `ShellWorktreeManager` lives in `ecc-infra`
- [ ] `ecc-app/src/worktree.rs` replaced by module directory
- [ ] `merge.rs` under 800 lines
- [ ] Domain crate has zero I/O imports
- [ ] All files under 800 lines

## SOLID Assessment

ISP violation identified (10-method `BacklogRepository`): **addressed** by splitting into 3 focused traits (`BacklogEntryStore`, `BacklogLockStore`, `BacklogIndexStore`). `FsBacklogRepository` implements all three. Each use case accepts only the trait(s) it consumes. All other SOLID principles PASS. Clean Architecture dependency rules PASS. Component principles PASS (ADP, SDP, SAP, REP, CCP).

## Robert's Oath Check

CLEAN — 0 oath warnings. Design self-corrected FC-003 (domain can't depend on ports). TDD approach with 59 PCs. Atomic commits per phase. Rework ratio 0.16 (healthy). 2 self-audit findings (LOW/MEDIUM, not blocking this design).

## Security Notes

CLEAR — 2 LOW findings:
1. Lock file parsing: validate ISO 8601 timestamp during parse(), not deferred to is_stale(). Handle \r\n line endings.
2. TOCTOU in stale lock removal: acceptable for advisory locking. list_available is read-heavy, worst case is last-writer-wins on concurrent lock creation.

No secrets, no shell injection, no path traversal (BL-NNN regex validates IDs), atomic writes via temp+rename.

## Rollback Plan

Reverse dependency order (undo last phase first):
1. Revert FC-017 (merge.rs split)
2. Revert FC-011 (worktree module split)
3. Revert FC-010 (ShellWorktreeManager move)
4. Revert FC-016 (workflow consolidation)
5. Revert FC-015 (CLI list command)
6. Revert FC-014 (app refactor)
7. Revert FC-008, FC-009 (infra adapter)
8. Revert FC-006, FC-007, FC-012, FC-013 (port traits + test doubles)
9. Revert FC-002, FC-003, FC-004, FC-005 (domain types)
10. FC-001 (build fix) — keep regardless

## Bounded Contexts Affected

| Context | Role | Files Modified |
|---------|------|----------------|
| Backlog | entity, value object, port | `ecc-domain/src/backlog/{entry.rs, lock.rs, mod.rs}`, `ecc-ports/src/backlog.rs`, `ecc-app/src/backlog.rs`, `ecc-infra/src/fs_backlog.rs`, `ecc-cli/src/commands/backlog.rs`, `ecc-workflow/src/commands/backlog.rs` |
| Worktree | adapter move, module split | `ecc-app/src/worktree/{mod.rs, gc.rs, status.rs}`, `ecc-infra/src/shell_worktree.rs` |
| Workflow | merge split | `ecc-workflow/src/commands/merge.rs`, `ecc-workflow/src/commands/merge_steps.rs` |

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/ARCHITECTURE.md` | Architecture | Update | Add 3 backlog port traits to ports listing | US-002 |
| 2 | `docs/adr/0059-backlog-repository-port.md` | Decision | Create | ADR for promoting backlog to full hexagonal concern | Decision 1 |
| 3 | `docs/domain/bounded-contexts.md` | Domain | Update | Add backlog context (fixes DOC-006) | US-002 |
| 4 | `docs/commands-reference.md` | Reference | Update | Add `ecc backlog list --available [--show-all]` | US-001 |
| 5 | `CLAUDE.md` | Onboarding | Update | Update test count, add list --available | US-001 |
| 6 | `CHANGELOG.md` | Changelog | Update | New CLI command + backlog hexagonal promotion | All |

## E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | BacklogEntryStore | FsBacklogRepository | BacklogEntryStore | Verify reads/writes BL-*.md on real fs | ignored | FsBacklogRepository modified |
| 2 | WorktreeManager→reindex | OsWorktreeManager | WorktreeManager | Verify reindex derives in-progress from git worktrees | ignored | reindex signature changed |
| 3 | CLI JSON | ecc-cli | All backlog ports | Verify list --available outputs valid JSON | ignored | CLI backlog modified |

## E2E Activation Rules

All 3 E2E boundaries touched. Un-ignore tests 1-3 during implementation.
