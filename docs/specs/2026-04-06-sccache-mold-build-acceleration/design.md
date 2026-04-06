# Solution: sccache + mold Build Acceleration

## Spec Reference
Concern: dev, Feature: sccache-mold-build-acceleration

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `.cargo/config.toml` | modify | Add mold linker for Linux targets (target-specific, macOS unaffected) | AC-001.3, AC-001.5 |
| 2 | `docs/getting-started.md` | modify | Add Build Acceleration section with sccache, mold, Cranelift docs | AC-001.1, AC-001.2, AC-001.4 |
| 3 | `CLAUDE.md` | modify | Add sccache note to Running Tests section | AC-001.6 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | lint | .cargo/config.toml has mold for linux-gnu targets | AC-001.3 | `grep -c 'x86_64-unknown-linux-gnu' .cargo/config.toml` | >= 1 |
| PC-002 | lint | getting-started.md has Build Acceleration section | AC-001.1 | `grep -c '## Build Acceleration' docs/getting-started.md` | >= 1 |
| PC-003 | lint | getting-started.md has RUSTC_WRAPPER instructions | AC-001.2 | `grep -c 'RUSTC_WRAPPER' docs/getting-started.md` | >= 1 |
| PC-004 | lint | getting-started.md has Cranelift documented | AC-001.4 | `grep -ci 'cranelift' docs/getting-started.md` | >= 1 |
| PC-005 | build | cargo build passes with mold config | AC-001.5 | `cargo build 2>&1 | tail -1` | Finished |
| PC-006 | lint | CLAUDE.md has sccache reference | AC-001.6 | `grep -c 'sccache' CLAUDE.md` | >= 1 |
| PC-007 | lint | getting-started.md has speedup percentages | AC-001.2 | `grep -cE '11.14%|30%' docs/getting-started.md` | >= 1 |

### Coverage Check

| AC | Covering PCs |
|----|-------------|
| AC-001.1 | PC-002 |
| AC-001.2 | PC-003, PC-007 |
| AC-001.3 | PC-001 |
| AC-001.4 | PC-004 |
| AC-001.5 | PC-005 |
| AC-001.6 | PC-006 |

All 6 ACs covered.

### E2E Test Plan
None -- config and documentation changes only.

### E2E Activation Rules
None.

## Test Strategy

TDD order:
1. PC-001: Add mold config to .cargo/config.toml
2. PC-005: Verify cargo build passes
3. PC-002..004, PC-007: Add Build Acceleration docs to getting-started.md
4. PC-006: Add sccache note to CLAUDE.md

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CHANGELOG.md | project | modify | Add BL-100 entry | all |

## SOLID Assessment
N/A -- config and documentation changes only.

## Robert's Oath Check
CLEAN -- improves developer experience.

## Security Notes
CLEAR -- local dev tooling, no attack surface.

## Rollback Plan
1. Revert CLAUDE.md
2. Revert docs/getting-started.md
3. Revert .cargo/config.toml

## Bounded Contexts Affected
No bounded contexts affected -- no domain files modified.
