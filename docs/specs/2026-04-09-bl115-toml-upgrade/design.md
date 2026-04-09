# Design: Upgrade toml 0.8 to 0.9 (BL-115)

## Spec Reference
`docs/specs/2026-04-09-bl115-toml-upgrade/spec.md`

## File Changes

| # | File | Change | Layer | Spec Ref |
|---|------|--------|-------|----------|
| 1 | `crates/ecc-infra/Cargo.toml` | modify: `toml = "0.8"` → `toml = "0.9"` | Infra (adapter) | AC-001.1 |
| 2 | `Cargo.lock` | auto-regenerated | Root | — |

## Pass Conditions

| PC | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | verify | Version bumped to 0.9 | AC-001.1 | `grep -E '^toml = "0.9"' crates/ecc-infra/Cargo.toml` | exit 0 |
| PC-002 | build | Workspace compiles | AC-001.2 | `cargo build -p ecc-infra` | exit 0 |
| PC-003 | lint | Clippy clean | AC-001.3 | `cargo clippy -p ecc-infra -- -D warnings` | exit 0 |
| PC-004 | unit | All file_config_store tests pass | AC-001.4 | `cargo test -p ecc-infra -- file_config_store` | PASS |
| PC-005 | verify | No source code changes | AC-001.5 | `test -z "$(git diff HEAD --name-only -- '*.rs')"` | exit 0 |

## Coverage Check

| AC | Covered by PC |
|----|---------------|
| AC-001.1 | PC-001 |
| AC-001.2 | PC-002 |
| AC-001.3 | PC-003 |
| AC-001.4 | PC-004 |
| AC-001.5 | PC-005 |

**5/5 ACs covered.**

## Implementation Strategy

Single wave — all PCs target the same file change:
1. Edit `crates/ecc-infra/Cargo.toml` line 24
2. Run PC-001 through PC-005 sequentially

## E2E Test Plan

No E2E tests needed — toml types never cross the port boundary.

## E2E Activation Rules

No E2E tests activated.

## Test Strategy

TDD order: PC-001 → PC-002 → PC-004 → PC-003 → PC-005. Build first to resolve Cargo.lock, then test, then lint, then verify no source changes.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CHANGELOG.md | root | Add entry | "Upgraded toml 0.8 → 0.9 in ecc-infra (BL-115)" | US-001 |
| 2 | MODULE-SUMMARIES.md | root | No action | Dependency version bump does not alter module interface | — |

## SOLID Assessment

**PASS.** toml confined to adapter layer. Port boundary clean. No SOLID violations.

## Robert's Oath Check

**CLEAN.** Single-line bump, independently shippable, 8 tests cover regression surface.

## Security Notes

**CLEAR.** No new attack surface. Local file parsing only. No CVEs for toml 0.8 or 0.9.

## Rollback Plan

Revert `crates/ecc-infra/Cargo.toml` to `toml = "0.8"` and run `cargo generate-lockfile`.

## Bounded Contexts Affected

No bounded contexts affected — `toml` is confined to the infra adapter layer.
