# Design: Add docs/cartography/ to Phase-Gate Allowlist (BL-142)

## Spec Reference
\`docs/specs/2026-04-09-bl142-cartography-allowlist/spec.md\`

## File Changes

| # | File | Change | Layer | Spec Ref |
|---|------|--------|-------|----------|
| 1 | \`crates/ecc-workflow/src/commands/phase_gate.rs\` | Add \`"docs/cartography/"\` to \`allowed_prefixes()\` | Adapter | AC-001.1 |

## Pass Conditions

| PC | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | verify | Allowlist contains cartography | AC-001.1 | \`grep -q 'docs/cartography/' crates/ecc-workflow/src/commands/phase_gate.rs\` | exit 0 |
| PC-002 | unit | Phase-gate allows cartography write during plan | AC-001.2 | \`cargo test -p ecc-workflow --bin ecc-workflow -- phase_gate_allows_cartography\` | PASS |
| PC-003 | unit | All existing phase_gate tests pass | AC-001.3 | \`cargo test -p ecc-workflow --bin ecc-workflow -- phase_gate\` | all PASS |
| PC-004 | lint | Clippy clean | AC-001.4 | \`cargo clippy -p ecc-workflow -- -D warnings\` | exit 0 |
| PC-005 | build | Build passes | AC-001.5 | \`cargo build -p ecc-workflow\` | exit 0 |

## Coverage Check

All 5 ACs covered by PC-001 through PC-005.

## Test Strategy

Single wave: add allowlist entry + test → verify regression → clippy → build.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CHANGELOG.md | root | Add entry | BL-142 cartography allowlist fix | US-001 |

## SOLID Assessment

**PASS.** 1-line addition to existing allowlist.

## Security Notes

**CLEAR.** Adding a documentation path to the allowlist. No new attack surface.

## Rollback Plan

Remove the \`"docs/cartography/"\` line from \`allowed_prefixes()\`.

## Bounded Contexts Affected

No bounded contexts affected — adapter-layer config change only.
