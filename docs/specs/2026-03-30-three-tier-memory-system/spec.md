# Spec: Three-Tier Memory System (BL-093)

## Problem Statement

ECC's current memory system is write-only noise. `docs/memory/` has 30+ work-item directories and an append-only `action-log.json` that nobody reads. Claude's auto-memory (MEMORY.md) is capped at 200 lines with no relevance-based retrieval — memories are loaded chronologically, not by relevance. There's no consolidation (stale entries persist forever), no deduplication, and no cross-session context injection. This is the #1 differentiator opportunity: no major AI coding tool has solved cross-session memory natively.

## Research Summary

- **Memory architecture consensus**: 4-tier model (episodic/semantic/working/procedural) is the research standard; ECC adopts 3 tiers (skip procedural for v1)
- **Rusqlite 0.38 + FTS5**: Production-ready, minimal deps; BM25 ranking, trigram tokenizers, boolean operators; use external-content FTS tables with triggers
- **Cross-session safety**: Tag all memories with session ID + project scope; implement memory decay/forgetting policies (PersistBench patterns)
- **Consolidation**: Jaccard similarity on word 3-grams for dedup; cap at 100 entries per hook run for 10s timeout budget
- **Anti-patterns**: Unbounded episodic growth without retention; coupling retrieval to specific model versions; single-source storage without consolidation
- **Hybrid retrieval**: FTS5 for full-text + metadata filters; add sqlite-vec later if corpus exceeds 10K entries

## Definitions

- **top-N**: 10 entries by default (configurable via `ECC_MEMORY_TOP_N` env var)
- **recency_factor**: `max(0.0, 1.0 - (age_days as f64 / 365.0))` — linear decay over 1 year
- **Jaccard similarity**: `|A ∩ B| / |A ∪ B|` on word 3-gram sets; threshold 0.8 for dedup

## Decisions Made

| # | Decision | Rationale | ADR? |
|---|----------|-----------|------|
| 1 | SQLite FTS5 as memory backend | Queryable, concurrent (WAL), no vector DB needed for <10K entries | Yes |
| 2 | Global DB with project_id column | Cross-project semantic memories + project-scoped episodic; single DB simpler | Yes (combined with #1) |
| 3 | CONTEXT.md separate from MEMORY.md | Avoid fighting Claude's auto-memory; CONTEXT.md loaded via session hook | Yes (ADR #2) |
| 4 | Three-tier classification model | Working (ephemeral), Episodic (preserved), Semantic (distilled) with explicit tagging + promotion | Yes (ADR #2) |
| 5 | Write-only to SQLite (breaking) | Stop writing to docs/memory/ files; one-time migration + export command for rollback | No |
| 6 | Cap consolidation at 100/run | 10s Stop hook timeout; full sweep via `ecc memory gc` | No |
| 7 | No encryption v1 | Same security model as Claude's memory; validate no-secrets at write | No |
| 8 | MemoryStore port trait | SQLite stays in ecc-infra behind port abstraction; hexagonal compliance | No |
| 9 | 100% test coverage all paths | User requirement | No |
| 10 | CLI uses flag syntax | `ecc memory add --type semantic --title "..." --tags rust,ddd` (not positional) | No |

## User Stories

### US-001: SQLite Memory Store Foundation

**As a** developer using ECC, **I want** a SQLite database with FTS5 at `~/.ecc/memory/memory.db` to store structured memory entries, **so that** memories are queryable across sessions without scanning markdown files.

#### Acceptance Criteria

- AC-001.1: Given ECC is installed, when `ecc memory add --type semantic --title "Prefer warn-not-block" --tags rust,ddd` is run, then a row is inserted with type=semantic, content, tags, timestamps, relevance_score=1.0
- AC-001.2: Given the database does not exist, when any `ecc memory` command runs, then the database and FTS5 virtual table are created automatically (auto-migration)
- AC-001.3: Given entries exist, when `ecc memory search "warn block"` is run, then FTS5 returns matching entries ranked by BM25 relevance
- AC-001.4: Given entries exist, when `ecc memory list --type semantic --tag rust` is run, then only semantic entries with "rust" tag are shown
- AC-001.5: Given SQLite WAL mode, when two concurrent sessions access the database, then reads never block and writes are serialized
- AC-001.6: Given a `MemoryStore` port trait defined in `ecc-ports`, when the SQLite adapter is in `ecc-infra`, then domain/app layers have zero SQLite imports
- AC-001.7: Given the `ecc-domain/src/memory/` module, when inspected, then it has zero I/O imports (pure types + logic)
- AC-001.8: Given the database file is corrupted, when any command runs, then it detects corruption, backs up the corrupt file as `memory.db.corrupt`, and recreates an empty database with a warning
- AC-001.9: Given an empty search result, when `ecc memory search` runs, then it displays "No matching memories found" (not an error)
- AC-001.10: Given content with Unicode characters (emoji, CJK), when stored and searched via FTS5, then unicode61 tokenizer handles them correctly

#### Dependencies
- None (foundation)

### US-002: Legacy Memory Migration + Export

**As a** developer, **I want** to migrate existing `docs/memory/` into SQLite and export back to files, **so that** historical context is preserved and the migration is reversible.

#### Acceptance Criteria

- AC-002.1: Given `docs/memory/work-items/` contains markdown files, when `ecc memory migrate` runs, then each work-item becomes an episodic entry
- AC-002.2: Given `docs/memory/action-log.json` exists, when `ecc memory migrate` runs, then each action becomes an episodic entry
- AC-002.3: Given migration has already run, when re-run, then no duplicates are created (idempotent, keyed on original file path)
- AC-002.4: Given migration completes, when checking entries, then `related_work_items` are populated from BL-NNN references
- AC-002.5: Given `action-log.json` contains malformed entries, when migrating, then invalid entries are skipped with count reported
- AC-002.6: Given `ecc memory export --output ./backup/`, when run, then all entries are exported as individual markdown files grouped by tier
- AC-002.7: Given export completes, when importing from the exported files, then the round-trip is lossless

#### Dependencies
- Depends on: US-001

### US-003: Session-End Consolidation

**As a** developer, **I want** memories automatically consolidated at session end, **so that** the store stays clean without manual maintenance.

#### Acceptance Criteria

- AC-003.1: Given a Stop hook fires, when consolidation runs, then entries with >80% Jaccard similarity (word 3-grams) are merged, keeping the newer entry
- AC-003.2: Given entries older than 90 days with zero references, when consolidation runs, then they are marked stale (not deleted)
- AC-003.3: Given consolidation runs, when computing scores, then `relevance_score = recency_factor * (1.0 + reference_count as f64 * 0.1)` where `recency_factor = max(0.0, 1.0 - (age_days as f64 / 365.0))`
- AC-003.4: Given consolidation runs, when it completes, then it finishes within the 10s hook timeout (capped at 100 most-recent entries per run)
- AC-003.5: Given another session is consolidating, when this session's consolidation starts, then it skips (non-blocking try-lock via ecc-flock)
- AC-003.6: Given two entries with fewer than 10 words each, when computing Jaccard, then the dedup is skipped (short entries produce unreliable similarity)
- AC-003.7: Given a merge operation, when it executes, then it runs within a SQLite transaction (atomic — either both delete+insert succeed or neither)

#### Dependencies
- Depends on: US-001, US-005

### US-004: CONTEXT.md Auto-Generation

**As a** developer, **I want** a CONTEXT.md file auto-generated from top-10 memories after consolidation, **so that** relevant cross-session knowledge is available without manual curation.

#### Acceptance Criteria

- AC-004.1: Given consolidation completes, when CONTEXT.md regeneration runs, then `~/.claude/projects/<hash>/memory/CONTEXT.md` is written with top-10 memories by relevance
- AC-004.2: Given CONTEXT.md content exceeds 200 lines, when generating, then entries are truncated by relevance until within limit
- AC-004.3: Given Claude's MEMORY.md exists, when CONTEXT.md is generated, then MEMORY.md is NOT modified
- AC-004.4: Given no memories in database, when generating, then CONTEXT.md contains "No memories stored"
- AC-004.5: Given CONTEXT.md is generated, when inspected, then each entry shows tier, title, relevance score, and truncated content

#### Dependencies
- Depends on: US-003

### US-005: Memory Tier Classification

**As a** developer, **I want** memories classified as working/episodic/semantic via explicit tagging and promotion, **so that** the three-tier system provides meaningful retrieval filtering.

#### Acceptance Criteria

- AC-005.1: Given `ecc memory add --type working --title "current task context"`, when added, then type is working
- AC-005.2: Given no --type flag, when adding, then default type is episodic
- AC-005.3: Given an episodic memory, when `ecc memory promote <id>` runs, then type changes to semantic and relevance_score is boosted 2x
- AC-005.4: Given working memories older than 24h, when consolidation runs, then they are promoted to episodic (if content >50 chars) or deleted
- AC-005.5: Given a semantic memory, when `ecc memory promote <id>` runs, then it's a no-op with message "Already semantic"

#### Dependencies
- Depends on: US-001

### US-006: Session-Start Context Injection

**As a** developer, **I want** the SessionStart hook to query SQLite and inject relevant memories, **so that** Claude starts with cross-session knowledge.

#### Acceptance Criteria

- AC-006.1: Given a new session starts, when the SessionStart hook fires, then it queries top-10 memories by `relevance_score * recency_factor`
- AC-006.2: Given project-scoped memories exist, when querying, then current project_id memories are boosted 1.5x
- AC-006.3: Given relevant memories found, when injecting, then they appear in hook stdout as `## Relevant Memories` markdown section
- AC-006.4: Given memory DB does not exist, when hook fires, then it silently passes through (exit 0, no output)
- AC-006.5: Given injected context exceeds 5000 chars, when injecting, then it's truncated to top-N that fits

#### Dependencies
- Depends on: US-001

### US-007: Manual Memory Management CLI

**As a** developer, **I want** commands to delete, gc, and view stats, **so that** I can manually curate memories.

#### Acceptance Criteria

- AC-007.1: Given a memory exists, when `ecc memory delete <id>` runs, then it's permanently removed from both table and FTS index
- AC-007.2: Given stale memories exist, when `ecc memory gc` runs, then stale entries >180 days are deleted
- AC-007.3: Given `ecc memory gc --dry-run`, when run, then it reports what would be deleted without deleting
- AC-007.4: Given `ecc memory stats`, when run, then it shows counts by type, stale count, DB file size, oldest/newest entry dates
- AC-007.5: Given a non-existent ID, when `ecc memory delete <id>` runs, then it returns error "Memory not found"

#### Dependencies
- Depends on: US-001

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| `ecc-domain/src/memory/` | Domain | New: MemoryEntry, MemoryTier, MemoryId, consolidation rules, classification |
| `ecc-ports/src/memory_store.rs` | Port | New: MemoryStore trait |
| `ecc-infra/src/sqlite_memory.rs` | Adapter | New: SqliteMemoryStore (rusqlite + FTS5) |
| `ecc-app/src/memory.rs` | App | New: add, search, list, consolidate, inject, migrate, export, gc use cases |
| `ecc-cli/src/commands/memory.rs` | CLI | New: ecc memory subcommands |
| `ecc-workflow/src/commands/memory_write.rs` | Workflow | Modify: redirect writes to SQLite via ecc-app |
| `ecc-test-support/` | Test | New: InMemoryMemoryStore test double |
| `hooks/hooks.json` | Config | Modify: add consolidation to Stop, injection to SessionStart |
| `Cargo.toml` | Build | Add rusqlite workspace dependency |

## Constraints

- `ecc-domain/src/memory/` must have zero I/O imports (pure domain)
- SQLite must stay behind `MemoryStore` port trait in `ecc-infra` only
- Consolidation must complete within 10s hook timeout (cap at 100 entries)
- CONTEXT.md must NOT touch Claude's MEMORY.md
- Global DB with `project_id` column, not per-project DBs
- WAL mode + flock for concurrency safety
- 100% test coverage on all paths
- No vector embeddings, no LLM-powered classification
- Consolidation merges must be transactional (SQLite transaction)
- `ecc memory export` must exist before the breaking change to file writes

## Non-Requirements

- Vector embeddings / semantic similarity (FTS5 sufficient for <10K entries)
- LLM-powered memory classification (rule-based only)
- Web UI for memory browsing
- Multi-user / team memory sharing
- Real-time sync across machines
- Encryption at rest (v1)
- Procedural memory tier (v1)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| MemoryStore (new) | New port + adapter | New E2E boundary: SQLite adapter needs integration tests |
| FileSystem | Existing | CONTEXT.md writes use existing FS port |
| FileLock | Existing | Consolidation uses existing flock |
| ShellExecutor | No change | No impact |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New CLI commands | CLAUDE.md | CLI table | Add `ecc memory` subcommands |
| Architecture update | docs/ARCHITECTURE.md | Crate list | Add memory module description |
| Domain glossary | docs/domain/glossary.md | Terms | Add 6 memory domain terms |
| Bounded contexts | docs/domain/bounded-contexts.md | Contexts | Add Memory bounded context |
| ADR | docs/adr/ | New ADR | SQLite memory backend |
| ADR | docs/adr/ | New ADR | Three-tier classification model |
| Changelog | CHANGELOG.md | Entry | Add memory system entry |
| Module summaries | docs/MODULE-SUMMARIES.md | Entries | Add memory module entries |

## Open Questions

None — all resolved during grill-me interview.
