# Spec: Deploy Poweruser Statusline via ecc install (BL-053)

## Problem Statement

The current ECC statusline script (`statusline/statusline-command.sh`) displays only model name, context bar, git repo, git branch, directory, and ECC version. Power users need full situational awareness: session cost, duration, lines changed, rate limits, token counts, and more. The script also lacks git branch caching (slow in large repos) and terminal-width-aware truncation (overflows on narrow terminals). BL-053 upgrades the script to maximum information density with ANSI color coding, adds a `StatuslineConfig` domain type for configuration modeling, and adds an `ecc validate statusline` subcommand for installation verification.

## Research Summary

- **Claude Code JSON schema**: Documented fields include `model.display_name`, `context_window.used_percentage`, `cost.total_cost_usd`, `cost.total_duration_ms`, `cost.total_lines_added/removed`, `rate_limits.five_hour.used_percentage`. No `vim.mode` or `worktree` fields documented — graceful omission required.
- **Community tools**: ccstatusline (YAML config + TTL), ClaudeCodeStatusLine (git + timestamps), cc-statusline (zero deps). ECC ships its own curated script.
- **Git caching**: Use `find -newer` trick for cross-platform TTL check (macOS + Linux). Key cache by working directory hash.
- **jq dependency**: jq is bundled with Claude Code installations. Script checks for jq at startup and emits error if missing.
- **Truncation**: Field-priority-based dropping using `$COLUMNS` or `tput cols`. No partial clipping.
- **Performance**: Script must complete in <50ms. Git caching mandatory for large repos.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Keep existing filename `statusline-command.sh` | Existing `ensure_statusline()` detection logic and `STATUSLINE_SCRIPT_FILENAME` constant key on this name. Changing breaks installs. | No |
| 2 | Graceful omission for undocumented fields (vim, worktree) via jq `// ""` | Fields may not exist. Silently omit, no placeholder. Separator also omitted. | No |
| 3 | Git cache keyed by working directory hash, atomic writes | Prevents stale branch + race conditions. Use `mktemp` + `mv` for atomic write. | No |
| 4 | `find -newer` for cross-platform TTL | Avoids macOS vs Linux `stat` divergence | No |
| 5 | StatuslineConfig extends existing `statusline.rs` alongside StatusLineResult | Same module, same concern. Config models validation rules, not runtime script config. | No |
| 6 | jq is a runtime dependency, checked at script startup | jq is bundled with Claude Code. Script emits error if missing. Documented in install output. | No |
| 7 | ECC version baked in at install time via `__ECC_VERSION__` placeholder | Existing `prepare_script()` mechanism. No runtime `ecc version` subprocess needed. | No |
| 8 | All git commands use `--no-optional-locks` | Prevents blocking concurrent git operations | No |
| 9 | No ADR needed | Enhancement of existing infrastructure | No |

## User Stories

### US-001: Extend statusline shell script with full power-user field set

**As a** power user, **I want** the statusline to display all available session fields with ANSI color coding, **so that** I have maximum situational awareness.

#### Acceptance Criteria

- AC-001.1: Given stdin JSON, when the script runs, then it displays: model name, context bar (color-coded), cost ($X.XX), duration (Xm Ys), lines (+N/-N), git branch, rate limit %, token counts (In:Xk Out:Yk), ECC version
- AC-001.2: Given context usage, when <60% then bar is green, 60-79% yellow, 80%+ red
- AC-001.3: Given null/missing fields, when extracted by jq, then the segment AND its separator are silently omitted
- AC-001.4: Given session cost, when formatted, then it shows exactly 2 decimal places
- AC-001.5: Given token counts, when formatted, then they show in `k` units rounded to 1 decimal
- AC-001.6: Given the script source, when measured, then it is under 200 lines
- AC-001.7: Given the script, when it contains `__ECC_VERSION__`, then the placeholder is replaced by `ecc install` at install time (existing mechanism)
- AC-001.8: Given the script, when it starts, then it checks for `jq` availability and emits "Error: jq not found" if missing
- AC-001.9: Given empty stdin or malformed JSON, when the script runs, then it outputs a degraded status showing only "ECC" and does not crash
- AC-001.10: Given all git commands in the script, when they execute, then they use `--no-optional-locks`

#### Dependencies

- Depends on: none

### US-002: Git branch caching with TTL

**As a** power user in a large repo, **I want** git branch cached with 5s TTL, **so that** the statusline renders in under 50ms.

#### Acceptance Criteria

- AC-002.1: Given a fresh invocation, when no cache exists, then `git rev-parse --abbrev-ref HEAD --no-optional-locks` is called and result cached
- AC-002.2: Given a cached branch, when cache is < 5s old, then the cached value is used
- AC-002.3: Given the cache file, when named, then it includes a hash of the working directory (e.g., `/tmp/ecc-sl-cache-$(echo $PWD | md5sum | cut -c1-8)`)
- AC-002.4: Given macOS and Linux, when checking cache age, then `find -newer` or equivalent cross-platform method is used
- AC-002.5: Given cache read/write failure, when it occurs, then the script falls back to live `git rev-parse` silently
- AC-002.6: Given concurrent Claude sessions, when writing cache, then atomic write via `mktemp` + `mv` prevents corruption

#### Dependencies

- Depends on: none

### US-003: Terminal-width-aware truncation

**As a** power user with varying terminal widths, **I want** fields dropped by priority when the terminal is too narrow, **so that** output never overflows.

#### Acceptance Criteria

- AC-003.1: Given the script, when it starts, then it reads terminal width via `$COLUMNS` or `tput cols`
- AC-003.2: Given truncation needed, when fields are dropped, then lowest-priority first: ECC version, worktree, vim mode, rate limits, duration, token counts, lines, cost, git branch, context bar. Model is never dropped.
- AC-003.3: Given truncation, when fields are dropped, then no partial segments appear
- AC-003.4: Given the model field, when truncation occurs, then it is always displayed
- AC-003.5: Given minimum terminal width, when width < 40 characters, then only model name is shown

#### Dependencies

- Depends on: US-001

### US-004: StatuslineConfig domain type

**As a** maintainer, **I want** a `StatuslineConfig` value object in ecc-domain, **so that** statusline validation rules are testable.

#### Acceptance Criteria

- AC-004.1: Given `StatuslineConfig` struct in `statusline.rs`, when defined, then it has fields: `cache_ttl_secs: u32`, `context_thresholds: ContextThresholds`, `field_order: Vec<StatuslineField>`
- AC-004.2: Given `StatuslineField` enum, when defined, then it has variants for each field
- AC-004.3: Given `StatuslineConfig::default()`, when called, then it returns the standard power-user config
- AC-004.4: Given the struct, when it derives traits, then Debug, Clone, PartialEq, Eq
- AC-004.5: Given ecc-domain, when checked, then zero I/O imports
- AC-004.6: Given unit tests, when run, then default construction is verified
- AC-004.7: Given the existing `StatusLineResult` and `ensure_statusline()`, when StatuslineConfig is added, then existing types are unchanged

#### Dependencies

- Depends on: none

### US-005: `ecc validate statusline` subcommand

**As a** user, **I want** to verify my statusline installation, **so that** I can diagnose issues.

#### Acceptance Criteria

- AC-005.1: Given `ValidateTarget::Statusline`, when added, then `validate_statusline()` function exists in ecc-app
- AC-005.2: Given script exists at expected path, when validated, then PASS
- AC-005.3: Given script contains `__ECC_VERSION__`, when validated, then FAIL (unresolved placeholder)
- AC-005.4: Given `settings.json` has `statusLine.command` pointing to script, when validated, then PASS
- AC-005.5: Given script has valid shebang (`#!/usr/bin/env bash`), when validated, then PASS
- AC-005.6: Given CLI `ecc validate statusline`, when run, then it dispatches to `validate_statusline`
- AC-005.7: Given unit tests with InMemoryFileSystem, when run, then all pass/fail scenarios covered
- AC-005.8: Given script contains `jq`, when validated, then PASS (script uses jq, indicating it's the power-user version)

#### Dependencies

- Depends on: US-004

### US-006: Install integration

**As a** user running `ecc install`, **I want** the upgraded script deployed, **so that** upgrading ECC activates the improved statusline.

#### Acceptance Criteria

- AC-006.1: Given `ecc install`, when run, then the upgraded script is deployed via existing `ensure_statusline_in_settings`
- AC-006.2: Given existing custom statusline, when `ecc install` runs, then it is not overwritten (AlreadyCustom)
- AC-006.3: Given the deployed script, when checked with `bash -n`, then syntax is valid
- AC-006.4: Given integration tests, when run, then they verify new script contains power-user fields (e.g., `total_cost_usd`)

#### Dependencies

- Depends on: US-001, US-004

### US-007: Quality gate and documentation

**As a** maintainer, **I want** all tests and docs updated, **so that** the codebase remains clean.

#### Acceptance Criteria

- AC-007.1: Given `cargo test`, when run, then all tests pass
- AC-007.2: Given `cargo clippy -- -D warnings`, when run, then zero warnings
- AC-007.3: Given ecc-domain, when checked, then zero I/O imports
- AC-007.4: Given CLAUDE.md, when checked, then CLI reference and test count updated
- AC-007.5: Given glossary, when checked, then StatuslineConfig entry exists
- AC-007.6: Given CHANGELOG, when checked, then BL-053 entry exists

#### Dependencies

- Depends on: all

## Affected Modules

| Module | Layer | Nature of Change |
|--------|-------|-----------------|
| `statusline/statusline-command.sh` | Shell script | Rewrite: full field set, caching, truncation, jq check |
| `crates/ecc-domain/src/config/statusline.rs` | Domain | Extend: StatuslineConfig, ContextThresholds, StatuslineField |
| `crates/ecc-app/src/validate.rs` | App | Extend: ValidateTarget::Statusline, validate_statusline() |
| `crates/ecc-cli/src/commands/validate.rs` | CLI | Extend: CliValidateTarget::Statusline |
| `CLAUDE.md` | Docs | Update: CLI Commands, test count |
| `docs/domain/glossary.md` | Docs | Add: StatuslineConfig entry |
| `CHANGELOG.md` | Docs | Add: BL-053 entry |

## Constraints

- `cargo test` must pass
- `cargo clippy -- -D warnings` must pass
- ecc-domain zero I/O imports
- Shell script under 200 lines, rendering under 50ms
- Keep filename `statusline-command.sh`
- Custom user statuslines not overwritten
- Cross-platform: macOS + Linux
- jq is a runtime dependency (checked at startup)
- All git commands use `--no-optional-locks`

## Non-Requirements

- Custom themes or user-configurable field order
- Windows support
- Interactive statusline
- vim.mode or worktree fields guaranteed
- Runtime config file for the shell script
- StatuslineConfig driving script behavior at runtime
- Rollback/backup of previous script version (users can `ecc install` to restore)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| FileSystem port | No new methods | validate_statusline uses existing read/exists |
| CLI → App → Domain | New ValidateTarget variant | Standard pattern |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New domain concept | Domain | `docs/domain/glossary.md` | Add StatuslineConfig entry |
| Feature entry | Project | `CHANGELOG.md` | Add BL-053 entry |
| CLI reference | Reference | `CLAUDE.md` | Update CLI Commands + test count |

## Open Questions

None — all resolved during grill-me interview and adversarial review.
