---
id: BL-097
title: "Spec backlog in-work filtering — hide entries claimed by other sessions"
scope: MEDIUM
target: "/spec-dev"
status: open
tags: [spec, backlog, filtering, session, concurrency, stale-detection]
created: 2026-03-29
related: [BL-065, BL-066]
---

# BL-097: Spec Backlog In-Work Filtering

## Problem

When multiple sessions run `/spec` concurrently, the backlog picker in Phase 1 shows all open entries — including entries already picked by another active session. Two sessions can claim the same backlog entry simultaneously, leading to duplicate work or conflicting implementations.

The same issue surfaces in `/backlog match` cross-reference suggestions: entries being actively worked in another session are still offered as candidates.

Currently:
- No session claims an entry when picking it from the backlog picker
- No signal distinguishes "open and available" from "open and in-work"
- A crashed session has no cleanup path — a claimed entry would be permanently locked with no recovery mechanism

## Proposed Solution

### Claim mechanism

When a session picks an entry from the `/spec` Phase 1 backlog picker, it writes a lock file:

```
docs/backlog/.locks/BL-NNN.lock
```

Lock file content (JSON):
```json
{"session_id": "session-20260329-abc123", "claimed_at": "2026-03-29T10:00:00Z", "pid": 12345}
```

The lock file is written atomically (tmp + mv) to avoid race conditions (consistent with BL-065's locking patterns).

### Filtering at both touch points

1. **`/spec` Phase 1 backlog picker** — before rendering the entry list, scan `.locks/` for existing claim files. Entries with a valid (non-stale) lock are hidden from the picker. A count is shown: "2 entries hidden (in work in other sessions)".

2. **`/backlog match` cross-reference** — the match response marks in-work entries distinctly rather than omitting them, since the user may intentionally want to bundle related work:

   | ID | Title | Confidence | Status | Suggestion |
   |----|-------|------------|--------|------------|
   | BL-NNN | ... | HIGH | in-work | Review before bundling |

### Stale lock detection

A lock is considered stale when either condition is true:
- `claimed_at` is older than **2 hours** (configurable via `ECC_LOCK_TTL_HOURS`, default 2)
- The PID in the lock file no longer exists on the local machine (`kill -0 <pid>` fails)

Stale locks are automatically removed before filtering runs. No entry can be permanently stuck.

### Lock release

The lock file is removed when:
- The spec pipeline completes (entry promoted or abandoned)
- The session exits cleanly
- Stale detection triggers

## Evidence

- BL-065 established session identity via worktree naming (`session-{timestamp}-{slug}`) — this entry reuses that identity for lock file naming
- BL-066 introduced deterministic backlog management with atomic writes — this entry extends the same pattern to session claiming

## Acceptance Criteria

- [ ] Picking an entry from `/spec` Phase 1 creates `.locks/BL-NNN.lock` atomically
- [ ] `/spec` Phase 1 picker hides entries with valid (non-stale) lock files
- [ ] Picker shows a count of hidden in-work entries
- [ ] `/backlog match` marks in-work entries with an `in-work` status column instead of hiding them
- [ ] Stale lock: removed when `claimed_at` > TTL hours ago
- [ ] Stale lock: removed when PID no longer exists
- [ ] `ECC_LOCK_TTL_HOURS` environment variable controls the TTL (default 2)
- [ ] Lock files stored under `docs/backlog/.locks/` (gitignored)
- [ ] Lock release on clean session exit
- [ ] Lock release on spec pipeline completion
- [ ] Two concurrent `/spec` sessions picking from same backlog never claim the same entry

## Scope Boundaries — Do NOT

- Do not add in-work status to the `BACKLOG.md` index (index stays as-is, locks are transient)
- Do not implement cross-machine locking (local PID check only)
- Do not block `/backlog list` — list shows all entries including in-work ones
- Do not require BL-065 to be fully implemented first (lock files work without full worktree isolation)

## Verification Steps

1. Open two terminals, both run `/spec` — both should see the same picker initially
2. Terminal 1 picks BL-050 — Terminal 2 should no longer see BL-050 in the picker
3. Kill Terminal 1's session — wait for TTL or verify PID check removes the lock
4. Terminal 2 now sees BL-050 again in the picker
5. Run `/backlog match "deferred summary tables"` while BL-050 is claimed — entry appears with `in-work` status

## Ready-to-Paste Prompt

```
/spec-dev Add in-work filtering to the /spec backlog picker and /backlog match.

When a session picks an entry from the /spec Phase 1 picker, write an atomic
lock file to docs/backlog/.locks/BL-NNN.lock containing session_id, claimed_at
timestamp, and PID. Two touch points must filter by these locks:

1. /spec Phase 1 backlog picker — hide entries with valid (non-stale) locks.
   Show a count: "N entries hidden (in work in other sessions)".
2. /backlog match cross-reference — mark in-work entries with an `in-work`
   status column rather than hiding them.

Stale lock detection (either condition): claimed_at older than ECC_LOCK_TTL_HOURS
(default 2h) OR PID no longer exists (kill -0 check). Stale locks auto-removed
before filtering runs. No entry can be permanently stuck.

Lock files are gitignored. Lock is released on clean session exit or pipeline
completion. Atomic write (tmp+mv) to avoid race conditions.

Acceptance: two concurrent /spec sessions never claim the same entry. Crashed
sessions auto-release within the TTL window.
```

## Challenge Log

Mode: backlog-mode (3 stages, max 2 questions each)

**Q1**: When is an entry considered "in work"?
**A**: When `/spec` picks the entry from the backlog picker (Phase 1).

**Q2**: Does filtering apply at both /spec Phase 1 AND /backlog match?
**A**: Yes — both touch points must respect in-work status.

**Q3**: How should in-work status be detected — lock files, status field update, or other?
**A**: Needs a guardrail so entries don't get stuck if a session crashes. Stale detection required.

**Q4**: What stale detection approach is acceptable?
**A**: Yes, required. No entry should ever be permanently stuck.
