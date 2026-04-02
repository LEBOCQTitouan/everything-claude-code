---
id: BL-065
title: Full concurrent session safety — worktree isolation, serialized merge, codebase audit fixes
status: "implemented"
created: 2026-03-26
promoted_to: ""
tags: [concurrency, worktree, git, safety, hooks, state, merge, audit]
scope: EPIC
target_command: /spec dev
---

## Optimized Prompt

Enable fully safe concurrent Claude Code sessions in the same project directory. Two or more sessions running `/spec` → `/design` → `/implement` (or `/backlog`) in parallel must never corrupt shared state, lose data, or produce broken builds. Each pipeline session gets its own git worktree; merges back to main are serialized with a fast verify gate.

**Project**: everything-claude-code (Rust, hexagonal architecture, 7 crates)

### Part 1 — Automatic Worktree Isolation

When a session enters a spec-driven pipeline (`/spec` → `/design` → `/implement`) or `/backlog`, it must:

1. **Auto-create a git worktree** for the session (e.g., `.worktrees/session-{timestamp}-{slug}/`)
2. **Branch** from current HEAD (e.g., `ecc/session-{timestamp}-{slug}`)
3. **Run all work** in the worktree — spec files, design files, TDD loop, commits all happen on the isolated branch
4. **Non-pipeline commands** (`/audit`, `/verify`, `/review`) remain on the main tree (they're read-only or safe)

This means two `/implement` sessions work on separate branches with separate `target/` directories — no cargo build races, no file write races.

### Part 2 — Serialized Merge-to-Main with Fast Verify Gate

When a worktree session completes its pipeline:

1. **Acquire a merge lock** (`.claude/workflow/.merge.lock` using `flock`)
2. **Rebase** the session branch onto current main: `git rebase main`
3. **Fast verify**: `cargo build && cargo test && cargo clippy -- -D warnings`
4. **If fast verify passes**: fast-forward merge to main, delete worktree + branch
5. **If rebase conflicts**: surface conflict to user, pause merge, release lock
6. **If fast verify fails**: surface failures to user, pause merge, release lock
7. **Release lock** after merge completes or pauses

Only one session can be in the merge step at a time. This prevents two sessions from merging simultaneously and creating conflicts.

### Part 3 — Shared State Concurrency Fixes

The codebase audit (2026-03-26) found these shared-state issues. All must be fixed:

#### CRITICAL

| # | File | Issue | Fix |
|---|------|-------|-----|
| 1 | `target/` (cargo build) | Two `cargo build` in same dir causes linker/rmeta races | Worktree isolation solves this — each worktree has its own `target/` |
| 2 | `docs/memory/action-log.json` | Read-modify-write race — jq reads, modifies, writes; concurrent sessions lose entries | Wrap in `flock "${MEMORY_DIR}/.action-log.lock"` |

#### HIGH

| # | File | Issue | Fix |
|---|------|-------|-----|
| 3 | `.claude/workflow/state.json` | TOCTOU — phase gate reads phase, another session transitions, gate check is stale | `flock ".claude/workflow/.state.lock"` around all read-modify-write in `phase-transition.sh`, `workflow-init.sh`, `toolchain-persist.sh` |
| 4 | `docs/backlog/BACKLOG.md` | Race on BL-NNN ID generation + lost appends | `flock "docs/backlog/.backlog.lock"` around ID read + file write + index append |
| 5 | `docs/memory/work-items/{date}-{slug}/*.md` | TOCTOU on file existence check — `[ -f ]` then write races | `flock` or use `set -o noclobber` with `>` for atomic create |

#### MEDIUM

| # | File | Issue | Fix |
|---|------|-------|-----|
| 6 | `docs/memory/daily/{date}.md` | Read-modify-write on awk-based append | `flock "${MEMORY_DIR}/.daily.lock"` |
| 7 | `~/.claude/projects/{hash}/memory/MEMORY.md` | Read-modify-write on index updates | `flock` around awk + mv |
| 8 | `.claude/workflow/.tdd-state` | State mutation across sessions (informational only) | Per-session TDD state file or `flock` |
| 9 | `/tmp/ecc-sl-cache-{hash}` | Statusline cache overwrite (harmless but noisy) | Already uses `mktemp+mv` — acceptable |

### Part 4 — Lock Infrastructure

Create a lightweight `flock`-based locking system for ECC hooks:

```bash
# Helper function for all hooks
ecc_flock() {
  local lock_file="$1"; shift
  flock "$lock_file" bash -c "$@"
}
```

Lock files directory: `.claude/workflow/.locks/` (gitignored)

Lock granularity:
- `.locks/state.lock` — workflow state.json
- `.locks/merge.lock` — merge-to-main serialization
- `.locks/action-log.lock` — memory action log
- `.locks/backlog.lock` — backlog index + ID generation
- `.locks/daily.lock` — daily memory files

### Part 5 — Worktree Lifecycle Management

- **Creation**: Auto-created at pipeline start in `.worktrees/` (gitignored)
- **Cleanup on success**: Worktree + branch deleted after successful merge
- **Cleanup on failure**: Worktree preserved for user inspection; user runs `ecc worktree clean` to remove
- **Cleanup on crash**: Add `ecc worktree gc` command to clean up stale worktrees (no active session)
- **CARGO_TARGET_DIR**: Each worktree uses its own `target/` naturally (different directory)

### Acceptance criteria

- [ ] Pipeline sessions (`/spec` → `/design` → `/implement`, `/backlog`) auto-create worktrees
- [ ] Non-pipeline commands (`/audit`, `/verify`, `/review`) run on main tree
- [ ] Merge-to-main is serialized via `flock` on `.locks/merge.lock`
- [ ] Fast verify (`cargo build && cargo test && cargo clippy`) runs before merge
- [ ] Rebase conflicts surfaced to user (merge paused, lock released)
- [ ] Fast verify failures surfaced to user (merge paused, lock released)
- [ ] `action-log.json` writes protected by `flock`
- [ ] `state.json` read-modify-write protected by `flock`
- [ ] `BACKLOG.md` ID generation + append protected by `flock`
- [ ] Work-item file creation uses atomic create (no TOCTOU)
- [ ] Daily memory writes protected by `flock`
- [ ] `.tdd-state` is per-session (in worktree) or locked
- [ ] Worktree cleanup on success (auto-delete)
- [ ] Worktree cleanup command for stale worktrees
- [ ] Two concurrent `/implement` sessions complete without data loss or conflicts
- [ ] CHANGELOG.md updated
- [ ] ADR created for concurrent session architecture

### Scope boundaries — do NOT

- Do not introduce external dependencies (Redis, etcd, SQLite) — use POSIX `flock` only
- Do not change the Claude Code tool behavior — work within existing tool semantics
- Do not make read-only commands (`/audit`, `/verify`) use worktrees — unnecessary overhead
- Do not support concurrent sessions across different machines (network locking out of scope)

### Verification steps

1. Open two terminals in same project, both run `/implement` on different specs → both complete, both merge cleanly
2. Open two terminals, both run `/backlog add` simultaneously → no duplicate BL IDs, no lost entries
3. Kill a session mid-pipeline → worktree preserved, `ecc worktree gc` cleans it up
4. Two sessions merge to main within 1 second of each other → serialized, no conflicts
5. One session's fast verify fails → merge paused, other session's merge proceeds
6. Rebase conflict → user notified, merge paused, lock released for other sessions

## Codebase Audit Results (2026-03-26)

### Files Audited

| File | Reads | Writes | Race Type | Severity |
|------|-------|--------|-----------|----------|
| `.claude/workflow/state.json` | phase-gate, stop-gate, tdd-enforcement | workflow-init, phase-transition, toolchain-persist | TOCTOU, lost write | HIGH |
| `docs/memory/action-log.json` | (read-only) | memory-writer | Read-modify-write | CRITICAL |
| `docs/memory/work-items/**/*.md` | (read-only) | memory-writer | TOCTOU on create | HIGH |
| `docs/memory/daily/{date}.md` | (read-only) | memory-writer | Read-modify-write | MEDIUM |
| `~/.claude/projects/{hash}/memory/MEMORY.md` | (read-only) | memory-writer | Read-modify-write | MEDIUM |
| `docs/backlog/BACKLOG.md` | backlog-curator | /backlog cmd | ID race, lost append | HIGH |
| `docs/specs/**/*` | gates | /spec, /design, /implement | File write race | MEDIUM |
| `.claude/workflow/.tdd-state` | tdd-enforcement | tdd-enforcement | State mutation | MEDIUM |
| `target/` | cargo | cargo | Linker/rmeta race | CRITICAL |
| `/tmp/ecc-sl-cache-{hash}` | statusline | statusline | Cache overwrite | LOW |

### Patterns Found

- **Atomic writes via `mktemp+mv`**: Used in workflow state, memory writer, statusline cache. Prevents partial writes but does NOT prevent read-modify-write races.
- **No locking**: Zero use of `flock`, advisory locks, or PID files anywhere in the codebase.
- **TOCTOU on file checks**: `[ -f "$file" ]` followed by write is used in memory-writer and is inherently racy.
- **Shell append (`>>`)**: Used for BACKLOG.md and daily memory — not atomic under concurrent access.

## Challenge Log

**Q1**: Same project directory or different directories?
**A1**: Same project directory.

**Q2**: Manual concurrent sessions or background agents?
**A2**: Concurrent sessions manually (multiple terminal tabs).

**Q3**: Should audit cover git conflicts too?
**A3**: Yes, all shared state including git.

**Q4**: Full concurrent safety or safe coexistence with warnings?
**A4**: Full concurrent safety.

**Q5**: Two `/implement` on same project — parallel or blocking?
**A5**: Parallel, but a merge reconciliation system ensures no failing code shipped.

**Q6**: Merge reconciliation — automatic or manual?
**A6**: Automatic, leveraging git worktrees/branches.

**Q7**: All sessions get worktrees or only `/implement`?
**A7**: All pipeline sessions (`/spec` → `/design` → `/implement`) + `/backlog`. Not read-only commands.

**Q8**: Should `/verify` run before merge (quality gate)?
**A8**: Full `/verify` is too slow. Fast verify (`cargo build && cargo test && cargo clippy`) is acceptable.

**Q9**: Fast verify (build+test+clippy) acceptable as merge gate?
**A9**: Yes.

**Q10**: Two branches conflict on merge — automatic rebase or surface to user?
**A10**: Attempt automatic rebase. But merge actions must be serialized (one at a time via lock) to prevent two sessions from conflicting during merge.

## Related Backlog Items

- BL-052: Replace shell hooks with Rust binaries — Rust binaries would use stdlib atomic file ops, solving many race conditions natively
- BL-031: Fresh context per TDD task via subagent isolation — already uses worktrees for TDD subagents; this extends to full session isolation
- BL-046: Phase-gate hook — directly affected by state.json TOCTOU race
