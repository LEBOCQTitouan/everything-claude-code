# Spec: Add cargo-llvm-cov Coverage Gate to CI

## Problem Statement

The project mandates 80% test coverage but only enforces it locally. PRs can merge without meeting the coverage floor because CI lacks a coverage gate. Adding cargo-llvm-cov (already used locally, rated "Adopt" in web radar) to CI closes this gap.

## Research Summary

- cargo-llvm-cov uses LLVM source-based instrumentation — more accurate than gcov-based tools, supports function/line/region coverage
- `--fail-under-functions` flag natively fails the command if coverage drops below threshold — no custom scripting needed
- `taiki-e/install-action@v2` is the recommended way to install cargo-llvm-cov in CI (handles llvm-tools-preview component)
- Instrumented builds produce incompatible artifacts — must use separate cache key from regular builds
- `if: always()` on artifact upload ensures LCOV is available even when the gate fails (critical for debugging)
- Path filtering via `dorny/paths-filter@v3` avoids wasting CI minutes on docs-only PRs; must be bypassed on `merge_group` events
- Workspace-level coverage (`--workspace`) is standard; exclude non-production crates (`xtask`, `ecc-test-support`) to avoid distortion

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Separate `coverage` job (not part of `validate`) | Cache incompatibility between instrumented/non-instrumented builds | No |
| 2 | Workspace aggregate 80% function threshold | Matches project rule; per-crate adds complexity | No |
| 3 | Artifact-only, no PR comments | Least-privilege (no `pull-requests: write` needed) | No |
| 4 | Path filter to skip on non-Rust changes | Avoids wasting CI minutes on docs/config PRs | No |
| 5 | Blocking required status check | 80% is a hard contract, not advisory | No |
| 6 | 20-minute timeout | Based on estimated instrumented build time of 8-12 min (warm cache); 20 min provides ~2x headroom. Cold-cache first run may approach 20 min — acceptable for one-time cost; subsequent runs use cache. | No |
| 7 | Add "coverage gate" to CLAUDE.md glossary | User requested | No |
| 8 | Baseline measurement before merge | Must confirm main >= 80% function coverage before enabling; if below, use current level as initial threshold | No |
| 9 | Exclude `xtask` and `ecc-test-support` from coverage | Non-production crates distort aggregate; `xtask` has no prod tests, `ecc-test-support` is pure helpers | No |
| 10 | `continue-on-error: false` (explicit) | Coverage is a hard gate unlike security/mutation which are advisory | No |
| 11 | `runs-on: ubuntu-latest` | Consistent with `validate` job; avoids platform-specific instrumentation differences | No |
| 12 | Transient install-action failures handled by re-run | Standard GHA practice; false green from `continue-on-error: true` is worse than CI downtime. Job-level 20-min timeout covers hung installs. | No |
| 13 | Pin `dorny/paths-filter@v3` | ECC convention: pin action versions for reproducibility | No |
| 14 | Coverage job uses workflow-level concurrency group (inherited) | No job-level concurrency needed — cancellation behavior matches existing jobs | No |

## User Stories

### US-001: CI coverage gate

**As a** contributor, **I want** CI to fail my PR if test coverage drops below 80%, **so that** the coverage floor is enforced automatically.

#### Acceptance Criteria

- AC-001.0: The coverage job includes a `dtolnay/rust-toolchain@stable` setup step before `taiki-e/install-action@v2`, matching the toolchain used in the `validate` job
- AC-001.1: Given a PR with Rust changes, when CI runs, then a `coverage` job (with `name: Coverage Gate`) executes `cargo llvm-cov --workspace --exclude xtask --exclude ecc-test-support --lcov --output-path lcov.info --fail-under-functions 80`
- AC-001.2: Given workspace function coverage < 80%, when the coverage job runs, then the job fails with a non-zero exit code
- AC-001.3: Given workspace function coverage >= 80%, when the coverage job runs, then the job passes
- AC-001.4: Given the coverage job completes (pass or fail), when artifacts are uploaded, then an artifact named `coverage-report` containing `lcov.info` is available with 14-day retention
- AC-001.5: Given the coverage job YAML, then all actions are pinned to explicit major versions (`taiki-e/install-action@v2`, `dorny/paths-filter@v3`, `actions/upload-artifact@v4`, `actions/cache@v4`) — no `@latest` tags
- AC-001.6: Given main function coverage < 80% at baseline measurement, when the coverage job is configured, then `--fail-under-functions` is set to `floor(measured_baseline_percent)` as an integer

#### Verification Note

Proof criteria require a CI run against a branch with function coverage at or above 80%. Below-threshold verification requires a synthetic branch with intentionally reduced test coverage. `--fail-under-functions N` is inclusive: coverage of exactly N% exits 0 (pass). [Human/operational] If baseline < 80%, the implementing PR description includes a `## Coverage Deviation` section naming the measured value and an ECC backlog item tracking restoration to 80%.

### US-002: Path filtering

**As a** contributor, **I want** the coverage job to skip when my PR only changes non-Rust files, **so that** CI minutes are not wasted.

#### Acceptance Criteria

- AC-002.1: Given a PR that changes no `*.rs`, `Cargo.toml`, or `Cargo.lock` files, when CI triggers via `pull_request`, then the `coverage` job is skipped (path filter uses `dorny/paths-filter@v3`). A GHA `skipped` status for `Coverage Gate` satisfies the branch protection required-check constraint.
- AC-002.2: Given a PR that changes any `*.rs`, `Cargo.toml`, or `Cargo.lock` file, when CI triggers, then the `coverage` job runs
- AC-002.3: Given the CI workflow is triggered by a `merge_group` event, when the coverage job evaluates path filter, then path filtering is bypassed and the job always runs. Mechanism: job-level `if:` condition combines path-filter output with event-name check (`github.event_name == 'merge_group' || steps.changes.outputs.rust == 'true'`)

#### Verification Note

AC-002.1 and AC-002.2 are verified by CI run against PRs with/without Rust changes. AC-002.3 is verifiable via static YAML inspection: the job's `if:` condition must explicitly include `github.event_name == 'merge_group'` — no live merge_group event required for structural verification.

#### Dependencies

- Depends on: US-001

### US-003: Cache isolation

**As a** CI pipeline, **I want** the coverage job to use a separate cache key, **so that** instrumented and non-instrumented builds don't thrash each other.

#### Acceptance Criteria

- AC-003.1: Given the coverage job, when caching via `actions/cache@v4`, then the cache key is `${{ runner.os }}-cargo-llvm-cov-${{ hashFiles('**/Cargo.lock') }}` with restore-keys prefix `${{ runner.os }}-cargo-llvm-cov-` (not `${{ runner.os }}-cargo-`, to prevent restore from `validate` job cache)

#### Dependencies

- Depends on: US-001

### US-004: Glossary update

**As a** developer reading CLAUDE.md, **I want** "coverage gate" defined in the glossary, **so that** the term is unambiguous.

#### Acceptance Criteria

- AC-004.1: Given CLAUDE.md, when reading the existing `Glossary:` bullet in `## Gotchas`, then "coverage gate" is appended with a definition containing the phrases "CI job", "function coverage", and the effective threshold value. Verified by: `grep 'coverage gate' CLAUDE.md`

#### Dependencies

- None

### US-005: Branch protection configuration

**As a** repo admin, **I want** the `coverage` job registered as a required status check in branch protection, **so that** it actually blocks merges.

#### Acceptance Criteria

- AC-005.1a: Given ci.yml, when inspecting the coverage job, then it has `name: Coverage Gate` — verifiable via `grep 'name: Coverage Gate' .github/workflows/ci.yml`
- AC-005.1b: [Operational] Given branch protection rules for `main` are configured by a repo admin, then `Coverage Gate` appears in the required status checks list (the string must match AC-005.1a character-for-character). This is a manual admin step, not a CI-testable criterion.

#### Verification Note

AC-005.1a is verified by grepping ci.yml. AC-005.1b is verified by running `gh api repos/{owner}/{repo}/branches/main/protection --jq '.required_status_checks.contexts[]'` and confirming `Coverage Gate` appears in the output.

#### Dependencies

- Depends on: US-001

## Preconditions

Before merging the PR that adds this gate:

1. Run `cargo llvm-cov --workspace --exclude xtask --exclude ecc-test-support --fail-under-functions 80` on `main`
2. If it fails, revise the threshold to the current coverage level and create a follow-up item to raise it to 80%

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `.github/workflows/ci.yml` | CI | Add `coverage` job |
| `CLAUDE.md` | Docs | Add glossary entry |

## Constraints

- Must follow existing CI conventions (see `rules/ecc/github-actions.md`): least-privilege permissions, pinned action versions, concurrency groups
- Must not modify the `validate` job or its cache key
- `coverage` job runs in parallel with `validate` (no `needs:` dependency)
- Separate cache key mandatory to avoid build artifact thrashing
- Job runs on `ubuntu-latest` consistent with existing jobs
- `continue-on-error: false` (blocking gate)
- Coverage job uses inherited workflow-level concurrency group — no job-level concurrency block

## Non-Requirements

- Codecov/Coveralls integration
- PR comment posting
- Branch/region coverage enforcement
- Per-crate coverage thresholds
- Coverage trend tracking over time

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| GitHub Actions CI | New job | New required status check in branch protection |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Glossary addition | Minor | CLAUDE.md | Add "coverage gate" definition |
| CI reference | Minor | CLAUDE.md | Already documents `cargo llvm-cov --workspace` |

## Rollback

To disable the gate:
1. Remove `Coverage Gate` from branch protection required checks (requires repo admin access — any repo admin can do this via Settings > Branches > Protection rules)
2. Optionally remove the `coverage` job from `ci.yml`
3. No data migration required

## Open Questions

None — all resolved during grill-me interview and adversarial review.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | What is explicitly OUT of scope? | Codecov integration, PR comments, branch coverage, trend tracking | Recommended |
| 2 | Skip coverage when no Rust files changed? | Yes, path-filter on *.rs, Cargo.toml, Cargo.lock | Recommended |
| 3 | Per-crate or workspace aggregate threshold? | Workspace aggregate 80% only | Recommended |
| 4 | CI timeout for coverage job? | 20 minutes, separate cache key cargo-llvm-cov-* | Recommended |
| 5 | Security implications? | None — read-only, no secrets, no external uploads | Recommended |
| 6 | Blocking required status check? | Yes, coverage job blocks PR merge if below 80% | Recommended |
| 7 | New domain terms for glossary? | Add coverage gate definition to CLAUDE.md glossary | User |
| 8 | ADR needed? | No — straightforward CI addition following established patterns | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | CI coverage gate | 7 | None |
| US-002 | Path filtering | 3 | US-001 |
| US-003 | Cache isolation | 1 | US-001 |
| US-004 | Glossary update | 1 | None |
| US-005 | Branch protection configuration | 2 | US-001 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.0 | Toolchain setup step before install-action | US-001 |
| AC-001.1 | Coverage job executes cargo llvm-cov with full flags | US-001 |
| AC-001.2 | Job fails when coverage < 80% | US-001 |
| AC-001.3 | Job passes when coverage >= 80% | US-001 |
| AC-001.4 | LCOV artifact uploaded as coverage-report | US-001 |
| AC-001.5 | All actions pinned to major versions | US-001 |
| AC-001.6 | Baseline < 80% uses floor(measured) as threshold | US-001 |
| AC-002.1 | Skipped on docs-only PRs via dorny/paths-filter@v3 | US-002 |
| AC-002.2 | Runs when *.rs/Cargo.toml/Cargo.lock changed | US-002 |
| AC-002.3 | merge_group bypasses path filter via if: condition | US-002 |
| AC-003.1 | Separate cache key with cargo-llvm-cov- prefix | US-003 |
| AC-004.1 | Glossary entry in CLAUDE.md Gotchas section | US-004 |
| AC-005.1a | YAML name: Coverage Gate (grep-verifiable) | US-005 |
| AC-005.1b | Branch protection includes Coverage Gate (operational) | US-005 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Ambiguity | 74 | PASS | Full command specified, action versions pinned, glossary placement defined |
| Edge Cases | 78 | PASS | merge_group bypass mechanism specified, cache restore-key isolation |
| Scope Creep | 90 | PASS | Non-Requirements exhaustive, boundaries tight |
| Dependencies | 85 | PASS | DAG correct, toolchain step added |
| Testability | 80 | PASS | All ACs deterministic or marked operational |
| Decision Completeness | 82 | PASS | 14 decisions covering all concerns |
| Rollback | 88 | PASS | Two-step rollback, no data migration |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-09-cargo-llvm-cov-ci-gate/spec.md | Full spec |
| docs/specs/2026-04-09-cargo-llvm-cov-ci-gate/spec-draft.md | Draft (pre-adversary) |
| docs/specs/2026-04-09-cargo-llvm-cov-ci-gate/campaign.md | Grill-me decisions |
