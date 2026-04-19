# Solution: BL-155 Foundation Concern Variant

Source: `docs/specs/2026-04-19-bl155-foundation-concern/spec.md` (PASS adversary round 2 at 86/100)
Design status: **PASS solution-adversary round 2 at 83/100**

## Spec Reference

Concern: `dev` | Feature: BL-155 Add Foundation variant to Concern enum

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/workflow/concern.rs` | modify | Add `Foundation` variant + Display arm + FromStr arm + update `UnknownConcern` text (alphabetical: `"dev, fix, foundation, or refactor"`) + 3 new tests | US-001 AC-001.1..7 |
| 2 | `crates/ecc-domain/src/workflow/state.rs` | modify | Extend `round_trips_all_variants` array to include `Concern::Foundation` | US-001 AC-001.5 |
| 3 | `crates/ecc-workflow/tests/init_foundation.rs` | create | `tempfile::TempDir` integration test (4 test fns) using `env!("CARGO_BIN_EXE_ecc-workflow")` walking init→plan→solution→implement→done | US-002 AC-002.1..4 |
| 4 | `commands/project-foundation.md` | modify | L18: revert `init dev` workaround; use `init foundation --feature-stdin` | US-003 AC-003.1 |
| 5 | `commands/catchup.md` | modify | L19: concern list `dev, fix, foundation, refactor` | US-003 AC-003.2 |
| 6 | `skills/campaign-manifest/SKILL.md` | modify | L18: include `foundation` in concern enumeration | US-003 AC-003.5 |
| 7 | `skills/artifact-schemas/SKILL.md` | modify | L69: include `foundation` in concern value example | US-003 AC-003.6 |
| 8 | `CHANGELOG.md` | modify | BL-155 entry under `## Unreleased → ### Added` | US-003 AC-003.4 |
| 9 | `Cargo.toml` (workspace) | modify | `version = "4.2.0"` → `"4.3.0"` (minor bump per Decision 1) | spec Decision 1 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | FromStr parses "foundation" | AC-001.1 | `cargo test -p ecc-domain workflow::concern::tests::from_str_parses_foundation` | pass |
| PC-002 | unit | Display yields "foundation" | AC-001.2 | `cargo test -p ecc-domain workflow::concern::tests::foundation_displays_as_lowercase` | pass |
| PC-003 | unit | serde serialize foundation | AC-001.3 | `cargo test -p ecc-domain workflow::concern::tests::foundation_serializes_as_lowercase` | pass |
| PC-004 | unit | serde deserialize foundation | AC-001.3 | `cargo test -p ecc-domain workflow::concern::tests::deserializes_foundation` | pass |
| PC-005 | unit | UnknownConcern text lists all 4 alphabetically | AC-001.4, AC-001.7 | `cargo test -p ecc-domain workflow::concern::tests::unknown_concern_error_text_lists_all_four_alphabetically` | pass |
| PC-006 | unit | Case-sensitive: "Foundation" → Err | AC-001.6 | `cargo test -p ecc-domain workflow::concern::tests::from_str_rejects_capitalized_foundation` | pass |
| PC-007 | unit | Round-trip includes Foundation | AC-001.3, AC-001.5 | `cargo test -p ecc-domain workflow::concern::tests::round_trips_all_variants` | pass (4 variants) |
| PC-008 | unit | All existing concern tests still pass | AC-001.5 | `cargo test -p ecc-domain workflow::concern::` | all pass |
| PC-009 | unit | state.rs round-trip covers Foundation | AC-001.5 | `cargo test -p ecc-domain workflow::state::` | pass |
| PC-010 | integration | init foundation via ecc-workflow | AC-002.1 | `cargo test -p ecc-workflow --test init_foundation init_foundation_writes_concern_foundation` | state.json has `"concern":"foundation"` |
| PC-011 | integration | init via `ecc workflow` delegator | AC-002.2 | `cargo test -p ecc-workflow --test init_foundation delegator_init_foundation_matches_direct` | identical state |
| PC-012 | integration | worktree-name foundation | AC-002.3 | `cargo test -p ecc-workflow --test init_foundation worktree_name_foundation_returns_valid_slug` | non-empty valid name |
| PC-013 | integration | Full FSM walk preserves concern | AC-002.4 | `cargo test -p ecc-workflow --test init_foundation foundation_concern_persists_through_full_fsm_walk` | concern=foundation at done |
| PC-014 | lint | project-foundation.md uses `init foundation` (location-agnostic per adversary F1) | AC-003.1 | `grep -q 'ecc-workflow init foundation --feature-stdin' commands/project-foundation.md && ! grep -q 'init dev' commands/project-foundation.md` | match + no workaround |
| PC-015 | lint | catchup.md lists 4 concerns | AC-003.2 | `grep -q 'dev, fix, foundation, refactor' commands/catchup.md` | match |
| PC-016 | lint | CHANGELOG BL-155 entry | AC-003.4 | `grep -q 'BL-155' CHANGELOG.md` | match under `## Unreleased` |
| PC-017 | lint | campaign-manifest skill lists foundation | AC-003.5 | `grep -q 'foundation' skills/campaign-manifest/SKILL.md` | match |
| PC-018 | lint | artifact-schemas skill lists foundation | AC-003.6 | `grep -q 'foundation' skills/artifact-schemas/SKILL.md` | match |
| PC-019 | lint | ecc validate commands | Constraint | `cargo run --bin ecc -- validate commands` | 0 errors |
| PC-020 | build | clippy clean | global | `cargo clippy --workspace --all-targets -- -D warnings` | 0 warnings |
| PC-021 | build | Full test suite | global | `cargo test --workspace` | all pass |
| PC-022 | build | Workspace version 4.3.0 | Decision 1 | `grep -n '^version = "4.3.0"' Cargo.toml` | match |
| PC-023 | lint | No existing tests deleted/modified (diff-based per adversary round 1 finding AC-001.5) | AC-001.5 | `test $(git diff $(git merge-base HEAD origin/main) -- crates/ecc-domain/src/workflow/concern.rs \| awk '/^-[^-]/ && $0 ~ /fn |#\[test\]/' \| wc -l) -eq 0` | exit 0 |
| PC-024 | build | cargo-semver-checks with stable baseline (merge-base, not origin/main per adversary F2) | Decision 1 | `cargo semver-checks --baseline-rev $(git merge-base HEAD origin/main)` | allow minor-bump variant addition (waiver applied OR passes due to 4.3.0 minor bump) |
| PC-025 | build | cargo fmt check | global | `cargo fmt --check` | exit 0 |

### Coverage Check

| AC | Covering PC(s) |
|----|-----|
| AC-001.1 | PC-001 |
| AC-001.2 | PC-002 |
| AC-001.3 | PC-003, PC-004, PC-007 |
| AC-001.4 | PC-005 |
| AC-001.5 | PC-007, PC-008, PC-009, PC-023 |
| AC-001.6 | PC-006 |
| AC-001.7 | PC-005 |
| AC-002.1 | PC-010 |
| AC-002.2 | PC-011 |
| AC-002.3 | PC-012 |
| AC-002.4 | PC-013 |
| AC-003.1 | PC-014 |
| AC-003.2 | PC-015 |
| AC-003.4 | PC-016 |
| AC-003.5 | PC-017 |
| AC-003.6 | PC-018 |

**All 16 ACs covered by ≥1 PC.** AC-003.3 does not exist (spec renumbering — intentional).

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | ecc-workflow CLI foundation accept | — | — | init → plan → solution → implement → done with concern=foundation via tempfile::TempDir | un-ignored (this PR) | Always (regression protection) |

### E2E Activation Rules

PC-010..PC-013 run un-ignored on this PR. After merge, remain un-ignored as permanent regression protection against future enum-extension breakage.

## Test Strategy

TDD order (7 commits per adversary F6 split):

1. **PC-001..PC-008 + PC-009** (domain unit) — Write 3 new tests RED → add Foundation variant GREEN.
   - Commit: `test: add foundation concern unit tests (RED)`
   - Commit: `feat: add Concern::Foundation domain variant (GREEN)`
2. **PC-009** — Extend state.rs round-trip array.
   - Commit: `test: extend state round-trip to cover foundation concern`
3. **PC-010..PC-013** (integration) — Create `init_foundation.rs` with `tempfile::TempDir` + `env!("CARGO_BIN_EXE_ecc-workflow")`, 4 test fns. Likely no workflow-crate code change since `FromStr` is the only gate.
   - Commit: `test: add foundation init integration tests`
4a. **PC-014** — Revert `commands/project-foundation.md` L18 workaround.
   - Commit: `docs: revert /project-foundation init-dev workaround (BL-155 AC-003.1)`
4b. **PC-015, PC-017, PC-018** — Update `catchup.md` + 2 skill files.
   - Commit: `docs: propagate foundation concern to doc sites (BL-155 AC-003.2, AC-003.5, AC-003.6)`
4c. **PC-016** — CHANGELOG entry.
   - Commit: `docs(changelog): BL-155 Foundation Concern variant entry`
5. **PC-022** — Workspace version bump.
   - Commit: `chore: bump workspace version to 4.3.0 (BL-155)`

Final gates PC-019, PC-020, PC-021, PC-023, PC-024, PC-025 run after all commits (verify phase of /implement).

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `CHANGELOG.md` | project | modify | BL-155 entry under `## Unreleased → ### Added` (foundation variant, minor bump note) | AC-003.4 |
| 2 | `commands/project-foundation.md` | project | modify | Revert L18 workaround; use `init foundation --feature-stdin` | AC-003.1 |
| 3 | `commands/catchup.md` | project | modify | Update L19 concern list `dev, fix, foundation, refactor` | AC-003.2 |
| 4 | `skills/campaign-manifest/SKILL.md` | project | modify | L18 concern enumeration include `foundation` | AC-003.5 |
| 5 | `skills/artifact-schemas/SKILL.md` | project | modify | L69 concern value example include `foundation` | AC-003.6 |

No ADRs required (spec Decisions table §4 all No).

## SOLID Assessment

**PASS** (uncle-bob via design-reviewer). Additive enum extension; no dependency direction change; `ecc-domain` stays I/O-free; `workflow/` module cohesion preserved.

## Robert's Oath Check

**CLEAN** (robert via design-reviewer). 25 PCs / 16 ACs (Oath 3), 7 atomic commits (Oath 4), workaround removal = net-negative entropy (Oath 2, Oath 5).

## Security Notes

**CLEAR** (security-reviewer via design-reviewer). `FromStr` retains explicit rejection + case-sensitivity. `tempfile::TempDir` isolates integration test. No new attack surface.

## Rollback Plan

All-or-nothing per Decision 5 (single-PR atomicity):

1. Primary: `git revert -m 1 <merge-sha>` reverses the PR in full. All 7 commits reverted atomically.
2. CHANGELOG entry revert: handled by the PR revert (entry lives in `## Unreleased`, not a tagged release — no semantic drift).
3. Workspace version bump revert: 4.3.0 → 4.2.0 is part of the PR revert; no external consumers affected.
4. Semver baseline: `--baseline-rev $(git merge-base HEAD origin/main)` freezes to PR's divergence point, not `origin/main` (per adversary F2), so revert doesn't drift future unrelated PRs' semver-checks results.
5. **Rollback window caveat** (per adversary F3): users running `ecc-workflow init foundation` between PR merge and revert would write `concern=foundation` state.json files. Post-revert, these deserialize-fail. **Manual remediation**: edit offending state.json `concern` field from `foundation` back to `dev` (the pre-BL-155 workaround value). Document this in the revert PR description if ever needed.

No partial rollback is valid — reverting the version bump alone leaves variant-addition on main at 4.2.0 release, which violates the semver waiver's premise.

## Bounded Contexts Affected

| Context | Role | Files Modified |
|---------|------|----------------|
| workflow | value object (`Concern`) + state aggregate round-trip test | `crates/ecc-domain/src/workflow/concern.rs`, `crates/ecc-domain/src/workflow/state.rs` |

Other domain modules: none.

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS | 0 |
| Robert | CLEAN | 0 warnings |
| Security | CLEAR | 0 findings |
| AC Coverage | 16/16 covered | 0 uncovered |

### Adversary Findings (2 rounds)

| Round | Verdict | Avg Score | Key Delta |
|-------|---------|-----------|-----------|
| 1 | CONDITIONAL | 78/100 | 6 fixes: AC-003.3 gap, semver baseline, AC-001.5 diff-test, rollback plan, PC-semver + PC-fmt, commit atomicity split |
| 2 | **PASS** | **83/100** | Fragility 55→72, Rollback 60→74, Missing PCs 75→80. 3 non-blocking residuals baked into final design |

### Per-Dimension Final Scores (round 2)

| Dimension | Score | Verdict |
|-----------|-------|---------|
| AC Coverage | 88 | Good |
| Execution Order | 88 | Good |
| Fragility | 72 | Good |
| Rollback Adequacy | 74 | Good |
| Architecture Compliance | 92 | Excellent |
| Blast Radius | 86 | Good |
| Missing Pass Conditions | 80 | Good |
| Doc Plan Completeness | 84 | Good |

### File Changes Summary

9 files, 7 atomic commits in single PR per Decision 5.

### Artifacts Persisted

| File | Action |
|------|--------|
| `docs/specs/2026-04-19-bl155-foundation-concern/spec.md` | Spec (14 decisions in campaign) |
| `docs/specs/2026-04-19-bl155-foundation-concern/design.md` | This design (2 adversary rounds passed) |
| `docs/specs/2026-04-19-bl155-foundation-concern/campaign.md` | 15 decisions across /spec + /design |
