# Solution: Add docs/cartography/ to phase-gate allowlist

## Spec Reference
Concern: fix, Feature: Add docs/cartography/ to phase-gate allowlist

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-workflow/src/commands/phase_gate.rs` | modify | Add 4 dir prefixes to allowlist + 4 unit tests | AC-001.1-001.6 |
| 2 | `CHANGELOG.md` | modify | Add fix entry | US-001 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | cartography dir allowed | AC-001.1 | `cargo test -p ecc-workflow phase_gate_allows_cartography_dir` | PASS |
| PC-002 | unit | domain dir allowed | AC-001.2 | `cargo test -p ecc-workflow phase_gate_allows_domain_dir` | PASS |
| PC-003 | unit | guides dir allowed | AC-001.3 | `cargo test -p ecc-workflow phase_gate_allows_guides_dir` | PASS |
| PC-004 | unit | diagrams dir allowed | AC-001.4 | `cargo test -p ecc-workflow phase_gate_allows_diagrams_dir` | PASS |
| PC-005 | unit | all phase_gate tests pass | AC-001.5 | `cargo test -p ecc-workflow -- phase_gate` | PASS |
| PC-006 | unit | traversal still blocked | AC-001.6 | `cargo test -p ecc-workflow phase_gate_blocks_encoded_traversal` | PASS |
| PC-007 | lint | clippy passes | regression | `cargo clippy -p ecc-workflow -- -D warnings` | exit 0 |
| PC-008 | build | build passes | regression | `cargo build -p ecc-workflow` | exit 0 |

### Coverage Check
All ACs covered:
- AC-001.1 → PC-001
- AC-001.2 → PC-002
- AC-001.3 → PC-003
- AC-001.4 → PC-004
- AC-001.5 → PC-005
- AC-001.6 → PC-006

### E2E Test Plan
No E2E boundaries affected.

### E2E Activation Rules
None.

## Test Strategy
TDD order: PC-001..004 (write 4 tests RED, add 4 prefixes GREEN) → PC-005..006 (regression) → PC-007..008 (lint, build)

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CHANGELOG.md | Minor | Add fix entry | "fix: add docs/cartography/ and 3 other dirs to phase-gate allowlist (BL-142)" | US-001 |

## SOLID Assessment
N/A — adding strings to an array in a private function.

## Robert's Oath Check
CLEAN — trivial proof-of-fix, small release, no mess.

## Security Notes
CLEAR — no new input boundaries, traversal guards unaffected.

## Rollback Plan
1. Remove 4 prefixes from `allowed_prefixes()` in phase_gate.rs
2. Remove 4 tests
3. Revert CHANGELOG entry

## Bounded Contexts Affected
No bounded contexts affected — ecc-workflow is infra/CLI, not domain.
