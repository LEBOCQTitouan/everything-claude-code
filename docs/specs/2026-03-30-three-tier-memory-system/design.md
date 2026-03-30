# Design: Three-Tier Memory System (BL-093)

## Overview

Implements a SQLite FTS5-backed memory system with three tiers (working/episodic/semantic), session-end consolidation, context injection, legacy migration, and CLI management. Follows hexagonal architecture: pure domain types in `ecc-domain`, port trait in `ecc-ports`, SQLite adapter in `ecc-infra`, use cases in `ecc-app`, CLI wiring in `ecc-cli`, test double in `ecc-test-support`.

## Sub-Spec Phases

| Phase | Stories | Scope |
|-------|---------|-------|
| A | US-001, US-002, US-007 | Foundation: domain types, port trait, SQLite adapter, CRUD + search + migration + gc CLI |
| B | US-003, US-004 | Consolidation: session-end dedup/decay, CONTEXT.md generation |
| C | US-005, US-006 | Classification: tier promotion, working memory expiry, session-start injection |

---

## Table 1: File Changes

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `Cargo.toml` (workspace) | MODIFY | Add `rusqlite = { version = "0.34", features = ["bundled", "modern_sqlite"] }` to `[workspace.dependencies]` | AC-001.1 |
| 2 | `crates/ecc-domain/src/lib.rs` | MODIFY | Add `pub mod memory;` | AC-001.7 |
| 3 | `crates/ecc-domain/src/memory/mod.rs` | CREATE | Module root: re-exports entry, tier, consolidation, classification, error submodules | AC-001.7 |
| 4 | `crates/ecc-domain/src/memory/entry.rs` | CREATE | `MemoryEntry` struct (id, title, content, tier, tags, project_id, session_id, relevance_score, created_at, updated_at, stale, related_work_items, source_path), `MemoryId` newtype | AC-001.1, AC-001.7 |
| 5 | `crates/ecc-domain/src/memory/tier.rs` | CREATE | `MemoryTier` enum (Working, Episodic, Semantic), Display, FromStr | AC-005.1, AC-005.2 |
| 6 | `crates/ecc-domain/src/memory/consolidation.rs` | CREATE | Pure functions: `jaccard_3gram_similarity`, `recency_factor`, `compute_relevance_score`, `should_merge`, `should_mark_stale`, `is_short_entry` | AC-003.1, AC-003.2, AC-003.3, AC-003.6 |
| 7 | `crates/ecc-domain/src/memory/classification.rs` | CREATE | Pure functions: `can_promote`, `promote_tier`, `should_expire_working` | AC-005.3, AC-005.4, AC-005.5 |
| 8 | `crates/ecc-domain/src/memory/error.rs` | CREATE | `MemoryError` enum: NotFound, AlreadySemantic, DatabaseCorrupted, MigrationFailed, InvalidTier, InvalidId, ExportFailed | AC-001.8, AC-005.5, AC-007.5 |
| 9 | `crates/ecc-domain/src/memory/context.rs` | CREATE | Pure function: `format_context_md` renders top-N entries as markdown, truncates to 200 lines | AC-004.1, AC-004.2, AC-004.4, AC-004.5 |
| 10 | `crates/ecc-domain/src/memory/migration.rs` | CREATE | Pure functions: `parse_work_item_md`, `parse_action_log_entry`, `extract_work_item_refs` | AC-002.1, AC-002.2, AC-002.4, AC-002.5 |
| 11 | `crates/ecc-domain/src/memory/export.rs` | CREATE | Pure function: `format_entry_as_md` renders a MemoryEntry to exportable markdown | AC-002.6, AC-002.7 |
| 12 | `crates/ecc-domain/src/memory/stats.rs` | CREATE | `MemoryStats` struct (counts by tier, stale_count, db_size_bytes, oldest, newest) | AC-007.4 |
| 13 | `crates/ecc-ports/src/lib.rs` | MODIFY | Add `pub mod memory_store;` | AC-001.6 |
| 14 | `crates/ecc-ports/src/memory_store.rs` | CREATE | `MemoryStore` trait: insert, get, update, delete, search_fts, list_filtered, list_recent, count_by_tier, stats, get_by_source_path; `MemoryStoreError` enum | AC-001.1, AC-001.3, AC-001.4, AC-001.6 |
| 15 | `crates/ecc-infra/Cargo.toml` | MODIFY | Add `rusqlite = { workspace = true }` | AC-001.1 |
| 16 | `crates/ecc-infra/src/lib.rs` | MODIFY | Add `pub mod sqlite_memory;` | AC-001.6 |
| 17 | `crates/ecc-infra/src/sqlite_memory.rs` | CREATE | `SqliteMemoryStore` implementing `MemoryStore`: auto-migration, FTS5 virtual table, WAL mode, corruption detection + recovery, unicode61 tokenizer | AC-001.1, AC-001.2, AC-001.3, AC-001.5, AC-001.8, AC-001.10 |
| 18 | `crates/ecc-test-support/Cargo.toml` | MODIFY | Add `ecc-domain = { workspace = true }` (needed for MemoryEntry in test double) | AC-001.6 |
| 19 | `crates/ecc-test-support/src/lib.rs` | MODIFY | Add `pub mod in_memory_memory_store;` and `pub use in_memory_memory_store::InMemoryMemoryStore;` | AC-001.6 |
| 20 | `crates/ecc-test-support/src/in_memory_memory_store.rs` | CREATE | `InMemoryMemoryStore` implementing `MemoryStore` via `HashMap` for deterministic unit tests | AC-001.6 |
| 21 | `crates/ecc-app/Cargo.toml` | MODIFY | Add `ecc-domain` features if needed (already dep) | — |
| 22 | `crates/ecc-app/src/lib.rs` | MODIFY | Add `pub mod memory;` | AC-001.6 |
| 23 | `crates/ecc-app/src/memory.rs` | CREATE | Use cases: `add`, `search`, `list`, `delete`, `gc`, `stats`, `migrate`, `export`, `consolidate`, `generate_context`, `promote`, `inject_context` | AC-001.1, AC-001.3, AC-001.4, AC-001.9, AC-002.1-7, AC-003.1-7, AC-004.1-5, AC-005.1-5, AC-006.1-5, AC-007.1-5 |
| 24 | `crates/ecc-cli/Cargo.toml` | MODIFY | Add `rusqlite = { workspace = true }` (for constructing SqliteMemoryStore) | — |
| 25 | `crates/ecc-cli/src/commands/mod.rs` | MODIFY | Add `pub mod memory;` | — |
| 26 | `crates/ecc-cli/src/commands/memory.rs` | CREATE | `MemoryArgs`, `MemoryAction` (Add, Search, List, Delete, Gc, Stats, Migrate, Export, Promote), routes to `ecc_app::memory::*` | AC-001.1, AC-001.3, AC-001.4, AC-001.9, AC-002.6, AC-007.1-5 |
| 27 | `crates/ecc-cli/src/main.rs` | MODIFY | Add `Memory(commands::memory::MemoryArgs)` variant to `Command` enum and match arm | — |
| 28 | `hooks/hooks.json` | MODIFY | Add consolidation to Stop hook, injection to SessionStart hook | AC-003.4, AC-006.1 |

---

## Table 2: Pass Conditions

### Sub-Spec A: Foundation (US-001, US-002, US-007)

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | `MemoryTier` enum has Working, Episodic, Semantic variants; Display + FromStr round-trip | AC-005.1, AC-005.2 | `cargo test -p ecc-domain memory::tier` | PASS |
| PC-002 | unit | `MemoryTier::from_str` defaults are correct; unknown string returns `InvalidTier` error | AC-005.2 | `cargo test -p ecc-domain memory::tier` | PASS |
| PC-003 | unit | `MemoryEntry` construction with all fields; immutable struct, Clone + Debug | AC-001.1, AC-001.7 | `cargo test -p ecc-domain memory::entry` | PASS |
| PC-004 | unit | `MemoryId` newtype wraps i64, Display, Eq, Hash | AC-001.1 | `cargo test -p ecc-domain memory::entry` | PASS |
| PC-005 | unit | `MemoryError` variants all implement Display with meaningful messages | AC-001.8, AC-007.5 | `cargo test -p ecc-domain memory::error` | PASS |
| PC-006 | unit | `recency_factor(0)` = 1.0, `recency_factor(365)` = 0.0, `recency_factor(180)` ~ 0.507 | AC-003.3 | `cargo test -p ecc-domain memory::consolidation` | PASS |
| PC-007 | unit | `compute_relevance_score(age_days=0, ref_count=0)` = 1.0; `(age_days=0, ref_count=5)` = 1.5 | AC-003.3 | `cargo test -p ecc-domain memory::consolidation` | PASS |
| PC-008 | unit | `jaccard_3gram_similarity("hello world foo", "hello world foo")` = 1.0; disjoint strings = 0.0 | AC-003.1 | `cargo test -p ecc-domain memory::consolidation` | PASS |
| PC-009 | unit | `should_merge` returns true when Jaccard > 0.8, false when <= 0.8 | AC-003.1 | `cargo test -p ecc-domain memory::consolidation` | PASS |
| PC-010 | unit | `is_short_entry` returns true for entries with <10 words, false for >=10 | AC-003.6 | `cargo test -p ecc-domain memory::consolidation` | PASS |
| PC-011 | unit | `should_mark_stale(age_days=91, ref_count=0)` = true; `(age_days=91, ref_count=1)` = false; `(age_days=89, ref_count=0)` = false | AC-003.2 | `cargo test -p ecc-domain memory::consolidation` | PASS |
| PC-012 | unit | `parse_work_item_md` extracts title, content, BL-NNN references from markdown | AC-002.1, AC-002.4 | `cargo test -p ecc-domain memory::migration` | PASS |
| PC-013 | unit | `parse_action_log_entry` extracts action fields from JSON; malformed JSON returns None | AC-002.2, AC-002.5 | `cargo test -p ecc-domain memory::migration` | PASS |
| PC-014 | unit | `extract_work_item_refs` finds all BL-NNN patterns in content | AC-002.4 | `cargo test -p ecc-domain memory::migration` | PASS |
| PC-015 | unit | `format_entry_as_md` produces valid markdown with tier, title, tags, content | AC-002.6, AC-002.7 | `cargo test -p ecc-domain memory::export` | PASS |
| PC-016 | unit | `format_context_md` with 0 entries returns "No memories stored" | AC-004.4 | `cargo test -p ecc-domain memory::context` | PASS |
| PC-017 | unit | `format_context_md` with 15 entries truncates to top-10 within 200 lines | AC-004.2, AC-004.5 | `cargo test -p ecc-domain memory::context` | PASS |
| PC-018 | unit | `format_context_md` entries show tier, title, relevance score, truncated content | AC-004.5 | `cargo test -p ecc-domain memory::context` | PASS |
| PC-019 | unit | `MemoryStats` struct holds counts_by_tier, stale_count, db_size_bytes, oldest, newest | AC-007.4 | `cargo test -p ecc-domain memory::stats` | PASS |
| PC-020 | arch | `ecc-domain/src/memory/` has zero `use std::fs`, `use std::io`, `use std::net` imports | AC-001.7 | `grep -rn "use std::fs\|use std::io\|use std::net" crates/ecc-domain/src/memory/` | 0 matches |
| PC-021 | unit | `MemoryStore` trait compiles with methods: insert, get, update, delete, search_fts, list_filtered, list_recent, count_by_tier, stats, get_by_source_path | AC-001.6 | `cargo test -p ecc-ports` | PASS |
| PC-022 | unit | `MemoryStoreError` enum covers NotFound, Database, Corruption variants | AC-001.6 | `cargo test -p ecc-ports memory_store` | PASS |
| PC-023 | unit | `InMemoryMemoryStore` implements all `MemoryStore` methods; insert + get round-trip | — | `cargo test -p ecc-test-support in_memory_memory_store` | PASS |
| PC-024 | unit | `InMemoryMemoryStore::search_fts` does substring match as FTS5 approximation | AC-001.3 | `cargo test -p ecc-test-support in_memory_memory_store` | PASS |
| PC-025 | unit | `InMemoryMemoryStore::list_filtered` filters by tier and tag | AC-001.4 | `cargo test -p ecc-test-support in_memory_memory_store` | PASS |
| PC-026 | integration | `SqliteMemoryStore::new` creates DB file + FTS5 table if missing (auto-migration) | AC-001.2 | `cargo test -p ecc-infra sqlite_memory` | PASS |
| PC-027 | integration | `SqliteMemoryStore` insert + search_fts returns BM25-ranked results for "warn block" | AC-001.1, AC-001.3 | `cargo test -p ecc-infra sqlite_memory` | PASS |
| PC-028 | integration | `SqliteMemoryStore` list_filtered with type=semantic, tag="rust" returns only matching | AC-001.4 | `cargo test -p ecc-infra sqlite_memory` | PASS |
| PC-029 | integration | `SqliteMemoryStore` enables WAL mode; PRAGMA journal_mode returns "wal" | AC-001.5 | `cargo test -p ecc-infra sqlite_memory` | PASS |
| PC-030 | integration | `SqliteMemoryStore` detects corruption, backs up as `.corrupt`, recreates empty DB | AC-001.8 | `cargo test -p ecc-infra sqlite_memory` | PASS |
| PC-031 | integration | `SqliteMemoryStore` search with no results returns empty vec (not error) | AC-001.9 | `cargo test -p ecc-infra sqlite_memory` | PASS |
| PC-032 | integration | `SqliteMemoryStore` stores and retrieves Unicode content (emoji, CJK) via FTS5 | AC-001.10 | `cargo test -p ecc-infra sqlite_memory` | PASS |
| PC-033 | integration | `SqliteMemoryStore::delete` removes from both main table and FTS index | AC-007.1 | `cargo test -p ecc-infra sqlite_memory` | PASS |
| PC-034 | unit | App `add` use case inserts entry with type, content, tags, relevance_score=1.0 | AC-001.1 | `cargo test -p ecc-app memory` | PASS |
| PC-035 | unit | App `search` use case returns FTS results; empty result returns empty vec + no error | AC-001.3, AC-001.9 | `cargo test -p ecc-app memory` | PASS |
| PC-036 | unit | App `list` use case filters by type and tag | AC-001.4 | `cargo test -p ecc-app memory` | PASS |
| PC-037 | unit | App `delete` use case removes entry; non-existent ID returns NotFound error | AC-007.1, AC-007.5 | `cargo test -p ecc-app memory` | PASS |
| PC-038 | unit | App `gc` use case deletes stale entries >180 days | AC-007.2 | `cargo test -p ecc-app memory` | PASS |
| PC-039 | unit | App `gc --dry-run` reports without deleting | AC-007.3 | `cargo test -p ecc-app memory` | PASS |
| PC-040 | unit | App `stats` use case returns counts by type, stale count, db size, dates | AC-007.4 | `cargo test -p ecc-app memory` | PASS |
| PC-041 | unit | App `migrate` converts work-item markdown files to episodic entries | AC-002.1 | `cargo test -p ecc-app memory` | PASS |
| PC-042 | unit | App `migrate` converts action-log.json entries to episodic entries | AC-002.2 | `cargo test -p ecc-app memory` | PASS |
| PC-043 | unit | App `migrate` is idempotent (keyed on source_path) | AC-002.3 | `cargo test -p ecc-app memory` | PASS |
| PC-044 | unit | App `migrate` populates related_work_items from BL-NNN refs | AC-002.4 | `cargo test -p ecc-app memory` | PASS |
| PC-045 | unit | App `migrate` skips malformed action-log entries, reports count | AC-002.5 | `cargo test -p ecc-app memory` | PASS |
| PC-046 | unit | App `export` writes individual markdown files grouped by tier | AC-002.6 | `cargo test -p ecc-app memory` | PASS |
| PC-047 | unit | App `export` then re-import is lossless (round-trip) | AC-002.7 | `cargo test -p ecc-app memory` | PASS |
| PC-048 | unit | CLI `ecc memory add` routes to app use case, parses --type/--title/--tags flags | — | `cargo test -p ecc-cli commands::memory` | PASS |
| PC-049 | unit | CLI `ecc memory search` routes to app use case | — | `cargo test -p ecc-cli commands::memory` | PASS |
| PC-050 | unit | CLI `ecc memory list --type semantic --tag rust` routes with filters | — | `cargo test -p ecc-cli commands::memory` | PASS |
| PC-051 | unit | CLI `ecc memory delete <id>` routes, non-existent ID prints error | — | `cargo test -p ecc-cli commands::memory` | PASS |
| PC-052 | unit | CLI `ecc memory gc [--dry-run]` routes correctly | — | `cargo test -p ecc-cli commands::memory` | PASS |
| PC-053 | unit | CLI `ecc memory stats` routes and prints formatted output | — | `cargo test -p ecc-cli commands::memory` | PASS |
| PC-054 | unit | CLI `ecc memory migrate` routes to app use case | — | `cargo test -p ecc-cli commands::memory` | PASS |
| PC-055 | unit | CLI `ecc memory export --output ./backup/` routes correctly | — | `cargo test -p ecc-cli commands::memory` | PASS |
| PC-056 | build | `cargo build --workspace` succeeds with zero errors | — | `cargo build --workspace` | exit 0 |
| PC-057 | lint | `cargo clippy --workspace -- -D warnings` passes with zero warnings | — | `cargo clippy --workspace -- -D warnings` | exit 0 |

### Sub-Spec B: Consolidation (US-003, US-004)

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-058 | unit | `can_promote` returns Working->Episodic, Episodic->Semantic, Semantic->None | AC-005.3, AC-005.5 | `cargo test -p ecc-domain memory::classification` | PASS |
| PC-059 | unit | `promote_tier` boosts relevance_score 2x for Episodic->Semantic | AC-005.3 | `cargo test -p ecc-domain memory::classification` | PASS |
| PC-060 | unit | `should_expire_working(age_hours=25, content_len=60)` = promote; `(25, 30)` = delete; `(23, 60)` = keep | AC-005.4 | `cargo test -p ecc-domain memory::classification` | PASS |
| PC-061 | unit | App `consolidate` merges entries with >80% Jaccard 3-gram similarity, keeps newer | AC-003.1 | `cargo test -p ecc-app memory` | PASS |
| PC-062 | unit | App `consolidate` marks entries >90 days with zero references as stale (not deleted) | AC-003.2 | `cargo test -p ecc-app memory` | PASS |
| PC-063 | unit | App `consolidate` recalculates relevance_score with correct formula | AC-003.3 | `cargo test -p ecc-app memory` | PASS |
| PC-064 | unit | App `consolidate` processes at most 100 most-recent entries | AC-003.4 | `cargo test -p ecc-app memory` | PASS |
| PC-065 | unit | App `consolidate` acquires try-lock; if held, returns Ok(skipped) | AC-003.5 | `cargo test -p ecc-app memory` | PASS |
| PC-066 | unit | App `consolidate` skips dedup for entries with <10 words | AC-003.6 | `cargo test -p ecc-app memory` | PASS |
| PC-067 | integration | `SqliteMemoryStore` merge operation runs within SQLite transaction | AC-003.7 | `cargo test -p ecc-infra sqlite_memory` | PASS |
| PC-068 | unit | App `generate_context` writes CONTEXT.md with top-10 by relevance_score * recency_factor | AC-004.1 | `cargo test -p ecc-app memory` | PASS |
| PC-069 | unit | App `generate_context` truncates to 200 lines | AC-004.2 | `cargo test -p ecc-app memory` | PASS |
| PC-070 | unit | App `generate_context` does NOT modify MEMORY.md | AC-004.3 | `cargo test -p ecc-app memory` | PASS |
| PC-071 | unit | App `generate_context` writes "No memories stored" when DB empty | AC-004.4 | `cargo test -p ecc-app memory` | PASS |
| PC-072 | unit | App `generate_context` entries show tier, title, score, truncated content | AC-004.5 | `cargo test -p ecc-app memory` | PASS |
| PC-073 | build | `cargo build --workspace` succeeds after Sub-Spec B | — | `cargo build --workspace` | exit 0 |
| PC-074 | lint | `cargo clippy --workspace -- -D warnings` passes after Sub-Spec B | — | `cargo clippy --workspace -- -D warnings` | exit 0 |

### Sub-Spec C: Classification + Injection (US-005, US-006)

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-075 | unit | App `add` with --type working sets tier to Working | AC-005.1 | `cargo test -p ecc-app memory` | PASS |
| PC-076 | unit | App `add` with no --type defaults to Episodic | AC-005.2 | `cargo test -p ecc-app memory` | PASS |
| PC-077 | unit | App `promote` changes Episodic to Semantic, boosts relevance 2x | AC-005.3 | `cargo test -p ecc-app memory` | PASS |
| PC-078 | unit | App `promote` on Semantic returns "Already semantic" message | AC-005.5 | `cargo test -p ecc-app memory` | PASS |
| PC-079 | unit | App `consolidate` promotes working entries >24h with >50 chars to episodic, deletes short ones | AC-005.4 | `cargo test -p ecc-app memory` | PASS |
| PC-080 | unit | App `inject_context` queries top-10 by `relevance_score * recency_factor` | AC-006.1 | `cargo test -p ecc-app memory` | PASS |
| PC-081 | unit | App `inject_context` boosts current project_id memories 1.5x | AC-006.2 | `cargo test -p ecc-app memory` | PASS |
| PC-082 | unit | App `inject_context` outputs `## Relevant Memories` markdown section | AC-006.3 | `cargo test -p ecc-app memory` | PASS |
| PC-083 | unit | App `inject_context` with no DB returns empty string, exit 0 | AC-006.4 | `cargo test -p ecc-app memory` | PASS |
| PC-084 | unit | App `inject_context` truncates output to 5000 chars | AC-006.5 | `cargo test -p ecc-app memory` | PASS |
| PC-085 | build | `cargo build --workspace` succeeds after Sub-Spec C | — | `cargo build --workspace` | exit 0 |
| PC-086 | lint | `cargo clippy --workspace -- -D warnings` passes after Sub-Spec C | — | `cargo clippy --workspace -- -D warnings` | exit 0 |
| PC-087 | arch | No SQLite imports in ecc-domain or ecc-app crates | AC-001.6 | `grep -rn "rusqlite" crates/ecc-domain/ crates/ecc-app/` | 0 matches |

---

## TDD Dependency Order

```
Phase A (13 TDD cycles):
  A1. ecc-domain/memory/error.rs        [Entity]           PC-005
  A2. ecc-domain/memory/tier.rs         [Entity]           PC-001, PC-002
  A3. ecc-domain/memory/entry.rs        [Entity]           PC-003, PC-004
  A4. ecc-domain/memory/consolidation.rs [Entity]          PC-006..PC-011
  A5. ecc-domain/memory/migration.rs    [Entity]           PC-012..PC-014
  A6. ecc-domain/memory/export.rs       [Entity]           PC-015
  A7. ecc-domain/memory/context.rs      [Entity]           PC-016..PC-018
  A8. ecc-domain/memory/stats.rs        [Entity]           PC-019
  A9. ecc-ports/memory_store.rs         [UseCase boundary] PC-021, PC-022
  A10. ecc-test-support/in_memory_memory_store.rs [Adapter] PC-023..PC-025
  A11. ecc-infra/sqlite_memory.rs       [Framework]        PC-026..PC-033, PC-067
  A12. ecc-app/memory.rs (CRUD+migrate+export+gc+stats) [UseCase] PC-034..PC-047
  A13. ecc-cli/commands/memory.rs       [Adapter]          PC-048..PC-055
  A14. Arch check + build + lint                           PC-020, PC-056, PC-057

Phase B (6 TDD cycles):
  B1. ecc-app/memory.rs (consolidate)   [UseCase]          PC-061..PC-066
  B2. ecc-app/memory.rs (generate_context) [UseCase]       PC-068..PC-072
  B3. Build + lint                                         PC-073, PC-074

Phase C (5 TDD cycles):
  C1. ecc-domain/memory/classification.rs [Entity]         PC-058..PC-060
  C2. ecc-app/memory.rs (promote, working expiry) [UseCase] PC-075..PC-079
  C3. ecc-app/memory.rs (inject_context) [UseCase]         PC-080..PC-084
  C4. hooks/hooks.json update           [Framework]        —
  C5. Arch check + build + lint                            PC-085..PC-087
```

## Layer Coverage Per Phase

| Phase | Layers Touched | Within 2-layer limit? |
|-------|---------------|----------------------|
| A1-A8 | Entity | Yes |
| A9 | UseCase (port definition) | Yes |
| A10 | Adapter (test double) | Yes |
| A11 | Framework | Yes |
| A12 | UseCase | Yes |
| A13 | Adapter (CLI) | Yes |
| B1-B2 | UseCase | Yes |
| C1 | Entity | Yes |
| C2-C3 | UseCase | Yes |
| C4 | Framework | Yes |

## MemoryStore Port Trait Signature

```rust
pub trait MemoryStore: Send + Sync {
    fn insert(&self, entry: &MemoryEntry) -> Result<MemoryId, MemoryStoreError>;
    fn get(&self, id: MemoryId) -> Result<MemoryEntry, MemoryStoreError>;
    fn update(&self, entry: &MemoryEntry) -> Result<(), MemoryStoreError>;
    fn delete(&self, id: MemoryId) -> Result<(), MemoryStoreError>;
    fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, MemoryStoreError>;
    fn list_filtered(
        &self,
        tier: Option<MemoryTier>,
        tag: Option<&str>,
        project_id: Option<&str>,
    ) -> Result<Vec<MemoryEntry>, MemoryStoreError>;
    fn list_recent(&self, limit: usize) -> Result<Vec<MemoryEntry>, MemoryStoreError>;
    fn count_by_tier(&self) -> Result<HashMap<MemoryTier, usize>, MemoryStoreError>;
    fn stats(&self) -> Result<MemoryStats, MemoryStoreError>;
    fn get_by_source_path(&self, path: &str) -> Result<Option<MemoryEntry>, MemoryStoreError>;
    fn delete_stale_older_than(&self, days: u64) -> Result<Vec<MemoryEntry>, MemoryStoreError>;
    fn merge_entries(
        &self,
        keep_id: MemoryId,
        remove_id: MemoryId,
        merged_content: &str,
    ) -> Result<(), MemoryStoreError>;
}
```

## SQLite Schema

```sql
CREATE TABLE IF NOT EXISTS memories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    tier TEXT NOT NULL CHECK(tier IN ('working', 'episodic', 'semantic')),
    tags TEXT NOT NULL DEFAULT '',          -- comma-separated
    project_id TEXT,
    session_id TEXT,
    relevance_score REAL NOT NULL DEFAULT 1.0,
    created_at TEXT NOT NULL,              -- ISO 8601
    updated_at TEXT NOT NULL,              -- ISO 8601
    stale INTEGER NOT NULL DEFAULT 0,
    related_work_items TEXT NOT NULL DEFAULT '', -- comma-separated BL-NNN
    source_path TEXT                        -- for migration idempotency
);

CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
    title, content, tags,
    content='memories',
    content_rowid='id',
    tokenize='unicode61'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS memories_ai AFTER INSERT ON memories BEGIN
    INSERT INTO memories_fts(rowid, title, content, tags)
    VALUES (new.id, new.title, new.content, new.tags);
END;

CREATE TRIGGER IF NOT EXISTS memories_ad AFTER DELETE ON memories BEGIN
    INSERT INTO memories_fts(memories_fts, rowid, title, content, tags)
    VALUES ('delete', old.id, old.title, old.content, old.tags);
END;

CREATE TRIGGER IF NOT EXISTS memories_au AFTER UPDATE ON memories BEGIN
    INSERT INTO memories_fts(memories_fts, rowid, title, content, tags)
    VALUES ('delete', old.id, old.title, old.content, old.tags);
    INSERT INTO memories_fts(rowid, title, content, tags)
    VALUES (new.id, new.title, new.content, new.tags);
END;
```

## Commit Cadence

Each TDD cycle (A1, A2, ..., C5) produces:
1. `test: add <module> tests` (RED)
2. `feat: implement <module>` (GREEN)
3. `refactor: improve <module>` (REFACTOR, only if needed)

Phase gate commits:
- `chore: add rusqlite workspace dependency` (before A11)
- `docs: update CLAUDE.md with ecc memory commands` (after C5)

## Review Findings (Incorporated)

### SOLID Assessment
NEEDS WORK (addressed):
1. CRITICAL: Split `ecc-app/src/memory.rs` into `memory/` directory with 4 submodules (crud, lifecycle, migration, consolidation) — SRP compliance
2. HIGH: Remove `rusqlite` from `ecc-cli/Cargo.toml` — only `ecc-infra` needs it
3. MEDIUM: ISP trade-off on MemoryStore (12 methods) — documented, split threshold at 15
4. MEDIUM: Consider merging classification.rs + consolidation.rs in domain if they always change together

### Robert's Oath Check
CLEAN — 0 warnings. All 9 oath items satisfied. 100% coverage achievable. Breaking change handled responsibly (export before break). Rework ratio 0.10.

### Security Notes
NOT CLEAR (addressed with 3 additional PCs):
- HIGH: Add `contains_likely_secret(content)` domain function + gate in `add` use case (PC-088)
- MEDIUM: Sanitize FTS5 queries by quoting user input before MATCH (PC-089)
- MEDIUM: Set DB directory to 0700, file to 0600 permissions (PC-090)
- LOW: Canonicalize migration source paths (defense-in-depth)
- CLEAR: SQL injection (rusqlite parameterized queries)

### Additional Pass Conditions
| ID | Type | Description | Verifies |
|----|------|-------------|----------|
| PC-088 | unit | `contains_likely_secret` detects API keys (sk-, ghp_, AKIA), connection strings, bearer tokens | Security |
| PC-089 | unit | FTS5 query sanitization quotes user input, strips operators | Security |
| PC-090 | integration | DB dir has 0700 perms, file has 0600 perms | Security |

## Rollback Plan

Reverse dependency order:
1. Revert hooks.json changes (SessionStart/Stop)
2. Revert ecc-cli/commands/memory.rs
3. Revert ecc-app/src/memory/
4. Revert ecc-infra/sqlite_memory.rs
5. Revert ecc-test-support/in_memory_memory.rs
6. Revert ecc-ports/memory_store.rs
7. Revert ecc-domain/src/memory/
8. Revert Cargo.toml rusqlite dependency
9. Delete ~/.ecc/memory/memory.db (user action)

## Doc Update Plan

| # | Doc File | Level | Action | Spec Ref |
|---|----------|-------|--------|----------|
| 1 | CLAUDE.md | Project | Add ecc memory subcommands | US-001 |
| 2 | docs/ARCHITECTURE.md | System | Add memory module | US-001 |
| 3 | docs/domain/glossary.md | Domain | Add 6 terms | Decisions |
| 4 | docs/domain/bounded-contexts.md | Domain | Add Memory context | US-001 |
| 5 | docs/adr/ | Decision | ADR: SQLite memory backend | Decision #1 |
| 6 | docs/adr/ | Decision | ADR: Three-tier classification | Decision #4 |
| 7 | CHANGELOG.md | Project | Add v5.0.0 entry | — |
| 8 | docs/MODULE-SUMMARIES.md | System | Add memory entries | US-001 |
