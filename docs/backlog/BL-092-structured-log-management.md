---
id: BL-092
title: "Structured log management — tracing + JSON rolling files + SQLite index + ecc log CLI"
scope: HIGH
target: "/spec dev"
status: implemented
tags: [observability, logging, tracing, sqlite, cli]
created: 2026-03-28
related: [BL-091]
---

# BL-092: Structured Log Management

## Problem

ECC hooks and commands produce ephemeral stderr/stdout output with zero persistence. When something goes wrong, there's no log trail to investigate. No way to search past sessions, find when a phase transition happened, or audit hook decisions over time. BL-091 adds tiered stderr verbosity; this item adds persistent, queryable storage.

## Proposed Solution

### Architecture: Dual-Layer Tracing + SQLite

```
tracing::info!("phase transition", from = %from, to = %to, session_id = %sid)
    │
    ├─► Layer 1: stderr (human-readable fmt, filtered by -v flags)
    │
    └─► Layer 2: JSON rolling file (~/.ecc/logs/ecc-YYYY-MM-DD.json)
         │
         └─► SQLite index (~/.ecc/logs/ecc.db) for queryable search
```

### Components

1. **tracing instrumentation**: Add `tracing::info!`, `debug!`, `warn!` with structured fields to every hook handler and workflow command. Fields: `session_id`, `hook_id`, `phase`, `verdict`, `duration_ms`.

2. **JSON rolling files**: `tracing-appender::rolling::daily()` writes newline-delimited JSON to `~/.ecc/logs/`. Each line has timestamp, level, target, span context, structured fields.

3. **SQLite FTS5 index**: On write, insert into `events(session_id, timestamp, level, target, message, fields_json)` with FTS5 full-text index. Enables fast SQL queries.

4. **`ecc log` CLI subcommands**:
   - `ecc log tail` — live tail of current session (like `tail -f`)
   - `ecc log search <query>` — FTS5 search across all sessions
   - `ecc log search --session <id>` — filter by session
   - `ecc log search --since 2d --level warn` — time + level filters
   - `ecc log prune` — manual cleanup (auto-prune also runs at startup)

5. **Session correlation ID**: Generated from `CLAUDE_SESSION_ID` env var at startup. Every event carries it. Enables "replay a session's action trail."

6. **30-day auto-prune**: At ecc startup, delete JSON files >30 days old. Prune matching SQLite rows. Configurable via `ecc config set log-retention 30d`.

### Dependencies
- `tracing` + `tracing-subscriber` (features: json, env-filter, fmt)
- `tracing-appender` (rolling file writer)
- `rusqlite` (features: bundled, fts5)
- BL-091 (tiered stderr verbosity — provides the tracing foundation)

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | JSON files + SQLite or one only? | Both: JSON for raw storage, SQLite for querying | Recommended |
| 2 | Relationship to BL-091? | Separate: BL-091 = stderr display, BL-092 = persistent storage | User |
| 3 | Retention? | 30 days auto-prune at startup, configurable | Recommended |
| 4 | Session correlation? | Yes, from CLAUDE_SESSION_ID env var | Recommended |
| 5 | Dependency cost (rusqlite)? | Acceptable — ~1MB binary increase | Recommended |

## Ready-to-Paste Prompt

```
/spec dev

Implement structured log management for ECC:

1. Add tracing instrumentation (info!/debug!/warn! with structured fields)
   to all hook handlers and workflow commands. Fields: session_id, hook_id,
   phase, verdict, duration_ms.

2. Dual-layer tracing subscriber: stderr (human fmt, from BL-091) + JSON
   rolling daily files via tracing-appender to ~/.ecc/logs/.

3. SQLite FTS5 index at ~/.ecc/logs/ecc.db. Insert on write. Schema:
   events(session_id, timestamp, level, target, message, fields_json).

4. `ecc log` CLI: tail, search (FTS5 + session/time/level filters), prune.

5. 30-day auto-prune at startup. Session correlation from CLAUDE_SESSION_ID.

Dependencies: tracing, tracing-subscriber, tracing-appender, rusqlite.
Depends on BL-091 for tracing foundation. See BL-092 for full analysis.
```
