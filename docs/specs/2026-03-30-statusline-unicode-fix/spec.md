# Spec: Fix statusline Unicode byte-counting bug (BL-076)

## Problem Statement

The statusline script (`statusline-command.sh`) uses `${#stripped}` to measure output width before deciding which segments to include. In bash, `${#var}` counts **bytes** when the locale is `C` or `POSIX` (common default in hooks/subprocesses). With a UTF-8 locale, `${#var}` correctly counts characters. Unicode characters used in the statusline (◆, ⎇, █, ░, ↑, ↓) are 3 bytes each, causing the script to think the line is ~2x wider than it actually is. This makes `build_output` stop adding segments early — rate limit bars (5h:, 7d:), duration, cost, and ECC version segments are silently dropped even when terminal width is sufficient.

## Research Summary

- `${#var}` in bash counts bytes when `LC_ALL=C` or with multi-byte UTF-8 strings
- `printf '%s' "$var" | wc -m` counts characters but spawns a subprocess
- Pure bash alternative: strip ANSI escape sequences first, then use `${#var}` with `LC_ALL=en_US.UTF-8` locale set
- The statusline already strips ANSI via `sed` — the locale is the remaining issue
- Bats test framework supports testing shell script functions

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Set `LC_ALL=C.UTF-8` at script top (once) | Ensures `${#var}` counts characters not bytes. `C.UTF-8` is more portable than `en_US.UTF-8`. | No |
| 2 | No helper function needed | With correct locale, `${#var}` works directly at all 4 sites | No |
| 3 | Check locale at startup, warn if `C.UTF-8` unavailable | One-time check, no subprocess per render | No |

## User Stories

### US-001: Fix Unicode width calculation

**As a** developer using Claude Code statusline, **I want** the statusline to correctly calculate visible character width of Unicode segments, **so that** rate limit bars and other segments appear when terminal width is sufficient.

#### Acceptance Criteria

- AC-001.1: Given a statusline with Unicode chars (◆, ⎇, █, ░) and COLUMNS=120, when build_output runs, then rate limit segments (5h:, 7d:) appear in the output
- AC-001.2: Given `LC_ALL=C.UTF-8` is set at script top, when `${#var}` is used on a string with multi-byte chars, then it returns the visible character count
- AC-001.3: Given a narrow terminal (COLUMNS=40), when build_output runs, then segments are still gracefully degraded (not all shown)
- AC-001.4: Given the script starts with `export LC_ALL=C.UTF-8`, when any of the 4 `${#stripped}`/`${#STRIPPED}` sites execute, then they count characters correctly (no code change needed at the sites)
- AC-001.5: Given existing Bats tests, when the fix is applied, then all 16 statusline tests still pass
- AC-001.6: Given `LC_ALL=C.UTF-8`, when `s="◆ main ⎇ spec"; echo ${#s}` runs, then it outputs 13 (visible chars, not 20 bytes)
- AC-001.7: Given `LC_ALL=C` (no UTF-8), when the script starts, then it detects the missing locale and sets `LC_ALL=C.UTF-8` before any width calculations

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `statusline/statusline-command.sh` | Shell script | Add `export LC_ALL=C.UTF-8` near top of script |
| `tests/statusline/*.bats` | Test | Add Unicode width test cases |

## Constraints

- No subprocess spawning in the hot path (statusline runs every few seconds)
- Must work in both bash and zsh (statusline is bash, but test for portability)
- Must not break existing 16 Bats tests
- Prefer locale-aware `${#var}` over `wc -m` subprocess

## Non-Requirements

- Changing the Unicode characters used in segments
- Terminal emulator compatibility testing (assume UTF-8 terminal)
- Changing segment priority order

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Shell script | Bug fix | Bats tests verify output |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Bug fix | CHANGELOG | CHANGELOG.md | Add entry under ### Fixed |

## Open Questions

None.
