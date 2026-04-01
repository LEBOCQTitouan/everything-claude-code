# Implementation Complete: BL-088 — ecc update Dual-Mode Deploy

## Spec Reference
Concern: dev, Feature: BL-088: ecc update dual-mode deploy

## Changes Made
| # | File | Action | Solution Ref | Tests | Status |
|---|------|--------|--------------|-------|--------|
| 1 | crates/ecc-domain/src/update/platform.rs | create | PC-001 | platform tests (6) | done |
| 2 | crates/ecc-domain/src/update/artifact.rs | modify | PC-002-005 | artifact tests (11) | done |
| 3 | crates/ecc-domain/src/update/error.rs | modify | PC-006 | error_display tests (6) | done |
| 4 | crates/ecc-domain/src/update/mod.rs | modify | PC-001 | — | done |
| 5 | crates/ecc-ports/src/env.rs | modify | PC-007,008 | re-export + current_exe tests | done |
| 6 | crates/ecc-ports/src/extract.rs | create | PC-010 | mock_extractor tests | done |
| 7 | crates/ecc-ports/src/release.rs | modify | PC-045 | compile check | done |
| 8 | crates/ecc-ports/src/lib.rs | modify | — | — | done |
| 9 | crates/ecc-test-support/src/mock_env.rs | modify | PC-009 | mock_env tests (3) | done |
| 10 | crates/ecc-test-support/src/mock_extractor.rs | create | PC-010 | mock_extractor tests (2) | done |
| 11 | crates/ecc-test-support/src/mock_release_client.rs | modify | PC-045 | compile check | done |
| 12 | crates/ecc-test-support/src/lib.rs | modify | — | — | done |
| 13 | crates/ecc-app/src/update/swap.rs | modify | PC-011-014 | rollback/cleanup/windows tests (4) | done |
| 14 | crates/ecc-app/src/update/orchestrator.rs | modify | PC-015-024 | orchestrator tests (10+) | done |
| 15 | crates/ecc-app/src/update/orchestrator_tests.rs | create | PC-015-024 | extracted tests | done |
| 16 | crates/ecc-app/src/update/context.rs | modify | PC-017,019 | — | done |
| 17 | crates/ecc-infra/src/github_release.rs | modify | PC-025-031,034-035 | 12 infra tests | done |
| 18 | crates/ecc-infra/src/tarball_extractor.rs | create | PC-021,032-033 | 3 extraction tests | done |
| 19 | crates/ecc-infra/src/os_env.rs | modify | PC-036 | os_env_current_exe | done |
| 20 | crates/ecc-infra/src/lib.rs | modify | — | — | done |
| 21 | crates/ecc-infra/Cargo.toml | modify | — | — | done |
| 22 | crates/ecc-cli/src/commands/update.rs | modify | PC-040-041 | — | done |
| 23 | xtask/src/deploy.rs | modify | PC-037-039 | 3 deploy tests | done |
| 24 | .github/workflows/release.yml | modify | PC-042 | grep check | done |
| 25 | Cargo.toml | modify | — | — | done |

## TDD Log
| PC ID | RED | GREEN | REFACTOR | Notes |
|-------|-----|-------|----------|-------|
| PC-001-006 | ✅ 19 compile errors | ✅ 993 domain tests pass | ⏭ clean | batched domain enums |
| PC-007 | ✅ type mismatch | ✅ 13 port tests pass | ⏭ clean | — |
| PC-008 | ✅ no method current_exe | ✅ build passes | ⏭ clean | — |
| PC-009 | ✅ E0046 missing impl | ✅ 3 mock_env tests | ⏭ clean | — |
| PC-010 | ✅ unresolved import | ✅ 2 mock_extractor tests | ⏭ clean | — |
| PC-045 | ✅ no method download_file | ✅ build passes | ⏭ clean | — |
| PC-011-014 | ✅ function not found | ✅ 4 swap tests pass | ⏭ clean | batched swap |
| PC-015-024 | ✅ missing context fields | ✅ all 10 commands pass | ⏭ clean | batched orchestrator |
| PC-025-036 | ✅ missing helper funcs | ✅ 62 infra tests pass | ⏭ clean | batched infra |
| PC-037-042 | ✅ missing functions | ✅ all 6 commands pass | ⏭ clean | batched CLI/xtask/CI |
| PC-043 | — | ✅ clippy zero warnings | — | final gate |
| PC-044 | — | ✅ release build succeeds | — | final gate |

## Pass Condition Results
| PC ID | Command | Expected | Actual | Status |
|-------|---------|----------|--------|--------|
| PC-001 | `cargo test -p ecc-domain platform` | PASS | PASS | ✅ |
| PC-002 | `cargo test -p ecc-domain resolves_macos_arm64` | PASS | PASS | ✅ |
| PC-003 | `cargo test -p ecc-domain artifact` | PASS | PASS | ✅ |
| PC-004 | `cargo test -p ecc-domain rejects_unsupported` | PASS | PASS | ✅ |
| PC-005 | `cargo test -p ecc-domain artifact` | PASS | PASS | ✅ |
| PC-006 | `cargo test -p ecc-domain error_display` | PASS | PASS | ✅ |
| PC-007 | `cargo test -p ecc-ports` | PASS | PASS | ✅ |
| PC-008 | `cargo build -p ecc-ports` | exit 0 | exit 0 | ✅ |
| PC-009 | `cargo test -p ecc-test-support mock_env` | PASS | PASS | ✅ |
| PC-010 | `cargo test -p ecc-test-support mock_extractor` | PASS | PASS | ✅ |
| PC-045 | `cargo build -p ecc-ports && cargo build -p ecc-test-support` | exit 0 | exit 0 | ✅ |
| PC-011 | `cargo test -p ecc-app rollback_swapped_restores` | PASS | PASS | ✅ |
| PC-012 | `cargo test -p ecc-app rollback_both_failures` | PASS | PASS | ✅ |
| PC-013 | `cargo test -p ecc-app cleanup_backups` | PASS | PASS | ✅ |
| PC-014 | `cargo test -p ecc-app windows_swap` | PASS | PASS | ✅ |
| PC-015 | `cargo test -p ecc-app full_upgrade_flow` | PASS | PASS | ✅ |
| PC-016 | `cargo test -p ecc-app permission_denied_before_download` | PASS | PASS | ✅ |
| PC-017 | `cargo test -p ecc-app update_locked` | PASS | PASS | ✅ |
| PC-018 | `cargo test -p ecc-app lock_released` | PASS | PASS | ✅ |
| PC-019 | `cargo test -p ecc-app full_upgrade_flow` | PASS | PASS | ✅ |
| PC-020 | `cargo test -p ecc-app corrupt_archive` | PASS | PASS | ✅ |
| PC-021 | `cargo test -p ecc-infra zip_slip_prevention` | PASS | PASS | ✅ |
| PC-022 | `cargo test -p ecc-app config_sync_triggers_rollback` | PASS | PASS | ✅ |
| PC-023 | `cargo test -p ecc-app cosign_not_installed_aborts` | PASS | PASS | ✅ |
| PC-024 | `cargo test -p ecc-app cosign_failed_aborts` | PASS | PASS | ✅ |
| PC-025 | `cargo test -p ecc-infra parse_latest_release` | PASS | PASS | ✅ |
| PC-026 | `cargo test -p ecc-infra get_specific_version` | PASS | PASS | ✅ |
| PC-027 | `cargo test -p ecc-infra streaming_download` | PASS | PASS | ✅ |
| PC-028 | `cargo test -p ecc-infra checksum_verification` | PASS | PASS | ✅ |
| PC-029 | `cargo test -p ecc-infra network_error` | PASS | PASS | ✅ |
| PC-030 | `cargo test -p ecc-infra rate_limited` | PASS | PASS | ✅ |
| PC-031 | `cargo test -p ecc-infra github_token_auth` | PASS | PASS | ✅ |
| PC-032 | `cargo test -p ecc-infra extract_valid_tarball` | PASS | PASS | ✅ |
| PC-033 | `cargo test -p ecc-infra windows_exe_preserved` | PASS | PASS | ✅ |
| PC-034 | `cargo test -p ecc-infra cosign_verify_bundle` | PASS | PASS | ✅ |
| PC-035 | `cargo test -p ecc-infra certificate_identity_constant` | PASS | PASS | ✅ |
| PC-036 | `cargo test -p ecc-infra os_env_current_exe` | PASS | PASS | ✅ |
| PC-037 | `cargo test -p xtask deploy_builds_three` | PASS | PASS | ✅ |
| PC-038 | `cargo test -p xtask deploy_installs_three` | PASS | PASS | ✅ |
| PC-039 | `cargo test -p xtask deploy_dry_run_lists_three` | PASS | PASS | ✅ |
| PC-040 | `cargo test -p ecc-app dry_run_no_writes` | PASS | PASS | ✅ |
| PC-041 | `cargo test -p ecc-app full_upgrade_flow` | PASS | PASS | ✅ |
| PC-042 | `grep -q 'cosign sign-blob.*--bundle' .github/workflows/release.yml` | exit 0 | exit 0 | ✅ |
| PC-043 | `cargo clippy -- -D warnings` | exit 0 | exit 0 | ✅ |
| PC-044 | `cargo build --release` | exit 0 | exit 0 | ✅ |

All pass conditions: 45/45 ✅

## E2E Tests
No E2E tests required by solution (all E2E tests are `#[ignore]` by design for network-dependent boundaries).

## Docs Updated
| # | Doc File | Level | What Changed |
|---|----------|-------|--------------|
| 1 | CLAUDE.md | project | Added ecc update command, updated test count to 2250 |
| 2 | CHANGELOG.md | project | Added BL-088 ecc update entry |
| 3 | docs/adr/0037-ureq-http-client.md | project | Created ADR for ureq choice |
| 4 | docs/adr/0038-keyless-sigstore-cosign.md | project | Created ADR for Sigstore verification |
| 5 | docs/backlog/BL-088-ecc-update-command.md | backlog | Status: implemented |
| 6 | docs/backlog/BACKLOG.md | backlog | BL-088 status: implemented |

## ADRs Created
| # | File | Decision |
|---|------|----------|
| 1 | docs/adr/0037-ureq-http-client.md | Use ureq over reqwest/self_update for sync CLI HTTP |
| 2 | docs/adr/0038-keyless-sigstore-cosign.md | Mandatory keyless Sigstore cosign verification |

## Supplemental Docs
| Subagent | Status | Output File | Commit SHA | Notes |
|----------|--------|-------------|------------|-------|
| module-summary-updater | dispatched | docs/MODULE-SUMMARIES.md | pending | background |
| diagram-updater | dispatched | docs/specs/.../diagrams.md | pending | background |

## Subagent Execution
| PC ID | Status | Commit Count | Files Changed Count |
|-------|--------|--------------|---------------------|
| PC-001-006 | success | 2 | 5 |
| PC-007-010,045 | success | 11 | 10 |
| PC-011-014 | success | 5 | 2 |
| PC-015-024 | success | 2 | 6 |
| PC-025-036 | success | 2 | 5 |
| PC-037-042 | success | 2 | 3 |
| PC-043 | success | 0 | 0 |
| PC-044 | success | 0 | 0 |

## Code Review
2 CRITICAL + 4 HIGH findings addressed in 6 fix commits:
- Fixed cosign certificate identity to use tag ref pattern
- Fixed bundle file download + path construction
- Added cleanup_backups call after successful update
- Handled include_prerelease flag in latest_version
- Extracted orchestrator tests to separate file (1297→345 lines)
- Removed dead compile-check functions from ports

## Suggested Commit
feat(update): complete ecc update self-update from GitHub Releases
