# Spec: Fix All HIGH Findings from Full Audit (2026-04-09)

## Problem Statement

The April 9 full codebase audit (Grade B, up from C) identified 5 HIGH findings and several overlapping MEDIUMs that stem from rapid growth outpacing hygiene: a failing time-dependent test in sqlite_bypass_store, 6 files exceeding the 800-line limit (phase_gate.rs at 592 lines is below threshold — corrected from audit), inconsistent regex compilation patterns (15 inline Regex::new() + 4 OnceLock in production), low doc-comment coverage (ecc-app 2.7%, ecc-infra 2.1%), and 15 production `let _ =` on Result in error-significant code paths (out of ~106 total, 48 acceptable cleanup, 35 test, 8 deliberate unused bindings). These findings compound: inline regex creates unwrap/expect calls, oversized files resist review, and swallowed errors hide failures.

## Research Summary

- Use `std::sync::LazyLock<Regex>` (stable since Rust 1.80) — initialization on first dereference, preferred over legacy lazy_static! or OnceLock
- Regex compilation takes microseconds-to-milliseconds; compiling in loops (secrets.rs: 6 regexes per call) is a measurable anti-pattern
- Split oversized Rust modules by extracting submodules with pub use re-exports to preserve API — do in 200-300 line increments
- Replace `let _ = expr` with `if let Err(e) = expr { tracing::warn!(...) }` — warn level for non-critical issues that might cascade
- Integrate tracing instrumentation with error types via tracing-error for enriched diagnostics
- Moving code between Rust functions changes ownership scope — test after each extraction
- OnceLock is semantically equivalent to LazyLock but more verbose; standardize on LazyLock

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Full structural fix for all 7 oversized files | User chose comprehensive over scope-limited | No |
| 2 | Clock injection via constructor on SqliteBypassStore | Existing Clock port trait at ecc-ports/src/clock.rs; inject at construction time, no BypassStore trait change needed | No |
| 3 | LazyLock<Regex> for static patterns only | Eliminates OnceLock (4) and inline (15) static patterns; dynamic regex (e.g., secrets.rs function args) stays as Regex::new() | No |
| 4 | Bundle overlapping MEDIUM findings | Same files touched, no extra blast radius | No |
| 5 | Comprehensive test coverage | Test every change category including re-exports | No |
| 6 | Prune equivalence test | Verify old/new prune produce identical results | No |
| 7 | Drop F2 (clippy lints) as invalid | Investigation confirmed clippy passes clean | No |
| 8 | Extract duplicate is_leap into ecc-domain | 5 copies across crates to 1 shared utility | No |
| 9 | Drop phase_gate.rs from decomposition | Actual line count is 592 (below 800 limit), audit figure was wrong | No |
| 10 | Scope swallowed errors to 15 error-significant sites | Out of ~106 total `let _ =`, only 15 are in error-significant production paths; 48 are acceptable cleanup, 35 test, 8 deliberate unused | No |

## User Stories

### US-001: Fix Failing Prune Test
**As a** developer, **I want** the sqlite_bypass_store prune test to be deterministic, **so that** CI stays green regardless of when tests run.

#### Acceptance Criteria
- AC-001.1: Given any system clock value, when `prune(older_than_days)` is called, then the cutoff is computed in Rust using the injected Clock (via constructor, not trait change) and passed as a bound SQL parameter
- AC-001.2: Given a test with injected Clock returning a fixed "now", when prune(1) is called, then records older than 1 day from the injected time are deleted and recent records survive
- AC-001.3: Given the old and new prune implementations with identical inputs and clock, when compared, then they produce identical deletion sets (equivalence test)
- AC-001.4: Given the prune SQL, when executed, then no format! string interpolation is used — only parameterized queries
- AC-001.5: Given the BypassStore port trait in ecc-ports, when the fix is applied, then the trait signature is unchanged (clock is a construction-time dependency of the adapter, not a method parameter)

#### Dependencies
- Depends on: none

### US-002: Decompose Oversized Files
**As a** developer, **I want** all files under the 800-line limit, **so that** code is reviewable and merge conflicts are reduced.

#### Acceptance Criteria
- AC-002.1: Given validate/patterns.rs (2,355 lines), when decomposed into submodules (frontmatter_validation, cross_ref_validation, section_validation, code_block_scanning, patterns_tests), then each submodule is under 800 lines and `cargo test -p ecc-app validate` passes
- AC-002.2: Given cartography/tests_helpers.rs (2,009 lines), when decomposed into submodules (start_test_helpers, stop_test_helpers, delta_test_helpers), then each is under 800 lines and all cartography tests pass
- AC-002.3: Given orchestrator_tests.rs (962 lines), when decomposed into submodules (happy_path_tests, error_tests, rollback_tests), then each is under 800 lines and `cargo test -p ecc-app update` passes
- AC-002.4: Given sqlite_memory.rs (907 lines), when decomposed into (memory_schema, memory_queries, memory_tests), then each is under 800 lines and `cargo test -p ecc-infra sqlite_memory` passes
- AC-002.5: Given hook/mod.rs (873 lines), when decomposed into (dispatch, bypass_handling, hook_tests), then each is under 800 lines and `cargo test -p ecc-app hook` passes
- AC-002.6: Given merge.rs (809 lines), when decomposed into (merge logic file + merge_tests), then each is under 800 lines and `cargo test -p ecc-workflow merge` passes
- AC-002.7: Given any decomposed file, when other crates import from it, then all public items are re-exported at the original module path (API preserved — `cargo build` succeeds without import changes in dependent crates)

#### Dependencies
- Depends on: none

### US-003: Standardize Regex on LazyLock
**As a** developer, **I want** all regex patterns compiled via LazyLock<Regex>, **so that** there are no per-call compilation costs and no production unwrap/expect calls on regex.

#### Acceptance Criteria
- AC-003.1: Given the 15 inline Regex::new() sites with static patterns in production code, when converted to LazyLock<Regex>, then no static-pattern Regex::new() remains in non-test production code (dynamic patterns like secrets.rs function args are excluded)
- AC-003.2: Given the 4 OnceLock<Regex> sites in ecc-domain (3 in report_validation.rs, 1 in dimension.rs), when converted to LazyLock<Regex>, then only LazyLock<Regex> is used for static regex across the workspace
- AC-003.3: Given the 1 #[allow(clippy::manual_is_multiple_of)] in bypass_mgmt.rs, when removed, then the function uses .is_multiple_of() and clippy passes
- AC-003.4: Given the 5 duplicate is_leap functions, when extracted to ecc-domain, then all 5 call sites use the shared function
- AC-003.5: Given any LazyLock<Regex> initialization, when the regex pattern is invalid, then the application panics on first access (LazyLock initializes on first dereference, not at binary startup) with a descriptive expect message

#### Dependencies
- Depends on: none

### US-004: Increase Doc-Comment Coverage
**As a** developer, **I want** doc comments on all public types and functions in ecc-app, ecc-infra, and ecc-domain, **so that** new contributors and AI agents understand module purposes and contracts.

#### Acceptance Criteria
- AC-004.1: Given ecc-domain public items, when doc coverage is measured, then coverage exceeds 50% (up from 4.7%)
- AC-004.2: Given ecc-app public items, when doc coverage is measured, then coverage exceeds 30% (up from 2.7%)
- AC-004.3: Given ecc-infra public items, when doc coverage is measured, then coverage exceeds 30% (up from 2.1%)
- AC-004.4: Given ecc-ports public items, when doc coverage is measured, then all port traits have at least a one-line doc comment

#### Dependencies
- Depends on: US-002 (file decomposition may move public items)

### US-005: Convert Swallowed Errors to Logged Warnings
**As a** developer, **I want** production `let _ =` on Result replaced with tracing::warn, **so that** failures are observable instead of silent.

#### Acceptance Criteria
- AC-005.1: Given the 15 production `let _ =` sites on Result, when converted, then each emits a tracing::warn with context (operation, path, error)
- AC-005.2: Given hook/mod.rs:294 bypass audit recording, when it fails, then a warning is emitted with hook_id and error
- AC-005.3: Given drift_check.rs:57,59 file operations, when they fail, then a warning is emitted with the target path
- AC-005.4: Given session_merge.rs:76 recovery file write, when it fails, then a warning is emitted
- AC-005.5: Given merge.rs:141 rebase abort, when it fails, then a warning is emitted
- AC-005.6: Given any `let _ =` on a fire-and-forget metric (hook/mod.rs:349, quality.rs:205), when kept as `let _ =`, then a code comment documents the intentional discard

#### Dependencies
- Depends on: US-002 (hook/mod.rs decomposition moves code)

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| ecc-infra/src/sqlite_bypass_store.rs | Adapter | Parameterize prune SQL, inject existing Clock via constructor |
| ecc-ports/src/clock.rs | Port | Already exists — no changes needed |
| ecc-app/src/validate/patterns.rs | App | Decompose into submodules |
| ecc-app/src/hook/mod.rs | App | Decompose + convert swallowed errors |
| ecc-app/src/hook/handlers/tier3_session/cartography/tests_helpers.rs | App | Decompose test helpers |
| ecc-app/src/update/orchestrator_tests.rs | App | Decompose test scenarios |
| ecc-infra/src/sqlite_memory.rs | Adapter | Decompose schema/queries |
| ecc-workflow/src/commands/merge.rs | Workflow | Decompose prod/tests + convert swallowed errors |
| ecc-domain/src/spec/*.rs | Domain | LazyLock regex conversion |
| ecc-domain/src/drift/*.rs | Domain | LazyLock regex conversion |
| ecc-domain/src/memory/*.rs | Domain | LazyLock regex conversion + swallowed errors |
| ecc-domain (multiple) | Domain | Doc comments on public items |
| ecc-app (multiple) | App | Doc comments + swallowed error conversion |

## Constraints

- Must not change any public API (all re-exports preserved after file splits)
- Must not introduce new dependencies (LazyLock and tracing are already in the workspace)
- Prune equivalence test required (user decision)
- All 2,569+ existing tests must continue to pass after each change
- ecc-domain must remain zero-I/O (no tracing in domain — swallowed errors in domain stay as-is or propagate via Result)

## Non-Requirements

- Bus factor improvement (EVOL-001) — out of scope for code changes
- CONV-004 duplicate type names — not addressed in this fix
- OBS-001 startup health checks — separate concern
- Full 80%+ doc coverage — targeting 30-50% improvement, not completeness

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Clock port (existing) | Constructor injection into SqliteBypassStore | Prune tests use injected clock; no trait change |
| SqliteBypassStore | Query change | Prune behavior verified by equivalence test |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Bug fix | CHANGELOG | CHANGELOG.md | Add entry for prune fix |
| Refactoring | CHANGELOG | CHANGELOG.md | Add entry for file decomposition |
| Code quality | CHANGELOG | CHANGELOG.md | Add entries for regex/errors/docs |
| Test count | CLAUDE.md | CLAUDE.md | Update test count after new tests added |

## Open Questions

None — all resolved during grill-me interview.
