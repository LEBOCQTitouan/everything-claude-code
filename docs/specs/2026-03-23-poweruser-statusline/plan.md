# Implementation Plan: BL-053 — Deploy Poweruser Statusline via ecc install

## Overview

Rewrite `statusline-command.sh` with full power-user fields (cost, duration, lines, rate limits, tokens), git branch caching, and terminal-width truncation. Add `StatuslineConfig` domain types in `ecc-domain`, a `validate_statusline()` use case in `ecc-app`, and wire `ecc validate statusline` in `ecc-cli`. Update docs (CLAUDE.md, glossary, CHANGELOG).

## File Changes Table (dependency order)

| # | File | Layer | Action | Depends On |
|---|------|-------|--------|------------|
| 1 | `statusline/statusline-command.sh` | Shell | Rewrite: full fields, caching, truncation, jq check | None |
| 2 | `crates/ecc-domain/src/config/statusline.rs` | Entity | Extend: add `StatuslineConfig`, `ContextThresholds`, `StatuslineField` | None |
| 3 | `crates/ecc-app/src/validate.rs` | UseCase | Extend: add `ValidateTarget::Statusline`, `validate_statusline()` | #2 |
| 4 | `crates/ecc-cli/src/commands/validate.rs` | Adapter | Extend: add `CliValidateTarget::Statusline`, wire mapping | #3 |
| 5 | `crates/ecc-app/src/install/helpers/settings.rs` | UseCase | Update test fixture: source script must contain power-user markers | #1 |
| 6 | `crates/ecc-app/src/install/global.rs` | UseCase | Update integration test: verify deployed script contains `total_cost_usd` | #1 |
| 7 | `CLAUDE.md` | Docs | Update test count, add `ecc validate statusline` to CLI reference | #1-#4 |
| 8 | `docs/domain/glossary.md` | Docs | Add `StatuslineConfig` entry | #2 |
| 9 | `CHANGELOG.md` | Docs | Add BL-053 entry | All |

## Implementation Steps

### Phase 1: Shell Script Rewrite (US-001, US-002, US-003)

Layers: [Framework]

1. **Rewrite `statusline/statusline-command.sh`** (File: `statusline/statusline-command.sh`)
   - Action: Replace the 53-line script with the power-user version (under 200 lines) implementing:
     - jq availability check at startup (emit "Error: jq not found" and exit 1)
     - Degraded output on empty stdin / malformed JSON (show only "ECC")
     - Fields: model name (bold), context bar (color-coded: green <60%, yellow 60-79%, red 80+%), cost ($X.XX), duration (Xm Ys), lines (+N/-N), git branch, rate limit %, token counts (In:Xk Out:Yk), ECC version
     - Null/missing fields silently omitted (segment AND separator)
     - `__ECC_VERSION__` placeholder preserved for install-time replacement
     - Git branch caching: `/tmp/ecc-sl-cache-$(echo $PWD | md5sum | cut -c1-8)`, 5s TTL via `find -newer`, atomic write via `mktemp` + `mv`, fallback to live `git rev-parse` on failure
     - All git commands use `--no-optional-locks`
     - Terminal-width truncation: read `$COLUMNS` or `tput cols`, drop fields lowest-priority-first (ECC version, worktree, vim mode, rate limits, duration, token counts, lines, cost, git branch, context bar), model never dropped, width <40 shows model only, no partial segments
   - Why: Core deliverable — all shell ACs (001.1-001.10, 002.1-002.6, 003.1-003.5)
   - Dependencies: None
   - Risk: Medium — cross-platform shell (macOS `md5` vs Linux `md5sum`)

#### Pass Conditions for Phase 1

| PC | AC(s) | Command | Expected |
|----|-------|---------|----------|
| PC-1.01 | 001.1 | `grep -c 'total_cost_usd' statusline/statusline-command.sh` | `1` or more |
| PC-1.02 | 001.1 | `grep -c 'total_duration_ms' statusline/statusline-command.sh` | `1` or more |
| PC-1.03 | 001.1 | `grep -c 'total_lines_added' statusline/statusline-command.sh` | `1` or more |
| PC-1.04 | 001.1 | `grep -c 'five_hour' statusline/statusline-command.sh` | `1` or more |
| PC-1.05 | 001.1 | `grep -c 'display_name' statusline/statusline-command.sh` | `1` or more |
| PC-1.06 | 001.2 | `grep -cE '\\\\033\[3[12]m' statusline/statusline-command.sh` | `1` or more (green/yellow/red ANSI) |
| PC-1.07 | 001.3 | `grep -c '// ""' statusline/statusline-command.sh` | `1` or more (jq fallback) |
| PC-1.08 | 001.4 | `grep -c 'printf.*%.2f' statusline/statusline-command.sh` | `1` or more |
| PC-1.09 | 001.6 | `test $(wc -l < statusline/statusline-command.sh) -lt 200` | exit 0 |
| PC-1.10 | 001.7 | `grep -c '__ECC_VERSION__' statusline/statusline-command.sh` | `1` or more |
| PC-1.11 | 001.8 | `grep -c 'jq not found\|command -v jq' statusline/statusline-command.sh` | `1` or more |
| PC-1.12 | 001.9 | `grep -cE 'ECC.*degraded\|fallback.*ECC\|echo.*ECC' statusline/statusline-command.sh` | `1` or more |
| PC-1.13 | 001.10 | `grep -c '\-\-no-optional-locks' statusline/statusline-command.sh` | equals count of git commands |
| PC-1.14 | 002.1 | `grep -c 'rev-parse.*--abbrev-ref.*HEAD' statusline/statusline-command.sh` | `1` or more |
| PC-1.15 | 002.3 | `grep -cE 'md5sum\|md5 ' statusline/statusline-command.sh` | `1` or more |
| PC-1.16 | 002.4 | `grep -c 'find.*-newer' statusline/statusline-command.sh` | `1` or more |
| PC-1.17 | 002.6 | `grep -c 'mktemp' statusline/statusline-command.sh` | `1` or more |
| PC-1.18 | 003.1 | `grep -cE 'COLUMNS\|tput cols' statusline/statusline-command.sh` | `1` or more |
| PC-1.19 | 003.4, 003.5 | `grep -cE 'model.*always\|width.*40\|MIN_WIDTH' statusline/statusline-command.sh` | `1` or more |
| PC-1.20 | 006.3 | `bash -n statusline/statusline-command.sh` | exit 0 |

#### Test Targets for Phase 1

- **Unit tests**: None (shell script, validated by grep PCs and `bash -n`)
- **Edge cases**: empty stdin, missing jq, narrow terminal (<40 cols)
- **Expected test file**: N/A (shell — verified via PCs)

---

### Phase 2: Domain Types (US-004)

Layers: [Entity]

1. **Add `StatuslineConfig`, `ContextThresholds`, `StatuslineField`** (File: `crates/ecc-domain/src/config/statusline.rs`)
   - Action: Below existing `prepare_script`, add:
     - `StatuslineField` enum: `Model`, `ContextBar`, `Cost`, `Duration`, `LinesChanged`, `GitBranch`, `RateLimit`, `TokenCounts`, `EccVersion`, `Worktree`, `VimMode` — derive `Debug, Clone, Copy, PartialEq, Eq`
     - `ContextThresholds` struct: `{ yellow_pct: u32, red_pct: u32 }` — derive `Debug, Clone, PartialEq, Eq`
     - `StatuslineConfig` struct: `{ cache_ttl_secs: u32, context_thresholds: ContextThresholds, field_order: Vec<StatuslineField> }` — derive `Debug, Clone, PartialEq, Eq`
     - `impl Default for StatuslineConfig` returning standard power-user config (5s TTL, yellow=60, red=80, full field order matching truncation priority)
   - Why: Testable domain model for validation rules (AC-004.1 through 004.7)
   - Dependencies: None
   - Risk: Low — pure types, no I/O

#### Pass Conditions for Phase 2

| PC | AC(s) | Command | Expected |
|----|-------|---------|----------|
| PC-2.01 | 004.1 | `cargo test -p ecc-domain statusline_config_default_construction` | PASS |
| PC-2.02 | 004.2 | `cargo test -p ecc-domain statusline_field_variants` | PASS |
| PC-2.03 | 004.3 | `cargo test -p ecc-domain statusline_config_default_values` | PASS |
| PC-2.04 | 004.4 | `cargo test -p ecc-domain statusline_config_derives` | PASS |
| PC-2.05 | 004.5, 007.3 | `! grep -rE 'std::fs\|std::process\|std::net\|tokio' crates/ecc-domain/src/` | exit 0 |
| PC-2.06 | 004.7 | `cargo test -p ecc-domain ensure_adds_to_empty_settings` | PASS (existing test still passes) |
| PC-2.07 | 004.7 | `cargo test -p ecc-domain prepare_script_replaces_placeholder` | PASS (existing test still passes) |

#### Test Targets for Phase 2

- **Interfaces to scaffold**: `StatuslineField`, `ContextThresholds`, `StatuslineConfig` in `crates/ecc-domain/src/config/statusline.rs`
- **Unit tests**: default construction verified (field_order length, cache_ttl_secs=5, yellow=60, red=80), Clone/PartialEq via assert_eq on two defaults, all field variants present
- **Edge cases**: ensure existing `StatusLineResult`, `ensure_statusline`, `prepare_script` unchanged
- **Expected test file**: `crates/ecc-domain/src/config/statusline.rs` (inline `#[cfg(test)]` block, extending existing)

#### Boy Scout Delta

Scan `statusline.rs` — rename the inconsistent casing `StatusLineResult` to... actually, leave it as-is since changing it would break downstream. Instead, look for a TODO/FIXME or unused import in nearby domain files.

---

### Phase 3: Validate Statusline Use Case (US-005)

Layers: [UseCase]

1. **Add `ValidateTarget::Statusline` and `validate_statusline()`** (File: `crates/ecc-app/src/validate.rs`)
   - Action:
     - Add `Statusline` variant to `ValidateTarget` enum
     - Add arm in `run_validate` match: `ValidateTarget::Statusline => validate_statusline(root, fs, terminal)`
     - Implement `validate_statusline()` checking:
       1. Script exists at `root/statusline/statusline-command.sh` (or deployed path — use `~/.claude/statusline-command.sh`). For validation, check `root` joined with the statusline filename.
       2. Script does NOT contain `__ECC_VERSION__` (unresolved placeholder = FAIL)
       3. Script starts with `#!/usr/bin/env bash` (valid shebang)
       4. Script contains `jq` (confirms power-user version)
       5. Read `settings.json` at `root/settings.json`, check `statusLine.command` points to a path containing `statusline-command.sh`
     - Print pass/fail per check
   - Why: AC-005.1 through 005.8
   - Dependencies: Phase 2 (conceptually, though the function itself just uses FileSystem port)
   - Risk: Low — follows existing validate pattern exactly

#### Pass Conditions for Phase 3

| PC | AC(s) | Command | Expected |
|----|-------|---------|----------|
| PC-3.01 | 005.1 | `cargo test -p ecc-app validate_statusline_pass_valid` | PASS |
| PC-3.02 | 005.2 | `cargo test -p ecc-app validate_statusline_fail_missing_script` | PASS |
| PC-3.03 | 005.3 | `cargo test -p ecc-app validate_statusline_fail_unresolved_placeholder` | PASS |
| PC-3.04 | 005.4 | `cargo test -p ecc-app validate_statusline_pass_settings_command` | PASS |
| PC-3.05 | 005.5 | `cargo test -p ecc-app validate_statusline_fail_bad_shebang` | PASS |
| PC-3.06 | 005.7 | `cargo test -p ecc-app validate_statusline` | all statusline tests PASS |
| PC-3.07 | 005.8 | `cargo test -p ecc-app validate_statusline_fail_no_jq` | PASS |

#### Test Targets for Phase 3

- **Unit tests**: valid script passes all checks; missing script fails; script with `__ECC_VERSION__` fails; script without shebang fails; script without `jq` fails; settings.json missing `statusLine.command` fails; settings.json with correct command passes
- **Integration tests**: N/A (uses InMemoryFileSystem)
- **Edge cases**: empty script file, settings.json is malformed JSON, script exists but settings.json missing
- **Expected test file**: `crates/ecc-app/src/validate.rs` (inline `#[cfg(test)]`, extending existing block)

---

### Phase 4: CLI Wiring (US-005.6)

Layers: [Adapter]

1. **Add `CliValidateTarget::Statusline`** (File: `crates/ecc-cli/src/commands/validate.rs`)
   - Action:
     - Add `Statusline` variant to `CliValidateTarget` enum with doc comment `/// Validate statusline installation`
     - Add mapping arm: `CliValidateTarget::Statusline => ecc_app::validate::ValidateTarget::Statusline`
   - Why: AC-005.6 — `ecc validate statusline` dispatches correctly
   - Dependencies: Phase 3
   - Risk: Low — mechanical wiring

#### Pass Conditions for Phase 4

| PC | AC(s) | Command | Expected |
|----|-------|---------|----------|
| PC-4.01 | 005.6 | `cargo build --release 2>&1` | exit 0 |
| PC-4.02 | 005.6 | `cargo run -- validate statusline --ecc-root . 2>&1; true` | runs without panic (may fail validation, that's OK) |

#### Test Targets for Phase 4

- **Unit tests**: N/A (thin wiring, tested via build + manual invocation)
- **Expected test file**: N/A

---

### Phase 5: Install Integration Tests (US-006)

Layers: [UseCase]

1. **Update install test fixtures** (File: `crates/ecc-app/src/install/helpers/settings.rs`)
   - Action: Update `statusline_source_fs()` fixture to use realistic power-user script content containing `total_cost_usd`, `jq`, shebang, etc. Add test verifying deployed script contains power-user markers.
   - Why: AC-006.4 — integration tests verify new script contains power-user fields
   - Dependencies: Phase 1
   - Risk: Low

2. **Add install integration test for power-user content** (File: `crates/ecc-app/src/install/global.rs`)
   - Action: Add test `install_deploys_poweruser_statusline` that verifies the installed script contains `total_cost_usd` after `install_global`.
   - Why: AC-006.1, 006.4
   - Dependencies: Phase 1
   - Risk: Low

#### Pass Conditions for Phase 5

| PC | AC(s) | Command | Expected |
|----|-------|---------|----------|
| PC-5.01 | 006.1 | `cargo test -p ecc-app statusline_installs_script_and_updates_settings` | PASS |
| PC-5.02 | 006.2 | `cargo test -p ecc-app statusline_does_not_overwrite_custom` | PASS |
| PC-5.03 | 006.4 | `cargo test -p ecc-app install_deploys_poweruser_statusline` | PASS |

#### Test Targets for Phase 5

- **Integration tests**: verify deployed script has `total_cost_usd`, verify AlreadyCustom preserves user script
- **Expected test file**: `crates/ecc-app/src/install/global.rs` (inline tests, extending existing block)

---

### Phase 6: Quality Gate and Documentation (US-007)

Layers: [Entity, Adapter]

1. **Update CLAUDE.md** (File: `CLAUDE.md`)
   - Action: Add `ecc validate statusline` to CLI Commands table. Update test count from 1185 to new count.
   - Why: AC-007.4
   - Dependencies: All phases complete
   - Risk: Low

2. **Add glossary entry** (File: `docs/domain/glossary.md`)
   - Action: Add `StatuslineConfig` entry describing the domain value object for statusline validation configuration.
   - Why: AC-007.5
   - Dependencies: Phase 2
   - Risk: Low

3. **Add CHANGELOG entry** (File: `CHANGELOG.md`)
   - Action: Add BL-053 entry under current version section.
   - Why: AC-007.6
   - Dependencies: All phases
   - Risk: Low

#### Pass Conditions for Phase 6

| PC | AC(s) | Command | Expected |
|----|-------|---------|----------|
| PC-6.01 | 007.1 | `cargo test` | exit 0, all tests pass |
| PC-6.02 | 007.2 | `cargo clippy -- -D warnings` | exit 0 |
| PC-6.03 | 007.3 | `! grep -rE 'std::fs\|std::process\|std::net\|tokio' crates/ecc-domain/src/` | exit 0 |
| PC-6.04 | 007.4 | `grep -c 'validate statusline\|validate.*statusline' CLAUDE.md` | `1` or more |
| PC-6.05 | 007.5 | `grep -c 'StatuslineConfig' docs/domain/glossary.md` | `1` or more |
| PC-6.06 | 007.6 | `grep -c 'BL-053' CHANGELOG.md` | `1` or more |

---

## Full AC-to-PC Traceability

| AC | PC(s) |
|----|-------|
| 001.1 | PC-1.01, PC-1.02, PC-1.03, PC-1.04, PC-1.05 |
| 001.2 | PC-1.06 |
| 001.3 | PC-1.07 |
| 001.4 | PC-1.08 |
| 001.5 | PC-1.01 (token field presence implies formatting logic) |
| 001.6 | PC-1.09 |
| 001.7 | PC-1.10 |
| 001.8 | PC-1.11 |
| 001.9 | PC-1.12 |
| 001.10 | PC-1.13 |
| 002.1 | PC-1.14 |
| 002.2 | PC-1.16 (TTL via find -newer) |
| 002.3 | PC-1.15 |
| 002.4 | PC-1.16 |
| 002.5 | PC-1.14 (fallback is live git rev-parse) |
| 002.6 | PC-1.17 |
| 003.1 | PC-1.18 |
| 003.2 | PC-1.18 (truncation logic presence) |
| 003.3 | PC-1.18 |
| 003.4 | PC-1.19 |
| 003.5 | PC-1.19 |
| 004.1 | PC-2.01, PC-2.03 |
| 004.2 | PC-2.02 |
| 004.3 | PC-2.03 |
| 004.4 | PC-2.04 |
| 004.5 | PC-2.05 |
| 004.6 | PC-2.01 |
| 004.7 | PC-2.06, PC-2.07 |
| 005.1 | PC-3.01 |
| 005.2 | PC-3.01 |
| 005.3 | PC-3.03 |
| 005.4 | PC-3.04 |
| 005.5 | PC-3.05 |
| 005.6 | PC-4.01, PC-4.02 |
| 005.7 | PC-3.06 |
| 005.8 | PC-3.07 |
| 006.1 | PC-5.01 |
| 006.2 | PC-5.02 |
| 006.3 | PC-1.20 |
| 006.4 | PC-5.03 |
| 007.1 | PC-6.01 |
| 007.2 | PC-6.02 |
| 007.3 | PC-6.03 |
| 007.4 | PC-6.04 |
| 007.5 | PC-6.05 |
| 007.6 | PC-6.06 |

## E2E Assessment

- **Touches user-facing flows?** Yes — `ecc validate statusline` is a new CLI subcommand
- **Crosses 3+ modules end-to-end?** Yes — CLI -> App -> Domain -> FileSystem
- **New E2E tests needed?** No — the validate subcommand follows an established pattern already covered by the existing test suite. The shell script is validated by grep-based PCs and `bash -n`. The install integration tests in ecc-app cover the deployment path. Run existing suite as gate.

## Test Strategy (TDD Order)

1. **Phase 2 (Domain)**: Write failing tests for `StatuslineConfig::default()`, field enum variants, derive checks. Then implement types. Then refactor.
2. **Phase 3 (App validate)**: Write failing tests for `validate_statusline()` (all pass/fail scenarios with InMemoryFileSystem). Then implement. Then refactor.
3. **Phase 1 (Shell)**: Write the script (no TDD for shell — validate via PCs). Run `bash -n` and all grep PCs.
4. **Phase 4 (CLI)**: Build and smoke-test `ecc validate statusline`.
5. **Phase 5 (Install integration)**: Write failing test `install_deploys_poweruser_statusline`. Update fixture. Run.
6. **Phase 6 (Docs + gate)**: Update docs, run full `cargo test`, `cargo clippy -- -D warnings`, verify all PCs.

## Commit Cadence

| Phase | Commits |
|-------|---------|
| 2 | `test: add StatuslineConfig domain tests (RED)` → `feat: add StatuslineConfig domain types (GREEN)` → `refactor: improve statusline domain (REFACTOR)` |
| 3 | `test: add validate_statusline tests (RED)` → `feat: implement validate_statusline (GREEN)` → `refactor: improve validate_statusline (REFACTOR)` |
| 1 | `feat: rewrite statusline-command.sh with power-user fields` |
| 4 | `feat: wire ecc validate statusline CLI subcommand` |
| 5 | `test: add install integration test for power-user statusline` |
| 6 | `docs: update CLAUDE.md, glossary, CHANGELOG for BL-053` |

## Risks & Mitigations

- **Risk**: macOS uses `md5` not `md5sum` for cache key hashing
  - Mitigation: Script detects platform: `command -v md5sum >/dev/null && md5sum || md5`
- **Risk**: `$COLUMNS` not set in non-interactive shells
  - Mitigation: Fall back to `tput cols 2>/dev/null || echo 120`
- **Risk**: jq not available in some environments
  - Mitigation: Script checks at startup, emits error, exits gracefully with degraded output
- **Risk**: Shell script exceeds 200-line limit with all features
  - Mitigation: Use compact shell idioms, collapse trivial logic into single lines

## Success Criteria

- [ ] All 37 ACs covered by at least one PC (see traceability table)
- [ ] `bash -n statusline/statusline-command.sh` passes
- [ ] Script is under 200 lines
- [ ] `cargo test` passes (all crates)
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `ecc-domain` has zero I/O imports
- [ ] `ecc validate statusline` runs without panic
- [ ] CLAUDE.md updated with new test count and CLI reference
- [ ] Glossary has `StatuslineConfig` entry
- [ ] CHANGELOG has BL-053 entry
