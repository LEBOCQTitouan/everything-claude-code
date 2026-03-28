# Design: BL-065 Sub-Spec B — Shared State Locking

## Spec Reference
Concern: dev, Feature: BL-065 Sub-Spec B: Shared state locking in ecc-workflow

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-flock/Cargo.toml` | CREATE | Shared flock utility crate (9th in workspace), libc only | US-002-005 |
| 2 | `crates/ecc-flock/src/lib.rs` | CREATE | FlockGuard + acquire + lock_dir + ensure_lock_dir | US-002-005 |
| 3 | `Cargo.toml` (workspace root) | MODIFY | Add ecc-flock to members + workspace deps | US-002-005 |
| 4 | `crates/ecc-infra/Cargo.toml` | MODIFY | Add ecc-flock dep | — |
| 5 | `crates/ecc-infra/src/flock_lock.rs` | MODIFY | Delegate raw flock to ecc-flock | — |
| 6 | `crates/ecc-workflow/Cargo.toml` | MODIFY | Add ecc-flock dep | US-002-005 |
| 7 | `crates/ecc-workflow/src/io.rs` | MODIFY | Add with_state_lock helper | US-003 |
| 8 | `crates/ecc-workflow/src/commands/init.rs` | MODIFY | Wrap in state lock | US-003, AC-003.2 |
| 9 | `crates/ecc-workflow/src/commands/transition.rs` | MODIFY | Wrap in state lock | US-003, AC-003.1 |
| 10 | `crates/ecc-workflow/src/commands/toolchain_persist.rs` | MODIFY | Wrap in state lock | US-003 |
| 11 | `crates/ecc-workflow/src/commands/phase_gate.rs` | MODIFY | Acquire state lock for reads | US-003, AC-003.3 |
| 12 | `crates/ecc-workflow/src/commands/memory_write.rs` | MODIFY | Per-type locks | US-002, US-005 |
| 13 | `crates/ecc-workflow/src/commands/backlog.rs` | CREATE | Atomic add-entry subcommand | US-004 |
| 14 | `crates/ecc-workflow/src/commands/mod.rs` | MODIFY | Add pub mod backlog | US-004 |
| 15 | `crates/ecc-workflow/src/main.rs` | MODIFY | Add Backlog subcommand | US-004 |
| 16 | CHANGELOG.md | MODIFY | Add Sub-Spec B entry | — |

## Architecture Note

ecc-flock is a raw utility crate (depends on `libc` only, no traits, no ports). It provides `FlockGuard` struct with RAII Drop and `acquire()` function. ecc-infra's `FlockLock` delegates to ecc-flock for raw flock mechanics while still implementing the `FileLock` port trait. ecc-workflow uses ecc-flock directly (raw, no trait). This respects spec Decision 8 — ecc-workflow does not depend on ecc-ports.

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | build | ecc-flock crate compiles | — | `cargo build -p ecc-flock` | exit 0 |
| PC-002 | unit | FlockGuard RAII drop releases lock | AC-003.1 | `cargo test -p ecc-flock -- guard_drop_releases` | PASS |
| PC-003 | unit | acquire creates lock dir and file | AC-003.1 | `cargo test -p ecc-flock -- acquire_creates_lock` | PASS |
| PC-004 | integration | ecc-infra FlockLock uses ecc-flock (no regression) | — | `cargo test -p ecc-infra` | PASS |
| PC-005 | integration | Two concurrent transitions serialize | AC-003.1 | `cargo test -p ecc-workflow --test state_lock_contention -- --ignored` | PASS |
| PC-006 | integration | Init concurrent with transition — no loss | AC-003.2 | `cargo test -p ecc-workflow --test state_lock_contention -- --ignored` | PASS |
| PC-007 | integration | Phase-gate reads post-transition state | AC-003.3 | `cargo test -p ecc-workflow --test state_lock_contention -- --ignored` | PASS |
| PC-008 | integration | Two concurrent action-log writes — both appear | AC-002.1, AC-002.2 | `cargo test -p ecc-workflow --test action_log_contention -- --ignored` | PASS |
| PC-009 | integration | Two concurrent daily writes — both appear | AC-005.1 | `cargo test -p ecc-workflow --test memory_lock_contention -- --ignored` | PASS |
| PC-010 | integration | Two concurrent MEMORY.md updates — both appear | AC-005.2 | `cargo test -p ecc-workflow --test memory_lock_contention -- --ignored` | PASS |
| PC-011 | integration | Two concurrent work-item writes — revision appended | AC-005.3 | `cargo test -p ecc-workflow --test memory_lock_contention -- --ignored` | PASS |
| PC-012 | unit | Each memory type uses dedicated lock name | AC-005.4 | `cargo test -p ecc-workflow -- uses_correct_lock` | PASS |
| PC-013 | integration | Two concurrent backlog add-entry — unique IDs | AC-004.1 | `cargo test -p ecc-workflow --test backlog_contention -- --ignored` | PASS |
| PC-014 | integration | Two concurrent backlog add-entry — both entries | AC-004.2 | `cargo test -p ecc-workflow --test backlog_contention -- --ignored` | PASS |
| PC-015 | unit | with_state_lock acquires before closure, releases after | AC-003.1 | `cargo test -p ecc-workflow -- with_state_lock_raii` | PASS |
| PC-016 | unit | init acquires state lock around archive+write | AC-003.2 | `cargo test -p ecc-workflow -- init_acquires_state_lock` | PASS |
| PC-017 | unit | phase_gate reads under state lock | AC-003.3 | `cargo test -p ecc-workflow -- phase_gate_reads_under_lock` | PASS |
| PC-018 | lint | clippy clean | — | `cargo clippy -- -D warnings` | exit 0 |
| PC-019 | build | cargo build succeeds | — | `cargo build` | exit 0 |
| PC-020 | unit | All tests pass | — | `cargo test` | all pass |

### Coverage Check
All 11 ACs covered — AC-002.1→PC-008, AC-002.2→PC-008, AC-003.1→PC-002/003/005/015, AC-003.2→PC-006/016, AC-003.3→PC-007/017, AC-004.1→PC-013, AC-004.2→PC-014, AC-005.1→PC-009, AC-005.2→PC-010, AC-005.3→PC-011, AC-005.4→PC-012.

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | ecc-flock | FlockGuard | — | Multi-process flock contention | ignored | ecc-flock modified |
| 2 | ecc-workflow | state.json | — | Concurrent transition serialization | ignored | lock logic modified |

### E2E Activation Rules
All contention tests (PC-005 through PC-014) run with `--ignored` flag during this implementation.

## Test Strategy

TDD order:
1. PC-001 (ecc-flock compiles)
2. PC-002 (FlockGuard RAII)
3. PC-003 (acquire creates lock)
4. PC-004 (ecc-infra regression)
5. PC-015 (with_state_lock RAII)
6. PC-016 (init acquires lock)
7. PC-005 (concurrent transitions)
8. PC-006 (init + transition concurrency)
9. PC-017 (phase-gate under lock)
10. PC-007 (phase-gate concurrency)
11. PC-008 (action-log concurrency)
12. PC-012 (lock name verification)
13. PC-009, PC-010, PC-011 (memory concurrency)
14. PC-013, PC-014 (backlog concurrency)
15. PC-018, PC-019, PC-020 (final gates)

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CHANGELOG.md | project | Add entry | BL-065 Sub-Spec B shared state locking | US-002-005 |

## SOLID Assessment
NEEDS WORK → resolved by extracting ecc-flock crate (eliminates DRY/CCP). Backlog TOCTOU resolved by single atomic add-entry.

## Robert's Oath Check
3 warnings → resolved by ecc-flock extraction (Oath 2, 5, 7).

## Security Notes
CLEAR — no findings.

## Rollback Plan
1. Remove backlog.rs, revert mod.rs/main.rs
2. Remove locks from memory_write.rs
3. Remove locks from phase_gate/toolchain_persist/transition/init
4. Remove with_state_lock from io.rs
5. Revert ecc-infra to direct libc::flock
6. Remove ecc-flock crate
