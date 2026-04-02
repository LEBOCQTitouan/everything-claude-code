# BL-071 Design: Deterministic Git Analytics CLI

## Overview

Add `ecc analyze` subcommand with four variants (changelog, hotspots, coupling, bus-factor) powered by pure domain logic and a `GitLogPort` trait. All git I/O is isolated in a single infra adapter; domain functions are pure string-parsing and counting.

## Architecture Decision

**ADR-0037: GitLogPort for git log abstraction** -- New port trait rather than reusing `ShellExecutor` directly in app layer. Rationale: app use cases should not construct git CLI arguments; the adapter owns all git command construction and output format selection. This keeps domain + app layers testable without mocking shell commands.

---

## File Changes

| # | Action | File | Rationale | Spec Ref |
|---|--------|------|-----------|----------|
| 1 | CREATE | `crates/ecc-domain/src/analyze/mod.rs` | Module root for analyze domain | US-001..006 |
| 2 | CREATE | `crates/ecc-domain/src/analyze/commit.rs` | `ConventionalCommit` VO, `CommitType` enum, `parse_conventional_commit()` | US-001 |
| 3 | CREATE | `crates/ecc-domain/src/analyze/changelog.rs` | `group_by_type()`, `format_changelog()` pure fns | US-003 |
| 4 | CREATE | `crates/ecc-domain/src/analyze/hotspot.rs` | `Hotspot` VO, `compute_hotspots()` | US-004 |
| 5 | CREATE | `crates/ecc-domain/src/analyze/coupling.rs` | `CouplingPair` VO, `compute_coupling()` with configurable thresholds | US-005 |
| 6 | CREATE | `crates/ecc-domain/src/analyze/bus_factor.rs` | `BusFactor` VO, `compute_bus_factor()` | US-006 |
| 7 | CREATE | `crates/ecc-domain/src/analyze/error.rs` | `AnalyzeError` enum (domain errors only) | US-001..006 |
| 8 | MODIFY | `crates/ecc-domain/src/lib.rs` | Add `pub mod analyze;` | -- |
| 9 | CREATE | `crates/ecc-ports/src/git_log.rs` | `GitLogPort` trait definition | US-002 |
| 10 | MODIFY | `crates/ecc-ports/src/lib.rs` | Add `pub mod git_log;` | -- |
| 11 | CREATE | `crates/ecc-infra/src/git_log_adapter.rs` | `GitLogAdapter` implementing `GitLogPort` via `ShellExecutor` | US-002 |
| 12 | MODIFY | `crates/ecc-infra/src/lib.rs` | Add `pub mod git_log_adapter;` | -- |
| 13 | CREATE | `crates/ecc-app/src/analyze.rs` | Use case functions: `changelog()`, `hotspots()`, `coupling()`, `bus_factor()` | US-003..006 |
| 14 | MODIFY | `crates/ecc-app/src/lib.rs` | Add `pub mod analyze;` | -- |
| 15 | CREATE | `crates/ecc-cli/src/commands/analyze.rs` | `AnalyzeArgs` + `AnalyzeAction` enum, `run()` | US-003..006 |
| 16 | MODIFY | `crates/ecc-cli/src/commands/mod.rs` | Add `pub mod analyze;` | -- |
| 17 | MODIFY | `crates/ecc-cli/src/main.rs` | Add `Analyze(commands::analyze::AnalyzeArgs)` variant | -- |
| 18 | CREATE | `crates/ecc-test-support/src/mock_git_log.rs` | `MockGitLog` implementing `GitLogPort` for tests | US-002 |
| 19 | MODIFY | `crates/ecc-test-support/src/lib.rs` | Add `pub mod mock_git_log;` + re-export | -- |
| 20 | CREATE | `docs/adr/0037-git-log-port.md` | ADR for GitLogPort decision | US-002 |

---

## Domain Types

### `crates/ecc-domain/src/analyze/commit.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitType {
    Feat, Fix, Refactor, Docs, Test, Chore, Perf, Ci, Style, Build, Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConventionalCommit {
    pub commit_type: CommitType,
    pub scope: Option<String>,
    pub breaking: bool,
    pub description: String,
    pub hash: String,
    pub author: String,
}

/// Parse a single commit message line into a ConventionalCommit.
/// Input format: "<hash> <author> <message>"
/// Returns None if the message doesn't match conventional commit format.
pub fn parse_conventional_commit(hash: &str, author: &str, message: &str) -> Option<ConventionalCommit>
```

### `crates/ecc-domain/src/analyze/hotspot.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hotspot {
    pub path: String,
    pub change_count: u32,
}

/// Count file change frequency from a list of (commit_files) groups.
/// Filters commits with > max_files_per_commit files.
/// Returns sorted by change_count descending, limited to top_n.
pub fn compute_hotspots(
    commit_files: &[Vec<String>],
    top_n: usize,
    max_files_per_commit: usize,
) -> Vec<Hotspot>
```

### `crates/ecc-domain/src/analyze/coupling.rs`

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct CouplingPair {
    pub file_a: String,
    pub file_b: String,
    pub coupling_ratio: f64,
    pub commits_together: u32,
}

/// Compute coupling pairs from commit file groups.
/// Formula: commits_together / max(commits_A, commits_B)
/// Filters: >= min_commits individual appearances, >= threshold coupling ratio.
/// Excludes commits with > max_files_per_commit files.
pub fn compute_coupling(
    commit_files: &[Vec<String>],
    threshold: f64,
    min_commits: u32,
    max_files_per_commit: usize,
) -> Vec<CouplingPair>
```

### `crates/ecc-domain/src/analyze/bus_factor.rs`

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct BusFactor {
    pub path: String,
    pub bus_factor: u32,
    pub top_authors: Vec<AuthorContribution>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthorContribution {
    pub author: String,
    pub commit_count: u32,
    pub percentage: f64,
}

/// Compute bus factor per file.
/// Bus factor = minimum authors whose combined commits exceed 50% of total.
/// Input: list of (file_path, author) tuples from git log.
pub fn compute_bus_factor(
    file_authors: &[(String, String)],
    top_n: usize,
) -> Vec<BusFactor>
```

---

## Port Trait

### `crates/ecc-ports/src/git_log.rs`

```rust
use std::path::Path;

/// Errors from git log operations.
#[derive(Debug, thiserror::Error)]
pub enum GitLogError {
    #[error("git not found on PATH")]
    GitNotFound,
    #[error("not a git repository: {0}")]
    NotARepo(String),
    #[error("invalid --since value: {0}")]
    InvalidSince(String),
    #[error("git command failed: {0}")]
    CommandFailed(String),
}

/// Raw commit record returned by the port.
#[derive(Debug, Clone)]
pub struct RawCommit {
    pub hash: String,
    pub author: String,
    pub message: String,
    pub files: Vec<String>,
}

/// Port for reading git log data.
pub trait GitLogPort: Send + Sync {
    /// Fetch commits with their changed files.
    /// `since` accepts git-compatible values: tags (v1.0.0), dates (2024-01-01), relative (90.days.ago).
    /// Always uses --no-merges.
    fn log_with_files(
        &self,
        repo_dir: &Path,
        since: Option<&str>,
    ) -> Result<Vec<RawCommit>, GitLogError>;

    /// Fetch (file_path, author) tuples for bus factor analysis.
    /// Always uses --no-merges.
    fn log_file_authors(
        &self,
        repo_dir: &Path,
        since: Option<&str>,
    ) -> Result<Vec<(String, String)>, GitLogError>;
}
```

---

## Infra Adapter

### `crates/ecc-infra/src/git_log_adapter.rs`

`GitLogAdapter` wraps a `&dyn ShellExecutor`. It:
1. Checks `command_exists("git")` -> `GitLogError::GitNotFound`
2. Constructs `git log` with `--no-merges --format=<format> --name-only` (or `--numstat` variant for authors)
3. Parses multi-line output (commit blocks separated by blank lines)
4. Validates `--since` by running a probe `git log --since=<value> -1` first; non-zero exit -> `GitLogError::InvalidSince`
5. Detects "not a git repository" in stderr -> `GitLogError::NotARepo`

```rust
pub struct GitLogAdapter<'a> {
    executor: &'a dyn ShellExecutor,
}
```

---

## App Use Cases

### `crates/ecc-app/src/analyze.rs`

Thin orchestration -- each function takes a `&dyn GitLogPort` + config params, calls the port, then calls domain logic.

```rust
pub fn changelog(port: &dyn GitLogPort, repo: &Path, since: Option<&str>) -> Result<String, AnalyzeAppError>
pub fn hotspots(port: &dyn GitLogPort, repo: &Path, since: Option<&str>, top_n: usize, max_files: usize) -> Result<Vec<Hotspot>, AnalyzeAppError>
pub fn coupling(port: &dyn GitLogPort, repo: &Path, since: Option<&str>, threshold: f64, min_commits: u32, max_files: usize) -> Result<Vec<CouplingPair>, AnalyzeAppError>
pub fn bus_factor(port: &dyn GitLogPort, repo: &Path, since: Option<&str>, top_n: usize) -> Result<Vec<BusFactor>, AnalyzeAppError>
```

`AnalyzeAppError` wraps `GitLogError` and `AnalyzeError`.

---

## CLI Subcommand

### `crates/ecc-cli/src/commands/analyze.rs`

```rust
#[derive(Args)]
pub struct AnalyzeArgs {
    #[command(subcommand)]
    pub action: AnalyzeAction,
}

#[derive(Subcommand)]
pub enum AnalyzeAction {
    /// Generate a changelog from conventional commits
    Changelog {
        #[arg(long)]
        since: Option<String>,
    },
    /// Show most frequently changed files
    Hotspots {
        #[arg(long, default_value = "10")]
        top: usize,
        #[arg(long, default_value = "90.days.ago")]
        since: Option<String>,
        #[arg(long, default_value = "20")]
        max_files_per_commit: usize,
    },
    /// Show files that change together
    Coupling {
        #[arg(long, default_value = "0.7")]
        threshold: f64,
        #[arg(long, default_value = "3")]
        min_commits: u32,
        #[arg(long, default_value = "90.days.ago")]
        since: Option<String>,
        #[arg(long, default_value = "20")]
        max_files_per_commit: usize,
    },
    /// Show files with low contributor diversity
    BusFactor {
        #[arg(long, default_value = "10")]
        top: usize,
        #[arg(long, default_value = "90.days.ago")]
        since: Option<String>,
    },
}
```

---

## Pass Conditions

| PC | Type | Description | AC Refs | Command | Expected |
|----|------|-------------|---------|---------|----------|
| PC-001 | unit | Parse "feat(cli): add analyze" into ConventionalCommit | AC-1.1 | `cargo test -p ecc-domain parse_feat_with_scope` | CommitType::Feat, scope=Some("cli") |
| PC-002 | unit | Parse "fix!: critical bug" as breaking | AC-1.2 | `cargo test -p ecc-domain parse_breaking_fix` | breaking=true |
| PC-003 | unit | Parse "updated readme" returns None (non-conventional) | AC-1.3 | `cargo test -p ecc-domain parse_non_conventional_returns_none` | None |
| PC-004 | unit | Parse all standard types (feat,fix,refactor,docs,test,chore,perf,ci,style,build) | AC-1.4 | `cargo test -p ecc-domain parse_all_commit_types` | correct CommitType variant each |
| PC-005 | unit | Unknown type "wip: stuff" -> CommitType::Unknown("wip") | AC-1.5 | `cargo test -p ecc-domain parse_unknown_type` | Unknown("wip".into()) |
| PC-006 | unit | MockGitLog returns canned RawCommits | AC-2.1 | `cargo test -p ecc-test-support mock_git_log_returns_commits` | Vec<RawCommit> matches |
| PC-007 | unit | GitLogAdapter detects git not found | AC-2.2 | `cargo test -p ecc-infra git_not_found_error` | GitLogError::GitNotFound |
| PC-008 | unit | GitLogAdapter detects not-a-repo | AC-2.3 | `cargo test -p ecc-infra not_a_repo_error` | GitLogError::NotARepo |
| PC-009 | unit | GitLogAdapter rejects invalid --since | AC-2.4 | `cargo test -p ecc-infra invalid_since_error` | GitLogError::InvalidSince |
| PC-010 | unit | GitLogAdapter parses multi-commit output | AC-2.5 | `cargo test -p ecc-infra parses_git_log_output` | correct Vec<RawCommit> |
| PC-011 | unit | GitLogAdapter always uses --no-merges | AC-2.6 | `cargo test -p ecc-infra uses_no_merges_flag` | "--no-merges" in args |
| PC-012 | unit | GitLogAdapter accepts tag as --since | AC-2.7 | `cargo test -p ecc-infra since_accepts_tag` | no error, "--since=v1.0.0" in args |
| PC-013 | unit | group_by_type groups commits correctly | AC-3.1 | `cargo test -p ecc-domain groups_commits_by_type` | HashMap with correct grouping |
| PC-014 | unit | format_changelog produces markdown with type headers | AC-3.2 | `cargo test -p ecc-domain format_changelog_markdown` | contains "## Features", "## Bug Fixes" |
| PC-015 | unit | Changelog with no commits returns empty section | AC-3.3 | `cargo test -p ecc-domain changelog_empty_input` | empty string or "No commits found" |
| PC-016 | unit | Changelog includes commit hash and description | AC-3.4 | `cargo test -p ecc-domain changelog_includes_hash` | each line has abbreviated hash |
| PC-017 | unit | Changelog scoped commits show scope | AC-3.5 | `cargo test -p ecc-domain changelog_shows_scope` | "**cli**: add analyze" |
| PC-018 | unit | Breaking changes get separate section | AC-3.6 | `cargo test -p ecc-domain changelog_breaking_section` | "## BREAKING CHANGES" section |
| PC-019 | unit | App changelog use case wires port to domain | AC-3.7 | `cargo test -p ecc-app changelog_use_case` | returns formatted markdown |
| PC-020 | unit | compute_hotspots counts correctly | AC-4.1 | `cargo test -p ecc-domain hotspot_counts` | path "a.rs" count=3 |
| PC-021 | unit | compute_hotspots sorts descending | AC-4.2 | `cargo test -p ecc-domain hotspots_sorted_descending` | first item has highest count |
| PC-022 | unit | compute_hotspots respects top_n | AC-4.3 | `cargo test -p ecc-domain hotspots_top_n_limit` | result.len() <= top_n |
| PC-023 | unit | compute_hotspots filters large commits | AC-4.4 | `cargo test -p ecc-domain hotspots_filters_large_commits` | commit with 25 files excluded |
| PC-024 | unit | App hotspots use case wires correctly | AC-4.5 | `cargo test -p ecc-app hotspots_use_case` | returns Vec<Hotspot> |
| PC-025 | unit | compute_coupling basic pair | AC-5.1 | `cargo test -p ecc-domain coupling_basic_pair` | a.rs+b.rs ratio=1.0 |
| PC-026 | unit | compute_coupling formula correct | AC-5.2 | `cargo test -p ecc-domain coupling_formula` | commits_together / max(commits_A, commits_B) |
| PC-027 | unit | compute_coupling filters by threshold | AC-5.3 | `cargo test -p ecc-domain coupling_threshold_filter` | pairs below 0.7 excluded |
| PC-028 | unit | compute_coupling filters by min_commits | AC-5.4 | `cargo test -p ecc-domain coupling_min_commits_filter` | files with <3 commits excluded |
| PC-029 | unit | compute_coupling filters large commits | AC-5.5 | `cargo test -p ecc-domain coupling_filters_large_commits` | commit with 25 files excluded |
| PC-030 | unit | compute_coupling sorts by ratio desc | AC-5.6 | `cargo test -p ecc-domain coupling_sorted_descending` | highest ratio first |
| PC-031 | unit | compute_coupling no self-pairs | AC-5.7 | `cargo test -p ecc-domain coupling_no_self_pairs` | file_a != file_b always |
| PC-032 | unit | App coupling use case wires correctly | AC-5.8 | `cargo test -p ecc-app coupling_use_case` | returns Vec<CouplingPair> |
| PC-033 | unit | compute_bus_factor single author = 1 | AC-6.1 | `cargo test -p ecc-domain bus_factor_single_author` | bus_factor=1 |
| PC-034 | unit | compute_bus_factor two equal authors = 2 | AC-6.2 | `cargo test -p ecc-domain bus_factor_two_equal` | bus_factor=2 |
| PC-035 | unit | compute_bus_factor dominant author = 1 | AC-6.3 | `cargo test -p ecc-domain bus_factor_dominant_author` | bus_factor=1, percentage > 50% |
| PC-036 | unit | compute_bus_factor top_n limits output | AC-6.4 | `cargo test -p ecc-domain bus_factor_top_n` | result.len() <= top_n |
| PC-037 | unit | App bus_factor use case wires correctly | AC-6.5 | `cargo test -p ecc-app bus_factor_use_case` | returns Vec<BusFactor> |
| PC-038 | integration | GitLogAdapter parses real git repo | AC-2.5 | `cargo test -p ecc-integration-tests git_log_adapter_real_repo -- --ignored` | parses commits from temp repo |
| PC-039 | build | Workspace compiles cleanly | -- | `cargo build --workspace` | exit 0 |
| PC-040 | lint | Zero clippy warnings | -- | `cargo clippy --workspace -- -D warnings` | exit 0 |
| PC-041 | lint | rustfmt check passes | -- | `cargo fmt --all -- --check` | exit 0 |

---

## TDD Order (dependency-first)

### Phase 1: Domain -- Commit Parser
**Layers: [Entity]**
**Files:** `crates/ecc-domain/src/analyze/mod.rs`, `crates/ecc-domain/src/analyze/error.rs`, `crates/ecc-domain/src/analyze/commit.rs`
**PCs:** PC-001 through PC-005
**Commits:**
1. `test: add conventional commit parser tests (PC-001..005)`
2. `feat: implement ConventionalCommit parser`
3. `refactor: improve commit parser` (if applicable)

### Phase 2: Domain -- Changelog Formatting
**Layers: [Entity]**
**Files:** `crates/ecc-domain/src/analyze/changelog.rs`
**PCs:** PC-013 through PC-018
**Commits:**
1. `test: add changelog formatting tests (PC-013..018)`
2. `feat: implement changelog grouping and formatting`
3. `refactor: improve changelog formatting` (if applicable)

### Phase 3: Domain -- Hotspot Computation
**Layers: [Entity]**
**Files:** `crates/ecc-domain/src/analyze/hotspot.rs`
**PCs:** PC-020 through PC-023
**Commits:**
1. `test: add hotspot computation tests (PC-020..023)`
2. `feat: implement compute_hotspots`

### Phase 4: Domain -- Coupling Computation
**Layers: [Entity]**
**Files:** `crates/ecc-domain/src/analyze/coupling.rs`
**PCs:** PC-025 through PC-031
**Commits:**
1. `test: add coupling computation tests (PC-025..031)`
2. `feat: implement compute_coupling`

### Phase 5: Domain -- Bus Factor Computation
**Layers: [Entity]**
**Files:** `crates/ecc-domain/src/analyze/bus_factor.rs`
**PCs:** PC-033 through PC-036
**Commits:**
1. `test: add bus factor computation tests (PC-033..036)`
2. `feat: implement compute_bus_factor`

### Phase 6: Port -- GitLogPort Trait
**Layers: [UseCase]**
**Files:** `crates/ecc-ports/src/git_log.rs`
**PCs:** PC-006 (mock compiles and returns data)
**Commits:**
1. `feat: add GitLogPort trait and GitLogError`

### Phase 7: Test Support -- MockGitLog
**Layers: [Adapter]**
**Files:** `crates/ecc-test-support/src/mock_git_log.rs`
**PCs:** PC-006
**Commits:**
1. `test: add MockGitLog test double`
2. `feat: implement MockGitLog`

### Phase 8: Infra -- GitLogAdapter
**Layers: [Framework]**
**Files:** `crates/ecc-infra/src/git_log_adapter.rs`
**PCs:** PC-007 through PC-012
**Commits:**
1. `test: add GitLogAdapter unit tests (PC-007..012)`
2. `feat: implement GitLogAdapter`

### Phase 9: App -- Use Cases
**Layers: [UseCase]**
**Files:** `crates/ecc-app/src/analyze.rs`
**PCs:** PC-019, PC-024, PC-032, PC-037
**Commits:**
1. `test: add analyze use case tests (PC-019,024,032,037)`
2. `feat: implement analyze use cases`

### Phase 10: CLI -- Analyze Subcommand
**Layers: [Adapter]**
**Files:** `crates/ecc-cli/src/commands/analyze.rs`, `crates/ecc-cli/src/commands/mod.rs`, `crates/ecc-cli/src/main.rs`
**PCs:** PC-039, PC-040, PC-041
**Commits:**
1. `feat: add ecc analyze CLI subcommand`

### Phase 11: Integration Test
**Layers: [Framework]**
**Files:** `crates/ecc-integration-tests/tests/git_log_adapter.rs`
**PCs:** PC-038
**Commits:**
1. `test: add GitLogAdapter integration test with temp git repo`

### Phase 12: ADR
**Layers: [--]**
**Files:** `docs/adr/0037-git-log-port.md`
**Commits:**
1. `docs: add ADR-0037 GitLogPort decision`

---

## E2E Assessment

- **Touches user-facing flows?** Yes -- new CLI subcommand
- **Crosses 3+ modules end-to-end?** Yes -- CLI -> App -> Port -> Infra -> git
- **New E2E tests needed?** No -- the integration test (PC-038) with a real temp git repo covers the full adapter stack. The CLI wiring is trivial (pattern already proven by 15+ existing subcommands). Run existing CI suite as gate.

---

## Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| Git log output format varies across git versions | Medium | Pin `--format` fields explicitly; integration test runs against actual git |
| Large repos produce huge git log output | Low | --since defaults to 90 days; --max-files-per-commit filters noise |
| Coupling computation is O(n^2) per commit | Low | --max-files-per-commit=20 caps pair generation; 90-day window limits commit count |
| ADR number 0037 already taken by parallel work | Low | Check before commit; renumber if needed |

---

## Success Criteria

- [ ] All 53 PCs pass (41 original + 12 from adversarial round 2)
- [ ] `cargo build --workspace` succeeds
- [ ] `cargo clippy --workspace -- -D warnings` has zero warnings
- [ ] `cargo fmt --all -- --check` passes
- [ ] ecc-domain/src/analyze/ has zero I/O imports
- [ ] Domain functions are pure (take typed input, return typed output)
- [ ] ADR-0037 documents the GitLogPort decision

---

## Adversarial Round 2 Additions

### Bus Factor Fix

Bus factor definition aligned to spec: `bus_factor = count(distinct authors)` per file (simple unique author count), NOT the "50% threshold" model. AC-006.1 says "lowest unique author count" — design must match.

### Additional Pass Conditions (PC-042..053)

| PC | Type | Description | AC Refs | Command | Expected |
|----|------|-------------|---------|---------|----------|
| PC-042 | unit | BREAKING CHANGE footer detection | AC-001.4 | `cargo test -p ecc-domain parse_breaking_change_footer` | breaking=true |
| PC-043 | unit | Empty git log returns empty Vec | AC-002.4 | `cargo test -p ecc-domain empty_log_returns_empty` | Vec empty |
| PC-044 | unit | Changelog "Other" section for non-conventional | AC-003.6 | `cargo test -p ecc-domain changelog_other_section` | contains "## Other" |
| PC-045 | unit | Changelog 90-day fallback with header note | AC-003.7 | `cargo test -p ecc-domain changelog_fallback_header` | contains "last 90 days" |
| PC-046 | unit | Hotspots include deleted files | AC-004.4 | `cargo test -p ecc-domain hotspots_include_deleted` | deleted file counted |
| PC-047 | unit | Hotspots --top 0 returns error | AC-004.5 | `cargo test -p ecc-domain hotspots_top_zero_error` | returns Err |
| PC-048 | unit | Coupling boundary: exactly N files included | AC-005.8 | `cargo test -p ecc-domain coupling_exactly_max_files` | commit included |
| PC-049 | unit | Bus factor=1 flagged as RISK | AC-006.5 | `cargo test -p ecc-domain bus_factor_risk_flag` | contains "RISK" |
| PC-050 | unit | --no-merges in log_file_authors | AC-002.6 | `cargo test -p ecc-infra file_authors_uses_no_merges` | "--no-merges" in args |
| PC-051 | unit | Coupling threshold=0.0 includes all | AC-005.2 | `cargo test -p ecc-domain coupling_threshold_zero` | all pairs included |
| PC-052 | unit | Coupling exactly min_commits boundary | AC-005.6 | `cargo test -p ecc-domain coupling_exactly_min_commits` | pair included |
| PC-053 | unit | parse_conventional_commit None handling | AC-001.3 | `cargo test -p ecc-domain parse_returns_none_non_conventional` | returns None |

---

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS (1 LOW: ISP split suggestion) | 1 |
| Robert | PASS | 0 |
| Security | Low risk (validate --since format) | 1 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Coverage | 88 | PASS | All 37 ACs mapped to 53 PCs |
| Order | 85 | PASS | Dependency chain preserved |
| Fragility | 82 | PASS | Boundary PCs avoid off-by-one |
| Rollback | 87 | PASS | All additive, no migrations |
| Architecture | 84 | PASS | Bus factor aligned, hex boundaries clean |
| Blast radius | 86 | PASS | Only pub mod additions to existing files |
| Missing PCs | 83 | PASS | Boundary and negative cases covered |
| Doc plan | 80 | PASS | ADR, CLAUDE.md, ARCHITECTURE.md, CHANGELOG planned |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | crates/ecc-domain/src/analyze/mod.rs | Create | US-001..006 |
| 2 | crates/ecc-domain/src/analyze/commit.rs | Create | US-001 |
| 3 | crates/ecc-domain/src/analyze/changelog.rs | Create | US-003 |
| 4 | crates/ecc-domain/src/analyze/hotspot.rs | Create | US-004 |
| 5 | crates/ecc-domain/src/analyze/coupling.rs | Create | US-005 |
| 6 | crates/ecc-domain/src/analyze/bus_factor.rs | Create | US-006 |
| 7 | crates/ecc-domain/src/analyze/error.rs | Create | US-001..006 |
| 8 | crates/ecc-domain/src/lib.rs | Modify | -- |
| 9 | crates/ecc-ports/src/git_log.rs | Create | US-002 |
| 10 | crates/ecc-ports/src/lib.rs | Modify | -- |
| 11 | crates/ecc-infra/src/git_log_adapter.rs | Create | US-002 |
| 12 | crates/ecc-infra/src/lib.rs | Modify | -- |
| 13 | crates/ecc-app/src/analyze.rs | Create | US-003..006 |
| 14 | crates/ecc-app/src/lib.rs | Modify | -- |
| 15 | crates/ecc-cli/src/commands/analyze.rs | Create | US-003..006 |
| 16 | crates/ecc-cli/src/commands/mod.rs | Modify | -- |
| 17 | crates/ecc-cli/src/main.rs | Modify | -- |
| 18 | crates/ecc-test-support/src/mock_git_log.rs | Create | US-002 |
| 19 | crates/ecc-test-support/src/lib.rs | Modify | -- |
| 20 | docs/adr/0037-git-log-port.md | Create | Decision #1 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-02-deterministic-git-analytics-cli/spec.md | Full spec + Phase Summary |
| docs/specs/2026-04-02-deterministic-git-analytics-cli/design.md | Full design + Phase Summary |
