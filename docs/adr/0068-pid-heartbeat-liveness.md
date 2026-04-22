# ADR 0068 — PID + Heartbeat Liveness (BL-156)

## Status

Accepted — 2026-04-21

## Context

`ecc worktree gc` historically used PID checks + filesystem mtime heuristics to detect abandoned worktrees. BL-150 (shipped earlier) added stricter PID resolution (`parent_id()`) but did not solve the core class of bugs:

1. **Idle-but-live sessions** exceed the 30-min `.git` recency window during long operations (grill-me interviews, slow agent calls) — GC deletes live work.
2. **PID reuse** on long-running macOS/Linux systems causes `kill -0` to lie in both directions — recycled unrelated process = false-positive alive; real live session using a child fork = false-negative dead.
3. **`ShellWorktreeManager` stubs silently defeat the fail-safe.** `unmerged_commit_count → Ok(0)` hard-coded; same for `has_uncommitted_changes`, `has_stash`, `has_untracked_files`, `is_pushed_to_remote`. The `unwrap_or(u64::MAX)` guard at `gc.rs:66` never fired in the automatic `SessionStart` hook path because the shell manager is used there instead of `OsWorktreeManager`.

The combination caused the `SessionStart`-triggered automatic GC to delete sibling sessions' live worktrees, losing uncommitted/unpushed work. The CLAUDE.md `TEMPORARY (BL-150)` warning ("do not run parallel Claude Code sessions in the same repo") was the documented workaround, removed in spec 2026-04-18-claude-md-temp-marker-lint on the assumption BL-156 would land.

Investigation considered POSIX flock with shared/exclusive semantics (live session holds shared, GC tries exclusive). **Rejected** because no long-lived ECC process exists to hold the shared flock — every ECC CLI invocation, hook handler, and `ecc-workflow` subcommand is short-lived; the only long-lived process in a session is Claude Code itself (not ours). A sidecar daemon was considered and rejected as disproportionate complexity for one feature.

## Decision

Adopt **PID + heartbeat hybrid**.

- Every ECC hook (`SessionStart`, `PostToolUse`, `Stop`) atomically writes `<worktree>/.ecc-session` via tmpfile + rename, containing JSON `{schema_version: 1, claude_code_pid: u32, last_seen_unix_ts: u64}`.
- `ecc worktree gc` treats a worktree as live iff `kill -0 claude_code_pid == 0` AND `now - last_seen < TTL` (default 3600s, override via `ECC_WORKTREE_LIVENESS_TTL_SECS`).
- Missing `.ecc-session` → fall back to existing BL-150 24h stale-age timer (backward compatible).
- Malformed `.ecc-session` → treated as missing (fall through).
- `claude_code_pid ∈ {0, 1}` or `last_seen > now + 60s` (clock skew / tampering) → treated as malformed.

Self-skip via `current_worktree()` resolver (`CLAUDE_PROJECT_DIR` → walk `.git` file for `gitdir:` → canonicalize) prevents the `SessionStart`-triggered GC from deleting its own worktree. If the resolver returns `None` (main-repo or unresolvable), GC conservatively skips all `ecc-session-*` worktrees younger than `ECC_WORKTREE_SELF_SKIP_FALLBACK_SECS` (default 3600s).

`ShellWorktreeManager`'s 5 previously-stubbed methods now invoke real `git` commands via the `ShellExecutor` port, so the `unwrap_or(u64::MAX)` fail-safe actually fires. This part of the work ships as a separable commit **before** the heartbeat code (per Decision #14 of the spec), so it can be reverted independently if needed.

Kill switch `ECC_WORKTREE_LIVENESS_DISABLED=1` short-circuits both the read path (GC skips heartbeat consult, falls back to BL-150 logic) and the write path (hooks suppress heartbeat writes). Mirrors `ECC_CLAUDE_MD_MARKERS_DISABLED` from BL-158.

Two-flag destructive override (`--force --kill-live`) mandatory interactive confirmation prompt unless `--yes` is passed. Non-TTY + `--force --kill-live` without `--yes` exits non-zero — prevents accidental scripted destruction.

## Consequences

**Positive**:

- Works without any long-lived ECC process — heartbeat written by every hook invocation, deterministic mtime comparison.
- No new port surface on `ecc-ports` (closure-based `now_fn` threaded through `LivenessChecker` instead of introducing a `Clock` port).
- No new crate dependencies.
- `LivenessRecord` VO + `is_live` pure fn live in `ecc-domain::worktree::liveness` with zero I/O imports (enforced by hook).
- Deterministic and testable via `InMemoryFileSystem` + `MockExecutor`.
- Degrades gracefully — old worktrees lacking `.ecc-session` fall through to BL-150 24h stale-timer.
- Revert path is revert-from-HEAD in dependency order, with US-004 (stub fix) being separately revertable per Decision #14.

**Negative**:

- Loses kernel-guaranteed release on crash that flock would have provided. False-negative cleanup window = TTL (60 min default). A crashed session's worktree lingers for up to TTL before being eligible for cleanup. Acceptable: a 60-min delay on cleanup of a crashed session is preferable to destroying a live one.
- Relies on `kill -0 PID == 0` for liveness signaling, so PID reuse within the TTL window can still produce false-positives. Mitigation: PID reuse on modern Linux/macOS has a much longer cycle than 60 min under normal load; heartbeat freshness AND PID liveness must both hold.
- Heartbeat is written on every `PostToolUse` — one atomic rename per Claude Code tool dispatch. Measured ~1ms overhead; acceptable.
- Stash detection caveat: `git stash list` is repo-global, not worktree-scoped. `ShellWorktreeManager::has_stash` carries a doc-comment caveat. Per-worktree filtering deferred (out of scope).

**Cross-machine caveat**: `kill -0` and flock are local-host only. NFS-shared worktrees are out of scope and documented as a limitation.

## Implementation notes

- Spec: `docs/specs/2026-04-18-safe-worktree-gc-flock/spec.md` (R2 PASS 87/100, 9 USes, 58 ACs).
- Design: `docs/specs/2026-04-18-safe-worktree-gc-flock/design.md` (R2 CONDITIONAL 79/100, 78 PCs, 29-commit TDD order).
- Commits: `8b359874` (US-004 tests) through the US-009 docs sweep.
- Resumable: every PC tagged in `tasks.md` with `red@ts`/`green@ts`/`done@ts`.

## Related

- Supersedes the BL-150 parent_id fix (which this spec's analysis proved was incomplete).
- Implements the removal-of-TEMPORARY-marker scenario from spec `2026-04-18-claude-md-temp-marker-lint` (AC-005.1). The CLAUDE.md warning was preemptively removed; this spec makes that removal honest.
- Companion follow-up: none in scope. Future work candidates: extract `Reason` enum from `LivenessVerdict::Stale` for richer status reporting; cross-machine liveness (NFS-safe); automated PID-recycling detection.
