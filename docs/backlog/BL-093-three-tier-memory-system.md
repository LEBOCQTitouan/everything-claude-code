---
id: BL-093
title: "Three-tier memory system — semantic/episodic/working memory with SQLite index, consolidation, auto-gen MEMORY.md"
scope: EPIC
target: "/spec dev"
status: open
tags: [memory, sqlite, retrieval, consolidation, context-management]
created: 2026-03-28
related: [BL-027, BL-092]
---

# BL-093: Three-Tier Memory System

## Problem

ECC's current memory is write-only noise. `docs/memory/` has 30+ work-item directories and an append-only action-log.json that nobody reads. Claude's auto-memory (MEMORY.md) is capped at 200 lines/25KB with no relevance-based retrieval. Memories are loaded chronologically, not by relevance. There's no consolidation — stale entries persist forever. This is the #1 differentiator opportunity: no major AI coding tool has solved cross-session memory natively.

## Proposed Solution

### Three Memory Tiers

| Tier | Purpose | Storage | Example |
|------|---------|---------|---------|
| **Working** | Current session context | Claude's context window | "Currently implementing BL-068 Phase 3" |
| **Episodic** | What happened | Markdown files + SQLite | "BL-068 implemented: 42 PCs, Idle phase added" |
| **Semantic** | Distilled facts & decisions | Markdown files + SQLite | "Prefer warn-not-block for side-effect failures (DDD pattern)" |

### Sub-Spec Phasing

**Sub-Spec A: SQLite Memory Index + CLI** (HIGH)
- Create `~/.ecc/memory/memory.db` with SQLite FTS5
- Schema: `memories(id, type, title, content, tags, created, updated, relevance_score, related_work_items)`
- `ecc memory add <type> <title>` — create a memory entry
- `ecc memory search <query>` — FTS5 + tag filter
- `ecc memory list [--type semantic|episodic] [--tag X]`
- Migrate existing `docs/memory/work-items/` into episodic entries
- Migrate existing `action-log.json` entries into episodic entries

**Sub-Spec B: Session-End Consolidation + MEMORY.md Auto-Gen** (HIGH)
- Stop hook triggers consolidation: dedup (>80% content overlap), mark stale (>90 days, no refs), update relevance scores (recency + reference count)
- Auto-regenerate `~/.claude/projects/.../memory/MEMORY.md` from top-N highest-relevance memories
- Stay within 200-line / 25KB limit
- MEMORY.md becomes a dynamic view, not a manual file
- Consolidation is pure Rust (no LLM cost)

**Sub-Spec C: Three-Tier Classification + Selective Injection** (HIGH)
- Classify memories as working/episodic/semantic based on content patterns
- At session start (SessionStart hook), query SQLite for memories relevant to the current task context
- Inject top-N memories into the prompt via a `## Relevant Context` section in MEMORY.md
- Working memory = ephemeral (cleared on session end)
- Episodic = what happened (preserved, consolidated)
- Semantic = distilled patterns (highest value, longest retention)

### Retrieval Strategy

SQLite FTS5 full-text search + metadata filters:
- `MATCH 'phase transition warn block'` — keyword relevance
- `WHERE type = 'semantic'` — tier filter
- `WHERE tags LIKE '%rust%'` — tag filter
- `ORDER BY relevance_score * recency_weight DESC` — combined ranking
- `LIMIT 10` — token budget (inject top-N only)

No vector embeddings needed for <10K memories. Add later if corpus grows.

### Concurrency Safety

Each session writes to its own namespaced temp entries. Consolidation runs under flock (same pattern as BL-065). SQLite WAL mode for concurrent read + single-writer.

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Priority across tiers? | All three tiers, phased via sub-specs | User |
| 2 | Retrieval method? | SQLite FTS5 + tag filtering, no vector DB | Recommended |
| 3 | Consolidation trigger? | Session-end auto-consolidation (Stop hook) | Recommended |
| 4 | MEMORY.md management? | Auto-generated from top-N highest-relevance | Recommended |
| 5 | Phasing? | Sub-specs A/B/C like BL-065 | Recommended |

## Dependencies

- BL-092 (structured logs) — shares SQLite infrastructure, tracing crate
- BL-065 (concurrent safety) — flock for consolidation writes

## Ready-to-Paste Prompt

```
/spec dev

Design a three-tier memory system for ECC (EPIC, split into sub-specs):

Sub-Spec A: SQLite memory index + CLI
- ~/.ecc/memory/memory.db with FTS5
- ecc memory add/search/list commands
- Migrate docs/memory/ work-items and action-log.json

Sub-Spec B: Session-end consolidation + MEMORY.md auto-gen
- Stop hook triggers: dedup, stale marking, relevance scoring
- Auto-regenerate MEMORY.md from top-N memories (200-line cap)
- Pure Rust, zero LLM cost

Sub-Spec C: Three-tier classification + selective injection
- Classify: working (ephemeral), episodic (what happened), semantic (distilled)
- SessionStart hook queries SQLite for task-relevant memories
- Inject top-N into prompt via MEMORY.md

Retrieval: SQLite FTS5, no vector DB. Concurrency: flock + WAL mode.
See BL-093 for full analysis.
```
