# Solution: Fix Worktree GC PID Bug (BL-150)

## Spec Reference
Concern: fix, Feature: worktree gc pid bug. Spec: `docs/specs/2026-04-17-worktree-gc-pid-bug/spec.md`.

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|---|---|---|---|
| 1 | `crates/ecc-workflow/src/commands/worktree_name.rs` | modify | `process::id()` → `parent_id()` | US-001, AC-001.1 |
| 2 | `crates/ecc-app/src/worktree/mod.rs` | modify | Rename `WorktreeError` → `WorktreeGcError` | US-004, AC-004.1 |
| 3 | `crates/ecc-app/src/worktree/gc.rs` | modify | Update import + `.unwrap_or(0)` → `.unwrap_or(u64::MAX)` | US-002, AC-002.1; US-004, AC-004.2 |
| 4 | `crates/ecc-app/src/worktree/status.rs` | modify | Update import `WorktreeGcError` | US-004, AC-004.2 |
| 5 | `crates/ecc-app/src/worktree/mod.rs` | modify | Add `RECENCY_SECS` constant + `worktree_path` param + recency check in `is_worktree_stale` | US-003, AC-003.1/5/6 |
| 6 | `crates/ecc-app/src/worktree/gc.rs` | modify | Pass `worktree_path` to `is_worktree_stale` (move path construction before staleness check) | US-003, AC-003.5 |
| 7 | `crates/ecc-app/src/worktree/status.rs` | modify | Update `is_worktree_stale` call site to pass worktree_path (4th param) | US-003, AC-003.5 |
| 8 | `CLAUDE.md` | modify | Add BL-150 temporary gotcha | US-005, AC-005.1 |
| 9 | `CHANGELOG.md` | modify | Add `[Unreleased]` Fixed entry | Doc impact |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|---|---|---|---|---|---|
| PC-001 | unit | parent_id in generated name, existing tests pass | AC-001.1, AC-001.2, AC-001.3 | `cargo test -p ecc-workflow -- worktree_name` | PASS |
| PC-002 | build | WorktreeGcError rename compiles | AC-004.1, AC-004.2, AC-004.3 | `cargo build --workspace` | exit 0 |
| PC-003 | lint | unwrap_or(u64::MAX) present in source | AC-002.1 | `grep -n 'unwrap_or(u64::MAX)' crates/ecc-app/src/worktree/gc.rs` | 1 match |
| PC-004 | unit | GC skips worktree when unmerged query fails | AC-002.2 | `cargo test -p ecc-app -- gc::tests::gc_skips_when_unmerged_query_fails` | PASS |
| PC-005 | unit | GC still removes stale when Ok(0) (regression) | AC-002.3 | `cargo test -p ecc-app -- gc::tests::removes_stale` | PASS |
| PC-006 | build | is_worktree_stale new signature compiles | AC-003.1, AC-003.5 | `cargo build --workspace` | exit 0 |
| PC-007 | unit | Recently modified worktree = not stale | AC-003.2 | `cargo test -p ecc-app -- worktree::tests::recently_modified_worktree_is_not_stale` | PASS |
| PC-008 | unit | Old + dead PID + old mtime = stale | AC-003.3 | `cargo test -p ecc-app -- worktree::tests::old_unmodified_worktree_is_stale` | PASS |
| PC-009 | unit | Stat failure = existing behavior | AC-003.4 | `cargo test -p ecc-app -- worktree::tests::stat_failure_preserves_existing_behavior` | PASS |
| PC-009b | unit | Malformed stat output (valid exit 0 but non-numeric stdout) = treat as stat failure | AC-003.4 | `cargo test -p ecc-app -- worktree::tests::malformed_stat_output_treated_as_failure` | PASS |
| PC-010 | unit | Live PID overrides old mtime | AC-003.6 | `cargo test -p ecc-app -- worktree::tests::live_pid_overrides_old_modification` | PASS |
| PC-011 | lint | CLAUDE.md gotcha present | AC-005.1 | `grep 'TEMPORARY (BL-150)' CLAUDE.md` | 1 match |
| PC-012 | build | Full test suite | all | `cargo test --workspace` | PASS |
| PC-013 | lint | Clippy clean | all | `cargo clippy --workspace -- -D warnings` | exit 0 |

### Coverage Check
All 16 ACs covered:
- AC-001.1/2/3 → PC-001
- AC-002.1 → PC-003; AC-002.2 → PC-004; AC-002.3 → PC-005
- AC-003.1/5 → PC-006; AC-003.2 → PC-007; AC-003.3 → PC-008; AC-003.4 → PC-009; AC-003.6 → PC-010
- AC-004.1/2/3 → PC-002
- AC-005.1 → PC-011

Zero uncovered.

### E2E Test Plan
None. No port/adapter contracts modified.

### E2E Activation Rules
None.

### Stat Command Specification

The recency check in `is_worktree_stale` uses the ShellExecutor to run `stat`. Platform handling:
- **macOS**: `stat -f %m <path>` → outputs epoch seconds on stdout
- **Linux**: `stat -c %Y <path>` → outputs epoch seconds on stdout
- **Implementation**: try macOS format first; if it fails (non-zero exit), try Linux format. If both fail → stat failure → AC-003.4 (skip recency guard).
- **Parsing**: `stdout.trim().parse::<u64>()`. If parse fails (non-numeric output, empty) → treat as stat failure (AC-003.4, PC-009b).

## Test Strategy

TDD order:
1. PC-001: Fix PID source + verify existing tests (US-001)
2. PC-002: Rename WorktreeError → WorktreeGcError (US-004)
3. PC-003: Change .unwrap_or(0) → .unwrap_or(u64::MAX) (US-002)
4. PC-004: Write test for unmerged query failure → GC skips (RED→GREEN)
5. PC-005: Verify removes_stale regression test still passes
6. PC-006: Add worktree_path param to is_worktree_stale + update call sites
7. PC-007/008/009/010: Write 4 recency guard tests (RED) → implement recency check (GREEN)
8. PC-011: Add CLAUDE.md gotcha
9. PC-012: Full test suite
10. PC-013: Clippy gate

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|---|---|---|---|---|
| 1 | `CLAUDE.md` | Project | Add gotcha | "TEMPORARY (BL-150): Do not run parallel Claude Code sessions in the same repo — worktree GC may delete active worktrees. Remove after BL-150 ships." | US-005, AC-005.1 |
| 2 | `CHANGELOG.md` | Project | Add entry | `[Unreleased]` Fixed: worktree GC PID bug, fail-safe unmerged count, recency guard | all |

No ADRs required (all decisions marked No in spec).

## SOLID Assessment
CLEAN (0 findings). DIP correct (stat via ShellExecutor port), SRP intact (one question: is stale?), OCP acceptable (pub(crate) 2-callsite change).

## Robert's Oath Check
CLEAN (0 warnings). Rework ratio 0.18 (healthy).

## Security Notes
CLEAR. parent_id() returns u32 — no injection. Stat mtime parser fails safely on non-numeric input (AC-003.4).

## Rollback Plan
Reverse order:
1. Revert CLAUDE.md gotcha
2. Revert CHANGELOG entry
3. Revert is_worktree_stale recency check + revert worktree_path param at call sites
4. Revert .unwrap_or(u64::MAX) → .unwrap_or(0)
5. Revert WorktreeGcError → WorktreeError rename
6. Revert parent_id() → process::id()

Each step is an independent atomic commit, revertable via `git revert <sha>`.

## Bounded Contexts Affected

No bounded contexts affected. The worktree module (`ecc-app/src/worktree/`) is not registered in `docs/domain/bounded-contexts.md`.

Other domain modules (not registered as bounded contexts):
- `worktree`: `crates/ecc-app/src/worktree/{mod.rs, gc.rs, status.rs}`
- `worktree_name`: `crates/ecc-workflow/src/commands/worktree_name.rs`
