# Solution: Fix cargo xtask deploy — ecc-flock not a binary

## Spec Reference
Concern: fix, Feature: cargo xtask deploy does not work

## File Changes

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | xtask/src/deploy.rs | Modify | Remove ecc-flock from packages_to_build + binaries_to_install, add tests | US-001 |
| 2 | CHANGELOG.md | Modify | Add fix entry | Convention |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | packages_to_build has no ecc-flock | AC-001.3 | `cargo test -p xtask deploy_packages_no_flock` | PASS |
| PC-002 | unit | binaries_to_install has no ecc-flock | AC-001.4 | `cargo test -p xtask deploy_binaries_no_flock` | PASS |
| PC-003 | integration | dry-run has no ecc-flock | AC-001.1 | `cargo run -p xtask -- deploy --dry-run 2>&1` | no ecc-flock |
| PC-004 | build | workspace builds | ALL | `cargo build --workspace` | exit 0 |
| PC-005 | lint | clippy clean | ALL | `cargo clippy --workspace -- -D warnings` | exit 0 |

## Rollback Plan

1. Revert xtask/src/deploy.rs (re-add ecc-flock to lists)
2. Revert CHANGELOG.md
