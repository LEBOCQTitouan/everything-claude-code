# ADR-0053: Pattern Memory Search Integration

## Status

Accepted (2026-04-05)

## Context

The pattern library contains 136 pattern files organized by category. Agents need to discover relevant patterns during code generation and review. Two approaches were considered: file-glob-only discovery (agents use `Glob` tool to find patterns by category/name) vs SQLite FTS5 indexing via the existing `ecc memory` system.

## Decision

Index patterns in the `ecc memory` SQLite store using transactional batch upsert. Patterns are stored as semantic-tier memory entries with name, category, tags, and languages as searchable fields. The `ecc memory reindex` command triggers re-indexing. Invalid frontmatter files are skipped with warnings.

### Alternatives Considered

1. **File-glob-only** — Agents use `Glob` to find `patterns/<category>/*.md`. Simple but no full-text search, no tag filtering, no cross-category discovery. Rejected for limited discoverability.
2. **SQLite FTS5 indexing (chosen)** — Reuses existing memory infrastructure. Transactional batch upsert ensures consistency. FTS5 enables natural-language pattern search.
3. **Separate search index** — New SQLite database just for patterns. Rejected: unnecessary infrastructure when memory system already exists.

## Consequences

- `batch_upsert_by_source_path` added to `MemoryStore` trait (ISP concern noted — may extract to separate trait in future)
- Pattern metadata searchable via `ecc memory search`
- Depends on BL-093 Sub-Spec A (SQLite Memory Index + CLI)
- Transactional indexing prevents partial state on failure
