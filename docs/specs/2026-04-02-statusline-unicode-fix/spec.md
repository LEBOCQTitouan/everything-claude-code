# Spec: BL-076 Statusline Unicode Byte-Counting Bug Fix

## Problem Statement

`statusline-command.sh` uses `${#stripped}` (bash string length) at 4 locations (lines 231, 266, 271, 276) to measure output width before deciding which segments to include. In non-UTF-8 locales (C/POSIX), `${#var}` counts bytes, not characters. Unicode characters (◆, ⎇, █, ░, ↑, ↓) are 3 bytes each in UTF-8, causing the script to overcount width by ~2x and silently drop rate limit bars and other segments even when the terminal has sufficient width.

## Research Summary

- **`${#var}` counts characters in UTF-8 locales but bytes in C/POSIX locales** -- the root cause of this bug
- **`printf '%s' "$var" | wc -m`** counts characters regardless of locale but spawns a subprocess (performance concern for statusline)
- **Pure-bash has no builtin for display width** -- `wcswidth()` is the authoritative C function, but overkill for our use case
- **For single-width Unicode characters (our case), `${#var}` in UTF-8 locale is sufficient** -- no East Asian wide chars or combining marks in statusline
- **Forcing `LC_ALL=C.UTF-8` at script start** ensures consistent behavior across all user locale configurations
- **ANSI stripping is already handled** by existing `strip_ansi()` function

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Force LC_ALL=C.UTF-8 at script start + add visible_width() helper | Global LC_ALL is safe because statusline-command.sh runs as a standalone subprocess (not sourced) -- it does not affect the user's shell locale. Belt-and-suspenders: locale ensures ${#var} counts chars; helper provides explicit intent and safety net | No |
| 2 | Proper structural fix (helper function + tests) over minimal patch | Prevents regression with explicit test coverage for Unicode width | No |

## User Stories

### US-001: Fix Width Measurement

**As a** developer using ECC statusline, **I want** the width measurement to count visible characters correctly, **so that** all segments (including rate limits) appear when the terminal has sufficient width.

#### Acceptance Criteria

- AC-001.1: Given `statusline-command.sh`, when loaded, then it sets `LC_ALL=C.UTF-8` immediately after the shebang and `set` options (before any function definitions or variable assignments) to ensure consistent character counting
- AC-001.2: Given a `visible_width()` function exists, when called with a string containing Unicode characters, then it returns the visible character count (not byte count)
- AC-001.3: Given `build_output()` runs with rate limits in the JSON input and COLUMNS=120, when segments are assembled, then rate limit bars (5h:, 7d:) are included in the output
- AC-001.4: Given all 4 width comparison lines (231, 266, 271, 276), when the fix is applied, then they use `visible_width` instead of `${#...}`
- AC-001.5: Given COLUMNS=50 (narrow terminal), when segments are assembled, then low-priority segments are still dropped gracefully

#### Dependencies

- Depends on: none

### US-002: Add Unicode Width Tests

**As a** developer, **I want** Bats tests verifying Unicode width handling, **so that** this bug doesn't regress.

#### Acceptance Criteria

- AC-002.1: Given a Bats test with rate_limits in the JSON and COLUMNS=120, when the statusline renders, then rate limit segments appear in the output
- AC-002.2: Given a Bats test with COLUMNS=50, when the statusline renders, then only high-priority segments appear (graceful degradation)
- AC-002.3: Given the `visible_width()` function, when tested with `"◆ Model"` (7 visible chars), `"████░░░░"` (8 visible chars), and `"⎇ branch"` (8 visible chars), then it returns 7, 8, and 8 respectively

#### Dependencies

- Depends on: US-001

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `statusline/statusline-command.sh` | Script | Add LC_ALL guard, visible_width() function, update 4 width comparisons |
| `tests/statusline/` | Test | Add Unicode width Bats tests |

## Constraints

- No subprocess spawning in hot path (statusline runs on every prompt)
- Must work on both macOS and Linux bash
- Existing `strip_ansi()` function must be reused, not duplicated
- All 16 existing Bats tests must continue to pass

## Non-Requirements

- Not handling East Asian wide characters (not used in statusline)
- Not handling combining marks (not used in statusline)
- Not refactoring the entire `build_output()` function
- No ADR needed (small targeted fix)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Statusline script | Fix | Rate limits and other low-priority segments now visible at adequate terminal widths |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Bug fix | LOW | CHANGELOG.md | Add fix entry |

## Open Questions

None -- all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| — | **Root cause** | ${#var} counts bytes in non-UTF-8 locales; confirmed root cause, not symptom | Recommended |
| 1 | Fix measurement or fix locale? | Both: force LC_ALL + add visible_width() | User |
| 2 | Minimal patch or proper fix? | Proper fix with helper + tests | Recommended |
| 3 | Add Unicode width tests? | Yes, Bats tests for width verification | Recommended |
| 4 | Regression risk? | No concerns, isolated to statusline | Recommended |
| 5 | Data impact? | No data impact, ephemeral display | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Fix Width Measurement | 5 | none |
| US-002 | Add Unicode Width Tests | 3 | US-001 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | LC_ALL=C.UTF-8 after shebang | US-001 |
| AC-001.2 | visible_width() returns char count | US-001 |
| AC-001.3 | Rate limits visible at COLUMNS=120 | US-001 |
| AC-001.4 | All 4 comparison lines use visible_width | US-001 |
| AC-001.5 | Graceful degradation at COLUMNS=50 | US-001 |
| AC-002.1 | Bats test: rate limits at 120 cols | US-002 |
| AC-002.2 | Bats test: degradation at 50 cols | US-002 |
| AC-002.3 | Bats test: visible_width with Unicode strings | US-002 |

### Adversary Findings

| Dimension | R1 Score | R2 Score | Verdict |
|-----------|----------|----------|---------|
| Ambiguity | 85 | 80 | PASS |
| Edge Cases | 70 | 78 | PASS |
| Scope | 90 | 88 | PASS |
| Dependencies | 90 | 86 | PASS |
| Testability | 80 | 82 | PASS |
| Decisions | 75 | 85 | PASS |
| Rollback | 60 | 75 | PASS |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-02-statusline-unicode-fix/spec.md` | Full spec + Phase Summary |
