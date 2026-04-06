# Spec: Miri Unsafe Code Verification for ecc-flock

## Problem Statement

ecc-flock uses libc and POSIX flock which involve unsafe boundaries. Miri can detect undefined behavior in unsafe Rust code at test time, but cannot interpret FFI calls to libc. The FFI unsafe blocks need manual audit documentation, while pure-Rust logic can be Miri-verified.

## Research Summary

- Miri cannot interpret foreign function calls (flock, close) -- "unsupported operation" error
- Miri CAN verify pure-Rust unsafe code (std::mem::forget patterns, pointer arithmetic)
- SAFETY comments on all 3 unsafe blocks already exist but are minimal
- ecc-flock has 6 existing tests, 3 of which use FFI (acquire, acquire_with_timeout, acquire_for)

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Miri on non-FFI tests only | FFI calls cause "unsupported operation" in Miri | No |
| 2 | Enhanced SAFETY comments | Document invariants for manual audit | No |
| 3 | Nightly-only, graceful skip | Miri requires nightly Rust; skip if not installed | No |

## User Stories

### US-001: Miri Verification

**As a** developer, **I want** Miri verification on ecc-flock's safe-Rust tests, **so that** I can detect undefined behavior in the non-FFI code paths.

#### Acceptance Criteria

- AC-001.1: Given `cargo +nightly miri test -p ecc-flock` is run, when Miri is available, then it executes the non-FFI tests successfully
- AC-001.2: Given the FFI tests (guard_drop_releases, acquire_creates_lock, acquire_with_timeout_succeeds_when_free, acquire_for_creates_lock), when run under Miri, then they are skipped with `#[cfg_attr(miri, ignore)]` and a comment explaining why
- AC-001.3: Given each unsafe block in ecc-flock, when reviewed, then it has a comprehensive SAFETY comment documenting: the invariant, why it's sound, and what could go wrong
- AC-001.4: Given CLAUDE.md Running Tests section, when updated, then it includes the Miri command with a note about the FFI limitation
- AC-001.5: Given a developer without nightly Rust, when they skip Miri, then no existing functionality is affected

#### Dependencies
- Depends on: none

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| `crates/ecc-flock/src/lib.rs` | Crate | Add cfg_attr(miri, ignore) to FFI tests, enhance SAFETY comments |
| `CLAUDE.md` | Docs | Add Miri command to Running Tests |

## Constraints

- Miri requires nightly Rust -- must be optional
- No changes to production code behavior
- FFI tests must still pass under normal cargo test

## Non-Requirements

- Making FFI calls Miri-compatible (impossible)
- CI integration for Miri
- Refactoring unsafe code to safe abstractions

## E2E Boundaries Affected
None.

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New command | CLAUDE.md | Running Tests | Add Miri command |
| Changelog | CHANGELOG.md | Add entry | all |

## Rollback Plan

1. Remove cfg_attr(miri, ignore) from tests
2. Revert SAFETY comment enhancements
3. Remove Miri line from CLAUDE.md

## Open Questions
None.
