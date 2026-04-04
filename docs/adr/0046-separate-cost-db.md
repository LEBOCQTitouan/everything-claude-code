# ADR 0046: Separate Cost Database from Logs Database

## Status

Accepted

## Context

BL-092 established `~/.ecc/logs/ecc.db` for structured logs. BL-096 needs a database for token usage records. The question is whether to add tables to the existing log database or create a separate one.

## Decision

Use a separate database at `~/.ecc/cost/cost.db` rather than adding tables to `~/.ecc/logs/ecc.db`.

## Consequences

- Independent schema evolution: cost schema changes don't require log database migrations
- Independent retention policies: `ecc cost prune` and `ecc log prune` operate on separate stores
- Bounded context isolation: logs and costs are distinct domain concepts with different lifecycles
- Slightly more connection overhead (two databases vs one), but negligible for a CLI tool
- Consistent with DDD principle of one data store per bounded context
