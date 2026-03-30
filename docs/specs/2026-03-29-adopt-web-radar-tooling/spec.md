# Spec: Adopt Web Radar Findings — 4 Tooling Upgrades

## Problem Statement

The web radar audit (2026-03-29) identified 4 "Adopt" findings: serde_yaml is deprecated and archived since March 2024 (security risk), the project lacks supply chain auditing, coverage measurement doesn't work on macOS (the dev platform), and the test runner is slower than necessary for 1400+ tests.

## Research Summary

- **serde_yaml → serde_yml**: API-compatible fork maintained by sebastienrousseau. Replace crate name in Cargo.toml and `use` statements. Only 2 source files affected.
- **cargo-nextest**: Drop-in test runner replacement. `cargo nextest run` instead of `cargo test`. ~60% faster with per-test process isolation and flaky test detection.
- **cargo-deny**: Config-driven supply chain auditor (`deny.toml`). Checks licenses, advisories, sources. `cargo deny init` generates starter config.
- **cargo-llvm-cov**: LLVM-instrumented coverage. Works on macOS (unlike tarpaulin). `cargo llvm-cov --workspace` for reports.

## Decisions Made

| # | Decision | Rationale | ADR? |
|---|----------|-----------|------|
| 1 | Replace serde_yaml with serde_yml | Deprecated dependency, API-compatible fork | No |
| 2 | Add tools as documented dev workflow, not CI gates | Start with local usage, gate in CI later | No |

## User Stories

### US-001: Replace serde_yaml with serde_yml

**As a** developer, **I want** the deprecated serde_yaml replaced with serde_yml, **so that** I'm not on an archived dependency with no security updates.

#### Acceptance Criteria

- AC-001.1: Given Cargo.toml workspace deps, when serde_yaml is replaced with serde_yml, then `cargo check` passes
- AC-001.2: Given backlog/entry.rs, when `use serde_yaml` is replaced with `use serde_yml`, then `cargo test -p ecc-domain` passes
- AC-001.3: Given `grep -r 'serde_yaml' crates/`, when run, then zero matches

#### Dependencies
- None

### US-002: Add cargo-nextest

**As a** developer, **I want** cargo-nextest documented as the recommended test runner, **so that** tests run faster with better isolation.

#### Acceptance Criteria

- AC-002.1: Given CLAUDE.md, when updated, then it documents `cargo nextest run` as alternative to `cargo test`
- AC-002.2: Given `.config/nextest.toml`, when created, then it configures default profile with retry and timeout

#### Dependencies
- None

### US-003: Add cargo-deny

**As a** developer, **I want** cargo-deny configured for supply chain auditing, **so that** license and advisory issues are caught.

#### Acceptance Criteria

- AC-003.1: Given `deny.toml`, when created at project root, then `cargo deny check` passes or reports only known acceptable issues
- AC-003.2: Given CLAUDE.md, when updated, then it documents `cargo deny check`

#### Dependencies
- None

### US-004: Add cargo-llvm-cov

**As a** developer, **I want** cargo-llvm-cov documented for coverage measurement, **so that** I can measure coverage on macOS.

#### Acceptance Criteria

- AC-004.1: Given CLAUDE.md, when updated, then it documents `cargo llvm-cov --workspace`

#### Dependencies
- None

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| Cargo.toml (workspace) | Config | serde_yaml → serde_yml |
| crates/ecc-domain/Cargo.toml | Config | serde_yaml → serde_yml |
| crates/ecc-domain/src/backlog/entry.rs | Domain | use serde_yml |
| deny.toml | Config | New file |
| .config/nextest.toml | Config | New file |
| CLAUDE.md | Docs | Add tool commands |

## Constraints

- serde_yml must be API-compatible (no code logic changes beyond use rename)
- All 1400+ tests must pass after serde_yml swap
- Tool configs are starter/default — not blocking CI

## Non-Requirements

- CI pipeline integration (future work)
- Coverage enforcement thresholds (future work)
- cargo-mutants (Trial ring, not Adopt)

## E2E Boundaries Affected

None — config and tooling changes only.

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|---|---|---|---|
| Update | Minor | CLAUDE.md | Add nextest, deny, llvm-cov commands |
| Add entry | Minor | CHANGELOG.md | Add tooling upgrade entry |

## Open Questions

None.
