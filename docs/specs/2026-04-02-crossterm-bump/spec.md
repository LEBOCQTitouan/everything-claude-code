# Spec: BL-105 Bump crossterm 0.28 to 0.29

## Problem Statement

crossterm 0.29.0 was released 2025-04-05. ECC uses 0.28.1. The project should stay current on dependencies to receive bug fixes and security patches. Only 2 APIs are used (`is_tty()`, `terminal::size()`), both behind the `TerminalIO` port trait in `ecc-infra`, making this a minimal-risk bump.

## Research Summary

Web research skipped -- routine minor dependency bump with no breaking changes documented.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Bump crossterm 0.28 to 0.29 | Stay current, receive fixes | No |
| 2 | Audit API usage for newer patterns | Review whether 0.29 offers better APIs | No |
| 3 | Add non-TTY test | Verify terminal_width() returns None on non-TTY | No |

## User Stories

### US-001: Bump crossterm Version

**As a** maintainer, **I want** crossterm updated to 0.29, **so that** the project receives the latest bug fixes and stays current.

#### Acceptance Criteria

- AC-001.1: Given Cargo.toml workspace dependencies, when inspected, then crossterm version is "0.29"
- AC-001.2: Given the bumped dependency, when `cargo build` runs, then it compiles without errors
- AC-001.3: Given the bumped dependency, when `cargo test` runs, then all tests pass
- AC-001.4: Given the bumped dependency, when `cargo clippy -- -D warnings` runs, then no warnings
- AC-001.5: Given the bumped dependency, when `cargo audit` runs, then no new advisories for crossterm
- AC-001.6: Given `ecc-infra/src/std_terminal.rs`, when the StdTerminal impl is reviewed, then it compiles correctly with crossterm 0.29 APIs

#### Dependencies

- Depends on: none

### US-002: Audit crossterm API Usage

**As a** maintainer, **I want** a review of whether crossterm 0.29 offers improved APIs, **so that** we use the best available patterns.

#### Acceptance Criteria

- AC-002.1: Given crossterm 0.29 API surface, when reviewed against current usage (is_tty, terminal::size), then a determination is made whether any API changes are beneficial
- AC-002.2: Given the audit finding, when documented in the commit message, then the rationale for "no code changes needed" or "migrated to X" is recorded

#### Dependencies

- Depends on: US-001

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `Cargo.toml` | Config | Version bump 0.28 to 0.29 |
| `Cargo.lock` | Config | Auto-updated by cargo |
| `crates/ecc-infra/src/std_terminal.rs` | Infra | Review only |

## Constraints

- Must compile on both macOS and Linux
- All existing tests must pass
- No new cargo audit advisories

## Non-Requirements

- No refactoring of TerminalIO port trait
- No new features from crossterm 0.29
- No ADR

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| TerminalIO / StdTerminal | Dependency bump | Compile-time verification only |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Dependency bump | LOW | CHANGELOG.md | Add chore entry |

## Open Questions

None.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Scope boundaries? | Bump + audit API usage | User |
| 2 | Edge cases? | Add non-TTY test | User |
| 3 | Test strategy? | Existing tests + build verification | Recommended |
| 4 | Performance? | No concerns | Recommended |
| 5 | Security? | Run cargo audit after bump | User |
| 6 | Breaking changes? | Verify trait impl compiles | User |
| 7 | Domain concepts? | None | Recommended |
| 8 | ADR? | No ADR needed | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Bump crossterm Version | 6 | none |
| US-002 | Audit crossterm API Usage | 2 | US-001 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Version is "0.29" in Cargo.toml | US-001 |
| AC-001.2 | cargo build compiles | US-001 |
| AC-001.3 | cargo test passes | US-001 |
| AC-001.4 | cargo clippy clean | US-001 |
| AC-001.5 | cargo audit clean | US-001 |
| AC-001.6 | StdTerminal compiles with 0.29 | US-001 |
| AC-002.1 | API audit determination | US-002 |
| AC-002.2 | Audit documented in commit | US-002 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-02-crossterm-bump/spec.md` | Full spec + Phase Summary |
