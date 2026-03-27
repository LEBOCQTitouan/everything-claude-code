# Solution: BL-065 Sub-Spec A — Lock Infrastructure

## Spec Reference
Concern: dev, Feature: BL-065 Concurrent session safety — Sub-Spec A: Lock Infrastructure (US-001, 8 ACs)

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `Cargo.toml` (workspace) | Modify | Add `libc = "0.2"` to workspace deps | US-001 |
| 2 | `crates/ecc-ports/src/lock.rs` | Create | FileLock trait (RAII LockGuard), LockError enum | AC-001.1, AC-001.8 |
| 3 | `crates/ecc-ports/src/lib.rs` | Modify | Add `pub mod lock;` | AC-001.1 |
| 4 | `crates/ecc-infra/Cargo.toml` | Modify | Add libc dep (cfg unix) + tempfile dev-dep | AC-001.2 |
| 5 | `crates/ecc-infra/src/flock_lock.rs` | Create | FlockLock (#[cfg(unix)]) + resolve_repo_root | AC-001.2-5 |
| 6 | `crates/ecc-infra/src/lib.rs` | Modify | Add `#[cfg(unix)] pub mod flock_lock;` | AC-001.2 |
| 7 | `crates/ecc-test-support/src/in_memory_lock.rs` | Create | InMemoryLock test double | AC-001.6 |
| 8 | `crates/ecc-test-support/src/lib.rs` | Modify | Add `pub mod in_memory_lock;` | AC-001.6 |
| 9 | `crates/ecc-integration-tests/tests/flock_contention.rs` | Create | Multi-process contention test | AC-001.7 |
| 10 | `.gitignore` | Modify | Add `.claude/workflow/.locks/` | AC-001.3 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | build | FileLock trait + LockError compiles | AC-001.1, AC-001.8 | `cargo build -p ecc-ports` | exit 0 |
| PC-002 | unit | ecc-ports tests pass | AC-001.1, AC-001.8 | `cargo test -p ecc-ports` | exit 0 |
| PC-003 | unit | FlockLock auto-creates .locks/ + LockGuard Drop releases | AC-001.3, AC-001.4 | `cargo test -p ecc-infra -- flock` | exit 0 |
| PC-004 | unit | FlockLock acquire/release round-trip + timeout | AC-001.2 | `cargo test -p ecc-infra -- flock` | exit 0 |
| PC-005 | unit | InMemoryLock test double | AC-001.6 | `cargo test -p ecc-test-support -- in_memory_lock` | exit 0 |
| PC-006 | unit | resolve_repo_root fallback | AC-001.5 | `cargo test -p ecc-infra -- resolve_repo_root` | exit 0 |
| PC-007 | integration | Multi-process flock contention (Command-based) | AC-001.2, AC-001.4, AC-001.7 | `cargo test -p ecc-integration-tests -- --ignored flock_contention` | exit 0 |
| PC-008 | lint | Clippy clean | All | `cargo clippy --workspace -- -D warnings` | exit 0 |
| PC-009 | build | Full workspace builds | All | `cargo build --workspace` | exit 0 |
| PC-010 | build | All non-ignored tests pass | All | `cargo test --workspace` | exit 0 |

### Coverage Check

All 8 ACs covered:
- AC-001.1: PC-001, PC-002
- AC-001.2: PC-004, PC-007
- AC-001.3: PC-003
- AC-001.4: PC-003, PC-007
- AC-001.5: PC-006
- AC-001.6: PC-005
- AC-001.7: PC-007
- AC-001.8: PC-001, PC-002

Zero uncovered.

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | FileLock | FlockLock | FileLock trait | Multi-process contention | ignored | ecc-infra lock module modified |

### E2E Activation Rules
Activate flock_contention test during this implementation.

## Test Strategy

TDD order:
1. **PC-001–002** (Port trait: FileLock + LockError in ecc-ports)
2. **PC-003–004** (FlockLock adapter: RAII Drop, auto-create, timeout)
3. **PC-005** (InMemoryLock test double)
4. **PC-006** (resolve_repo_root standalone function)
5. **PC-007** (Multi-process integration test)
6. **PC-008–010** (Final gate)

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CHANGELOG.md | Minor | Add entry | "Lock infrastructure — FileLock port trait + FlockLock adapter (BL-065-A)" | US-001 |

ADR deferred to Sub-Spec D (full EPIC documentation).

## SOLID Assessment
**PASS** — Clean hexagonal: trait in ports, impl in infra, double in test-support. DIP respected. Send+Sync bound.

## Robert's Oath Check
**CLEAN** — RAII prevents lock leaks (Oath 2: no mess). Multi-level tests (Oath 3: proof). Per-phase commits (Oath 4: small releases).

## Security Notes
**CLEAR** — Local POSIX advisory locking. Minimal `unsafe` for libc::flock (necessary, documented). No secrets, no network.

## Rollback Plan
1. Remove flock_contention.rs integration test
2. Remove InMemoryLock
3. Remove FlockLock + resolve_repo_root
4. Remove FileLock trait + LockError
5. Remove libc workspace dep
6. Revert .gitignore

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS | 0 |
| Robert | CLEAN | 0 |
| Security | CLEAR | 0 |

### Adversary Findings

| Dimension | Verdict | Key Rationale |
|-----------|---------|---------------|
| Coverage | PASS (round 3) | RAII guard, LockError, timeout, path separation, Command-based test |
| Order | PASS | Hexagonal layer order |
| Fragility | PASS | cfg(unix) gate, tempdir isolation |
| Rollback | PASS | Purely additive |
| Architecture | PASS (round 3) | Path resolution separated from lock trait |
| Blast radius | PASS | 10 files, 4 new + 6 minor modifications |
| Missing PCs | PASS (round 3) | AC-001.8 added, timeout AC added |
| Doc plan | PASS | CHANGELOG only, ADR deferred |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | Cargo.toml | Modify | US-001 |
| 2 | crates/ecc-ports/src/lock.rs | Create | AC-001.1, AC-001.8 |
| 3 | crates/ecc-ports/src/lib.rs | Modify | AC-001.1 |
| 4 | crates/ecc-infra/Cargo.toml | Modify | AC-001.2 |
| 5 | crates/ecc-infra/src/flock_lock.rs | Create | AC-001.2-5 |
| 6 | crates/ecc-infra/src/lib.rs | Modify | AC-001.2 |
| 7 | crates/ecc-test-support/src/in_memory_lock.rs | Create | AC-001.6 |
| 8 | crates/ecc-test-support/src/lib.rs | Modify | AC-001.6 |
| 9 | crates/ecc-integration-tests/tests/flock_contention.rs | Create | AC-001.7 |
| 10 | .gitignore | Modify | AC-001.3 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-27-concurrent-session-safety/spec.md | Full spec (updated ACs) |
| docs/specs/2026-03-27-concurrent-session-safety/design-sub-spec-a.md | Full design + phase summary |
