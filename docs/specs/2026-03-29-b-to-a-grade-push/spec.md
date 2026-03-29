# Spec: B-to-A Grade Push — 5 Remaining HIGHs

## Problem Statement

The post-remediation audit (2026-03-28) graded the project at B with 5 remaining HIGH findings: a missing Transitionable trait implementation, one file marginally over the 800-line limit (aliases.rs at 814 lines; merge/mod.rs at 362 lines is already compliant), 4 functions using `Result<T, String>` in package_manager.rs, and 90 functions exceeding the 50-line limit. Fixing these eliminates all HIGH findings, achieving 0 CRITICAL + 0 HIGH which qualifies for an A grade.

## Research Summary

Web research skipped: mechanical fixes with established patterns from the previous remediation (Wave 2 file splits, Wave 3 error type migration).

## Decisions Made

| # | Decision | Rationale | ADR? |
|---|----------|-----------|------|
| 1 | Scope function splits to top 10 worst offenders | 90 functions is too many for one spec; top 10 covers the most impactful | No |
| 2 | Follow same split/migration patterns from v4.3.0 | Proven patterns, no new risk | No |

## User Stories

### US-001: Complete Transitionable Implementation

**As a** developer, **I want** WorkflowState to implement the Transitionable trait, **so that** domain tests compile and the trait is usable.

#### Acceptance Criteria

- AC-001.1: Given WorkflowState, when transition_to(Phase) is called, then it delegates to resolve_transition_by_name and returns Result<Self, WorkflowError>
- AC-001.2: Given make_state() test helper in io.rs, when constructing WorkflowState, then it uses Concern::Dev (not String) and Timestamp("...".into()) (not String)
- AC-001.3: Given `cargo test -p ecc-domain`, when run, then all domain tests pass including transitionable_impl tests

#### Dependencies
- None

### US-002: Split session/aliases.rs (814 lines)

**As a** developer, **I want** aliases.rs under 800 lines, **so that** it complies with the file size convention.

#### Acceptance Criteria

- AC-002.1: Given aliases.rs, when split, then each resulting file is < 400 lines
- AC-002.2: Given the split, when cargo test runs, then all existing session tests pass

#### Dependencies
- None

### US-003: Type package_manager.rs Errors

**As a** developer, **I want** the 4 functions in package_manager.rs to use typed errors, **so that** no `Result<T, String>` remains in ecc-domain.

#### Acceptance Criteria

- AC-003.1: Given 4 functions returning Result<T, String>, when migrated to a PackageManagerError enum, then all callers compile
- AC-003.2: Given `cargo test -p ecc-domain`, when run, then all tests pass
- AC-003.3: Given `grep 'Result<.*String>' crates/ecc-domain/src/detection/package_manager.rs`, when run, then zero matches

#### Dependencies
- None

### US-004: Extract Top 10 Longest Functions

**As a** developer, **I want** the 10 longest functions (>70 lines) refactored below 50 lines, **so that** the function-size convention is better enforced.

#### Acceptance Criteria

- AC-004.1: Given run_validate_design (171 lines), when extracted into helpers, then the main function is < 50 lines
- AC-004.2: Given transition::run (145 lines), when extracted into helpers, then the main function is < 50 lines
- AC-004.3: Given merge_skills (121 lines), when extracted into helpers, then the main function is < 50 lines
- AC-004.4: Given SessionManager::default (95 lines), when extracted, then < 50 lines
- AC-004.5: Given run_validate_spec (85 lines), when extracted, then < 50 lines
- AC-004.6: Given worktree::gc (83 lines), when extracted, then < 50 lines
- AC-004.7: Given get_package_manager (81 lines), when extracted, then < 50 lines
- AC-004.8: Given get_all_sessions (80 lines), when extracted, then < 50 lines
- AC-004.9: Given rename_alias (79 lines), when extracted, then < 50 lines
- AC-004.10: Given format_side_by_side_diff (78 lines), when extracted, then < 50 lines
- AC-004.11: Given all extractions, when cargo test runs, then all tests pass

#### Dependencies
- US-002 (aliases.rs split may affect function locations)

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| ecc-domain/src/workflow/state.rs | Domain | Transitionable impl |
| ecc-domain/src/workflow/transition.rs | Domain | Used by Transitionable impl |
| ecc-domain/src/detection/package_manager.rs | Domain | PackageManagerError enum |
| ecc-app/src/merge/mod.rs | App | Split into submodules |
| ecc-app/src/session/aliases.rs | App | Split into submodules |
| ecc-app/src/validate_design.rs | App | Function extraction |
| ecc-app/src/validate_spec.rs | App | Function extraction |
| ecc-app/src/worktree.rs | App | Function extraction |
| ecc-app/src/session/mod.rs | App | Function extraction |
| ecc-domain/src/session/manager.rs | Domain | Function extraction |
| ecc-domain/src/diff/formatter.rs | Domain | Function extraction |
| ecc-domain/src/config/merge.rs | Domain | Function extraction |
| ecc-workflow/src/commands/transition.rs | Binary | Function extraction |
| ecc-workflow/src/io.rs | Binary | Fix make_state() helper |

## Constraints

- All steps behavior-preserving (tests pass before and after)
- Build must pass after each change (cargo test + cargo clippy)
- No new features — pure refactoring only

## Non-Requirements

- Remaining 80 functions >50 lines (deferred — top 10 covers worst offenders)
- anyhow removal from ecc-app (blocked by worktree.rs — separate ticket)

## E2E Boundaries Affected

None — all changes are internal structural refactoring.

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|---|---|---|---|
| Update | Minor | CLAUDE.md | Update test count if new tests added |
| Add entry | Minor | CHANGELOG.md | Add B-to-A remediation entry |

## Open Questions

None — all resolved during grill-me interview.
