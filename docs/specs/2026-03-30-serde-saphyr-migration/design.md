# Solution: Migrate serde_yml to serde-saphyr (BL-099)

## Spec Reference
Concern: refactor, Feature: Migrate serde_yml to serde-saphyr (BL-099)

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `Cargo.toml` | Modify | Replace `serde_yml = "0.0.12"` with `serde-saphyr = "0.0.22"` in workspace deps | AC-001.1 |
| 2 | `crates/ecc-domain/Cargo.toml` | Modify | Replace `serde_yml = { workspace = true }` with `serde-saphyr = { workspace = true }` | AC-001.2 |
| 3 | `crates/ecc-domain/src/backlog/entry.rs` | Modify | Replace 5 `serde_yml::from_str` calls with `serde_saphyr::from_str` | AC-001.3 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | build | serde_yml absent from workspace Cargo.toml | AC-001.1 | `grep -c "serde_yml" Cargo.toml` | 0 |
| PC-002 | build | serde-saphyr present in ecc-domain Cargo.toml | AC-001.2 | `grep -c "serde-saphyr" crates/ecc-domain/Cargo.toml` | >= 1 |
| PC-003 | build | No serde_yml references in entry.rs | AC-001.3 | `grep -c "serde_yml" crates/ecc-domain/src/backlog/entry.rs` | 0 |
| PC-004 | unit | Backlog YAML tests pass with serde-saphyr | AC-001.4 | `cargo test -p ecc-domain backlog::entry` | All PASS |
| PC-005 | build | cargo deny check passes | AC-001.5 | `cargo deny check` | exit 0 |
| PC-006 | unit | No tests assert on error text | AC-001.6 | `grep -c 'MalformedYaml("' crates/ecc-domain/src/backlog/entry.rs` | 0 |
| PC-007 | lint | Clippy clean | AC-001.7 | `cargo clippy --workspace -- -D warnings` | exit 0 |
| PC-008 | test | Full workspace tests pass | AC-001.8 | `cargo test --workspace` | All PASS |

### Coverage Check

All 8 ACs covered:
- AC-001.1 → PC-001
- AC-001.2 → PC-002
- AC-001.3 → PC-003
- AC-001.4 → PC-004
- AC-001.5 → PC-005
- AC-001.6 → PC-006
- AC-001.7 → PC-007
- AC-001.8 → PC-008

Zero uncovered.

### E2E Test Plan

No E2E tests — pure domain-layer dependency swap with no port/adapter changes.

### E2E Activation Rules

None activated.

## Test Strategy

TDD order (single phase — all changes are atomic):
1. **PC-001, PC-002, PC-003**: Swap crate in Cargo.toml files and update import paths
2. **PC-004**: Run backlog entry tests to verify serde-saphyr compatibility
3. **PC-006**: Verify no error text assertions exist
4. **PC-005**: Run cargo deny check for supply chain health
5. **PC-007**: Run clippy
6. **PC-008**: Full workspace test suite

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/adr/0034-serde-saphyr-migration.md` | ADR | Create | Decision: serde-saphyr over serde-yaml-ng; resolves RUSTSEC-2025-0068 | US-001, Decision 1 |
| 2 | `CHANGELOG.md` | Project | Add entry | BL-099: serde-saphyr migration | US-001 |
| 3 | `docs/backlog/BL-099-*.md` | Metadata | Mark implemented | status: open → implemented | US-001 |
| 4 | `docs/backlog/BACKLOG.md` | Metadata | Update index row | BL-099 → implemented | US-001 |

## SOLID Assessment

**PASS** — Pure dependency swap. No structural change. Same function signatures, different underlying crate. No SOLID principles are affected.

## Robert's Oath Check

**CLEAN** — Resolves a known security advisory (RUSTSEC-2025-0068). Replaces unsafe code with a panic-free pure-Rust alternative. Small, focused change. Well-tested.

## Security Notes

**CLEAR** — This change *improves* security by removing a crate with a known segfault advisory and replacing it with a pure-Rust, panic-free alternative that eliminates the unsafe-libyaml dependency chain.

## Rollback Plan

Reverse dependency order:
1. Revert `crates/ecc-domain/src/backlog/entry.rs` — restore `serde_yml::from_str` calls
2. Revert `crates/ecc-domain/Cargo.toml` — restore `serde_yml = { workspace = true }`
3. Revert `Cargo.toml` — restore `serde_yml = "0.0.12"`
4. Run `cargo update` to regenerate Cargo.lock
