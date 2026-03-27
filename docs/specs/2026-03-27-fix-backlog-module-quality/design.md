# Solution: Fix backlog module — UTF-8 bug, error types, code quality

## Spec Reference
Concern: fix, Feature: Fix backlog module — UTF-8 Levenshtein bug, thiserror enum, code quality

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `ecc-domain/src/backlog/entry.rs` | Modify | Add BacklogError thiserror enum, BacklogStatus typed enum, update parse_frontmatter return type | US-002 AC-002.1-2, US-003 AC-003.5 |
| 2 | `ecc-domain/src/backlog/similarity.rs` | Modify | Char-based Levenshtein, named constants, normalized composite score | US-001 AC-001.1-3, US-003 AC-003.1-2 |
| 3 | `ecc-domain/src/backlog/index.rs` | Modify | Use BacklogStatus enum in generate_stats | US-003 AC-003.5 |
| 4 | `ecc-domain/src/backlog/mod.rs` | Modify | Re-export BacklogError, BacklogStatus | US-002 |
| 5 | `ecc-app/src/backlog.rs` | Modify | BacklogError return types, load_entries helper, temp file cleanup | US-002 AC-002.3, US-003 AC-003.3-4 |
| 6 | `ecc-cli/src/commands/backlog.rs` | Modify | Map BacklogError to anyhow | US-002 AC-002.4 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Levenshtein em-dash "—" vs "x" = 1 | AC-001.1 | `cargo test -p ecc-domain -- backlog::similarity::tests::levenshtein_unicode_chars` | PASS |
| PC-002 | unit | Normalized Levenshtein identical multi-byte = 1.0 | AC-001.2 | `cargo test -p ecc-domain -- backlog::similarity::tests::normalized_levenshtein_unicode_identical` | PASS |
| PC-003 | unit | Existing levenshtein_known_pair still passes | AC-001.3 | `cargo test -p ecc-domain -- backlog::similarity::tests::levenshtein_known_pair` | PASS |
| PC-004 | unit | BacklogError enum has 5 variants | AC-002.1 | `cargo test -p ecc-domain -- backlog::entry::tests::backlog_error_variants` | PASS |
| PC-005 | unit | parse_frontmatter returns BacklogError | AC-002.2 | `cargo test -p ecc-domain -- backlog::entry::tests::parse_frontmatter_malformed` | PASS |
| PC-006 | unit | Named constants for weights/thresholds | AC-003.1 | `cargo test -p ecc-domain -- backlog::similarity::tests::constants_are_defined` | PASS |
| PC-007 | unit | Composite score in [0.0, 1.0] | AC-003.2 | `cargo test -p ecc-domain -- backlog::similarity::tests::composite_score_normalized_range` | PASS |
| PC-008 | unit | BacklogStatus serde round-trip | AC-003.5 | `cargo test -p ecc-domain -- backlog::entry::tests::backlog_status_serde` | PASS |
| PC-009 | unit | BacklogStatus Unknown fallback | AC-003.5 | `cargo test -p ecc-domain -- backlog::entry::tests::backlog_status_unknown_fallback` | PASS |
| PC-010 | unit | next_id returns BacklogError | AC-002.3 | `cargo test -p ecc-app -- backlog::tests::next_id_missing_dir` | PASS |
| PC-011 | unit | check_duplicates returns BacklogError | AC-002.3 | `cargo test -p ecc-app -- backlog::tests::check_duplicates_empty_query` | PASS |
| PC-012 | unit | load_entries helper works | AC-003.3 | `cargo test -p ecc-app -- backlog::tests::next_id_sequential` | PASS |
| PC-013 | unit | Temp file cleanup on rename failure | AC-003.4 | `cargo test -p ecc-app -- backlog::tests::reindex_cleanup_on_failure` | PASS |
| PC-014 | unit | All domain backlog tests pass | AC-001.3, AC-003.6 | `cargo test -p ecc-domain -- backlog` | PASS |
| PC-015 | unit | All app backlog tests pass | AC-003.6 | `cargo test -p ecc-app -- backlog` | PASS |
| PC-016 | build | CLI builds with BacklogError mapping | AC-002.4 | `cargo build -p ecc-cli` | exit 0 |
| PC-017 | lint | Clippy zero warnings | All | `cargo clippy -- -D warnings` | exit 0 |
| PC-018 | build | Workspace builds | All | `cargo build --workspace` | exit 0 |

### Coverage Check

All 13 ACs covered:

| AC | Covered By |
|----|------------|
| AC-001.1 | PC-001 |
| AC-001.2 | PC-002 |
| AC-001.3 | PC-003, PC-014 |
| AC-002.1 | PC-004 |
| AC-002.2 | PC-005 |
| AC-002.3 | PC-010, PC-011 |
| AC-002.4 | PC-016 |
| AC-003.1 | PC-006 |
| AC-003.2 | PC-007 |
| AC-003.3 | PC-012 |
| AC-003.4 | PC-013 |
| AC-003.5 | PC-008, PC-009 |
| AC-003.6 | PC-014, PC-015 |

### E2E Test Plan

No E2E tests — all changes are internal to domain and app layers.

### E2E Activation Rules

None.

## Test Strategy

TDD order (bottom-up):
1. PC-001..003: Fix Levenshtein to use chars (domain)
2. PC-004..005: Add BacklogError enum (domain)
3. PC-006..007: Extract constants + normalize score (domain)
4. PC-008..009: Add BacklogStatus enum (domain)
5. PC-010..013: Update app layer (BacklogError, load_entries, cleanup)
6. PC-014..015: Regression verification (all existing tests)
7. PC-016..018: CLI build + lint + workspace build

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CHANGELOG.md | Project | Modify | "fix: correct Levenshtein UTF-8 byte bug and improve backlog module code quality" | All |

## SOLID Assessment

PASS — 0 findings. Fix improves SRP (error types in dedicated enum) and DRY (shared load_entries helper).

## Robert's Oath Check

CLEAN — fixing known bugs and code quality issues. Adding proof (3 new tests). Small contained release.

## Security Notes

CLEAR — no auth, secrets, network, or injection surface changes.

## Rollback Plan

1. Revert `ecc-cli/src/commands/backlog.rs` (restore String error mapping)
2. Revert `ecc-app/src/backlog.rs` (restore String errors, inline loops)
3. Revert `ecc-domain/src/backlog/index.rs` (restore String status)
4. Revert `ecc-domain/src/backlog/similarity.rs` (restore byte-based Levenshtein, inline numbers)
5. Revert `ecc-domain/src/backlog/entry.rs` (remove BacklogError, BacklogStatus)
6. Revert `ecc-domain/src/backlog/mod.rs` (remove re-exports)
