# Spec: BL-155 Foundation Concern Variant

Source: BL-155 | Scope: MEDIUM | Target: /spec-dev | Status: **PASS adversary round 2** (86/100)

## Problem Statement

`/project-foundation` (BL-143) was designed to initialize workflow state with `concern=foundation`, but `Concern::from_str("foundation")` returns `UnknownConcern`. The command applies an `init dev` workaround with explicit comment — foundation sessions are indistinguishable from dev sessions in `state.json`, muddling audit trails. This ticket finishes BL-143 by adding the missing `Concern::Foundation` domain variant.

## Research Summary

- `Concern` is an exhaustive public enum (no `#[non_exhaustive]`); adding a variant is semver-breaking per `cargo-semver-checks` + Rust RFC 1105. ECC CI runs `cargo semver-checks`.
- serde `rename_all = "lowercase"` applies uniformly to new variants — no per-variant annotation.
- No exhaustive `match` on `Concern` exists outside `concern.rs`; only `state.rs` tests enumerate variants.
- `clap` `ValueEnum` could layer on later for `--help` auto-enumeration; current `FromStr`-based parsing suffices.
- BL-143 `/project-foundation` workaround is the canonical reference for intended semantics.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | MINOR version bump (4.2.0 → 4.3.0) with semver-checks waiver via `--baseline-rev` override (preferred over `[package.metadata.semver-checks]` for transience) | `ecc-domain` has no external consumers; rationale aligns with minor, not major | No |
| 2 | Update 5 hardcoded-concern doc sites + test UnknownConcern text | Prevents doc-code drift | No |
| 3 | Same FSM as dev/fix/refactor (no Foundation-specific transitions) | BL-155 §Out-of-scope excludes phase-transition changes | No |
| 4 | Alphabetical order in `UnknownConcern`: `"dev, fix, foundation, or refactor"` | Deterministic, reviewer-friendly | No |
| 5 | Ship US-001/US-002/US-003 in a single PR | Prevents temporary drift between doc promising `init foundation` and enum still rejecting it | No |

## User Stories

### US-001: Add `Foundation` variant to domain enum

**As a** `/project-foundation` command, **I want** `Concern::Foundation` to exist in the domain, **so that** `ecc-workflow init foundation` succeeds.

- AC-001.1: `Concern::from_str("foundation")` → `Ok(Concern::Foundation)`.
- AC-001.2: `Concern::Foundation.to_string()` (via `fmt::Display`) → `"foundation"`.
- AC-001.3: serde serialize → `"foundation"` and round-trips via deserialize.
- AC-001.4: `UnknownConcern(s).to_string()` for unknown `s` produces exactly `"unknown concern: {s} (expected dev, fix, foundation, or refactor)"`.
- AC-001.5: All existing tests in `concern.rs` pass unchanged; no existing test is modified or deleted.
- AC-001.6: `Concern::from_str("Foundation")` → `Err(UnknownConcern("Foundation"))` — documents case-sensitive parsing; case-handling unchanged from existing Dev/Fix/Refactor behavior.
- AC-001.7: New unit test asserts `UnknownConcern("xyz".to_owned()).to_string()` equals exactly `"unknown concern: xyz (expected dev, fix, foundation, or refactor)"`.

Depends on: none.

### US-002: CLI accepts `foundation` + integration test

**As a** `/project-foundation` author, **I want** `ecc-workflow init foundation --feature-stdin` to succeed, **so that** the workaround can be reverted.

- AC-002.1: `ecc-workflow init foundation "demo" --feature-stdin` → exit 0 + state.json contains `"concern": "foundation"`.
- AC-002.2: Same command via `ecc workflow init` delegator → identical result.
- AC-002.3: `ecc-workflow worktree-name foundation "demo project"` → valid worktree name string.
- AC-002.4: Integration test under `crates/ecc-workflow/tests/` uses `tempfile::TempDir` for isolated state directory; walks `init foundation → transition plan → transition solution → transition implement → transition done`; each step exits 0; final state.json `concern="foundation"` preserved across all transitions.

Depends on: US-001.

### US-003: Revert `/project-foundation` workaround + doc updates

**As a** ECC user, **I want** doc sites to accurately list all 4 concerns, **so that** stale doc content doesn't drift from the domain enum.

- AC-003.1: `commands/project-foundation.md:18` instructs `ecc-workflow init foundation --feature-stdin`; zero workaround text, zero backlog-reference text.
- AC-003.2: `commands/catchup.md:19` concern list reads `dev, fix, foundation, refactor`.
- AC-003.4: `CHANGELOG.md` contains BL-155 entry under `## Unreleased`.
- AC-003.5: `skills/campaign-manifest/SKILL.md:18` concern enumeration includes `foundation` (e.g., `dev|fix|foundation|refactor`).
- AC-003.6: `skills/artifact-schemas/SKILL.md:69` concern value examples include `foundation`.

Depends on: US-001, US-002.

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `crates/ecc-domain/src/workflow/concern.rs` | domain | add `Foundation` variant + Display + FromStr + UnknownConcern text + 3 new tests |
| `crates/ecc-domain/src/workflow/state.rs` | domain | extend `round_trips_all_variants` array |
| `crates/ecc-workflow/tests/init_foundation.rs` (new) | bin | TempDir-isolated integration test walking the full FSM |
| `commands/project-foundation.md` | docs | revert workaround at line 18 |
| `commands/catchup.md` | docs | update concern list at line 19 |
| `skills/campaign-manifest/SKILL.md` | docs | include `foundation` in concern enumeration at line 18 |
| `skills/artifact-schemas/SKILL.md` | docs | include `foundation` in concern value example at line 69 |
| `CHANGELOG.md` | docs | add BL-155 entry under `## Unreleased` |
| workspace `Cargo.toml` | meta | version bump `4.2.0 → 4.3.0` |

## Constraints

- `cargo semver-checks` in CI will flag variant addition; use `--baseline-rev` override for this commit.
- `ecc validate commands` must pass post-edit.
- Existing state.json files with `concern: dev|fix|refactor` must deserialize unchanged.
- All three stories ship in a single commit/PR (Decision 5) to avoid doc-code drift mid-merge.

## Non-Requirements

- Foundation-specific phase transitions (plan→implement shortcut, etc.).
- Backfilling existing `concern=dev` state.json entries — no live foundation state exists.
- Spec-directory slug convention overhaul (`foundation-` prefix).
- Migrating `FromStr` to clap `ValueEnum` derive.
- Retroactively adding `#[non_exhaustive]` to `Concern`.
- Downgrade-path compatibility (absent in ECC generally).

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| `ecc-workflow` CLI | string-accept widening | Integration test (US-002.4) using `tempfile::TempDir` |
| `/project-foundation` command template | content change | `ecc validate commands` smoke-test |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CHANGELOG | project | CHANGELOG.md | BL-155 entry under `## Unreleased` |
| Command | project | commands/project-foundation.md | Revert BL-143 workaround (line 18) |
| Command | project | commands/catchup.md | Update concern list (line 19) |
| Skill | project | skills/campaign-manifest/SKILL.md | Update concern enumeration (line 18) |
| Skill | project | skills/artifact-schemas/SKILL.md | Update concern value example (line 69) |

## Open Questions

None.
