# Tasks: BL-065 Sub-Spec B — Shared State Locking

## Pass Conditions

- [ ] PC-001: ecc-flock crate compiles | `cargo build -p ecc-flock` | pending@2026-03-28T12:00:00Z
- [ ] PC-002: FlockGuard RAII drop releases lock | `cargo test -p ecc-flock -- guard_drop_releases` | pending@2026-03-28T12:00:00Z
- [ ] PC-003: acquire creates lock dir and file | `cargo test -p ecc-flock -- acquire_creates_lock` | pending@2026-03-28T12:00:00Z
- [ ] PC-004: ecc-infra FlockLock uses ecc-flock | `cargo test -p ecc-infra` | pending@2026-03-28T12:00:00Z
- [ ] PC-015: with_state_lock RAII correctness | `cargo test -p ecc-workflow -- with_state_lock_raii` | pending@2026-03-28T12:00:00Z
- [ ] PC-016: init acquires state lock | `cargo test -p ecc-workflow -- init_acquires_state_lock` | pending@2026-03-28T12:00:00Z
- [ ] PC-005: Two concurrent transitions serialize | `cargo test -p ecc-workflow --test state_lock_contention -- --ignored` | pending@2026-03-28T12:00:00Z
- [ ] PC-006: Init concurrent with transition — no loss | `cargo test -p ecc-workflow --test state_lock_contention -- --ignored` | pending@2026-03-28T12:00:00Z
- [ ] PC-007: Phase-gate reads post-transition state | `cargo test -p ecc-workflow --test state_lock_contention -- --ignored` | pending@2026-03-28T12:00:00Z
- [ ] PC-017: phase_gate reads under state lock | `cargo test -p ecc-workflow -- phase_gate_reads_under_lock` | pending@2026-03-28T12:00:00Z
- [ ] PC-008: Two concurrent action-log writes | `cargo test -p ecc-workflow --test action_log_contention -- --ignored` | pending@2026-03-28T12:00:00Z
- [ ] PC-012: Each memory type uses dedicated lock name | `cargo test -p ecc-workflow -- uses_correct_lock` | pending@2026-03-28T12:00:00Z
- [ ] PC-009: Two concurrent daily writes | `cargo test -p ecc-workflow --test memory_lock_contention -- --ignored` | pending@2026-03-28T12:00:00Z
- [ ] PC-010: Two concurrent MEMORY.md updates | `cargo test -p ecc-workflow --test memory_lock_contention -- --ignored` | pending@2026-03-28T12:00:00Z
- [ ] PC-011: Two concurrent work-item writes — revision | `cargo test -p ecc-workflow --test memory_lock_contention -- --ignored` | pending@2026-03-28T12:00:00Z
- [ ] PC-013: Two concurrent backlog add-entry — unique IDs | `cargo test -p ecc-workflow --test backlog_contention -- --ignored` | pending@2026-03-28T12:00:00Z
- [ ] PC-014: Two concurrent backlog add-entry — both entries | `cargo test -p ecc-workflow --test backlog_contention -- --ignored` | pending@2026-03-28T12:00:00Z
- [ ] PC-018: clippy clean | `cargo clippy -- -D warnings` | pending@2026-03-28T12:00:00Z
- [ ] PC-019: cargo build succeeds | `cargo build` | pending@2026-03-28T12:00:00Z
- [ ] PC-020: All tests pass | `cargo test` | pending@2026-03-28T12:00:00Z

## Post-TDD

- [ ] E2E tests | pending@2026-03-28T12:00:00Z
- [ ] Code review | pending@2026-03-28T12:00:00Z
- [ ] Doc updates | pending@2026-03-28T12:00:00Z
- [ ] Supplemental docs | pending@2026-03-28T12:00:00Z
- [ ] Write implement-done.md | pending@2026-03-28T12:00:00Z
