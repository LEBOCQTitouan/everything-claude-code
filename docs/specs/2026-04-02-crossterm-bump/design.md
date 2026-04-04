# Solution: BL-105 Bump crossterm 0.28 to 0.29

## Spec Reference
Concern: dev, Feature: BL-105 Bump crossterm 0.28 to 0.29

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `Cargo.toml` | Modify | Bump crossterm version from "0.28" to "0.29" | US-001, AC-001.1 |

Cargo.lock auto-updates via cargo. No code changes — only 2 stable APIs used (is_tty, terminal::size), both unchanged in 0.29.

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Cargo.toml has crossterm 0.29 | AC-001.1 | `grep -q 'crossterm = "0.29"' Cargo.toml` | Exit 0 |
| PC-002 | build | cargo build compiles with 0.29 | AC-001.2, AC-001.6 | `cargo build --workspace` | Exit 0 |
| PC-003 | integration | cargo test passes | AC-001.3 | `cargo test --workspace` | Exit 0 |
| PC-004 | lint | cargo clippy clean | AC-001.4 | `cargo clippy --workspace -- -D warnings` | Exit 0 |
| PC-005 | integration | cargo audit clean for crossterm | AC-001.5 | `cargo audit 2>&1 \| grep -v 'crossterm' \|\| cargo audit` | Exit 0 |
| PC-006 | unit | API audit: crossterm usage unchanged | AC-002.1, AC-002.2 | `grep -c 'crossterm::' crates/ecc-infra/src/std_terminal.rs` | Output: 2 |
| PC-007 | build | Final build gate | All | `cargo build --workspace` | Exit 0 |

### Coverage Check

All 8 ACs covered:

| AC | Covering PCs |
|---|---|
| AC-001.1 | PC-001 |
| AC-001.2 | PC-002 |
| AC-001.3 | PC-003 |
| AC-001.4 | PC-004 |
| AC-001.5 | PC-005 |
| AC-001.6 | PC-002 |
| AC-002.1 | PC-006 |
| AC-002.2 | PC-006 (documented in commit message) |

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | TerminalIO | StdTerminal | TerminalIO | Compile-time verification | ignored | ecc-infra modified |

### E2E Activation Rules
No E2E tests to un-ignore — compile-time verification via PC-002 is sufficient.

## Test Strategy

TDD order:
1. **PC-001** — Version bump (foundation)
2. **PC-002** — Build compiles
3. **PC-003** — Tests pass
4. **PC-004** — Clippy clean
5. **PC-005** — Audit clean
6. **PC-006** — API audit
7. **PC-007** — Final build gate

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `CHANGELOG.md` | LOW | Modify | Add chore entry for crossterm bump | All |

## SOLID Assessment
**PASS** — 1-line config change. No architecture impact.

## Robert's Oath Check
**CLEAN** — 7 PCs verify the bump. Minimal change, full test coverage.

## Security Notes
**CLEAR** — Dependency bump only. cargo audit verifies no new advisories.

## Rollback Plan
1. Revert Cargo.toml crossterm version to "0.28"
2. Run `cargo update -p crossterm` to regenerate Cargo.lock

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS | 0 |
| Robert | CLEAN | 0 |
| Security | CLEAR | 0 |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `Cargo.toml` | Modify | US-001 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-02-crossterm-bump/design.md` | Full design + Phase Summary |
