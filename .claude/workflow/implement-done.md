# Implementation Complete: ECC Diagnostics — Tiered Verbosity

## Spec Reference
Concern: dev, Feature: ECC diagnostics tiered verbosity

## Changes Made
| # | File | Action | Solution Ref | Tests | Status |
|---|------|--------|--------------|-------|--------|
| 1 | crates/ecc-domain/src/config/log_level.rs | create | US-001 | 5 unit tests | done |
| 2 | crates/ecc-ports/src/config_store.rs | create | US-005 | — | done |
| 3 | crates/ecc-test-support/src/in_memory_config_store.rs | create | US-005 | — | done |
| 4 | crates/ecc-infra/src/file_config_store.rs | create | US-005 | 5 unit tests | done |
| 5 | crates/ecc-infra (tracing migration) | modify | US-001 | — | done |
| 6 | crates/ecc-app (~25 files) | modify | US-001 | 727 existing pass | done |
| 7 | crates/ecc-app/src/diagnostics.rs | create | US-004 | 3 unit tests | done |
| 8 | crates/ecc-app/src/config_cmd.rs | create | US-005 | 11 unit tests | done |
| 9 | crates/ecc-app/src/hook/mod.rs | modify | US-003 | — | done |
| 10 | crates/ecc-cli/src/main.rs | modify | US-002 | — | done |
| 11 | crates/ecc-cli/src/commands/status.rs | create | US-004 | — | done |
| 12 | crates/ecc-cli/src/commands/config.rs | create | US-005 | — | done |
| 13 | crates/ecc-workflow/src/main.rs | modify | US-002 | 95 existing pass | done |
| 14 | crates/ecc-workflow (instrumentation) | modify | US-003 | — | done |
| 15 | docs/adr/0032-tracing-cross-cutting-concern.md | create | US-006 | — | done |
| 16 | CLAUDE.md | modify | US-004 | — | done |
| 17 | CHANGELOG.md | modify | — | — | done |

## Pass Condition Results
All pass conditions verified via cargo clippy + cargo test + cargo build. 0 failures across entire workspace.

All pass conditions: 52/52 PASS

## E2E Tests
No E2E tests required by solution.

## Docs Updated
| # | Doc File | Level | What Changed |
|---|----------|-------|--------------|
| 1 | CHANGELOG.md | project | Added v4.7.0 tiered diagnostics entry |
| 2 | CLAUDE.md | project | Added ecc status + ecc config commands |
| 3 | docs/adr/0032-tracing-cross-cutting-concern.md | ADR | Tracing layer restrictions |

## ADRs Created
| # | File | Decision |
|---|------|----------|
| 1 | docs/adr/0032-tracing-cross-cutting-concern.md | Tracing forbidden in domain/ports |

## Supplemental Docs
No supplemental docs generated — MODULE-SUMMARIES update deferred to post-implementation.

## Subagent Execution
| PC ID | Status | Commit Count | Files Changed Count |
|-------|--------|--------------|---------------------|
| Phase 1 (domain+ports) | success | 3 | 6 |
| Phase 2 (infra) | success | 2 | 4 |
| Phase 3a (app migration) | success | 1 | 27 |
| Phase 3b (app use cases) | success | 3 | 3 |
| Phase 3c (instrumentation) | success | 1 | 5 |
| Phase 4 (CLI) | success | 2 | 8 |
| Phase 5 (workflow) | success | 1 | 3 |
| Phase 6 (docs) | success | 4 | 4 |

## Code Review
Deferred to /verify — all PCs pass, clippy clean, full test suite green.

## Suggested Commit
feat(diagnostics): add tiered verbosity with tracing, ecc status, ecc config
