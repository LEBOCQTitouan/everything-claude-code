# Solution: Worktree Auto-Merge and Cleanup Enforcement

## Spec Reference
Concern: dev, Feature: force merge at end of work on worktree with auto-delete after merge, safety check before deletion, enforced in workflow.

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/worktree.rs` | Modify | Add `WorktreeSafetyInput` struct, `SafetyViolation` enum, `assess_safety()` pure function | US-001, AC-001.1-8 |
| 2 | `crates/ecc-ports/src/worktree.rs` | Create | New `WorktreeManager` trait with 8 methods, `WorktreeInfo` struct, `WorktreeError` enum | US-002, AC-002.1-4 |
| 3 | `crates/ecc-ports/src/lib.rs` | Modify | Re-export `worktree` module | US-002 |
| 4 | `crates/ecc-infra/src/os_worktree.rs` | Create | `OsWorktreeManager` using git CLI, `--` before all user args | US-002, AC-002.2, AC-002.4 |
| 5 | `crates/ecc-infra/src/lib.rs` | Modify | Re-export `os_worktree` module | US-002 |
| 6 | `crates/ecc-test-support/src/mock_worktree.rs` | Create | `MockWorktreeManager` builder-pattern mock | US-002, AC-002.3 |
| 7 | `crates/ecc-test-support/src/lib.rs` | Modify | Re-export `MockWorktreeManager` | US-002 |
| 8 | `crates/ecc-workflow/src/commands/merge.rs` | Modify | Add safety data gathering (raw Command), cleanup orchestration after ff-merge inside lock | US-003, AC-003.1-12 |
| 9 | `crates/ecc-app/src/worktree.rs` | Modify | Refactor GC to accept `&dyn WorktreeManager`, add merge-status check, wire `--force` | US-004, AC-004.1-7 |
| 10 | `crates/ecc-cli/src/commands/worktree.rs` | Modify | Wire `OsWorktreeManager` to GC, add `Status` subcommand with tab-separated output | US-004, US-005, AC-005.1-5 |
| 11 | `crates/ecc-app/src/hook/handlers/tier3_session/session_merge.rs` | Modify | Update messages: "merged and cleaned up" / "cleanup blocked" | US-006, AC-006.1-5 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Uncommitted changes returns SafetyViolation::UncommittedChanges | AC-001.1 | `cargo test -p ecc-domain -- worktree::tests::assess_uncommitted_changes` | PASS |
| PC-002 | unit | Untracked files returns SafetyViolation::UntrackedFiles | AC-001.2 | `cargo test -p ecc-domain -- worktree::tests::assess_untracked_files` | PASS |
| PC-003 | unit | Unmerged commits returns SafetyViolation::UnmergedCommits | AC-001.3 | `cargo test -p ecc-domain -- worktree::tests::assess_unmerged_commits` | PASS |
| PC-004 | unit | Stashed changes returns SafetyViolation::StashedChanges | AC-001.4 | `cargo test -p ecc-domain -- worktree::tests::assess_stashed_changes` | PASS |
| PC-005 | unit | Unpushed commits returns SafetyViolation::UnpushedCommits | AC-001.5 | `cargo test -p ecc-domain -- worktree::tests::assess_unpushed_commits` | PASS |
| PC-006 | unit | All clean returns empty vec | AC-001.6 | `cargo test -p ecc-domain -- worktree::tests::assess_all_clean` | PASS |
| PC-007 | unit | assess_safety is pure: no std::process/fs/net imports in domain worktree | AC-001.7 | `cargo test -p ecc-domain -- worktree::tests::assess_safety_is_pure` | PASS |
| PC-008 | unit | Multiple unsafe conditions all collected (not short-circuited) | AC-001.8 | `cargo test -p ecc-domain -- worktree::tests::assess_collects_all_failures` | PASS |
| PC-009 | build | WorktreeManager trait compiles with 8 methods | AC-002.1 | `cargo build -p ecc-ports` | exit 0 |
| PC-010 | integration | OsWorktreeManager detects uncommitted changes in temp repo | AC-002.2 | `cargo test -p ecc-infra -- os_worktree::tests::detects_uncommitted` | PASS |
| PC-011 | integration | OsWorktreeManager detects untracked files in temp repo | AC-002.2 | `cargo test -p ecc-infra -- os_worktree::tests::detects_untracked` | PASS |
| PC-012 | integration | OsWorktreeManager counts unmerged commits | AC-002.2 | `cargo test -p ecc-infra -- os_worktree::tests::counts_unmerged` | PASS |
| PC-013 | integration | OsWorktreeManager detects stash | AC-002.2 | `cargo test -p ecc-infra -- os_worktree::tests::detects_stash` | PASS |
| PC-014 | integration | OsWorktreeManager checks push status | AC-002.2 | `cargo test -p ecc-infra -- os_worktree::tests::checks_push_status` | PASS |
| PC-015 | integration | OsWorktreeManager removes worktree | AC-002.2 | `cargo test -p ecc-infra -- os_worktree::tests::removes_worktree` | PASS |
| PC-016 | integration | OsWorktreeManager deletes branch | AC-002.2 | `cargo test -p ecc-infra -- os_worktree::tests::deletes_branch` | PASS |
| PC-017 | integration | OsWorktreeManager lists worktrees | AC-002.2 | `cargo test -p ecc-infra -- os_worktree::tests::lists_worktrees` | PASS |
| PC-018 | unit | MockWorktreeManager returns configured values | AC-002.3 | `cargo test -p ecc-test-support -- mock_worktree::tests` | PASS |
| PC-019 | unit | All OsWorktreeManager git commands use `--` before user args | AC-002.4 | `cargo test -p ecc-infra -- os_worktree::tests::uses_double_dash` | PASS |
| PC-020 | integration | Safety check runs after ff-merge in temp repo | AC-003.1 | `cargo test -p ecc-workflow -- tests::cleanup_runs_after_merge` | PASS |
| PC-021 | integration | Safe worktree is removed via `git worktree remove` | AC-003.2 | `cargo test -p ecc-workflow -- tests::safe_worktree_removed` | PASS |
| PC-022 | integration | Safe worktree branch deleted via `git branch -d` | AC-003.3 | `cargo test -p ecc-workflow -- tests::safe_branch_deleted` | PASS |
| PC-023 | integration | Commands use `current_dir(repo_root)` for deletion | AC-003.4 | `cargo test -p ecc-workflow -- tests::cwd_set_to_repo_root` | PASS |
| PC-024 | integration | Unsafe worktree preserved with failed checks listed | AC-003.5 | `cargo test -p ecc-workflow -- tests::unsafe_worktree_preserved` | PASS |
| PC-025 | integration | Worktree remove failure is warning, not merge failure | AC-003.6 | `cargo test -p ecc-workflow -- tests::remove_failure_is_warning` | PASS |
| PC-026 | unit | Success message says "cleaned up successfully" | AC-003.7 | `cargo test -p ecc-workflow -- tests::success_message_cleaned_up` | PASS |
| PC-027 | integration | Branch delete failure after worktree remove is warning | AC-003.8 | `cargo test -p ecc-workflow -- tests::branch_delete_failure_warning` | PASS |
| PC-028 | unit | CWD failure aborts cleanup and preserves worktree | AC-003.9 | `cargo test -p ecc-workflow -- tests::cwd_failure_aborts_cleanup` | PASS |
| PC-029 | unit | Safety data gathered via std::process::Command, not port | AC-003.10 | `cargo test -p ecc-workflow -- tests::uses_raw_commands` | PASS |
| PC-030 | integration | Safety check is inside merge lock critical section | AC-003.11 | `cargo test -p ecc-workflow -- tests::safety_inside_lock` | PASS |
| PC-031 | integration | Missing worktree dir: prune metadata + branch delete | AC-003.12 | `cargo test -p ecc-workflow -- tests::missing_dir_prunes_metadata` | PASS |
| PC-032 | unit | GC accepts &dyn WorktreeManager | AC-004.1 | `cargo test -p ecc-app -- worktree::tests::gc_uses_worktree_manager` | PASS |
| PC-033 | unit | list_worktrees replaces porcelain parsing | AC-004.2 | `cargo test -p ecc-app -- worktree::tests::gc_uses_list_worktrees` | PASS |
| PC-034 | unit | remove_worktree + delete_branch replace manual commands | AC-004.3 | `cargo test -p ecc-app -- worktree::tests::gc_uses_port_methods` | PASS |
| PC-035 | unit | Existing GC tests pass with MockWorktreeManager | AC-004.4 | `cargo test -p ecc-app -- worktree::tests` | PASS |
| PC-036 | unit | GC still uses age + PID staleness | AC-004.5 | `cargo test -p ecc-app -- worktree::tests::gc_staleness_unchanged` | PASS |
| PC-037 | unit | Unmerged worktrees skipped | AC-004.6 | `cargo test -p ecc-app -- worktree::tests::gc_skips_unmerged` | PASS |
| PC-038 | unit | --force overrides merge-status safety | AC-004.7 | `cargo test -p ecc-app -- worktree::tests::gc_force_overrides_merge_check` | PASS |
| PC-039 | unit | Status returns all 8 columns per worktree | AC-005.1 | `cargo test -p ecc-app -- worktree::tests::status_returns_all_columns` | PASS |
| PC-040 | unit | Non-session worktrees excluded | AC-005.2 | `cargo test -p ecc-app -- worktree::tests::status_excludes_non_session` | PASS |
| PC-041 | unit | Output is human-readable table | AC-005.3 | `cargo test -p ecc-app -- worktree::tests::status_table_format` | PASS |
| PC-042 | unit | Exit code 0 on success | AC-005.4 | `cargo test -p ecc-cli -- commands::worktree::tests::status_exit_zero` | PASS |
| PC-043 | unit | Tab-separated columns with snapshot test | AC-005.5 | `cargo test -p ecc-app -- worktree::tests::status_snapshot` | PASS |
| PC-044 | unit | Hook calls ecc-workflow merge (unchanged mechanism) | AC-006.1 | `cargo test -p ecc-app -- hook::handlers::tier3_session::session_merge::tests::calls_merge_in_worktree` | PASS |
| PC-045 | unit | Success message says "merged and cleaned up" | AC-006.2 | `cargo test -p ecc-app -- hook::handlers::tier3_session::session_merge::tests::merge_success_cleaned_up` | PASS |
| PC-046 | unit | Merge failure behavior unchanged | AC-006.3 | `cargo test -p ecc-app -- hook::handlers::tier3_session::session_merge::tests::rebase_conflict_preserves_worktree` | PASS |
| PC-047 | unit | Cleanup failure lists failed checks | AC-006.4 | `cargo test -p ecc-app -- hook::handlers::tier3_session::session_merge::tests::cleanup_failure_lists_checks` | PASS |
| PC-048 | unit | Bypass via ECC_WORKFLOW_BYPASS=1 works | AC-006.5 | `cargo test -p ecc-app -- hook::handlers::tier3_session::session_merge::tests::bypass_skips_merge` | PASS |
| PC-049 | lint | Clippy passes with zero warnings | All | `cargo clippy -- -D warnings` | exit 0 |
| PC-050 | build | Full workspace builds | All | `cargo build` | exit 0 |
| PC-051 | unit | Full test suite passes | All | `cargo test` | exit 0 |

### Coverage Check

All 41 ACs covered:
- AC-001.1→PC-001, AC-001.2→PC-002, AC-001.3→PC-003, AC-001.4→PC-004, AC-001.5→PC-005, AC-001.6→PC-006, AC-001.7→PC-007, AC-001.8→PC-008
- AC-002.1→PC-009, AC-002.2→PC-010..017, AC-002.3→PC-018, AC-002.4→PC-019
- AC-003.1→PC-020, AC-003.2→PC-021, AC-003.3→PC-022, AC-003.4→PC-023, AC-003.5→PC-024, AC-003.6→PC-025, AC-003.7→PC-026, AC-003.8→PC-027, AC-003.9→PC-028, AC-003.10→PC-029, AC-003.11→PC-030, AC-003.12→PC-031
- AC-004.1→PC-032, AC-004.2→PC-033, AC-004.3→PC-034, AC-004.4→PC-035, AC-004.5→PC-036, AC-004.6→PC-037, AC-004.7→PC-038
- AC-005.1→PC-039, AC-005.2→PC-040, AC-005.3→PC-041, AC-005.4→PC-042, AC-005.5→PC-043
- AC-006.1→PC-044, AC-006.2→PC-045, AC-006.3→PC-046, AC-006.4→PC-047, AC-006.5→PC-048

**Zero uncovered ACs.**

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | WorktreeManager | OsWorktreeManager | WorktreeManager | Full worktree lifecycle: list, safety check, remove, branch delete | ignored | ecc-infra/os_worktree.rs modified |
| 2 | Session Hook | session_merge.rs | ShellExecutor | Session end triggers merge+cleanup, reports result | ignored | session_merge.rs modified |
| 3 | CLI Worktree | worktree.rs (CLI) | WorktreeManager | `ecc worktree status` table output validation | ignored | ecc-cli/commands/worktree.rs modified |
| 4 | Merge+Cleanup | merge.rs | N/A (standalone) | Full merge→safety→cleanup in real git repo | ignored | ecc-workflow/commands/merge.rs modified |

### E2E Activation Rules

All 4 E2E tests un-ignored for this implementation (all 4 boundary files are modified).

## Test Strategy

TDD order (dependency-driven):
1. **PC-001..008** (Phase 1) — Domain pure function, no dependencies
2. **PC-009..019** (Phase 2) — Port trait + adapters, depends on domain types
3. **PC-020..031** (Phase 3) — Merge cleanup, depends on domain `assess_safety`
4. **PC-032..038** (Phase 4) — GC refactor, depends on WorktreeManager port + mock
5. **PC-039..043** (Phase 5) — Status command, depends on WorktreeManager port + mock
6. **PC-044..048** (Phase 6) — Hook update, depends on merge output format (Phase 3)
7. **PC-049..051** (Cross-cutting) — Final quality gates

Within each phase: RED (write failing test, commit) → GREEN (implement, commit) → REFACTOR (clean up, commit).

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `CLAUDE.md` | Project | Update | Add `ecc worktree status` to CLI Commands; update Gotchas worktree lifecycle | US-005, US-003 |
| 2 | `docs/ARCHITECTURE.md` | Architecture | Update | Add WorktreeManager to ports, OsWorktreeManager to adapters | US-002 |
| 3 | `docs/adr/0054-post-merge-worktree-auto-deletion.md` | Decision | Create | ADR covering all 11 decisions from spec | Decisions 1-11 |
| 4 | `CHANGELOG.md` | Release | Update | Entry for worktree auto-merge-cleanup feature | All US |
| 5 | `docs/MODULE-SUMMARIES.md` | Reference | Update | Entries for os_worktree, mock_worktree, worktree port | US-002 |

## SOLID Assessment

**NEEDS WORK** (2 MEDIUM findings from uncle-bob):
1. **MEDIUM — Rename `UnsafeReason` to `SafetyViolation`**: Avoid confusion with Rust's `unsafe` keyword. Applied in this design.
2. **MEDIUM — Evaluate 8-method WorktreeManager for ISP**: During implementation, verify all consumers use at least 5/8 methods. Split into `WorktreeQuery` + `WorktreeLifecycle` if two distinct client groups emerge.

No SRP, OCP, LSP, DIP violations. Clean Architecture dependency rules respected.

## Robert's Oath Check

**CLEAN** — 0 oath warnings. Safety check is "maximum proof" (5-point, collect-all). TDD order enables small releases. GC refactor is Boy Scout improvement. No harmful code patterns.

## Security Notes

**CLEAR** — All git commands safe. `--` separators consistent before user-derived values. `WorktreeName::parse()` allowlist (`[a-zA-Z0-9/_-]`) blocks shell metacharacters. `std::process::Command` passes args as argv (no shell expansion). No TOCTOU concerns (merge lock serializes operations). No auth, secrets, or network calls.

## Rollback Plan

Reverse dependency order — if implementation fails, undo in this order:
1. Revert `session_merge.rs` hook messages (US-006)
2. Revert `worktree.rs` CLI status subcommand (US-005)
3. Revert `worktree.rs` GC refactor (US-004) — restore ShellExecutor usage
4. Revert `merge.rs` cleanup additions (US-003) — restore "preserved" message
5. Remove `mock_worktree.rs` + lib.rs export (US-002)
6. Remove `os_worktree.rs` + lib.rs export (US-002)
7. Remove `worktree.rs` port + lib.rs export (US-002)
8. Revert `worktree.rs` domain additions (US-001) — remove SafetyViolation, WorktreeSafetyInput

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | NEEDS WORK | 2 MEDIUM (rename UnsafeReason → SafetyViolation; evaluate ISP for 8-method trait) |
| Robert | CLEAN | 0 oath warnings |
| Security | CLEAR | 0 findings |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Coverage | 95 | PASS | All 41 ACs covered by 51 PCs, zero gaps |
| Order | 90 | PASS | 6-phase TDD order follows crate dependency graph correctly |
| Fragility | 80 | PASS | PC-007 purity test is fragile but redundant with arch hook |
| Rollback | 75 | PASS | Reverse dependency order correct; doc artifacts omitted but git-revertable |
| Architecture | 90 | PASS | Domain pure, port ISP, ecc-workflow raw commands per Decision 9 |
| Blast Radius | 85 | PASS | 11 files modified, well under 20-file threshold |
| Missing PCs | 80 | PASS | No missing PCs; --exact flag on cargo test commands is impl-time fix |
| Doc Plan | 85 | PASS | 5 doc actions, ADR included, CHANGELOG included |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `crates/ecc-domain/src/worktree.rs` | Modify | US-001 |
| 2 | `crates/ecc-ports/src/worktree.rs` | Create | US-002 |
| 3 | `crates/ecc-ports/src/lib.rs` | Modify | US-002 |
| 4 | `crates/ecc-infra/src/os_worktree.rs` | Create | US-002 |
| 5 | `crates/ecc-infra/src/lib.rs` | Modify | US-002 |
| 6 | `crates/ecc-test-support/src/mock_worktree.rs` | Create | US-002 |
| 7 | `crates/ecc-test-support/src/lib.rs` | Modify | US-002 |
| 8 | `crates/ecc-workflow/src/commands/merge.rs` | Modify | US-003 |
| 9 | `crates/ecc-app/src/worktree.rs` | Modify | US-004 |
| 10 | `crates/ecc-cli/src/commands/worktree.rs` | Modify | US-004, US-005 |
| 11 | `crates/ecc-app/src/hook/handlers/tier3_session/session_merge.rs` | Modify | US-006 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-06-worktree-auto-merge-cleanup/design.md | Full design + Phase Summary |
