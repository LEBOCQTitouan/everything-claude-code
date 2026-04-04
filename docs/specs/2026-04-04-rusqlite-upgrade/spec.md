# Spec: BL-113 Upgrade rusqlite 0.34 to 0.38

## Problem Statement

rusqlite 0.34 is 4 minor versions behind current (0.38). The upgrade brings SQLite 3.51.3 support, accumulated bug fixes, and keeps the dependency current. Research confirms all 4 known breaking changes between 0.34-0.38 do NOT affect this codebase: no u64/usize ToSql usage, no multi-statement execute calls, no prepare_cached, no hook registration.

## Research Summary

- **u64/usize ToSql/FromSql disabled by default (0.38)** -- NOT affected; no u64/usize passed to rusqlite params
- **execute() rejects multi-statement SQL (0.35)** -- NOT affected; all execute calls use single statements
- **Statement cache optional (0.38)** -- NOT affected; no prepare_cached usage
- **Hook ownership check (0.38)** -- NOT affected; no commit_hook/update_hook/rollback_hook used
- **Minimum SQLite bumped to 3.34.1** -- NOT affected; we use `bundled` feature
- **0.36 and 0.37 had no breaking changes** -- additive features only

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Bump rusqlite 0.34 to 0.38 | Stay current, receive bug fixes, SQLite 3.51.3 | No |

## User Stories

### US-001: Bump rusqlite Version

**As a** maintainer, **I want** rusqlite updated to 0.38, **so that** the project receives latest SQLite support and bug fixes.

#### Acceptance Criteria

- AC-001.1: Given Cargo.toml workspace dependencies, when inspected, then rusqlite version is "0.38"
- AC-001.2: Given the bumped dependency, when `cargo build` runs, then it compiles without errors
- AC-001.3: Given the bumped dependency, when `cargo test` runs, then all tests pass
- AC-001.4: Given the bumped dependency, when `cargo clippy -- -D warnings` runs, then no warnings
- AC-001.5: Given the 3 SQLite files (sqlite_log_store.rs, sqlite_memory.rs, log_schema.rs), when compiled with rusqlite 0.38, then no API changes needed

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `Cargo.toml` | Config | Version bump 0.34 to 0.38 |
| `Cargo.lock` | Config | Auto-updated |
| `crates/ecc-infra/src/` | Infra | Compile verification only (no changes expected) |

## Constraints

- Must compile on both macOS and Linux
- All existing tests must pass
- Features unchanged: `["bundled", "modern_sqlite"]`

## Non-Requirements

- No API migration (breaking changes don't apply)
- No new rusqlite features adoption
- No ADR

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| SQLite adapters | Dependency bump | Compile-time verification only |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Dependency bump | LOW | CHANGELOG.md | Add chore entry |

## Open Questions

None.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1-8 | All 8 mandatory questions | Accepted all recommendations (clean bump, no breaking changes apply) | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Bump rusqlite Version | 5 | none |

### Adversary Findings

Adversarial review: PASS (user override -- trivial dependency bump with confirmed zero breaking changes).

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-04-rusqlite-upgrade/spec.md` | Full spec + Phase Summary |
