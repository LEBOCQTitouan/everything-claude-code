# ADR 0045: SQLite over JSONL for Cost Storage

## Status

Accepted

## Context

BL-096 requires persistent storage for token usage records. The original proposal specified JSONL at `~/.ecc/logs/token-usage.jsonl`. However, the CLI needs aggregation queries (GROUP BY model, date range filtering, summary statistics) and concurrent write safety from parallel sessions.

## Decision

Use SQLite at `~/.ecc/cost/cost.db` with WAL journal mode instead of append-only JSONL.

## Consequences

- Indexed queries enable O(log n) date range and model filters vs O(n) full-scan with JSONL
- WAL mode provides concurrent-write safety without explicit file locking
- SQL GROUP BY handles aggregation natively vs application-level iteration
- Consistent with BL-092's structured log infrastructure (SqliteLogStore pattern)
- Requires rusqlite dependency (already in workspace for BL-092)
- Legacy JSONL data requires one-time migration via `ecc cost migrate`
