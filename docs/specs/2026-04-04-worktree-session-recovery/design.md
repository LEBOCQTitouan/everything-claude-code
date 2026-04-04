# Solution: Worktree Session CWD Orphaning Fix

## Spec Reference
Concern: fix, Feature: worktree-session-recovery

## File Changes (dependency order)
| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-app/src/hook/handlers/tier3_session/session_merge.rs` | modify | Remove `git worktree remove --force .` from zero-commit path; update success message to not claim "cleaned up" | US-001, AC-001.2, AC-001.3, AC-001.5 |
| 2 | `crates/ecc-workflow/src/commands/merge.rs` | modify | Remove `cleanup_worktree()` call from `execute_merge()`; update success message; remove dead `cleanup_worktree` fn, `CleanupFailed` variant, and associated test(s) | US-001, AC-001.1, AC-001.3, AC-001.5 |
| 3 | `crates/ecc-app/src/hook/handlers/tier3_session/lifecycle.rs` | modify | Add best-effort `crate::worktree::gc()` call at top of `session_start()`. Resolve `project_dir` via `ports.shell.run_command("git", &["rev-parse", "--show-toplevel"])`. Swallow all errors. | US-002, AC-002.1, AC-002.2, AC-002.3, AC-002.4 |
| 4 | `CLAUDE.md` | modify | Update Gotchas section: worktree deletion deferred, session-start gc added | US-001, US-002 |
| 5 | `CHANGELOG.md` | modify | Add fix entry | US-001, US-002 |

## Pass Conditions
| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Empty worktree defers to gc (no removal, no "cleaned up") | AC-001.2, AC-001.3, AC-001.5 | `cargo test -p ecc-app empty_worktree_defers_to_gc` | PASS |
| PC-002 | unit | Merge success message has no "cleaned up" claim | AC-001.5 | `cargo test -p ecc-app merge_success_message_no_cleanup_claim` | PASS |
| PC-003 | unit | execute_merge preserves worktree directory (no worktree remove call) | AC-001.1, AC-001.3 | `cargo test -p ecc-workflow merge_preserves_worktree_directory` | PASS |
| PC-004 | unit | execute_merge success message no "cleaned up" | AC-001.5 | `cargo test -p ecc-workflow merge_success_message_no_cleanup` | PASS |
| PC-005 | unit | execute_merge still deletes branch after merge | AC-001.1 | `cargo test -p ecc-workflow merge_deletes_branch_after_merge` | PASS |
| PC-006 | unit | Merge failure preserves worktree with recovery file (existing) | AC-001.4 | `cargo test -p ecc-app rebase_conflict_preserves_worktree` | PASS |
| PC-007 | unit | session_start runs gc and removes stale worktrees | AC-002.1 | `cargo test -p ecc-app session_start_runs_gc` | PASS |
| PC-008 | unit | session_start gc skips worktrees with alive PID | AC-002.2, AC-002.4 | `cargo test -p ecc-app session_start_gc_skips_alive` | PASS |
| PC-009 | unit | session_start gc failure does not block session | AC-002.3 | `cargo test -p ecc-app session_start_gc_failure_non_blocking` | PASS |
| PC-010 | lint | clippy passes both crates | All | `cargo clippy -p ecc-app -p ecc-workflow -- -D warnings` | 0 warnings |
| PC-011 | build | workspace builds | All | `cargo build --workspace` | exit 0 |
| PC-012 | suite | full test suite passes | All | `cargo test --workspace` | exit 0 |
| PC-013 | lint | cargo fmt check passes | All | `cargo fmt --check` | exit 0 |

### Coverage Check
| AC | Covered By |
|----|-----------|
| AC-001.1 | PC-003, PC-005 |
| AC-001.2 | PC-001 |
| AC-001.3 | PC-001, PC-003 |
| AC-001.4 | PC-006 |
| AC-001.5 | PC-001, PC-002, PC-004 |
| AC-002.1 | PC-007 |
| AC-002.2 | PC-008 |
| AC-002.3 | PC-009 |
| AC-002.4 | PC-008 |

All 9 ACs covered. Zero uncovered.

### E2E Test Plan
| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | Session hooks | session_merge | ShellExecutor | Merge completes without worktree deletion | ignored | session_merge.rs modified |
| 2 | Session hooks | lifecycle | ShellExecutor | GC runs at session start | ignored | lifecycle.rs modified |

### E2E Activation Rules
Both boundaries activated for this implementation. Fully testable as unit tests with MockExecutor — no separate E2E infrastructure required.

## Test Strategy
TDD order:
1. PC-001, PC-002 — session_merge.rs tests (adapter, no dependencies)
2. PC-003, PC-004, PC-005 — merge.rs tests (standalone binary, independent)
3. PC-006 — verify existing test unchanged
4. PC-007, PC-008, PC-009 — lifecycle.rs tests (depends on worktree.rs gc being callable)
5. PC-010, PC-011, PC-012 — lint, build, suite gates

## Doc Update Plan
| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CLAUDE.md | Gotchas | Update | Worktree deletion deferred to gc; session-start gc added | US-001, US-002 |
| 2 | CHANGELOG.md | Release | Add | `fix: defer worktree deletion from merge/hook paths to prevent CWD orphaning` | US-001, US-002 |

## SOLID Assessment
CLEAN (uncle-bob). No SOLID violations. All changes respect hex architecture dependency rules. `lifecycle.rs` calling `crate::worktree::gc()` is an intra-crate same-layer call.

## Robert's Oath Check
CLEAN. 0 oath warnings. Design is purely subtractive in critical path, additive only in non-critical gc path. 9 unit tests planned. Rework ratio 0.16 (healthy).

## Security Notes
CLEAR. One LOW informational: theoretical PID-reuse TOCTOU in gc, mitigated by 24h age check + large PID space. No actionable findings.

## Rollback Plan
Reverse order:
1. Revert lifecycle.rs (remove gc from session_start)
2. Revert merge.rs (restore cleanup_worktree call and success message)
3. Revert session_merge.rs (restore git worktree remove in zero-commit path)
4. Revert CLAUDE.md and CHANGELOG.md

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | CLEAN | 0 |
| Robert (oath) | CLEAN | 0 |
| Security | CLEAR | 1 LOW (informational) |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Coverage | 90 | PASS | All 9 ACs mapped to PCs |
| Order | 85 | PASS | TDD dependency order correct |
| Fragility | 80 | PASS | Subtractive changes, low risk |
| Rollback | 90 | PASS | Reverse dependency order specified |
| Architecture | 88 | PASS | No layer violations, intra-crate calls |
| Blast Radius | 85 | PASS | 3 files, ~50-60 lines |
| Missing PCs | 85 | PASS | PC-013 fmt added in round 2 |
| Doc Plan | 85 | PASS | CLAUDE.md + CHANGELOG covered |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `crates/ecc-app/src/hook/handlers/tier3_session/session_merge.rs` | modify | US-001, AC-001.2, AC-001.3, AC-001.5 |
| 2 | `crates/ecc-workflow/src/commands/merge.rs` | modify | US-001, AC-001.1, AC-001.3, AC-001.5 |
| 3 | `crates/ecc-app/src/hook/handlers/tier3_session/lifecycle.rs` | modify | US-002, AC-002.1-AC-002.4 |
| 4 | `CLAUDE.md` | modify | US-001, US-002 |
| 5 | `CHANGELOG.md` | modify | US-001, US-002 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-04-worktree-session-recovery/design.md | Full design |
| docs/specs/2026-04-04-worktree-session-recovery/spec.md | Full spec |
| docs/specs/2026-04-04-worktree-session-recovery/campaign.md | Campaign manifest |
