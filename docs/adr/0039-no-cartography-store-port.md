# ADR 0039: No CartographyStore Port

## Status
Accepted

## Context
The cartography system persists data as Markdown files and JSON deltas. A dedicated `CartographyStore` port trait could abstract this, but the existing `FileSystem` and `FileLock` ports already cover all required operations.

## Decision
Do not create a `CartographyStore` port. Domain types use serde for JSON serialization (same pattern as `WorkflowState`, `BacklogEntry`). The app layer uses `FileSystem` for read/write and `FileLock` for concurrency control. The Markdown output format IS the interface — no structured query capability is needed.

## Consequences
- No new port traits to maintain
- JSON serialization lives in domain types via serde derives (not I/O — serde is a data format library)
- App layer handlers are responsible for file path construction and atomic write patterns
- If cartography ever needs structured queries (e.g., "find all journeys mentioning actor X"), a port may be warranted — but this is premature for v1
