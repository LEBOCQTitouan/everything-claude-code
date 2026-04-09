# Spec: Upgrade toml 0.8 to 0.9 (BL-115)

## Problem Statement

The `toml` crate in `ecc-infra` is at version 0.8. Version 0.9 (released July 2025) unifies the internal AST with `toml_edit`, improves parser alignment with TOML 1.1, and delivers bug fixes. Staying current reduces maintenance risk and keeps the dependency chain modern.

## Research Summary

- toml 0.9 shares AST with toml_edit — unified parser, improved error messages
- Serde bridge API (`from_str`, `to_string`) unchanged between 0.8 and 0.9
- No breaking changes for serde-based usage patterns
- `toml_edit` is not a workspace dependency — no version conflict
- Low-risk bump: only 2 call sites in the entire workspace

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Use `toml = "0.9"` (not exact pin) | Allows patch updates within 0.9.x | No |
| 2 | Do not promote to workspace dep | Only ecc-infra uses it; no second consumer | No |
| 3 | No new tests needed | 7 existing tests cover all call sites | No |

## User Stories

### US-001: Bump toml dependency

**As a** maintainer, **I want** toml at version 0.9, **so that** ecc-infra benefits from upstream bug fixes and parser improvements.

#### Acceptance Criteria

- AC-001.1: `crates/ecc-infra/Cargo.toml` changes `toml = "0.8"` to `toml = "0.9"`
- AC-001.2: `cargo build` compiles without errors
- AC-001.3: `cargo clippy -- -D warnings` exits zero
- AC-001.4: `cargo test` — all tests pass, including all file_config_store tests
- AC-001.5: No source code changes required

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `crates/ecc-infra/Cargo.toml` | Infra (adapter) | Version bump |
| `Cargo.lock` | Root | Auto-regenerated |

## Constraints

- No source code changes — serde API is stable
- Must not affect any other crate's compilation

## Non-Requirements

- Not promoting toml to workspace dependency
- Not adding new tests
- Not upgrading toml_edit (not a dependency)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| ConfigStore (FileConfigStore) | Internal dep bump | None — toml types never cross port boundary |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CHANGELOG | root | CHANGELOG.md | Add dep upgrade entry |

## Open Questions

None.
