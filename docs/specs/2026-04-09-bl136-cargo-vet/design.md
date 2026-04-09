# Design: Add cargo-vet for SLSA Level 2 (BL-136)

## Spec Reference
\`docs/specs/2026-04-09-bl136-cargo-vet/spec.md\`

## File Changes

| # | File | Change | Layer | Spec Ref |
|---|------|--------|-------|----------|
| 1 | \`supply-chain/config.toml\` | create via cargo vet init, add imports + policy | Root | AC-001.1, AC-002.1/2, AC-003.2 |
| 2 | \`supply-chain/audits.toml\` | create via cargo vet init | Root | AC-001.2, AC-003.3 |
| 3 | \`supply-chain/imports.lock\` | create via cargo vet fetch-imports | Root | AC-001.3, AC-002.3 |
| 4 | \`.github/workflows/ci.yml\` | add cargo vet --locked step | CI | AC-004.1/2/3 |
| 5 | \`supply-chain/README.md\` | create audit policy doc | Docs | AC-005.1 |
| 6 | \`CLAUDE.md\` | add cargo vet to Running Tests | Docs | AC-005.2 |
| 7 | \`CHANGELOG.md\` | add BL-136 entry | Docs | AC-005.3 |

## Pass Conditions

| PC | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | verify | config.toml exists | AC-001.1 | \`test -f supply-chain/config.toml\` | exit 0 |
| PC-002 | verify | audits.toml exists | AC-001.2 | \`test -f supply-chain/audits.toml\` | exit 0 |
| PC-003 | verify | imports.lock committed | AC-001.3 | \`test -f supply-chain/imports.lock\` | exit 0 |
| PC-004 | verify | cargo vet check passes | AC-001.4, AC-003.1 | \`cargo vet check\` | exit 0 |
| PC-005 | verify | Mozilla import | AC-002.1 | \`grep -q 'mozilla' supply-chain/config.toml\` | exit 0 |
| PC-006 | verify | Google import | AC-002.2 | \`grep -q 'google' supply-chain/config.toml\` | exit 0 |
| PC-007 | verify | Dev deps safe-to-run | AC-003.2 | \`grep -q 'safe-to-run' supply-chain/config.toml\` | exit 0 |
| PC-008 | verify | CI step exists | AC-004.1 | \`grep -q 'cargo vet' .github/workflows/ci.yml\` | exit 0 |
| PC-009 | verify | README exists | AC-005.1 | \`test -f supply-chain/README.md\` | exit 0 |
| PC-010 | verify | CLAUDE.md cargo vet | AC-005.2 | \`grep -q 'cargo vet' CLAUDE.md\` | exit 0 |
| PC-011 | verify | CHANGELOG BL-136 | AC-005.3 | \`grep -q 'BL-136' CHANGELOG.md\` | exit 0 |

## Coverage Check

All 16 ACs covered by PC-001 through PC-011.

## Test Strategy

Sequential: US-001 (init) → US-002 (imports) → US-003 (certify) → US-004 (CI) → US-005 (docs).

## E2E Test Plan

No E2E tests — config/CI only.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CHANGELOG.md | root | Add entry | cargo-vet integration | AC-005.3 |
| 2 | CLAUDE.md | root | Add line | cargo vet in Running Tests | AC-005.2 |

## SOLID Assessment

**PASS.** Config/CI only, no code.

## Robert's Oath Check

**CLEAN.** Supply chain hardening.

## Security Notes

**CLEAR.** Adds security tooling, doesn't introduce attack surface.

## Rollback Plan

1. Remove supply-chain/ directory
2. Revert ci.yml cargo vet step
3. Revert CLAUDE.md and CHANGELOG lines

## Bounded Contexts Affected

No bounded contexts affected — config/CI/docs only.
