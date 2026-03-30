# Solution: Adopt Web Radar Findings — 4 Tooling Upgrades

## Spec Reference
Concern: refactor, Feature: Adopt 4 Adopt findings from web radar

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | Cargo.toml | modify | Replace serde_yaml workspace dep with serde_yml | AC-001.1 |
| 2 | crates/ecc-domain/Cargo.toml | modify | Replace serde_yaml with serde_yml | AC-001.1 |
| 3 | crates/ecc-domain/src/backlog/entry.rs | modify | Replace `use serde_yaml` with `use serde_yml` | AC-001.2 |
| 4 | .config/nextest.toml | create | Default nextest profile | AC-002.2 |
| 5 | deny.toml | create | cargo-deny starter config | AC-003.1 |
| 6 | CLAUDE.md | modify | Add nextest, deny, llvm-cov commands | AC-002.1, AC-003.2, AC-004.1 |
| 7 | CHANGELOG.md | modify | Add tooling entry | doc |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | build | cargo check after serde_yml swap | AC-001.1 | `cargo check` | exit 0 |
| PC-002 | unit | ecc-domain tests pass | AC-001.2 | `cargo test -p ecc-domain` | PASS |
| PC-003 | lint | No serde_yaml in crates/ | AC-001.3 | `! grep -r 'serde_yaml' crates/` | exit 0 |
| PC-004 | lint | CLAUDE.md mentions nextest | AC-002.1 | `grep 'nextest' CLAUDE.md` | exit 0 |
| PC-005 | lint | nextest.toml exists | AC-002.2 | `test -f .config/nextest.toml` | exit 0 |
| PC-006 | lint | deny.toml exists | AC-003.1 | `test -f deny.toml` | exit 0 |
| PC-007 | lint | CLAUDE.md mentions deny | AC-003.2 | `grep 'deny check' CLAUDE.md` | exit 0 |
| PC-008 | lint | CLAUDE.md mentions llvm-cov | AC-004.1 | `grep 'llvm-cov' CLAUDE.md` | exit 0 |
| PC-009 | lint | Zero clippy warnings | all | `cargo clippy -- -D warnings` | exit 0 |
| PC-010 | build | Release build | all | `cargo build --release` | exit 0 |
| PC-011 | build | Full test suite | all | `cargo test` | PASS |

### Coverage Check
All 9 ACs covered. AC-001.1→PC-001, AC-001.2→PC-002, AC-001.3→PC-003, AC-002.1→PC-004, AC-002.2→PC-005, AC-003.1→PC-006, AC-003.2→PC-007, AC-004.1→PC-008.

### E2E Test Plan
None — config and tooling changes only.

## Test Strategy
PC-001..003 (serde_yml) → PC-004..005 (nextest) → PC-006..007 (deny) → PC-008 (llvm-cov) → PC-009..011 (gates)

## Doc Update Plan
| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CLAUDE.md | Minor | Update | Add nextest, deny, llvm-cov to test commands | US-002..004 |
| 2 | CHANGELOG.md | Minor | Add entry | Tooling upgrades: serde_yml, nextest, deny, llvm-cov | all |

## SOLID Assessment
PASS — config file changes only, no architectural impact.

## Robert's Oath Check
CLEAN — replacing deprecated dependency (Oath 5: fearless improvement).

## Security Notes
CLEAR — serde_yml is the maintained fork, cargo-deny improves security posture.

## Rollback Plan
1. Revert CHANGELOG.md
2. Revert CLAUDE.md
3. Delete deny.toml
4. Delete .config/nextest.toml
5. Revert backlog/entry.rs (serde_yml → serde_yaml)
6. Revert crates/ecc-domain/Cargo.toml
7. Revert Cargo.toml workspace dep
