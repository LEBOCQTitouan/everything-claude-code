---
id: BL-047
title: Automatic session-to-memory persistence with daily files
status: implemented
created: 2026-03-21
scope: HIGH
target_command: /spec dev
tags: [memory, hooks, session, persistence, daily, auto-commit]
---

## Optimized Prompt

Add automatic session-to-memory persistence that is transparent and non-blocking to the user. Two trigger points: (1) incremental writes during the session — after /spec, /design, /implement phase completions, write activity + inferred insights to the daily memory file; (2) a Stop hook writes a final session summary. Daily memory lives in `memory/daily/YYYY-MM-DD.md` with one file per day. Each daily file contains two sections: `## Activity` (commands ran, files changed, PCs passed, commits made) and `## Insights` (preferences detected, decisions made, feedback given — with links to typed memory files when significant enough to promote). When a new day's file is created and it would be empty, the system initializes it with a `## Context from previous sessions` section containing links to the most recent non-empty daily files (up to 3) for continuity. MEMORY.md index gets a `## Daily` section with date-ordered links. The auto-commit must not block the user's workflow — writes happen in the background after phase transitions, not before. Typed memory files (user, feedback, project, reference) continue to be written manually for significant insights; the daily file is the automatic capture layer.

## Framework Source

- **User request**: Transparent, non-blocking auto-commit to memory with day-by-day structure and cross-day linking

## Design Notes

- Incremental writes: Hook into phase-transition.sh (already writes to state.json and memory-writer.sh) to also append to daily file
- Stop hook: New or extended Stop hook that summarizes the session
- Daily file initialization: When creating YYYY-MM-DD.md, check for recent daily files and add context links
- Promotion: When an insight in the daily file is significant, promote it to a typed memory file and add a cross-reference

## Related Backlog Items

- Related to: BL-027 (cross-session memory system — already implemented, covers action-log.json and work-item files at docs/memory/)
- This extends the user-facing memory at ~/.claude/projects/<project>/memory/ — distinct from BL-027's project-level memory
