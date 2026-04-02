# Spec: BL-116 cargo-mutants Mutation Testing Integration

## Problem Statement

ECC has 2148+ tests with cargo-nextest and 80%+ line coverage targets, but no mechanism to measure test *quality* — whether tests actually detect behavioral changes. Audit TEST-008 (flagged 2026-03-14, reconfirmed 2026-03-29) identifies "No mutation testing configured" as a gap. cargo-mutants injects mutations into compiled code and checks if tests catch them, revealing untested behavioral paths that line coverage misses.

## Research Summary

- **nextest integration recommended**: `--test-tool=nextest` gives 30-40% faster mutation runs via early exit on first test failure
- **Well-tested Rust projects achieve 80-95% mutation scores**; pragmatic target is 75-85%
- **CI strategy**: incremental `--in-diff` on PRs (fast), full suite periodically (thorough)
- **No source modifications required**: works out-of-box on any Rust project
- **Key pitfall**: flaky tests cause false "undetected mutant" results — test suite must be deterministic
- **cargo-mutants is the de facto standard** — mutest-rs exists but is less mature, less documented, no nextest integration
- **Workspace support**: use `mutants.toml` to scope packages; testing full workspace multiplies runtime

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | cargo-mutants over mutest-rs | Actively maintained, nextest integration, no code annotations, workspace support | Yes |
| 2 | Scope to ecc-domain + ecc-app only | Highest ROI (pure business logic + orchestration); infra/cli adapter mutations are noisy | Yes |
| 3 | Aspirational thresholds: 100% domain validation, 85% app (TBD after baseline) | Domain validation is pure business rules; app has I/O-backed orchestration. Thresholds will be revised after US-002/003 baseline measurement. | Yes |
| 4 | Non-blocking CI initially | Too slow for blocking gate; use continue-on-error until scores stabilize | No |
| 5 | --in-diff for /verify integration | Full mutation runs too slow for pre-PR gate; diff-scoped keeps under 5 min | No |
| 6 | nextest as test tool for mutations | 30-40% faster via early-exit on first failure, already used by project | No |
| 7 | Graceful degradation when not installed | /mutants, /verify, xtask all check for binary and print install instructions | No |
| 8 | xtask subcommand for structured execution | Follows existing xtask pattern (deploy), enables structured output and flags | No |

## User Stories

### US-001: Install and Configure cargo-mutants

**As a** developer, **I want** cargo-mutants installed and configured for the ECC workspace, **so that** I can run mutation testing against targeted crates.

#### Acceptance Criteria

- AC-001.1: Given a fresh checkout, when `mutants.toml` exists at workspace root, then it configures nextest integration, 120s per-mutant timeout, and scopes to ecc-domain + ecc-app
- AC-001.2: Given `.gitignore`, when updated, then `mutants.out/` is excluded from version control
- AC-001.3: Given CLAUDE.md, when updated, then it documents `cargo mutants` alongside existing test commands
- AC-001.4: Given `docs/sources.md`, when updated, then cargo-mutants is listed as a knowledge source under testing

#### Dependencies

- Depends on: none

### US-002: Establish Mutation Testing Baseline for ecc-domain

**As a** developer, **I want** an initial mutation testing report for ecc-domain, **so that** I know which domain modules have the weakest test quality.

#### Acceptance Criteria

- AC-002.1: Given ecc-domain crate, when `cargo mutants -p ecc-domain` runs, then it completes without infrastructure errors
- AC-002.2: Given mutation results, when analyzed, then a baseline report is persisted to `docs/audits/mutation-baseline-ecc-domain.md`
- AC-002.3: Given the report, when reviewed, then it identifies top modules with most surviving mutants
- AC-002.4: Given the report, when reviewed, then it includes total mutants tested, killed, survived, timed-out counts

#### Dependencies

- Depends on: US-001

### US-003: Establish Mutation Testing Baseline for ecc-app

**As a** developer, **I want** an initial mutation testing report for ecc-app, **so that** I know which application modules have the weakest test quality.

#### Acceptance Criteria

- AC-003.1: Given ecc-app crate, when `cargo mutants -p ecc-app` runs, then it completes without infrastructure errors
- AC-003.2: Given mutation results, when analyzed, then a baseline report is persisted to `docs/audits/mutation-baseline-ecc-app.md`
- AC-003.3: Given the report, when reviewed, then it identifies top modules with most surviving mutants
- AC-003.4: Given the report, when reviewed, then it includes total mutants tested, killed, survived, timed-out counts

#### Dependencies

- Depends on: US-001

### US-004: Add xtask mutants Subcommand

**As a** developer, **I want** a `cargo xtask mutants` subcommand, **so that** I can run structured mutation testing with flags for package selection, timeout, and diff mode.

#### Acceptance Criteria

- AC-004.1: Given `xtask/src/mutants.rs`, when created, then it defines a `Mutants` subcommand with `--package`, `--timeout`, `--in-diff`, and `--nextest` flags
- AC-004.2: Given `cargo xtask mutants`, when run, then it invokes `cargo mutants` with configured defaults from mutants.toml
- AC-004.3: Given `cargo xtask mutants --in-diff`, when run, then it passes `--in-diff` to cargo-mutants using `origin/main` as the diff base for branch-scoped testing
- AC-004.4: Given cargo-mutants not installed, when xtask mutants is run, then it prints a clear install message and exits with code 1

#### Dependencies

- Depends on: US-001

### US-005: Add /mutants Slash Command

**As a** developer, **I want** a `/mutants` slash command in Claude Code, **so that** I can run targeted mutation testing within sessions.

#### Acceptance Criteria

- AC-005.1: Given `commands/mutants/COMMAND.md`, when created, then it defines a slash command with arguments for crate targeting and diff mode
- AC-005.2: Given `/mutants ecc-domain`, when invoked, then it runs `cargo xtask mutants --package ecc-domain` and reports results
- AC-005.3: Given `/mutants --diff`, when invoked, then it runs diff-scoped mutation testing on changed files
- AC-005.4: Given the output, when complete, then it presents killed/survived/timeout counts and top surviving mutants

#### Dependencies

- Depends on: US-004

### US-006: Integrate Mutation Testing into /verify

**As a** developer, **I want** `/verify` to optionally run diff-scoped mutation testing, **so that** I can include mutation quality in my pre-PR gate.

#### Acceptance Criteria

- AC-006.1: Given `/verify full --mutation`, when invoked with the `--mutation` flag, then it runs the full verify pipeline plus diff-scoped mutation testing as an additional step after architecture review
- AC-006.2: Given `/verify full` or `/verify quick`, when invoked without `--mutation` flag, then mutation testing is NOT run and output is identical to current behavior
- AC-006.3: Given verify output with mutations, when displayed, then it shows a Mutation: line with killed/survived/timeout
- AC-006.4: Given surviving mutants in /verify, when reported, then they do NOT block "Ready for PR" verdict

#### Dependencies

- Depends on: US-004

### US-007: Add Non-Blocking CI Mutation Job

**As a** maintainer, **I want** a non-blocking mutation testing job in CI, **so that** mutation scores are tracked automatically on merges.

#### Acceptance Criteria

- AC-007.1: Given `.github/workflows/ci.yml`, when updated, then a `mutation` job runs cargo-mutants on ecc-domain and ecc-app
- AC-007.2: Given the mutation job, when configured, then it uses `continue-on-error: true` (non-blocking)
- AC-007.3: Given the mutation job, when it completes, then mutation report is uploaded as artifact
- AC-007.4: Given the mutation job, when configured, then it has 30-minute timeout and separate cache key
- AC-007.5: Given the mutation CI job, when cargo-mutants is not pre-installed, then it installs a pinned version (e.g., `cargo install cargo-mutants@25.1.1`) before running
- AC-007.6: Given the mutation CI job, when configured, then it uses full mutation testing (not `--in-diff`) since CI runs on merged code, and uses `fetch-depth: 0` to ensure full git history is available

#### Dependencies

- Depends on: US-001

### US-008: Mutation Score Dashboard

**As a** developer, **I want** a living document tracking mutation scores per crate, **so that** I can prioritize test improvement efforts.

#### Acceptance Criteria

- AC-008.1: Given `docs/audits/mutation-scores.md`, when created, then it contains per-crate mutation scores table
- AC-008.2: Given the dashboard, when updated after baseline runs, then it shows latest scores with dates
- AC-008.3: Given the dashboard, when reviewed, then crates are ranked weakest to strongest
- AC-008.4: Given the dashboard, when reviewed, then it includes per-module breakdown for ecc-domain and ecc-app

#### Dependencies

- Depends on: US-002, US-003

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| `mutants.toml` (new) | Configuration | Create workspace-scoped mutation config |
| `xtask/src/mutants.rs` (new) | Developer tooling | New xtask subcommand |
| `xtask/src/main.rs` | Developer tooling | Register mutants subcommand |
| `commands/mutants/COMMAND.md` (new) | Claude Code commands | New slash command |
| `commands/verify/COMMAND.md` | Claude Code commands | Add mutation mode |
| `.github/workflows/ci.yml` | CI | Add non-blocking mutation job |
| `.gitignore` | Configuration | Add mutants.out/ |
| CLAUDE.md | Documentation | Add cargo mutants to test commands |
| `docs/sources.md` | Documentation | Add cargo-mutants source |
| `docs/audits/mutation-baseline-*.md` (new) | Documentation | Baseline reports |
| `docs/audits/mutation-scores.md` (new) | Documentation | Living dashboard |

## Constraints

- cargo-mutants is an external tool installed via `cargo install`, NOT a Cargo dependency
- No Rust `ecc mutants` CLI subcommand — mutation testing has no domain logic (architect flag)
- CI job must be non-blocking until scores stabilize
- Per-mutant timeout 120s to catch infinite loops
- CI job timeout 30 minutes
- Must not break existing /verify quick or /verify full flows
- cargo-mutants version must be pinned in CI for reproducible baselines
- Baseline reports must distinguish timed-out mutants from survived mutants (flaky test mitigation)
- CI mutation job uses full mutation (not --in-diff) with fetch-depth: 0

## Non-Requirements

- Full workspace mutation testing (only ecc-domain + ecc-app)
- Blocking CI gate (non-blocking only)
- Mutation testing of infra/cli/workflow/flock crates
- Real-time mutation score tracking (manual dashboard updates for now)
- Custom mutant operators or mutation strategies

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | N/A | No hexagonal boundaries crossed — pure tooling integration |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New command | CLAUDE.md | Test commands section | Add `cargo mutants` and `cargo xtask mutants` |
| New source | docs/sources.md | Testing section | Add cargo-mutants entry |
| New report | docs/audits/ | Mutation baselines | Create baseline + dashboard files |
| ADR | docs/adr/ | Two new ADRs | Tool choice + threshold strategy |

## Open Questions

None — all questions resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Scope exclusions | Accept recommended + include CI non-blocking job + include xtask subcommand | User |
| 2 | Edge case: cargo-mutants not installed | Graceful degradation with install message | Recommended |
| 3 | Test thresholds | 100% domain validation, 85% app orchestration (aspirational, TBD after baseline) | Recommended |
| 4 | Performance constraints | 120s per-mutant, 30min CI cap, --in-diff for /verify | Recommended |
| 5 | Security implications | None — local-only tool, contents:read CI permission | Recommended |
| 6 | Breaking changes | None — all additions are additive | Recommended |
| 7 | Domain glossary | Add mutation score, surviving mutant, diff-scoped mutation testing | Recommended |
| 8 | ADR decisions | Two ADRs: tool choice/scoping + threshold strategy | User |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Install and Configure cargo-mutants | 4 | none |
| US-002 | Establish Baseline for ecc-domain | 4 | US-001 |
| US-003 | Establish Baseline for ecc-app | 4 | US-001 |
| US-004 | Add xtask mutants Subcommand | 4 | US-001 |
| US-005 | Add /mutants Slash Command | 4 | US-004 |
| US-006 | Integrate into /verify | 4 | US-004 |
| US-007 | Non-Blocking CI Mutation Job | 6 | US-001 |
| US-008 | Mutation Score Dashboard | 4 | US-002, US-003 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | mutants.toml with nextest, 120s timeout, scoped crates | US-001 |
| AC-001.2 | .gitignore excludes mutants.out/ | US-001 |
| AC-001.3 | CLAUDE.md documents cargo mutants | US-001 |
| AC-001.4 | docs/sources.md lists cargo-mutants | US-001 |
| AC-002.1 | ecc-domain mutation run completes | US-002 |
| AC-002.2 | Baseline report persisted | US-002 |
| AC-002.3 | Top modules with surviving mutants identified | US-002 |
| AC-002.4 | Counts: tested, killed, survived, timed-out | US-002 |
| AC-003.1 | ecc-app mutation run completes | US-003 |
| AC-003.2 | Baseline report persisted | US-003 |
| AC-003.3 | Top modules with surviving mutants identified | US-003 |
| AC-003.4 | Counts: tested, killed, survived, timed-out | US-003 |
| AC-004.1 | xtask Mutants subcommand with --package, --timeout, --in-diff, --nextest | US-004 |
| AC-004.2 | Invokes cargo mutants with defaults | US-004 |
| AC-004.3 | --in-diff uses origin/main as diff base | US-004 |
| AC-004.4 | Graceful error when cargo-mutants not installed | US-004 |
| AC-005.1 | /mutants COMMAND.md created | US-005 |
| AC-005.2 | /mutants ecc-domain runs targeted mutation testing | US-005 |
| AC-005.3 | /mutants --diff runs diff-scoped testing | US-005 |
| AC-005.4 | Summary with killed/survived/timeout counts | US-005 |
| AC-006.1 | /verify full --mutation adds mutation step | US-006 |
| AC-006.2 | /verify without --mutation is unchanged | US-006 |
| AC-006.3 | Mutation: summary line in output | US-006 |
| AC-006.4 | Surviving mutants don't block verdict | US-006 |
| AC-007.1 | CI mutation job in ci.yml | US-007 |
| AC-007.2 | continue-on-error: true | US-007 |
| AC-007.3 | Mutation report uploaded as artifact | US-007 |
| AC-007.4 | 30-min timeout, separate cache key | US-007 |
| AC-007.5 | CI installs pinned cargo-mutants version | US-007 |
| AC-007.6 | CI uses full mutation, fetch-depth: 0 | US-007 |
| AC-008.1 | Per-crate mutation scores table | US-008 |
| AC-008.2 | Latest scores with dates | US-008 |
| AC-008.3 | Crates ranked weakest to strongest | US-008 |
| AC-008.4 | Per-module breakdown for domain and app | US-008 |

### Adversary Findings

| Dimension | Score (R1→R2) | Verdict | Key Rationale |
|-----------|---------------|---------|---------------|
| Completeness | 72→92 | PASS | CI install, fetch-depth, version pin added |
| Testability | 65→90 | PASS | Given/When/Then, timed-out distinction |
| Consistency | n/a→95 | PASS | No contradictions |
| Feasibility | n/a→88 | PASS | All implementable with current tooling |
| Scope | 78→93 | PASS | Well-bounded non-requirements |
| Dependencies | 82→94 | PASS | External deps clearly called out |
| Risk | n/a→89 | PASS | Ambiguities resolved, thresholds deferred |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-31-cargo-mutants/spec.md | Full spec + Phase Summary |
