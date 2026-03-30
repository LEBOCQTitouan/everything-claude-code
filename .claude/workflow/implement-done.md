# Implementation Complete: Knowledge Sources Registry — Hardening & Completion

## Spec Reference
Concern: dev, Feature: Knowledge sources registry — curated reference list with quadrant organization and command integration

## Changes Made
| # | File | Action | Solution Ref | Tests | Status |
|---|------|--------|--------------|-------|--------|
| 1 | `crates/ecc-domain/src/sources/entry.rs` | modify | PC-001→006 | source_url_parse_*, source_url_as_str | done |
| 2 | `crates/ecc-domain/src/sources/parser.rs` | modify | PC-008, PC-010 | parse_stale_bare_flag, parse_full_document | done |
| 3 | `crates/ecc-domain/src/sources/serializer.rs` | modify | PC-009, PC-011 | stale_flag_round_trip, round_trip | done |
| 4 | `crates/ecc-domain/src/sources/registry.rs` | modify | PC-007 | add_*, list_*, reindex_*, find_* | done |
| 5 | `crates/ecc-app/src/sources.rs` | modify | PC-012→014 | add_uses_injected_date, check_clears_stale | done |
| 6 | `crates/ecc-cli/src/commands/sources.rs` | modify | PC-015 | sources_list_routes_to_app_use_case | done |
| 7 | `crates/ecc-integration-tests/tests/sources_flow.rs` | create | PC-016→019 | list_outputs_entries, add_creates_entry, reindex_moves_inbox, reindex_dry_run_no_write | done |
| 8 | `commands/audit-evolution.md` | modify | PC-020→022 | grep verification | done |
| 9 | `commands/audit-full.md` | modify | PC-023→024 | grep verification | done |

## TDD Log
| PC ID | RED | GREEN | REFACTOR | Notes |
|-------|-----|-------|----------|-------|
| PC-001→009 | ✅ | ✅ passes, 29 domain tests green | ✅ removed dead validate_url | Atomic domain migration |
| PC-010 | ✅ | ✅ parse_stale_bare_flag passes | ⏭ no refactor needed | — |
| PC-011 | ✅ | ✅ stale_flag_round_trip passes | ⏭ no refactor needed | — |
| PC-012 | ✅ | ✅ add_uses_injected_date passes | ⏭ no refactor needed | — |
| PC-013 | ✅ | ✅ check_clears_stale passes with correct format | ⏭ no refactor needed | Fixed test data from `stale: true` to bare `stale` |
| PC-014 | ✅ | ✅ all 13 app tests pass | ⏭ no refactor needed | — |
| PC-015 | ✅ | ✅ CLI test passes | ⏭ no refactor needed | — |
| PC-016→019 | ✅ | ✅ 4 integration tests pass | ⏭ no refactor needed | Real binary via EccTestEnv |
| PC-020→024 | ✅ | ✅ grep checks all >= 1 | ⏭ no refactor needed | Markdown-only |
| PC-025 | ✅ | ✅ CHANGELOG entry present | ⏭ no refactor needed | — |
| PC-026 | — | ✅ cargo build --workspace exit 0 | — | — |
| PC-027 | — | ✅ cargo clippy clean | — | — |
| PC-028 | — | ✅ cargo test --workspace all pass | — | 1845 tests total |

## Pass Condition Results
| PC ID | Command | Expected | Actual | Status |
|-------|---------|----------|--------|--------|
| PC-001 | `cargo test --lib -p ecc-domain sources::entry::tests::source_url_parse_valid_https` | PASS | PASS | ✅ |
| PC-002 | `cargo test --lib -p ecc-domain sources::entry::tests::source_url_parse_valid_http` | PASS | PASS | ✅ |
| PC-003 | `cargo test --lib -p ecc-domain sources::entry::tests::source_url_parse_rejects_no_scheme` | PASS | PASS | ✅ |
| PC-004 | `cargo test --lib -p ecc-domain sources::entry::tests::source_url_parse_rejects_empty` | PASS | PASS | ✅ |
| PC-005 | `cargo test --lib -p ecc-domain sources::entry::tests::source_url_as_str` | PASS | PASS | ✅ |
| PC-006 | `cargo test --lib -p ecc-domain sources::entry::tests::source_url_parse_rejects_ftp` | PASS | PASS | ✅ |
| PC-007 | `cargo test --lib -p ecc-domain sources` | All PASS | All PASS | ✅ |
| PC-008 | `cargo test --lib -p ecc-domain sources::parser` | All PASS | All PASS | ✅ |
| PC-009 | `cargo test --lib -p ecc-domain sources::serializer` | All PASS | All PASS | ✅ |
| PC-010 | `cargo test --lib -p ecc-domain sources::parser::tests::parse_stale_bare_flag` | PASS | PASS | ✅ |
| PC-011 | `cargo test --lib -p ecc-domain sources::serializer::tests::stale_flag_round_trip` | PASS | PASS | ✅ |
| PC-012 | `cargo test --lib -p ecc-app sources::tests::add_uses_injected_date` | PASS | PASS | ✅ |
| PC-013 | `cargo test --lib -p ecc-app sources::tests::check_clears_stale` | PASS | PASS | ✅ |
| PC-014 | `cargo test --lib -p ecc-app sources` | All PASS | All PASS | ✅ |
| PC-015 | `cargo test -p ecc-cli sources` | PASS | PASS | ✅ |
| PC-016 | `cargo test -p ecc-integration-tests --test sources_flow list_outputs_entries` | PASS | PASS | ✅ |
| PC-017 | `cargo test -p ecc-integration-tests --test sources_flow add_creates_entry` | PASS | PASS | ✅ |
| PC-018 | `cargo test -p ecc-integration-tests --test sources_flow reindex_moves_inbox` | PASS | PASS | ✅ |
| PC-019 | `cargo test -p ecc-integration-tests --test sources_flow reindex_dry_run_no_write` | PASS | PASS | ✅ |
| PC-020 | `grep -c "Sources Re-interrogation" commands/audit-evolution.md` | >= 1 | 1 | ✅ |
| PC-021 | `grep -c "docs/sources.md" commands/audit-evolution.md` | >= 1 | 3 | ✅ |
| PC-022 | `grep -ci "if.*sources.md.*exist\|skip.*silently\|does not exist" commands/audit-evolution.md` | >= 1 | 2 | ✅ |
| PC-023 | `grep -c "Sources Re-interrogation" commands/audit-full.md` | >= 1 | 1 | ✅ |
| PC-024 | `grep -ci "if.*sources.md.*exist\|skip.*silently\|does not exist" commands/audit-full.md` | >= 1 | 2 | ✅ |
| PC-025 | `grep -ci "knowledge.sources\|BL-086\|source.url.*newtype" CHANGELOG.md` | >= 1 | 2 | ✅ |
| PC-026 | `cargo build --workspace` | exit 0 | exit 0 | ✅ |
| PC-027 | `cargo clippy --workspace -- -D warnings` | exit 0 | exit 0 | ✅ |
| PC-028 | `cargo test --workspace` | All PASS | All PASS | ✅ |

All pass conditions: 28/28 ✅

## E2E Tests
No additional E2E tests required — PC-016 through PC-019 cover all activated E2E boundaries.

## Docs Updated
| # | Doc File | Level | What Changed |
|---|----------|-------|--------------|
| 1 | CHANGELOG.md | project | Added BL-086 hardening entry in v4.6.2 |
| 2 | docs/backlog/BL-086-*.md | metadata | status: open → implemented |
| 3 | docs/backlog/BACKLOG.md | metadata | BL-086 row updated to implemented |
| 4 | docs/domain/bounded-contexts.md | reference | Added SourceUrl value object to Sources context |
| 5 | CLAUDE.md | project | Updated test count 1698 → 1845 |

## ADRs Created
None required.

## Supplemental Docs
No supplemental docs generated — change scope did not warrant module summary or diagram updates.

## Subagent Execution
Inline execution — subagent dispatch not used.

## Code Review
WARNING — 1 HIGH (polyadic add() with 9 params, pre-existing, out of scope), 4 MEDIUM (pre-existing check() size, dead validate_url fixed, duplicated date math, days_to_ymd guard), 2 LOW. Actionable item (dead validate_url) addressed. HIGH deferred to backlog.

## Suggested Commit
feat(sources): harden knowledge sources registry — SourceUrl newtype, date injection, stale fix, integration tests (BL-086)
