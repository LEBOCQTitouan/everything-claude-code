# Solution: Deploy Poweruser Statusline via ecc install (BL-053)

## Spec Reference
Concern: dev, Feature: BL-053 deploy poweruser statusline via ecc install

## File Changes (dependency order)
| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/config/statusline.rs` | modify | Add StatuslineConfig, ContextThresholds, StatuslineField domain types | US-004, AC-004.1–004.7 |
| 2 | `crates/ecc-app/src/validate.rs` | modify | Add ValidateTarget::Statusline, validate_statusline() | US-005, AC-005.1–005.8 |
| 3 | `statusline/statusline-command.sh` | modify | Rewrite with full fields, caching, truncation, jq check | US-001–003, AC-001.1–003.5 |
| 4 | `crates/ecc-cli/src/commands/validate.rs` | modify | Add CliValidateTarget::Statusline, wire mapping | US-005, AC-005.6 |
| 5 | `crates/ecc-app/src/install/helpers/settings.rs` | modify | Update test fixture with power-user script content | US-006, AC-006.4 |
| 6 | `crates/ecc-app/src/install/global.rs` | modify | Add integration test for power-user content | US-006, AC-006.1, AC-006.4 |
| 7 | `CLAUDE.md` | modify | CLI Commands + test count | US-007, AC-007.4 |
| 8 | `docs/domain/glossary.md` | modify | StatuslineConfig entry | US-007, AC-007.5 |
| 9 | `CHANGELOG.md` | modify | BL-053 entry | US-007, AC-007.6 |

## Pass Conditions
| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | StatuslineConfig default construction | AC-004.1, AC-004.3, AC-004.6 | `cargo test -p ecc-domain statusline_config_default_construction` | pass |
| PC-002 | unit | StatuslineField enum variants | AC-004.2 | `cargo test -p ecc-domain statusline_field_variants` | pass |
| PC-003 | unit | StatuslineConfig default values (TTL=5, yellow=60, red=80) | AC-004.3 | `cargo test -p ecc-domain statusline_config_default_values` | pass |
| PC-004 | unit | StatuslineConfig derives (Debug, Clone, PartialEq, Eq) | AC-004.4 | `cargo test -p ecc-domain statusline_config_derives` | pass |
| PC-005 | lint | Domain zero I/O | AC-004.5, AC-007.3 | `! grep -rE 'std::fs\|std::process\|std::net\|tokio' crates/ecc-domain/src/` | exit 0 |
| PC-006 | unit | Existing ensure_statusline tests pass | AC-004.7 | `cargo test -p ecc-domain ensure_adds_to_empty_settings` | pass |
| PC-007 | unit | Existing prepare_script tests pass | AC-004.7 | `cargo test -p ecc-domain prepare_script_replaces_placeholder` | pass |
| PC-008 | unit | validate_statusline pass valid | AC-005.1, AC-005.2 | `cargo test -p ecc-app validate_statusline_pass_valid` | pass |
| PC-009 | unit | validate_statusline fail missing script | AC-005.2 | `cargo test -p ecc-app validate_statusline_fail_missing_script` | pass |
| PC-010 | unit | validate_statusline fail unresolved placeholder | AC-005.3 | `cargo test -p ecc-app validate_statusline_fail_unresolved_placeholder` | pass |
| PC-011 | unit | validate_statusline pass settings command | AC-005.4 | `cargo test -p ecc-app validate_statusline_pass_settings_command` | pass |
| PC-012 | unit | validate_statusline fail bad shebang | AC-005.5 | `cargo test -p ecc-app validate_statusline_fail_bad_shebang` | pass |
| PC-013 | unit | validate_statusline all tests pass | AC-005.7 | `cargo test -p ecc-app validate_statusline` | pass |
| PC-014 | unit | validate_statusline fail no jq | AC-005.8 | `cargo test -p ecc-app validate_statusline_fail_no_jq` | pass |
| PC-015 | unit | Script has total_cost_usd | AC-001.1 | `grep -c 'total_cost_usd' statusline/statusline-command.sh \| grep -v '^0$'` | exit 0 |
| PC-016 | unit | Script has total_duration_ms | AC-001.1 | `grep -c 'total_duration_ms' statusline/statusline-command.sh \| grep -v '^0$'` | exit 0 |
| PC-017 | unit | Script has total_lines_added | AC-001.1 | `grep -c 'total_lines_added' statusline/statusline-command.sh \| grep -v '^0$'` | exit 0 |
| PC-018 | unit | Script has rate limit field | AC-001.1 | `grep -c 'five_hour' statusline/statusline-command.sh \| grep -v '^0$'` | exit 0 |
| PC-019 | unit | Script has display_name | AC-001.1 | `grep -c 'display_name' statusline/statusline-command.sh \| grep -v '^0$'` | exit 0 |
| PC-020 | unit | Script has ANSI color codes | AC-001.2 | `grep -cE '\\\\033\[' statusline/statusline-command.sh \| grep -v '^0$'` | exit 0 |
| PC-021 | unit | Script has jq null fallback | AC-001.3 | `grep -c '// ""' statusline/statusline-command.sh \| grep -v '^0$'` | exit 0 |
| PC-022 | unit | Script has cost formatting | AC-001.4 | `grep -c 'printf.*%.2f' statusline/statusline-command.sh \| grep -v '^0$'` | exit 0 |
| PC-023 | unit | Script under 250 lines | AC-001.6 | `test $(wc -l < statusline/statusline-command.sh) -lt 250` | exit 0 |
| PC-024 | unit | Script has __ECC_VERSION__ | AC-001.7 | `grep -c '__ECC_VERSION__' statusline/statusline-command.sh \| grep -v '^0$'` | exit 0 |
| PC-025 | unit | Script checks for jq | AC-001.8 | `grep -c 'command -v jq' statusline/statusline-command.sh \| grep -v '^0$'` | exit 0 |
| PC-026 | unit | Script has degraded output | AC-001.9 | `grep -qi 'ECC' statusline/statusline-command.sh` | exit 0 |
| PC-027 | unit | Script has --no-optional-locks | AC-001.10 | `grep -c '\-\-no-optional-locks' statusline/statusline-command.sh \| grep -v '^0$'` | exit 0 |
| PC-028 | unit | Script has git rev-parse | AC-002.1 | `grep -c 'rev-parse' statusline/statusline-command.sh \| grep -v '^0$'` | exit 0 |
| PC-029 | unit | Script has cache hash | AC-002.3 | `grep -cE 'md5sum\|md5 ' statusline/statusline-command.sh \| grep -v '^0$'` | exit 0 |
| PC-030 | unit | Script has find -newer or TTL check | AC-002.4 | `grep -cE 'find.*-newer\|mtime\|cache.*age' statusline/statusline-command.sh \| grep -v '^0$'` | exit 0 |
| PC-031 | unit | Script has atomic write | AC-002.6 | `grep -c 'mktemp' statusline/statusline-command.sh \| grep -v '^0$'` | exit 0 |
| PC-032 | unit | Script has terminal width | AC-003.1 | `grep -cE 'COLUMNS\|tput cols' statusline/statusline-command.sh \| grep -v '^0$'` | exit 0 |
| PC-033 | unit | Script has model always / min width | AC-003.4, AC-003.5 | `grep -cE 'model.*always\|MIN_WIDTH\|width.*40' statusline/statusline-command.sh \| grep -v '^0$'` | exit 0 |
| PC-034 | unit | Script passes bash -n | AC-006.3 | `bash -n statusline/statusline-command.sh` | exit 0 |
| PC-035 | build | CLI builds with statusline validate | AC-005.6 | `cargo build -p ecc-cli` | exit 0 |
| PC-036 | unit | Install test: script installs and updates settings | AC-006.1 | `cargo test -p ecc-app statusline_installs_script_and_updates_settings` | pass |
| PC-037 | unit | Install test: custom not overwritten | AC-006.2 | `cargo test -p ecc-app statusline_does_not_overwrite_custom` | pass |
| PC-038 | unit | Install test: power-user content | AC-006.4 | `cargo test -p ecc-app install_deploys_poweruser_statusline` | pass |
| PC-039 | build | All tests pass | AC-007.1 | `cargo test` | pass |
| PC-040 | lint | Clippy clean | AC-007.2 | `cargo clippy -- -D warnings` | exit 0 |
| PC-041 | lint | Domain zero I/O | AC-007.3 | `! grep -rE 'std::fs\|std::process\|std::net\|tokio' crates/ecc-domain/src/` | exit 0 |
| PC-042 | unit | CLAUDE.md has validate statusline | AC-007.4 | `grep -q 'validate statusline' CLAUDE.md` | exit 0 |
| PC-043 | unit | Glossary has StatuslineConfig | AC-007.5 | `grep -q 'StatuslineConfig' docs/domain/glossary.md` | exit 0 |
| PC-044 | unit | CHANGELOG has BL-053 | AC-007.6 | `grep -q 'BL-053' CHANGELOG.md` | exit 0 |
| PC-045 | unit | Token format in k units | AC-001.5 | `grep -qi 'token\|input_tokens\|output_tokens' statusline/statusline-command.sh` | exit 0 |

| PC-046 | integration | Script outputs formatted cost from JSON | AC-001.4, AC-001.5 | `echo '{"model":{"display_name":"Opus"},"cost":{"total_cost_usd":1.23},"context_window":{"used_percentage":50}}' \| bash statusline/statusline-command.sh 2>/dev/null \| grep -q '1.23'` | exit 0 |
| PC-047 | integration | Script omits null segments | AC-001.3 | `echo '{"model":{"display_name":"Opus"},"context_window":{"used_percentage":50}}' \| bash statusline/statusline-command.sh 2>/dev/null \| grep -qv 'null'` | exit 0 |
| PC-048 | integration | Script shows only model at width 40 | AC-003.4, AC-003.5 | `echo '{"model":{"display_name":"Opus"},"context_window":{"used_percentage":50}}' \| COLUMNS=40 bash statusline/statusline-command.sh 2>/dev/null \| grep -q 'Opus'` | exit 0 |
| PC-049 | integration | Script falls back to live git on cache failure | AC-002.5 | `echo '{"model":{"display_name":"Opus"}}' \| TMPDIR=/nonexistent bash statusline/statusline-command.sh 2>/dev/null; test $? -eq 0` | exit 0 |

### Coverage Check
All 37 ACs covered by 49 PCs. No uncovered ACs.

### Design Clarifications (from adversarial review)

**Cross-platform md5 strategy**: Script uses `md5sum 2>/dev/null || md5 -q` to handle Linux vs macOS. The cache key computation falls back gracefully.

**200-line budget**: Raised to 250 lines to accommodate caching + truncation + full fields. If the script approaches 250, consider extracting a `_statusline_helpers.sh` sourced file.

**Bad shebang in PC-012**: A "bad shebang" means any first line that is not `#!/usr/bin/env bash` or `#!/bin/bash` (e.g., `#!/usr/bin/python`, missing shebang, or `#!/bin/sh`).

**find -newer TTL**: The script creates a sentinel file at cache write time. On next invocation, `find $CACHE_FILE -newer $SENTINEL_FILE` determines freshness. If `find` is unavailable, falls back to live git.

### E2E Test Plan
No E2E tests needed — follows established validate pattern.

### E2E Activation Rules
No E2E tests to activate.

## Test Strategy
TDD order:
1. **Phase 1 (PC-001–007)**: Domain types — StatuslineConfig, ContextThresholds, StatuslineField
2. **Phase 2 (PC-008–014)**: App validate — validate_statusline() with InMemoryFileSystem
3. **Phase 3 (PC-015–034)**: Shell script rewrite — full fields, caching, truncation
4. **Phase 4 (PC-035)**: CLI wiring — CliValidateTarget::Statusline
5. **Phase 5 (PC-036–038)**: Install integration tests
6. **Phase 6 (PC-039–045)**: Docs + quality gate

## Doc Update Plan
| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/domain/glossary.md` | Domain | Add entry | StatuslineConfig definition | AC-007.5 |
| 2 | `CHANGELOG.md` | Project | Add entry | BL-053 feature | AC-007.6 |
| 3 | `CLAUDE.md` | Reference | Update | CLI Commands + test count | AC-007.4 |

## SOLID Assessment
PASS. SRP clean (config in domain, validation in app, wiring in CLI). OCP (existing types unchanged). DIP (domain zero I/O).

## Robert's Oath Check
CLEAN. Read-only script + pure domain types. 45 PCs with TDD. 6 atomic phases.

## Security Notes
CLEAR. Trusted stdin JSON, no network, no secrets, jq checked, git uses --no-optional-locks.

## Rollback Plan
1. Revert `CHANGELOG.md`
2. Revert `docs/domain/glossary.md`
3. Revert `CLAUDE.md`
4. Revert `crates/ecc-app/src/install/global.rs`
5. Revert `crates/ecc-app/src/install/helpers/settings.rs`
6. Revert `crates/ecc-cli/src/commands/validate.rs`
7. Revert `crates/ecc-app/src/validate.rs`
8. Revert `crates/ecc-domain/src/config/statusline.rs`
9. Revert `statusline/statusline-command.sh`
