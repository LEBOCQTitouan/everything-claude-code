# Spec: Migrate serde_yml to serde-saphyr (BL-099)

## Problem Statement

`serde_yml 0.0.12` has a RUSTSEC-2025-0068 security advisory (segfault in `Serializer.emitter`), is archived, and was flagged by the community for AI-generated unsound code. It's used in `ecc-domain` — the most stable, most-depended-upon crate — making it a supply chain risk that propagates to all downstream crates ([CORR-006] from the full audit 2026-03-29).

## Research Summary

- **RUSTSEC-2025-0068**: serde_yml has a documented segfault in `Serializer.emitter`; crate is archived and unmaintained
- **serde-saphyr**: Pure Rust YAML parser, panic-free on malformed input, no `unsafe-libyaml` dependency, faster than the serde-yaml fork family in benchmarks
- **serde-yaml-ng**: Near drop-in API replacement for the original serde-yaml, but still uses `unsafe-libyaml` under the hood (same class of unsafety concerns)
- **Our usage is minimal**: Only `serde_saphyr::from_str()` for YAML frontmatter parsing in backlog entries — no `Value`, `Mapping`, `to_value`, or serialization
- **Community recommendation**: serde-saphyr for security-sensitive work; serde-yaml-ng for minimal-churn migrations
- **Migration scope**: 1 production call site + 4 test call sites in `crates/ecc-domain/src/backlog/entry.rs`

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Use serde-saphyr (not serde-yaml-ng) | Pure Rust, no unsafe, panic-free, faster. Our usage is just `from_str()` — API differences don't affect us | Yes |
| 2 | Single atomic change | 1 file, 5 call sites — too small to split into multiple steps | No |
| 3 | Include `cargo deny check` verification | Verify supply chain health after swap; confirm no new advisories introduced | No |
| 4 | Verify error message format doesn't break tests | serde-saphyr may produce different error strings; must check no test asserts on error text | No |

## User Stories

### US-001: Replace serde_yml with serde-saphyr

**As a** maintainer, **I want** to replace `serde_yml` with `serde-saphyr` across the workspace, **so that** the domain layer is free from the RUSTSEC-2025-0068 advisory and uses a pure-Rust, panic-free YAML parser.

#### Acceptance Criteria

- AC-001.1: Given `Cargo.toml` workspace deps, when checked, then `serde_yml` is absent and `serde-saphyr` is present
- AC-001.2: Given `crates/ecc-domain/Cargo.toml`, when checked, then it references `serde-saphyr` (not `serde_yml`)
- AC-001.3: Given `crates/ecc-domain/src/backlog/entry.rs`, when checked, then all `serde_yml::` references are replaced with `serde_saphyr::`
- AC-001.4: Given all existing YAML parsing tests, when run, then they pass with the new crate
- AC-001.5: Given `cargo deny check`, when run, then no advisories are reported for the YAML dependency
- AC-001.6: Given error messages from YAML parsing failures, when compared, then no test asserts on specific error text (or assertions are updated if needed)
- AC-001.7: Given `cargo clippy --workspace -- -D warnings`, when run, then zero warnings
- AC-001.8: Given `cargo test --workspace`, when run, then all tests pass

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `Cargo.toml` (workspace) | Config | Replace `serde_yml = "0.0.12"` with `serde-saphyr` in `[workspace.dependencies]` |
| `crates/ecc-domain/Cargo.toml` | Config | Update `serde_yml = { workspace = true }` to `serde-saphyr = { workspace = true }` |
| `crates/ecc-domain/src/backlog/entry.rs` | Domain | Replace 5 `serde_yml::from_str` call sites with `serde_saphyr::from_str` |

## Constraints

- All existing tests must pass after the swap — behavior-preserving refactor
- `cargo deny check` must pass (no new advisories introduced)
- `ecc-domain` must remain pure (no I/O imports) — serde-saphyr is a pure Rust crate, so this is satisfied
- Error format changes are acceptable if no tests assert on specific error text

## Non-Requirements

- Restructuring how YAML parsing is used (e.g., abstracting behind a port trait)
- Adding YAML serialization support (we only use `from_str` deserialization)
- Migrating other serde-related dependencies
- Supporting multiple YAML backends

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | N/A | Pure domain-layer dependency swap, no port/adapter/E2E impact |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Dependency change | ADR | `docs/adr/0034-serde-saphyr-migration.md` | Create ADR documenting crate choice and RUSTSEC resolution |
| Backlog status | Metadata | `docs/backlog/BL-099-*.md` | Mark `status: implemented` |
| CHANGELOG | Project | `CHANGELOG.md` | Add entry for serde-saphyr migration |

## Rollback Plan

If serde-saphyr causes issues after merge:
1. Revert `Cargo.toml` workspace deps: restore `serde_yml = "0.0.12"`
2. Revert `crates/ecc-domain/Cargo.toml`: restore `serde_yml = { workspace = true }`
3. Revert `crates/ecc-domain/src/backlog/entry.rs`: restore `serde_yml::from_str` calls
4. Run `cargo update` to regenerate `Cargo.lock`
5. Verify `cargo test --workspace` passes

## Version Pin

Use the latest stable serde-saphyr release at implementation time. Pin to exact minor version (e.g., `serde-saphyr = "0.2"`) to avoid surprise breaking changes. Check `crates.io/crates/serde-saphyr` for current version.

## Open Questions

None — all resolved during grill-me interview.
