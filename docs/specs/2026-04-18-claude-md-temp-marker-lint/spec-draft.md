# Spec: CLAUDE.md `TEMPORARY (BL-NNN)` Marker Lint

## Problem Statement

CLAUDE.md can accumulate "TEMPORARY (BL-NNN)" warning comments that reference backlog IDs. When the underlying work ships, the warning becomes stale; when the BL-NNN is never actually filed, the warning is orphaned from day 1. Both forms rot silently — no tool detects the drift. Current evidence: `CLAUDE.md:108` has said `TEMPORARY (BL-150): Do not run parallel Claude Code sessions…` for multiple sprints even though `CHANGELOG.md:35` records that BL-150's work shipped. The drift surfaced only by accident during a /backlog session. A lint rule closes this class of drift permanently.

## Research Summary

- **todocheck** (Go) is the closest prior art: validates TODO comments against real issue IDs in GitHub/GitLab/Jira, blocks CI on unresolved or missing references. The pattern ports directly to BL-NNN marker validation.
- **Softwire/todo-checker** and **todo-actions** use two-level severity: unparseable marker / missing reference / stale reference — three distinct codes recommended for maximum clarity. v1 scope: missing reference only (others deferred).
- **flake8-todos** validates TODO format first, then resolves references — reinforces regex-first approach before cross-context lookup.
- **pulldown-cmark** is the Rust ecosystem's canonical CommonMark parser and is used by rust-clippy's `doc_markdown` lint. Decision: not adopted in v1 — the existing `extract_claims` hand-rolled fence-skip in the same `claude_md.rs` file is sufficient and consistent.
- **Fiberplane's drift-documentation-linter** frames this drift class explicitly and solves it with CI-enforced anchor validation — validates the approach at scale.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | CLI shape: subcommand (`ecc validate claude-md markers`), not flag stacking | Aligns with existing `ecc validate <target>` subcommand pattern (agents/commands/skills/hooks/rules); scoped flags (`--strict`, `--audit-report`) only apply to the subcommand that supports them; `--counts` kept as deprecated alias for one release | No |
| 2 | Default severity: WARN (exit 0); `--strict` upgrades to ERROR (exit 1) | Raw request; safer day-1 adoption; can flip polarity in v2 after burn-in | No |
| 3 | File scope: both `CLAUDE.md` and `AGENTS.md` recursively from v1 | Forward-compat with ADR 0062 (AGENTS.md additive alignment); zero AGENTS.md files today so identical behavior in practice | No |
| 4 | Archived-status semantics: presence-only (archived = resolved) | File on disk = satisfied. Archived means tracked and decision was made. Simpler logic, no frontmatter parsing, zero cross-context coupling | No |
| 5 | Markdown parser: hand-rolled fence skip mirroring `extract_claims` | Zero new deps; consistent with sibling function in same file; nested-fence risk accepted (not used in ECC CLAUDE.md) | No |
| 6 | Stale BL-150 marker: remove `CLAUDE.md:108` as part of this spec | Atomically ships lint + fixes known drift; first CI run green; clean end-to-end story | No |
| 7 | CI wiring: `ecc validate claude-md markers --strict` in `ci.yml` validate job from day 1 | Maximum drift prevention; BL-150 line removed in this spec so first CI run is green | No |
| 8 | No dedicated ADR for the `docs↔backlog` ACL pattern | Architectural choice is visible in code + tests; CHANGELOG entry sufficient; ADRs reserved for decisions invisible in code | No |
| 9 | `TemporaryMarker` lives in `ecc-domain::docs` (not `backlog`) | Marker is a documentation artifact, not a backlog entity; cross-context reference via ACL | No |
| 10 | Lift `matches_backlog_filename(filename, id) -> bool` into `ecc-domain::backlog::entry` | Prevents `ecc-app → ecc-infra` dependency violation; pure predicate; refactors existing `fs_backlog::find_entry_path` to delegate | No |
| 11 | BL-NNN regex upper bound: widen to `\d{1,6}` | Current max is BL-152; `\d{1,6}` gives 6500× headroom. `\d{1,4}` silently stops matching at BL-10000 — too narrow for silent-failure risk | No |
| 12 | `--audit-report` flag is permanent CLI feature (not spec-artifact-only) | Low implementation cost (already tested); future audits may re-run; keeps the CLI surface orthogonal to any one spec's lifecycle | No |
| 13 | Kill switch: env var `ECC_CLAUDE_MD_MARKERS_DISABLED=1` short-circuits to exit 0 | `--strict` from day 1 in CI has zero rollback path otherwise; env var provides emergency brake without requiring a code revert | No |
| 14 | `--counts` alias removal: in the **next minor** release after this spec ships | Semver-anchored deprecation removes ambiguity; CI test asserts the deprecation warning fires | No |
| 15 | Archived = resolved today; frontmatter-aware v2 tracked as a new backlog entry | Closes the adversary's governance-loophole concern: anyone can silence a warning by archiving, but the counterweight is filed as future work. Presence-only stays simple for v1 | No |

## User Stories

### US-001: Detect `TEMPORARY (BL-NNN)` markers with missing backlog files

**As a** repo contributor, **I want** `ecc validate claude-md markers` to flag TEMPORARY markers pointing to non-existent BL-NNN backlog files, **so that** stale warning comments cannot linger past the backlog entry's lifecycle.

#### Acceptance Criteria

- AC-001.1: Given a CLAUDE.md containing `TEMPORARY (BL-156)`, when `docs/backlog/BL-156-*.md` exists on disk, then no diagnostic is emitted.
- AC-001.2: Given a CLAUDE.md containing `TEMPORARY (BL-999)`, when no file matches `docs/backlog/BL-999*.md`, then a diagnostic is emitted to stderr: `WARN: <path>:<line>: TEMPORARY (BL-999) references missing backlog entry. Remediation: (a) file docs/backlog/BL-999-<slug>.md or (b) remove the stale warning if work has shipped.`
- AC-001.3: Given the detection regex `(?i)TEMPORARY\s*\(BL-0*(\d{1,6})\)`, when scanning `CLAUDE.md`, then it matches `TEMPORARY (BL-150)`, `temporary (bl-150)`, `Temporary (Bl-150)`, `TEMPORARY  (BL-0150)`, `TEMPORARY (BL-100000)` (all normalized to their minimum-digit canonical form via `BL-{:03}` lookup when ≤999, else `BL-{id}` for ≥1000) and does NOT match prose like "this is temporary" or `TEMPORARY: note` (no parenthesized `BL-` capture).
- AC-001.4: Given fenced code blocks in `CLAUDE.md`, when a marker appears inside a ``` ... ``` block, then it is ignored (reuse the hand-rolled fence-skipping loop from `extract_claims`).
- AC-001.5: Given a marker `TEMPORARY (BL-137)` where BL-137 has status=archived on disk, when scanning, then no diagnostic is emitted (presence-only semantics).
- AC-001.6: Default severity: diagnostics go to stderr, process exits 0 when missing markers are the only issue.
- AC-001.7: Domain extractor returns `Vec<TemporaryMarker { backlog_id: u32, line_number: usize, raw_text: String }>`. Zero I/O in `ecc-domain`.
- AC-001.8: Duplicate markers in the same file (same BL-NNN, two lines) produce two diagnostics (one per occurrence) preserving line info.
- AC-001.9: Malformed markers like `TEMPORARY (BL-)` or `TEMPORARY (BL-ABC)` do NOT match regex, produce no diagnostic, and do not crash.
- AC-001.10: Baseline — given a CLAUDE.md with zero TEMPORARY markers, when scanning, then zero diagnostics emitted and exit code is 0 regardless of `--strict`.
- AC-001.11: Missing backlog directory — given no `docs/backlog/` directory exists in the repo, when the lint runs and any TEMPORARY marker exists, then all markers report as `missing` (directory absence = backlog empty).
- AC-001.12: Governance-loophole acknowledgement — given a marker `TEMPORARY (BL-NNN)` where the file exists with `status: archived`, then it is treated as resolved in v1 (presence-only, decision #4). A companion backlog entry will be filed in Phase 10 of this spec tracking frontmatter-aware v2 as future work.
- AC-001.13: Deprecation warning — given `ecc validate claude-md --counts` (the legacy flag form), when executed, then stderr emits `DEPRECATED: use 'ecc validate claude-md counts' (subcommand form); --counts will be removed in the next minor release.` Exit code and primary output behavior unchanged from prior release.
- AC-001.14: Within-file ordering — given a file with multiple TEMPORARY markers, when emitting diagnostics, then they appear in ascending line-number order. Combined with AC-003.5 (lexicographic cross-file order), output is fully deterministic.

#### Dependencies

- Depends on: none (foundation)

### US-002: `--strict` flag escalates warnings to errors

**As a** CI maintainer, **I want** `--strict` to turn missing-marker warnings into non-zero exit codes, **so that** CI blocks PRs that introduce or retain stale TEMPORARY markers.

#### Acceptance Criteria

- AC-002.1: Given `ecc validate claude-md markers` (no `--strict`), when any missing marker is found, then stderr emits `WARN: ...` lines and exit code is `0`.
- AC-002.2: Given `ecc validate claude-md markers --strict`, when any missing marker is found, then stderr emits `ERROR: ...` lines (prefix flip) and exit code is `1`.
- AC-002.3: Given `--strict`, when zero missing markers are found, then exit code is `0` and stdout emits `All TEMPORARY markers reference valid backlog entries`.
- AC-002.4: `--strict` flag is scoped to the `markers` subcommand only; it does NOT affect `counts`.
- AC-002.5: Default mode (no `--strict`) with zero missing markers emits **nothing** to stdout and exit code is 0. The "All TEMPORARY markers reference valid backlog entries" success message is emitted to stdout ONLY under `--strict` (from AC-002.3).

#### Dependencies

- Depends on: US-001

### US-003: Multi-file CLAUDE.md + AGENTS.md scan

**As a** the lint's consumer, **I want** the rule to walk every `CLAUDE.md` and `AGENTS.md` under the project root, **so that** stale markers in nested files (`crates/CLAUDE.md`, `examples/CLAUDE.md`) are also caught.

#### Acceptance Criteria

- AC-003.1: Given a repo with `CLAUDE.md`, `crates/CLAUDE.md`, `examples/CLAUDE.md`, when the lint runs, then all three files are scanned and per-file line numbers are reported.
- AC-003.2: Given any `AGENTS.md` files (none today; forward-compat), when the lint runs, then they are scanned identically to `CLAUDE.md`.
- AC-003.3: Given nested files at any depth, when scanning, then the walker skips `.git/`, `target/`, `node_modules/`, `.claude/worktrees/` (hardcoded deny-list; deterministic).
- AC-003.4: Given symlinks to files outside the repo, when walking, then they are NOT followed.
- AC-003.5: Scan order is deterministic (lexicographic by path) so diagnostic output is stable across runs.
- AC-003.6: Non-UTF8 or binary content — given a `*.md` file that cannot be decoded as UTF-8, when walking, then it is skipped with a single `WARN: <path>: skipping non-UTF8 file` on stderr; exit code is NOT affected (skip is advisory, not fatal).
- AC-003.7: Walker is hand-rolled on top of `FileSystem::read_dir` (no new crate dep). Max recursion depth: 64 (defense against symlink loops / pathological nesting); at depth limit, skip with `WARN: <path>: depth limit` on stderr.

#### Dependencies

- Depends on: US-001

### US-004: One-time repo audit report

**As a** the user filing this spec, **I want** a one-time audit report listing every current `TEMPORARY (BL-NNN)` marker in the repo with resolution status, **so that** we verify the known drift is closed and catch any hidden cases before merging.

#### Acceptance Criteria

- AC-004.1: Given `ecc validate claude-md markers --audit-report`, when executed against the current repo, then stdout emits a markdown table: `| File | Line | Marker ID | Status |` with `Status ∈ {missing, resolved}`. Every found marker produces **exactly one row**; Status is `resolved` when the backing file exists on disk, else `missing`.
- AC-004.2: Given the pre-fix repo state (BL-150 line still present), when the audit runs, then row `| CLAUDE.md | 108 | BL-150 | missing |` appears. This is the regression anchor.
- AC-004.3: Given the post-fix repo state (BL-150 line removed), when the audit runs, then no `missing` rows are present. This is the exit criterion for this spec.
- AC-004.4: The audit output is committed as `docs/specs/2026-04-18-claude-md-temp-marker-lint/audit-report.md` alongside the spec.

#### Dependencies

- Depends on: US-001, US-003

### US-005: Remove stale `CLAUDE.md:108 TEMPORARY (BL-150)` line

**As a** the maintainer, **I want** the stale BL-150 warning removed from CLAUDE.md, **so that** CI turns green on first merge and the known drift is closed atomically with the lint rule.

#### Acceptance Criteria

- AC-005.1: Given `CLAUDE.md` after this spec ships, when grepping for `TEMPORARY (BL-`, then zero matches are returned in `CLAUDE.md`.
- AC-005.2: Given `ecc validate claude-md markers --strict` run against the post-fix repo, when executed, then exit code is `0` and stdout emits "All TEMPORARY markers reference valid backlog entries".
- AC-005.3: CHANGELOG.md receives an entry under the current release documenting the lint rule + BL-150 line removal.

#### Dependencies

- Depends on: US-001, US-002 (need --strict to validate)

### US-006: CI wiring

**As a** a repo contributor, **I want** `ecc validate claude-md markers --strict` run in CI, **so that** PRs introducing new stale markers are automatically blocked.

#### Acceptance Criteria

- AC-006.1: Given `.github/workflows/ci.yml`, when reading the `validate` job, then a step runs `ecc validate claude-md markers --strict` after the existing `ecc validate claude-md --counts` step.
- AC-006.2: Given a PR that introduces a new `TEMPORARY (BL-999)` marker without filing BL-999, when the CI run executes, then the `validate` job fails with exit code 1 and the error message pinpoints the file, line, and ID.
- AC-006.3: Given the post-fix repo at main HEAD, when CI runs, then the new step passes (exit 0).
- AC-006.4: Rollback path — the ci.yml addition is a single atomic commit scoped to one added step in the `validate` job. Rollback = `git revert <ci-commit>` on main, single PR, no dependencies.
- AC-006.5: Kill switch — given `ECC_CLAUDE_MD_MARKERS_DISABLED=1` in the environment, when `ecc validate claude-md markers` runs, then it short-circuits to exit 0 with `stderr: ecc validate claude-md markers: disabled via ECC_CLAUDE_MD_MARKERS_DISABLED`. Allows emergency CI bypass without requiring a code revert. Env var is undocumented in README but documented in `docs/commands-reference.md` with a "do not rely on" warning.
- AC-006.6: Refactor-regression guard — given the existing `ecc-integration-tests` for `ecc backlog next-id`, `ecc backlog list`, and `ecc backlog update-status`, when running against the refactored `fs_backlog::find_entry_path` (decision #10), then all prior assertions hold unchanged (test diff = zero).

#### Dependencies

- Depends on: US-001, US-002, US-005

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `ecc-domain::docs::claude_md` | domain | Add `TemporaryMarker` VO + `extract_temporary_markers(content: &str) -> Vec<TemporaryMarker>`; mirrors `CountClaim` / `extract_claims` pattern; zero I/O. |
| `ecc-domain::backlog::entry` | domain | Add pure predicate `matches_backlog_filename(filename: &str, id: u32) -> bool` (normalizes `BL-{:03}` matching). |
| `ecc-app::validate_claude_md` | app | New use case `run_validate_temporary_markers(fs, terminal, project_root, strict: bool, audit_report: bool) -> bool`; walks file tree, reads markers, probes `docs/backlog/` via `FileSystem::read_dir`, emits diagnostics. |
| `ecc-infra::fs_backlog` | adapter | Refactor existing `find_entry_path` to delegate to new domain predicate (no behavior change). |
| `ecc-cli::commands::validate` | CLI | Restructure `ClaudeMd` variant from `{ counts: bool }` to subcommand enum `{ Counts, Markers { strict, audit_report }, All { strict, audit_report } }`; `--counts` kept as deprecated alias with stderr warning. |
| `ecc-integration-tests` | test harness | New tests: `validate_claude_md_markers.rs` (InMemoryFileSystem fixtures), `validate_claude_md_repo_audit.rs` (real-repo anchor). |
| `.github/workflows/ci.yml` | CI | Add `ecc validate claude-md markers --strict` to `validate` job after `--counts` step. |
| `CLAUDE.md` | docs | Remove line 108 stale TEMPORARY marker. |
| `CHANGELOG.md` | docs | Add entry under current release. |
| `docs/commands-reference.md` | docs | Document new subcommand and `--strict` / `--audit-report` flags. |

## Constraints

- **No new crate deps**: hand-rolled regex + fence-skip sufficient (decision #5).
- **Hexagonal compliance**: `ecc-domain` remains zero-I/O; no `ecc-app → ecc-infra` edge (decision #10).
- **Backward compat**: `ecc validate claude-md --counts` must work for one release with deprecation warning on stderr.
- **First CI run must pass**: BL-150 line removed in this spec ensures green day-1.
- **Zero frontmatter parsing**: presence-only semantics simplify the validator (decision #4).
- **Audit findings from full-2026-04-18.md**: no CRITICAL/HIGH blockers relevant to this feature area.
- **Companion backlog entry requirement** (governance loophole counterweight): before this spec's work merges, file a new BL entry titled "Frontmatter-aware TEMPORARY marker validation (v2)" capturing the decision-#4 loophole (archived=resolved allows silencing warnings by archiving without doing work). The companion entry is a tracked follow-up — NOT a blocker for v1 shipping.
- **Rollback contract**: the CI wiring (US-006) is a single atomic commit; kill switch env var is the emergency brake. Documented in AC-006.4 and AC-006.5.

## Non-Requirements (OUT of scope for v1)

- Orphaned-entry inverse lint (BL-NNN filed but no code references anywhere) — separate future BL.
- Arbitrary user-provided markdown paths — v1 scans only repo-local CLAUDE.md/AGENTS.md.
- `--fix` auto-removal of stale lines — requires human judgment; manual fix only in v1.
- doc-validator / docs-suite pipeline integration — standalone subcommand for v1.
- GitHub Actions annotation output format (e.g., `::warning file=...::`) — stderr diagnostics only in v1.
- Frontmatter status checks (`status: archived` vs `status: implemented`) — presence-only in v1.
- `pulldown-cmark` adoption — hand-rolled fence-skip in v1.
- AGENTS.md-specific marker vocabulary — AGENTS.md treated identically to CLAUDE.md.

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| `FileSystem::read_dir` | reuse | No change to port; new consumer in app layer. Walks repo for markdown files + `docs/backlog/` for presence probe. |
| `TerminalIO::stderr_write` / `stdout_write` | reuse | New diagnostic strings emitted; integration test asserts exact format. |
| CLI → app use case (`ecc validate claude-md markers`) | new binding | Integration test required: real binary + tempdir fixture. |
| CI job (`validate`) | new step | One added line in `ci.yml` runs the subcommand with `--strict`. Existing ≥80% coverage gate and clippy gate unaffected. |
| App → domain (`extract_temporary_markers`) | new pure fn | Unit-testable in `ecc-domain` with string inputs. |
| App → domain (`matches_backlog_filename`) | new pure fn | Unit-testable in `ecc-domain`. Also used by refactored `ecc-infra::fs_backlog::find_entry_path` — behavior-preserving refactor must not break existing `ecc backlog` surface. |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New CLI subcommand | upper-level | `docs/commands-reference.md` | Add row under validate section: `ecc validate claude-md markers [--strict] [--audit-report]`. |
| New CLI subcommand | upper-level | `CLAUDE.md` (CLI Commands top 10) | Update `ecc validate <target>` line to mention new `markers` subcommand. |
| Stale-marker removal | upper-level | `CLAUDE.md` | Delete line 108 entirely (no replacement). |
| Deprecation | upper-level | `CLAUDE.md` + `docs/commands-reference.md` | Note: `ecc validate claude-md --counts` deprecated in favor of `ecc validate claude-md counts`; remove in next minor. |
| New CI step | upper-level | `rules/ecc/github-actions.md` | Update the `ci.yml` validate-job description to include the markers step. |
| Changelog | upper-level | `CHANGELOG.md` | Add entry under current unreleased / current release. |
| Audit deliverable | spec-artifact | `docs/specs/2026-04-18-claude-md-temp-marker-lint/audit-report.md` | New file: markdown table of current markers and resolution status (before and after BL-150 line removal). |
| Spec file | spec-artifact | `docs/specs/2026-04-18-claude-md-temp-marker-lint/spec.md` | This spec, persisted after adversarial PASS. |

## Open Questions

(None — all resolved in grill-me interview.)
