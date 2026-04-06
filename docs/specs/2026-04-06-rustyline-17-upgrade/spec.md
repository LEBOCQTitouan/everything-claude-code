# Spec: BL-114 Upgrade rustyline 15 → 17

## Problem Statement

rustyline is 2 major versions behind (v15 → v17.0.2). Used by NanoClaw REPL (`ecc claw`). Web radar flagged as trial-ring upgrade.

## Research Summary

- v16.0.0: grapheme cluster support, SIGINT handling, new Prompt trait
- v17.0.0: maintenance — dependency bumps, deletion refresh optimization
- No breaking API changes to DefaultEditor, readline(), add_history_entry(), load_history(), save_history()
- Upgrade is a version bump only — zero code changes expected

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Bump version in workspace Cargo.toml only | No API changes between v15 and v17 | No |

## User Stories

### US-001: Upgrade rustyline dependency

**As a** maintainer, **I want** rustyline updated to v17, **so that** we stay current with security patches and dependency updates.

#### Acceptance Criteria

- AC-001.1: rustyline version in workspace Cargo.toml is "17"
- AC-001.2: `cargo build --workspace` succeeds
- AC-001.3: `cargo test --workspace` passes (no regressions)
- AC-001.4: `cargo clippy -- -D warnings` passes

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| Cargo.toml (workspace) | root | version bump |
| Cargo.lock | root | auto-updated |

## Constraints

- Zero code changes expected
- If API breaking changes surface at compile time, address minimally

## Non-Requirements

- NanoClaw feature additions
- rustyline configuration changes

## Open Questions

None.
