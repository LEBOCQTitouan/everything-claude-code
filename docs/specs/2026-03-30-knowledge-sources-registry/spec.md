# Spec: Knowledge Sources Registry — Hardening & Completion

## Problem Statement

The knowledge sources registry (BL-086) was implemented across all hexagonal layers — domain types, parser, serializer, app use cases, CLI commands, and command integrations — but has 2 bugs (hardcoded date in `add()`, stale flag parsing inconsistency), a DDD anti-pattern (primitive obsession on URL field), missing integration tests, and incomplete audit command integrations. These gaps prevent marking BL-086 as fully implemented and leave subtle correctness issues in the stale-tracking workflow.

## Research Summary

- **lychee-lib** is the leading Rust URL reachability crate (async, Tokio-based) — but current shell-based curl approach is sufficient for ~100 sources and avoids adding a heavy dependency
- **Markdown table parsers** (comrak, pulldown-cmark) avoid fragile regex — current hand-rolled parser works but has a stale flag format bug where `stale: true` (key:value) is silently dropped instead of parsed as a flag
- **Staleness best practice**: track `date_added` and `last_checked` per entry, flag sources not verified within configurable window (already implemented)
- **AOE Technology Radar** (React + Markdown entries) validates the quadrant organization pattern already used
- **DDD newtype pattern**: validated value objects make invalid states unrepresentable at compile time — standard Rust idiom per the codebase's own rules

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Keep curl-based checks (no reqwest/lychee) | Existing pattern works, avoids new deps, sequential is fine for ~100 sources | No |
| 2 | URL newtype (`SourceUrl`) in domain layer | DDD anti-primitive-obsession, compile-time safety, follows codebase newtype convention | No |
| 3 | Accept `date: &str` parameter in `add()` | Pure/testable, follows hexagonal patterns — CLI injects today's date | No |
| 4 | Sequential checks with 10s timeout | Sufficient for ~100 sources, no performance SLA | No |
| 5 | Allow HTTP + HTTPS in URL newtype | Some internal docs and localhost tools use HTTP | No |

## User Stories

### US-001: Fix hardcoded date in add() use case

**As a** CLI user, **I want** `ecc sources add` to use today's date, **so that** entries have accurate timestamps.

#### Acceptance Criteria

- AC-001.1: Given a call to `add()`, when no date is provided, then `added_date` uses the current date (injected as `date: &str` parameter, not hardcoded)
- AC-001.2: Given the CLI `ecc sources add` command, when executed, then it passes `today_date()` to the app layer `add()` function
- AC-001.3: Given unit tests for `add()`, when date is injected as "2026-01-15", then the entry has that exact date

#### Dependencies

- Depends on: none

### US-002: Fix stale flag parsing inconsistency

**As a** sources registry user, **I want** the stale flag to round-trip correctly, **so that** `ecc sources check` accurately tracks and clears stale entries.

#### Acceptance Criteria

- AC-002.1: Given a sources.md with `stale` as a bare flag in pipe-separated metadata, when parsed, then `entry.stale == true`
- AC-002.2: Given a stale entry serialized by the serializer, when re-parsed by the parser, then the stale flag is preserved (round-trip)
- AC-002.3: Given the test `check_clears_stale`, when the test data uses the correct stale flag format (`| stale` not `| stale: true`), then the test validates clearing a genuinely stale entry

#### Dependencies

- Depends on: none

### US-003: Introduce URL newtype value object

**As a** domain developer, **I want** URLs wrapped in a validated `SourceUrl` newtype, **so that** invalid URLs cannot exist in domain structs.

#### Acceptance Criteria

- AC-003.1: Given a `SourceUrl::parse("https://example.com")`, when the URL has a valid scheme and host, then construction succeeds
- AC-003.2: Given a `SourceUrl::parse("not-a-url")`, when the URL lacks a scheme, then construction returns `Err(SourceError::InvalidUrl)`
- AC-003.3: Given `SourceEntry.url` field, when typed as `SourceUrl` instead of `String`, then all call sites (parser, add use case, serializer, registry, tests) compile and pass
- AC-003.4: Given a `SourceUrl` instance, when `as_str()` is called, then it returns the inner string for serialization and display

#### Dependencies

- Depends on: none

### US-004: Add integration tests

**As a** developer, **I want** integration tests for the sources CLI commands, **so that** end-to-end behavior is validated beyond unit tests.

#### Acceptance Criteria

- AC-004.1: Given `ecc sources list` with a valid sources.md, when executed end-to-end via the app layer with real file system doubles, then it outputs entries in the correct format
- AC-004.2: Given `ecc sources add` with valid arguments, when executed, then the entry appears in the file with correct date and URL
- AC-004.3: Given `ecc sources reindex` with inbox entries, when executed, then inbox entries are moved to correct quadrants
- AC-004.4: Given `ecc sources reindex --dry-run`, when executed, then the file is unchanged but output shows the proposed result

#### Dependencies

- Depends on: US-001, US-002, US-003

### US-005: Add sources integration to audit-evolution and audit-full

**As an** auditor, **I want** `/audit-evolution` and `/audit-full` to consult the sources registry, **so that** relevant sources are re-interrogated during audits.

#### Acceptance Criteria

- AC-005.1: Given the `audit-evolution` command markdown, when `docs/sources.md` exists, then a "Sources Re-interrogation" step lists sources in relevant subjects for re-interrogation
- AC-005.2: Given the `audit-full` command markdown, when it orchestrates sub-audits, then it includes a "Sources Re-interrogation" section that consults `docs/sources.md`
- AC-005.3: Given no `docs/sources.md` file, when either audit runs, then sources consultation is silently skipped

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `ecc-domain/src/sources/entry.rs` | Domain | Add `SourceUrl` newtype with `parse()` and `as_str()`, change `SourceEntry.url` from `String` to `SourceUrl` |
| `ecc-domain/src/sources/parser.rs` | Domain | Use `SourceUrl::parse()` when constructing entries from parsed markdown |
| `ecc-domain/src/sources/serializer.rs` | Domain | Use `entry.url.as_str()` for serialization output |
| `ecc-domain/src/sources/registry.rs` | Domain | Update `find_by_url()` to accept `&str` and compare via `as_str()` |
| `ecc-app/src/sources.rs` | App | Accept `date: &str` parameter in `add()`, use `SourceUrl::parse()` for URL construction |
| `ecc-cli/src/commands/sources.rs` | CLI | Pass `today_date()` result to `add()` as the date parameter |
| `ecc-integration-tests/tests/sources.rs` | Test | New integration test file covering list, add, reindex, reindex --dry-run |
| `commands/audit-evolution.md` | Command | Add sources consultation section for re-interrogation |
| `commands/audit-full.md` | Command | Add sources re-interrogation orchestration step |

## Constraints

- `ecc-domain` must have zero I/O imports — `SourceUrl` validation must be pure string parsing (no `url` crate if it pulls I/O)
- Stale flag format must remain `stale` (bare flag) in pipe-separated metadata — never `stale: true`
- All existing 36 sources-related tests must continue to pass after changes
- Atomic write pattern (temp file + rename) must be preserved for all file mutations
- Test data in `check_clears_stale` must use the serializer's format, not a divergent format

## Non-Requirements

- `CheckStatus`/`CheckResult` type relocation from app to domain layer
- `--dry-run` flag on `add` subcommand
- Domain events (`SourceAdded`, `SourceMarkedStale`, `SourceDeprecated`)
- reqwest or lychee-lib replacement for curl-based reachability checks
- Parallel URL checks or async check execution
- Driving port traits for the sources feature (consistent with codebase convention)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| FileSystem port | No change | Existing `read_to_string`, `write`, `rename` sufficient |
| ShellExecutor port | No change | curl-based `check` unchanged |
| CLI (`ecc sources add`) | Parameter addition (`date`) | Integration test covers new parameter flow |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Backlog status | Metadata | `docs/backlog/BL-086-*.md` | Mark `status: implemented` after completion |
| Test count | CLAUDE.md | `CLAUDE.md` | Update test count after integration tests added |
| Bounded contexts | Reference | `docs/domain/bounded-contexts.md` | Add `SourceUrl` value object to Sources context description |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Scope boundaries | Focus on 5 gaps + URL newtype refactor. Out of scope: CheckStatus relocation, --dry-run on add, domain events | User |
| 2 | Curl unavailable edge case | Clear error + exit with non-zero status. No fallback. | Recommended |
| 3 | Test coverage strategy | 100%: parser round-trip, add with date, stale flag, URL newtype. 80%: CLI, check, reindex | Recommended |
| 4 | Performance constraints | Sequential checks + 10s timeout per URL. No SLA. | Recommended |
| 5 | URL scheme restriction | Allow both HTTP and HTTPS. Validate format only. | Recommended |
| 6 | Breaking changes from URL newtype | No concerns — no external consumers, persisted format unchanged | Recommended |
| 7 | New domain terms | None needed — existing terms sufficient | Recommended |
| 8 | ADR decisions | None needed — standard DDD patterns and bug fixes | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Fix hardcoded date in add() use case | 3 | none |
| US-002 | Fix stale flag parsing inconsistency | 3 | none |
| US-003 | Introduce URL newtype value object | 4 | none |
| US-004 | Add integration tests | 4 | US-001, US-002, US-003 |
| US-005 | Add sources integration to audit-evolution and audit-full | 3 | none |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | add() uses injected date parameter, not hardcoded | US-001 |
| AC-001.2 | CLI passes today_date() to app layer | US-001 |
| AC-001.3 | Unit tests verify injected date appears on entry | US-001 |
| AC-002.1 | Bare `stale` flag in metadata parses as stale=true | US-002 |
| AC-002.2 | Stale flag round-trips through serialize/parse | US-002 |
| AC-002.3 | check_clears_stale test uses correct flag format | US-002 |
| AC-003.1 | SourceUrl::parse succeeds for valid URLs | US-003 |
| AC-003.2 | SourceUrl::parse fails for invalid URLs | US-003 |
| AC-003.3 | All call sites compile with SourceUrl type | US-003 |
| AC-003.4 | SourceUrl::as_str() returns inner string | US-003 |
| AC-004.1 | list integration test outputs correct format | US-004 |
| AC-004.2 | add integration test creates entry in file | US-004 |
| AC-004.3 | reindex integration test moves inbox entries | US-004 |
| AC-004.4 | reindex --dry-run leaves file unchanged | US-004 |
| AC-005.1 | audit-evolution consults sources.md | US-005 |
| AC-005.2 | audit-full includes sources re-interrogation | US-005 |
| AC-005.3 | Audits skip silently when no sources.md | US-005 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Ambiguity | 88 | PASS | ACs precise with Given/When/Then; minor vagueness on re-interrogation semantics |
| Edge Cases | 72 | PASS | Stale round-trip well-specified; minor gaps on malformed date, URL whitespace |
| Scope | 90 | PASS | 6 explicit non-requirements, low scope creep risk |
| Dependencies | 85 | PASS | US-004 depends correctly on US-001/002/003; no circular deps |
| Testability | 82 | PASS | All ACs testable; AC-003.3 is compile-time (valid but unconventional) |
| Decisions | 92 | PASS | All 5 decisions documented with rationale |
| Rollback | 78 | PASS | Bug fixes minimal blast radius; SourceUrl touches 6 files but build fails safely |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-30-knowledge-sources-registry/spec.md | Full spec with phase summary |
