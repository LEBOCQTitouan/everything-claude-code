# Spec: Fix cargo xtask deploy — ecc-flock binary not found

## Problem Statement

`cargo xtask deploy` fails after a successful release build with "No such file or directory (os error 2)". The root cause is that `deploy.rs` lists `ecc-flock` in both `packages_to_build()` and `binaries_to_install()`, but `ecc-flock` is a library crate (`lib.rs` only) and does not produce a binary artifact. The `std::fs::copy("target/release/ecc-flock", ...)` call fails because no such file exists.

## Research Summary

- Web research skipped: no search tool available.
- ecc-flock is a shared POSIX flock utility consumed as a library dependency
- Built transitively when ecc-cli or ecc-workflow are built
- Deploy script was written when ecc-flock may have been planned as a standalone binary

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Remove ecc-flock from both deploy lists | Lib crate built transitively; no binary to copy | No |
| 2 | Add validation test | Prevent future regressions | No |

## User Stories

### US-001: Fix deploy binary list

**As a** developer, **I want** `cargo xtask deploy` to complete successfully, **so that** I can deploy ECC to my local machine.

#### Acceptance Criteria

- AC-001.1: Given `cargo xtask deploy --dry-run`, when executed, then it completes without mentioning ecc-flock
- AC-001.2: Given `cargo xtask deploy`, when executed, then it completes with exit 0
- AC-001.3: Given `packages_to_build()`, when inspected, then it returns `["ecc-cli", "ecc-workflow"]`
- AC-001.4: Given `binaries_to_install()`, when inspected, then it returns `["ecc", "ecc-workflow"]`
- AC-001.5: Given a unit test, when it checks binaries_to_install entries, then all correspond to actual binary crates

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| xtask/src/deploy.rs | Tooling (outside hex) | Remove ecc-flock from lists, add test |

## Constraints

- Must not break existing deploy --dry-run output format
- ecc-flock must still compile as a transitive dependency

## Non-Requirements

- Converting ecc-flock to a binary crate
- Changing deploy dry-run output format

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | N/A | Tooling-only fix |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CHANGELOG | Project | CHANGELOG.md | Add fix entry |

## Open Questions

None.
