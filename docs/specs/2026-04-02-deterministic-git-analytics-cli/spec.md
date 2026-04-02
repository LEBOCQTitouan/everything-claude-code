# Spec: BL-071 Deterministic Git Analytics CLI

## Problem Statement

Three LLM-driven agents (changelog-gen skill, evolution-analyst agent) perform git history analysis that is entirely mechanical: parsing conventional commits, counting file changes, computing co-change frequency, and measuring author diversity. These operations take 30-60s through LLM agents but could run in <2s as deterministic Rust code. The results are not reproducible (LLM variance) and cannot run in CI without an LLM API key.

## Research Summary

- Web research skipped: no search tool available.
- Conventional commit parsing is well-specified (conventionalcommits.org) with regex-based parsing
- Hotspot analysis pattern from "Your Code as a Crime Scene" (Adam Tornhill) -- file change frequency as a proxy for risk
- Co-change coupling is a standard software evolution metric -- `commits_together / max(commits_A, commits_B)`
- Bus factor per file is `count(distinct authors)` -- simple git log aggregation
- Prior art in ECC: evolution-analyst agent already runs these exact git commands, just via LLM orchestration
- Audit reports (EVO-002, CORR-001) already consume hotspot and coupling data -- deterministic CLI makes these reproducible

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Introduce GitLogPort trait in ecc-ports | 4 subcommands all need git log data; dedicated port avoids leaking git CLI args into app layer; proper hex arch | Yes |
| 2 | Coupling formula: commits_together / max(A,B) | Matches existing evolution-analyst behavior; standard metric | No |
| 3 | Configurable thresholds for coupling | --min-commits (default 3), --max-files-per-commit (default 20) as flags | No |
| 4 | Unified --since accepting tags and dates | All 4 subcommands use same --since semantics for consistency | No |
| 5 | Defaults: hotspots/coupling/bus-factor 90 days, changelog last tag | Practical defaults matching typical dev workflow | No |
| 6 | Command namespace: `ecc analyze` | Self-descriptive subcommands; "analyze" is clear enough | No |

## User Stories

### US-001: Conventional Commit Parser (Domain)

**As a** developer, **I want** git commit messages parsed into structured conventional commit types, **so that** changelog and analytics features share a tested parsing foundation.

#### Acceptance Criteria

- AC-001.1: Given a commit message `"feat(cli): add analyze command"`, when parsed, then type=feat, scope=Some("cli"), description="add analyze command", breaking=false
- AC-001.2: Given a commit message `"feat!: remove legacy API"`, when parsed, then breaking=true
- AC-001.3: Given a non-conventional message `"update readme"`, when parsed, then type=other (no panic)
- AC-001.4: Given a commit with `BREAKING CHANGE:` in the footer, when parsed, then breaking=true
- AC-001.5: Given empty or malformed input, when parsed, then a sensible default is returned (no panic)

#### Dependencies

- Depends on: none

### US-002: GitLogPort Trait and Adapter

**As a** developer, **I want** a GitLogPort trait abstracting git log operations, **so that** all analyze subcommands depend on a testable port instead of raw shell commands.

#### Acceptance Criteria

- AC-002.1: Given the GitLogPort trait in ecc-ports, when inspected, then it has methods returning domain types (Vec<GitCommit> with hash, author, date, subject, body, changed_files)
- AC-002.2: Given the infra adapter, when called with a date range, then it runs `git log` with correct format flags and parses output into domain types
- AC-002.3: Given the infra adapter, when called with a tag as --since value, then it uses `<tag>..HEAD` revision range
- AC-002.4: Given an empty git log result (no commits), when the adapter runs, then it returns an empty Vec (no error)
- AC-002.5: Given the adapter, when tested with a temp git repo, then 100% of output parsing paths are covered
- AC-002.6: Given `git` binary not found on PATH, when any analyze subcommand runs, then a clear error message is returned (not a panic)
- AC-002.7: Given `--since` with a nonexistent tag or malformed date, when any subcommand runs, then a descriptive error is returned

#### Dependencies

- Depends on: US-001 (needs ConventionalCommit type for commit parsing)

### US-003: Changelog Generation Subcommand

**As a** CLI user, **I want** `ecc analyze changelog [--since <tag|date>]`, **so that** I get a markdown changelog grouped by conventional commit type.

#### Acceptance Criteria

- AC-003.1: Given a repo with conventional commits since tag v1.0.0, when running `ecc analyze changelog --since v1.0.0`, then output is markdown with sections: Added (feat), Fixed (fix), Changed (refactor), Performance (perf), Documentation (docs), Maintenance (chore), CI/CD (ci), Testing (test)
- AC-003.2: Given `--since 2026-01-01`, when running, then only commits after that date are included
- AC-003.3: Given no `--since` flag, when running, then all commits since the last semver tag are used (or last 90 days if no tags)
- AC-003.4: Given breaking changes, when generating changelog, then a "BREAKING CHANGES" section appears at the top
- AC-003.5: Given empty sections, when generating changelog, then those sections are omitted
- AC-003.6: Given non-conventional commits, when generating changelog, then they appear in an "Other" section
- AC-003.7: Given a repo with no semver tags and no `--since` flag, when running changelog, then the 90-day fallback is applied and documented in output header

#### Dependencies

- Depends on: US-001, US-002

### US-004: Hotspot Analysis Subcommand

**As a** CLI user, **I want** `ecc analyze hotspots [--top N] [--since <tag|date>]`, **so that** I can identify most frequently changed files.

#### Acceptance Criteria

- AC-004.1: Given a repo with 500 commits, when running `ecc analyze hotspots --top 10`, then the 10 most frequently changed files are listed with change counts, sorted descending
- AC-004.2: Given `--since 2026-01-01`, when running, then only commits after that date are counted
- AC-004.3: Given --top not specified, when running, then default is 10
- AC-004.4: Given deleted files, when computing hotspots, then they are still counted (historical analysis)
- AC-004.5: Given --top 0 or negative, when running, then an error is returned with a helpful message

#### Dependencies

- Depends on: US-002

### US-005: Co-Change Coupling Subcommand

**As a** CLI user, **I want** `ecc analyze coupling [--threshold 0.7] [--min-commits 3] [--max-files-per-commit 20] [--since <tag|date>]`, **so that** I can identify files with hidden coupling.

#### Acceptance Criteria

- AC-005.1: Given files A and B in 8 of the same 10 commits, when running with threshold 0.7, then pair (A,B) appears with score 0.80
- AC-005.2: Given --threshold 0.5, when running, then pairs >= 0.5 are included
- AC-005.3: Given no --threshold, when running, then default is 0.7
- AC-005.4: Given output, when rendered, then pairs are sorted by coupling score descending
- AC-005.5: Given a commit touching 20+ files, when computing, then that commit is excluded (configurable via --max-files-per-commit)
- AC-005.6: Given a file pair where both have fewer than 3 commits, when computing, then the pair is excluded (configurable via --min-commits)
- AC-005.7: Given coupling formula, when computed, then it uses commits_together / max(commits_A, commits_B)
- AC-005.8: Given `--max-files-per-commit 20` and a commit touching exactly 20 files, when computing coupling, then that commit IS included (excluded only when strictly exceeding threshold)

#### Dependencies

- Depends on: US-002

### US-006: Bus Factor Subcommand

**As a** CLI user, **I want** `ecc analyze bus-factor [--top N] [--since <tag|date>]`, **so that** I can identify files with single-author risk.

#### Acceptance Criteria

- AC-006.1: Given a repo, when running `ecc analyze bus-factor --top 10`, then the 10 files with lowest unique author count are listed
- AC-006.2: Given output, when rendered, then each line shows file path, unique author count, and total commits
- AC-006.3: Given --since, when running, then only commits in range are considered
- AC-006.4: Given --top not specified, when running, then default is 10
- AC-006.5: Given a file with bus factor = 1, when listed, then it is flagged as "RISK: single author"

#### Dependencies

- Depends on: US-002

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| ecc-domain/src/analyze/ | Domain | New: ConventionalCommit, CommitType, Hotspot, CouplingPair, BusFactor VOs; pure scoring/formatting fns |
| ecc-ports/src/git_log.rs | Port | New: GitLogPort trait with methods returning domain types |
| ecc-infra/src/git_log.rs | Infra/Adapter | New: GitLogAdapter implementing GitLogPort, runs git log, parses output |
| ecc-app/src/analyze/ | App | New: 4 thin use cases calling GitLogPort then domain functions |
| ecc-cli/src/commands/analyze.rs | CLI | New: Clap subcommand with Changelog/Hotspots/Coupling/BusFactor variants |

## Constraints

- ecc-domain must have zero I/O imports (enforced by hook)
- GitLogPort adapter must handle repos with 10,000+ commits without timeout
- Conventional commit parsing must handle non-conventional messages gracefully
- All --since values must accept both ISO 8601 dates and git tags/refs
- Coupling computation must cap per-commit file count to avoid O(n^2) blowup
- All subcommands use `--no-merges` by default to avoid double-counting files from merge commits

## Non-Requirements

- JSON output format (defer to follow-up)
- Complexity trend analysis (requires multi-commit checkout -- different beast)
- Updating changelog-gen skill or evolution-analyst agent (separate BL item)
- LLM-enhanced changelog rewriting
- Git blame integration (beyond bus factor author counting)
- Visualization or charts

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| GitLogPort / GitLogAdapter | New port + adapter | Integration tests against temp git repos needed |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CLI commands | Project | CLAUDE.md | Add ecc analyze subcommands |
| ADR | Project | docs/adr/ | ADR for GitLogPort decision |
| Backlog | Project | docs/backlog/BL-071 | Status -> implemented |
| CHANGELOG | Project | CHANGELOG.md | Add BL-071 entry |
| Architecture | Project | docs/ARCHITECTURE.md | Note new GitLogPort in ports layer |

## Open Questions

None -- all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Scope boundaries | 4 subcommands: changelog, hotspots, coupling, bus-factor. OUT: complexity trends, JSON output, skill/agent updates | User |
| 2 | Architecture | GitLogPort trait in ecc-ports (proper hex arch) | Recommended |
| 3 | Test strategy | Everything 100% including infra adapter with temp git repos | User |
| 4 | Performance | No hard constraint, aim for fast | User |
| 5 | Security | No implications -- pure local git log parsing | Recommended |
| 6 | Breaking changes + formula | No breaking changes, coupling uses max() denominator | Recommended |
| 7 | Config + --since | Configurable thresholds, unified --since accepting tags+dates, 90-day defaults | Recommended |
| 8 | ADR decisions | ADR for GitLogPort decision | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Conventional Commit Parser (Domain) | 5 | none |
| US-002 | GitLogPort Trait and Adapter | 7 | US-001 |
| US-003 | Changelog Generation Subcommand | 7 | US-001, US-002 |
| US-004 | Hotspot Analysis Subcommand | 5 | US-002 |
| US-005 | Co-Change Coupling Subcommand | 8 | US-002 |
| US-006 | Bus Factor Subcommand | 5 | US-002 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Conventional commit parsing with scope | US-001 |
| AC-001.2 | Breaking change via ! suffix | US-001 |
| AC-001.3 | Non-conventional message -> type=other | US-001 |
| AC-001.4 | BREAKING CHANGE footer | US-001 |
| AC-001.5 | Empty/malformed input -> sensible default | US-001 |
| AC-002.1 | GitLogPort trait returns domain types | US-002 |
| AC-002.2 | Adapter runs git log with date range | US-002 |
| AC-002.3 | Adapter handles tag as --since | US-002 |
| AC-002.4 | Empty git log -> empty Vec | US-002 |
| AC-002.5 | 100% parsing path coverage | US-002 |
| AC-002.6 | Git binary not found -> clear error | US-002 |
| AC-002.7 | Invalid --since -> descriptive error | US-002 |
| AC-003.1 | Changelog sections by commit type | US-003 |
| AC-003.2 | Date-based --since filtering | US-003 |
| AC-003.3 | Default to last semver tag or 90 days | US-003 |
| AC-003.4 | Breaking changes section at top | US-003 |
| AC-003.5 | Empty sections omitted | US-003 |
| AC-003.6 | Non-conventional in "Other" section | US-003 |
| AC-003.7 | No tags fallback with header note | US-003 |
| AC-004.1 | Top N hotspots sorted descending | US-004 |
| AC-004.2 | Date-based --since filtering | US-004 |
| AC-004.3 | Default --top 10 | US-004 |
| AC-004.4 | Deleted files still counted | US-004 |
| AC-004.5 | Invalid --top -> error | US-004 |
| AC-005.1 | Coupling score computation example | US-005 |
| AC-005.2 | Threshold filtering | US-005 |
| AC-005.3 | Default threshold 0.7 | US-005 |
| AC-005.4 | Sorted by score descending | US-005 |
| AC-005.5 | Max-files-per-commit exclusion | US-005 |
| AC-005.6 | Min-commits exclusion | US-005 |
| AC-005.7 | Formula: commits_together / max(A,B) | US-005 |
| AC-005.8 | Boundary: exactly N files included | US-005 |
| AC-006.1 | Top N lowest unique author count | US-006 |
| AC-006.2 | Output: path, authors, commits | US-006 |
| AC-006.3 | --since range filtering | US-006 |
| AC-006.4 | Default --top 10 | US-006 |
| AC-006.5 | Bus factor = 1 flagged as RISK | US-006 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Ambiguity | 82 | PASS | ACs concrete with examples; minor output header vagueness |
| Edge Cases | 80 | PASS | All round 1 gaps addressed (binary not found, invalid --since, boundary, merges) |
| Scope | 88 | PASS | Non-requirements explicit, no creep vectors |
| Dependencies | 85 | PASS | Acyclic DAG, correct ordering |
| Testability | 83 | PASS | 100% coverage mandated, numeric examples in ACs |
| Decisions | 87 | PASS | 6 decisions with rationale, formula pinned |
| Rollback | 75 | PASS | All additive, no migrations |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-02-deterministic-git-analytics-cli/spec.md | Full spec + Phase Summary |
