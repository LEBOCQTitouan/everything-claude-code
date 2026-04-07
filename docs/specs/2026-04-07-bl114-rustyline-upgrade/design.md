# Design: BL-114 — Upgrade rustyline 15 → 17

**Spec:** `docs/specs/2026-04-07-bl114-rustyline-upgrade/spec.md`
**Date:** 2026-04-07
**Status:** Approved

---

## File Changes

| # | File | Change | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `Cargo.toml:56` | `rustyline = "15"` → `rustyline = "17"` | Version bump | AC-1.1 |
| 2 | `crates/ecc-infra/src/rustyline_input.rs:18` | `Result<Self, String>` → `Result<Self, TerminalError>` | Typed error convention | AC-2.1 |
| 3 | `crates/ecc-infra/src/rustyline_input.rs:20` | `.map_err(\|e\| format!(...))` → `.map_err(\|e\| TerminalError::Io(format!(...)))` | Error wrapping pattern | AC-2.2 |
| 4 | `crates/ecc-cli/src/commands/claw.rs:49` | `.map_err(\|e\| anyhow::anyhow!(e))` → `?` | Typed error enables direct propagation | AC-2.3 |

## Pass Conditions

| PC | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Create editor without history | AC-1.2, AC-2.1, AC-2.4 | `cargo test -p ecc-infra create_without_history -- --exact` | PASS |
| PC-002 | unit | Create editor with temp history | AC-1.2, AC-2.1, AC-2.4 | `cargo test -p ecc-infra create_with_temp_history -- --exact` | PASS |
| PC-003 | build | Full test suite regression | AC-1.3 | `cargo test` | exit 0 |
| PC-004 | lint | Clippy clean | AC-1.4 | `cargo clippy -- -D warnings` | exit 0 |
| PC-005 | build | Cargo check | AC-1.2 | `cargo check` | exit 0 |
| PC-006 | build | Release build | AC-1.2 | `cargo build --release` | exit 0 |
| PC-007 | lint | Format check | AC-1.4 | `cargo fmt --check` | exit 0 |
| PC-008 | grep | No String error type in adapter | AC-2.1, AC-2.2 | `! grep -q 'Result<Self, String>' crates/ecc-infra/src/rustyline_input.rs` | exit 0 |
| PC-009 | grep | TerminalError::Io wrapping present | AC-2.2 | `grep -c 'TerminalError::Io(format!' crates/ecc-infra/src/rustyline_input.rs` | ≥1 |
| PC-010 | grep | No map_err workaround in claw.rs | AC-2.3 | `grep 'RustylineInput::new' crates/ecc-cli/src/commands/claw.rs \| grep -vc map_err` | 1 |
| PC-011 | grep | CHANGELOG entry exists | doc plan | `grep -q 'BL-114' CHANGELOG.md` | exit 0 |

## Coverage Check

8/8 ACs covered. All ACs have ≥1 PC with direct verification.

## E2E Test Plan

None required — no port boundary changes.

## Test Strategy

Existing 2 unit tests cover construction paths. No new tests needed — API is compatible. Full suite regression via PC-003. Grep-based PCs verify structural changes.

## Doc Update Plan

| Doc File | Action | Content | Spec Ref |
|----------|--------|---------|----------|
| CHANGELOG.md | Add entry | "Upgrade rustyline 15 → 17, fix String error type in RustylineInput (BL-114)" | US-1, US-2 |

## SOLID Assessment

No violations. Typed error return strengthens Interface Segregation.

## Robert's Oath Check

CLEAN. Small, focused, tested change.

## Security Notes

CLEAR. Local REPL only, no network or auth surface.

## Rollback Plan

Revert `rustyline = "17"` → `"15"`, revert error type and caller changes. Single atomic revert.

## Bounded Contexts Affected

None. Infrastructure adapter only.

## Adversarial Review

**Round 1**: CONDITIONAL (72/100). Missing PCs for AC-2.2, AC-2.3, structural checks.
**Resolution**: Added PC-006 through PC-011.
