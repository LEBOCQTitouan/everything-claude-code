# Spec: Fix backlog module — UTF-8 bug, error types, code quality

## Problem Statement

The BL-066 backlog module has a correctness bug (Levenshtein byte-vs-char) and 6 code quality issues flagged by `/verify`. The UTF-8 bug produces incorrect edit distances for multi-byte characters (em-dash "—" is 3 bytes but 1 char, but `levenshtein_distance` uses `as_bytes()` treating it as 3 units). The `String` error types violate the project's `thiserror` convention established by `FsError`. Magic numbers, duplicated loops, non-normalized scores, orphaned temp files, and stringly-typed status compound maintenance risk.

## Research Summary

Web research skipped: findings are code-level, not pattern-research territory. Root causes fully identified by `/verify` code-reviewer and rust-reviewer agents.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Use `.chars().collect::<Vec<_>>()` for Levenshtein | Correct for all Unicode; negligible perf cost on short titles | No |
| 2 | Create `BacklogError` thiserror enum in domain | Matches `FsError` pattern in ecc-ports; enables variant matching | No |
| 3 | Extract scoring constants as named `const` | Self-documenting, tunable without hunting through code | No |
| 4 | Extract `load_entries()` shared helper | DRY; both `check_duplicates` and `reindex` use identical loops | No |
| 5 | Normalize composite score to [0.0, 1.0] | Divide by max possible (1.0 + TAG_BOOST_CAP) for intuitive range | No |
| 6 | Add `BacklogStatus` enum with `Unknown(String)` fallback | Type safety for known statuses, extensible for unknown ones | No |

## User Stories

### US-001: Fix Levenshtein UTF-8 correctness bug

**As a** backlog-curator agent, **I want** `levenshtein_distance` to operate on Unicode chars not bytes, **so that** duplicate detection produces correct scores for titles with non-ASCII characters.

#### Acceptance Criteria

- AC-001.1: Given input "—" (em-dash, 3 bytes, 1 char) and "x", when levenshtein_distance runs, then it returns 1 (not 3)
- AC-001.2: Given identical multi-byte strings, when normalized_levenshtein_similarity runs, then it returns 1.0
- AC-001.3: All existing Levenshtein tests still pass with updated char-based implementation

#### Dependencies

- Depends on: none

### US-002: Replace String errors with BacklogError enum

**As a** developer, **I want** typed error variants in the backlog module, **so that** callers can match on specific error types.

#### Acceptance Criteria

- AC-002.1: `BacklogError` enum exists in `ecc-domain/src/backlog/` with variants: `NoFrontmatter`, `MalformedYaml(String)`, `DirectoryNotFound(PathBuf)`, `EmptyQuery`, `IoError(String)`
- AC-002.2: `parse_frontmatter` returns `Result<BacklogEntry, BacklogError>`
- AC-002.3: `next_id`, `check_duplicates`, `reindex` in ecc-app return `Result<_, BacklogError>`
- AC-002.4: CLI layer maps `BacklogError` to anyhow at the boundary

#### Dependencies

- Depends on: none

### US-003: Extract constants, normalize scores, deduplicate loops

**As a** developer, **I want** clean code patterns in the backlog module, **so that** the code is maintainable and scores are intuitive.

#### Acceptance Criteria

- AC-003.1: Weights (0.7, 0.3) and thresholds (0.15, 0.3, 0.6) are named constants
- AC-003.2: `composite_score` returns values in [0.0, 1.0] range (normalized)
- AC-003.3: A shared `load_entries` helper replaces the duplicated loop in `check_duplicates` and `reindex`
- AC-003.4: `reindex` cleans up temp file on rename failure
- AC-003.5: `BacklogStatus` enum replaces `status: String` in `BacklogEntry`
- AC-003.6: All 33+ existing tests pass (with updated threshold assertions where needed)

#### Dependencies

- Depends on: US-002 (BacklogError used in load_entries helper)

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `ecc-domain/src/backlog/entry.rs` | Domain | BacklogStatus enum, BacklogError, parse_frontmatter signature |
| `ecc-domain/src/backlog/similarity.rs` | Domain | Char-based Levenshtein, constants, normalized score |
| `ecc-domain/src/backlog/index.rs` | Domain | Use BacklogStatus enum |
| `ecc-app/src/backlog.rs` | App | BacklogError return types, load_entries helper, temp cleanup |
| `ecc-cli/src/commands/backlog.rs` | CLI | Map BacklogError to anyhow |

## Constraints

- Must not change the CLI's public interface (output format stays identical)
- Must not change the `check-duplicates` JSON output schema
- All 33+ existing tests must pass (with updated assertions where needed)
- No port/adapter changes required
- `ecc-domain` must remain free of I/O imports

## Non-Requirements

- Not changing the FileSystem port
- Not adding new CLI subcommands
- Not changing BACKLOG.md table format
- Not adding new CLI flags or options

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | None | No E2E impact — all changes are internal |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Bug fix + quality | CHANGELOG.md | Project | Add fix entry |

## Open Questions

None — all questions resolved in grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| Root cause | UTF-8 byte bug is correctness; others are quality | All 7 are root causes in their domains | Recommended |
| 1 | Root cause vs symptom | All 7 are root causes, bundle together | Recommended |
| 2 | Minimal vs proper fix | Proper structural fix — 4 files, ~150 lines | Recommended |
| 3 | Missing tests | 3 new tests: UTF-8 chars, error variants, temp cleanup | Recommended |
| 4 | Regression risk | Contained in 4 files, 33 tests catch regressions, update thresholds | Recommended |
| 5 | Related audit findings | No audit overlap | Recommended |
| 6 | Reproducibility | em-dash "—" vs "x": expected 1, currently 3 | Recommended |
| 7 | Data impact | No persisted data affected | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Fix Levenshtein UTF-8 correctness bug | 3 | none |
| US-002 | Replace String errors with BacklogError enum | 4 | none |
| US-003 | Extract constants, normalize scores, deduplicate loops | 6 | US-002 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | em-dash vs "x" returns 1 | US-001 |
| AC-001.2 | identical multi-byte returns 1.0 | US-001 |
| AC-001.3 | existing Levenshtein tests pass | US-001 |
| AC-002.1 | BacklogError enum with 5 variants | US-002 |
| AC-002.2 | parse_frontmatter returns BacklogError | US-002 |
| AC-002.3 | app functions return BacklogError | US-002 |
| AC-002.4 | CLI maps to anyhow | US-002 |
| AC-003.1 | named constants for weights/thresholds | US-003 |
| AC-003.2 | normalized score [0.0, 1.0] | US-003 |
| AC-003.3 | shared load_entries helper | US-003 |
| AC-003.4 | temp file cleanup on failure | US-003 |
| AC-003.5 | BacklogStatus enum | US-003 |
| AC-003.6 | all existing tests pass | US-003 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-27-fix-backlog-module-quality/spec.md | Full spec |
