# ADR 0034: Dual Read/Write Paths for Structured Log Storage

## Status

Accepted (2026-03-30)

## Context

BL-092 adds persistent, queryable log storage via JSON rolling files and SQLite FTS5. The architecture must handle two distinct access patterns: passive event capture (write path) and active user queries (read path).

The write path is a tracing subscriber concern — events flow through the tracing ecosystem into JSON files, which a background indexer thread tails into SQLite. This is infrastructure plumbing.

The read path is a user-facing capability — `ecc log search/tail/prune/export` commands query the SQLite database. This is an application use case requiring testability and hexagonal isolation.

## Decision

Separate the read and write paths architecturally:

**Write path** (infrastructure-only, no port):
- `tracing-appender` writes NDJSON daily files to `~/.ecc/logs/`
- Background indexer thread in `ecc-infra` tails JSON files and batch-inserts into SQLite FTS5
- No port trait — this is passive subscriber plumbing, not an orchestrated use case

**Read path** (full hexagonal):
- `LogStore` port trait in `ecc-ports` with `search`, `tail`, `prune`, `export` methods
- `SqliteLogStore` adapter in `ecc-infra` implements the port
- `InMemoryLogStore` test double in `ecc-test-support`
- App use cases in `ecc-app/src/log_mgmt.rs` take `&dyn LogStore`

Both paths share the same SQLite database file via a shared `ensure_schema()` function.

## Consequences

- JSON files are the source of truth; SQLite is a derived index
- The indexer and adapter are independent — either can be deployed/tested separately
- Shared `ensure_schema()` prevents schema version conflicts
- Write path failure is non-fatal (logs still go to stderr and JSON files)
- Read path failure is reported to the user (query errors surfaced)
- Future BL-093 (memory system) can follow the same dual-path pattern
