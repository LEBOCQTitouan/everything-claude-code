# 5. File-based cross-session memory system

Date: 2026-03-21

## Status

Accepted

## Context

ECC has no cross-session persistence. Each Claude Code session starts from scratch with no record of prior plans, solutions, or implementations. Developers must manually reconstruct context by reading git history or re-running commands. Several options were considered: database storage, Claude's native memory system, vector stores with semantic search, and file-based deterministic storage.

## Decision

Use a file-based, deterministic memory system with two categories: (1) an append-only JSON action log at `docs/memory/action-log.json` recording workflow phase completions, and (2) grouped Markdown work item files at `docs/memory/work-items/YYYY-MM-DD-slug/` containing plan, solution, and implementation artifacts. Memory is written by a shell script (`memory-writer.sh`) called from `phase-transition.sh`. Only designated consumer agents/commands (drift-checker, catchup, robert) read memory — it is never injected broadly.

## Consequences

**Positive:**
- No external dependencies — pure shell + jq, same tooling as existing hooks
- Git-trackable (README checked in, data files gitignored)
- Deterministic paths enable grep-based tooling and debugging
- Scoped consumer access prevents context pollution
- Append-only log is safe for concurrent sessions

**Negative:**
- No semantic search — consumers must know file paths and read linearly
- No automatic cleanup — data grows unboundedly until BL-023 lifecycle management is implemented
- Work item content is template-based, not populated from actual command output (requires future command integration to enrich)
