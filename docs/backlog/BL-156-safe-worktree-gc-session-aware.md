---
id: BL-156
title: Safe worktree GC — skip active session worktrees
status: implemented
created: 2026-04-18
promoted_to: ""
tags: [worktree, gc, session-safety, cli, concurrency]
scope: MEDIUM
target_command: /spec-fix
---

## Optimized Prompt

```
/spec-fix

Fix `ecc worktree gc` so it never deletes a worktree that is actively in use
by a parallel Claude Code session.

Context:
- ECC Rust CLI, hexagonal architecture (ecc-domain / ecc-infra / ecc-cli crates)
- `ecc worktree gc` already exists; `session:start` hook triggers it automatically (best-effort)
- BL-065 implemented per-session worktree isolation; BL-097 uses transient lock files for
  session-liveness detection across concurrent backlog operations — reuse that lock-file pattern
- CLAUDE.md carries a TEMPORARY warning: "Do not run parallel Claude Code sessions in the same
  repo — worktree GC may delete active worktrees. Remove after BL-150 ships." This fix IS BL-150.
  Removing that warning is a required acceptance criterion.

Problem:
`ecc worktree gc` cannot distinguish (a) an abandoned worktree from a crashed session from
(b) a worktree actively in use by a live parallel session. It currently deletes both, causing
data loss for the live session.

Proposed approach — session heartbeat lock file:
1. Each Claude Code session writes a lock file into its worktree directory on startup:
   `<worktree-path>/.ecc-session.lock` containing the owning PID and session ID.
2. The lock file is held open (POSIX flock, shared lock) for the session lifetime.
   On crash, the OS releases the flock automatically.
3. `ecc worktree gc` checks each worktree for `.ecc-session.lock`:
   - If absent → eligible for GC (existing stale-age logic applies unchanged)
   - If present and flock is acquirable (exclusive try-lock succeeds) → session is dead,
     eligible for GC; release lock and proceed
   - If present and flock is NOT acquirable → session is live, SKIP with a warning:
     "Skipping <name>: active session detected (PID <pid>)"
4. `ecc worktree gc --force` bypasses the stale-age check but still respects live-session
   locks (prints warning, does not delete live worktrees). Provide `--force --kill-live` as
   a separate, explicit opt-in for destruction of live worktrees (prints loud warning, requires
   confirmation prompt unless `--yes`).
5. The `session:start` hook's automatic GC call uses default (non-force) behavior — consistent
   with the manual command.

Acceptance criteria:
- [ ] `.ecc-session.lock` is written to the worktree directory by ecc-workflow at session init
- [ ] Lock file uses POSIX flock (shared) so crash/kill releases automatically; use ecc-flock crate
- [ ] `ecc worktree gc` skips worktrees with an active flock and prints a per-worktree warning
- [ ] `ecc worktree gc` GCs worktrees whose lock file exists but flock is acquirable (dead session)
- [ ] `ecc worktree gc` GCs worktrees with no lock file (existing behavior preserved)
- [ ] `--force` skips age check but still skips live-session worktrees (with warning)
- [ ] `--force --kill-live` + confirmation deletes even live-session worktrees
- [ ] `ecc worktree status` shows a "Live" / "Stale" / "Dead-lock" status column
- [ ] TEMPORARY BL-150 warning in CLAUDE.md is removed
- [ ] Unit tests: lock-acquisition logic, GC skip/delete decisions
- [ ] Integration test: simulate dead lock (file exists, flock free) → GC deletes; simulate live lock → GC skips

Scope boundaries (do NOT):
- Do not change the session:start hook trigger cadence or make it blocking
- Do not introduce a daemon or background heartbeat process
- Do not touch backlog lock-file logic (BL-097) — reuse the ecc-flock crate, don't couple the modules
- Do not change worktree creation or merge logic (BL-122)
```

## Original Input

> "I want a clean up git repo command that do not delete worktrees where other sessions work in"

## Challenge Log

Mode: backlog-mode | Profile: standard | Stages completed: 2/3 | Questions answered: 3/3

### Stage 1: Clarity

**Q1** [Clarification] Which liveness detection mechanism — heartbeat file, PID check, or flock-based lock?
**A** (resolved from context): BL-097 already uses transient lock files with POSIX flock for session
liveness; ecc-flock crate exists for exactly this purpose. Lock-file + flock is the natural fit —
consistent with existing patterns, zero polling, OS-guaranteed release on crash.
**Status**: answered — resolved from codebase context, no ambiguity

**Q2** [Assumption] Should `--force` bypass the liveness check and delete active session worktrees?
**A** (resolved from user intent): User said "do not delete worktrees where other sessions work in"
— even `--force` should warn and skip live worktrees. A separate `--force --kill-live` + confirmation
is provided as an explicit opt-in escape hatch.
**Status**: answered

### Stage 2: Assumptions

**Q1** [Assumption] Should the `session:start` hook's automatic GC call behave differently from the
manual command (e.g., stricter or looser)?
**A** (resolved from context): Consistent behavior — hook uses same default (non-force) mode.
**Status**: answered

## Related Backlog Items

- BL-065: Session-aware worktree isolation — provides the worktree-per-session infrastructure this fix hardens
- BL-097: Transient lock files for in-work backlog filtering — same flock pattern, same ecc-flock crate
- BL-122: Worktree auto-merge and cleanup at session end — GC is the other cleanup path; must stay consistent
