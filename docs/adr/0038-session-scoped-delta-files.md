# ADR 0038: Session-Scoped Delta Files

## Status
Accepted

## Context
Multiple Claude sessions can run concurrently (e.g., in worktrees). The cartography Stop hook must write session change data without risking corruption from concurrent writes to a shared manifest.

## Decision
Each session writes its own `pending-delta-<session_id>.json` file. No shared manifest is written by the hook. Only the SessionStart handler writes to shared docs, protected by a `cartography-merge` file lock.

## Consequences
- Zero concurrent write races at the hook level
- Multiple delta files may accumulate for the same files (the merge algorithm handles deduplication)
- The SessionStart handler acquires a file lock before processing, preventing read races
- Delta files are session-scoped and immutable once written
