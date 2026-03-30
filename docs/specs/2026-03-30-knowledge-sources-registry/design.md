# Solution: Knowledge Sources Registry — Hardening & Completion

## Spec Reference
Concern: dev, Feature: Knowledge sources registry — curated reference list with quadrant organization and command integration

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/sources/entry.rs` | Modify | Add `SourceUrl` newtype with `parse()`, `as_str()`, `Display`, `Debug`, `Clone`, `PartialEq`, `Eq`. Change `SourceEntry.url` from `String` to `SourceUrl`. Remove standalone `validate_url()` (replaced by `SourceUrl::parse()`). | US-003 (AC-003.1, AC-003.2, AC-003.3, AC-003.4) |
| 2 | `crates/ecc-domain/src/sources/parser.rs` | Modify | Use `SourceUrl::parse()` instead of `validate_url()` when constructing entries from parsed markdown. | US-003 (AC-003.3) |
| 3 | `crates/ecc-domain/src/sources/serializer.rs` | Modify | Use `entry.url.as_str()` in `serialize_entry()`. Update `make_entry` test helper to use `SourceUrl`. | US-003 (AC-003.3, AC-003.4) |
| 4 | `crates/ecc-domain/src/sources/registry.rs` | Modify | Update `find_by_url()` to compare via `e.url.as_str() == url`. Update `add()` error message to use `entry.url.as_str()`. Update test helpers. | US-003 (AC-003.3) |
| 5 | `crates/ecc-app/src/sources.rs` | Modify | (a) Add `date: &str` parameter to `add()`, replace hardcoded `"2026-03-29"`. (b) Use `SourceUrl::parse()` for URL construction. (c) Fix `check_clears_stale` test data from `stale: true` to bare `stale` flag. | US-001 (AC-001.1, AC-001.3), US-002 (AC-002.3), US-003 (AC-003.3) |
| 6 | `crates/ecc-cli/src/commands/sources.rs` | Modify | Pass `&today_date()` as date parameter to `ecc_app::sources::add()`. | US-001 (AC-001.2) |
| 7 | `crates/ecc-integration-tests/tests/sources_flow.rs` | Create | Integration tests for list, add, reindex, reindex --dry-run via app-layer with `InMemoryFileSystem`. | US-004 (AC-004.1, AC-004.2, AC-004.3, AC-004.4) |
| 8 | `commands/audit-evolution.md` | Modify | Add "Sources Re-interrogation" step. Check if `docs/sources.md` exists; if yes, list sources in relevant subjects. Skip silently if file missing. | US-005 (AC-005.1, AC-005.3) |
| 9 | `commands/audit-full.md` | Modify | Add "Sources Re-interrogation" section in analysis phase. Check if `docs/sources.md` exists; if yes, consult. Skip silently if missing. | US-005 (AC-005.2, AC-005.3) |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | `SourceUrl::parse` succeeds for valid HTTPS URL | AC-003.1 | `cargo test --lib -p ecc-domain sources::entry::tests::source_url_parse_valid_https -- --exact` | PASS |
| PC-002 | unit | `SourceUrl::parse` succeeds for valid HTTP URL | AC-003.1 | `cargo test --lib -p ecc-domain sources::entry::tests::source_url_parse_valid_http -- --exact` | PASS |
| PC-003 | unit | `SourceUrl::parse` rejects no-scheme string | AC-003.2 | `cargo test --lib -p ecc-domain sources::entry::tests::source_url_parse_rejects_no_scheme -- --exact` | PASS |
| PC-004 | unit | `SourceUrl::parse` rejects empty string | AC-003.2 | `cargo test --lib -p ecc-domain sources::entry::tests::source_url_parse_rejects_empty -- --exact` | PASS |
| PC-005 | unit | `SourceUrl::as_str` returns inner string | AC-003.4 | `cargo test --lib -p ecc-domain sources::entry::tests::source_url_as_str -- --exact` | PASS |
| PC-006 | unit | `SourceUrl::parse` rejects non-HTTP scheme (ftp://) | AC-003.2 | `cargo test --lib -p ecc-domain sources::entry::tests::source_url_parse_rejects_ftp -- --exact` | PASS |
| PC-007 | unit | All domain sources tests pass with SourceUrl type | AC-003.3 | `cargo test --lib -p ecc-domain sources` | All PASS |
| PC-008 | unit | Parser produces SourceUrl entries, full doc parses | AC-003.3 | `cargo test --lib -p ecc-domain sources::parser` | All PASS |
| PC-009 | unit | Serializer round-trip with SourceUrl | AC-003.3 | `cargo test --lib -p ecc-domain sources::serializer` | All PASS |
| PC-010 | unit | Stale bare flag parses correctly | AC-002.1 | `cargo test --lib -p ecc-domain sources::parser::tests::parse_stale_bare_flag -- --exact` | PASS |
| PC-011 | unit | Stale flag round-trips through serialize then parse | AC-002.2 | `cargo test --lib -p ecc-domain sources::serializer::tests::stale_flag_round_trip -- --exact` | PASS |
| PC-012 | unit | `add()` uses injected date parameter | AC-001.1, AC-001.3 | `cargo test --lib -p ecc-app sources::tests::add_uses_injected_date -- --exact` | PASS |
| PC-013 | unit | `check_clears_stale` test uses correct stale format | AC-002.3 | `cargo test --lib -p ecc-app sources::tests::check_clears_stale -- --exact` | PASS |
| PC-014 | unit | All existing app-layer sources tests pass | AC-001.1, AC-003.3 | `cargo test --lib -p ecc-app sources` | All PASS |
| PC-015 | unit | CLI sources test compiles and passes with new add() signature | AC-001.2 | `cargo test --lib -p ecc-cli commands::sources` | PASS |
| PC-016 | integration | `ecc sources list` outputs entries | AC-004.1 | `cargo test -p ecc-integration-tests sources_flow::list_outputs_entries -- --exact` | PASS |
| PC-017 | integration | `ecc sources add` creates entry with correct date | AC-004.2 | `cargo test -p ecc-integration-tests sources_flow::add_creates_entry -- --exact` | PASS |
| PC-018 | integration | `ecc sources reindex` moves inbox entries | AC-004.3 | `cargo test -p ecc-integration-tests sources_flow::reindex_moves_inbox -- --exact` | PASS |
| PC-019 | integration | `ecc sources reindex --dry-run` leaves file unchanged | AC-004.4 | `cargo test -p ecc-integration-tests sources_flow::reindex_dry_run_no_write -- --exact` | PASS |
| PC-020 | command | `audit-evolution.md` contains "Sources Re-interrogation" section | AC-005.1 | `grep -c "Sources Re-interrogation" commands/audit-evolution.md` | >= 1 |
| PC-021 | command | `audit-evolution.md` mentions docs/sources.md | AC-005.1, AC-005.3 | `grep -c "docs/sources.md" commands/audit-evolution.md` | >= 1 |
| PC-022 | command | `audit-evolution.md` contains conditional skip language | AC-005.3 | `grep -ci "if.*sources.md.*exist\|skip.*silently\|does not exist" commands/audit-evolution.md` | >= 1 |
| PC-023 | command | `audit-full.md` contains "Sources Re-interrogation" section | AC-005.2 | `grep -c "Sources Re-interrogation" commands/audit-full.md` | >= 1 |
| PC-024 | command | `audit-full.md` mentions skip-if-missing logic | AC-005.3 | `grep -ci "if.*sources.md.*exist\|skip.*silently\|does not exist" commands/audit-full.md` | >= 1 |
| PC-025 | doc | CHANGELOG.md contains knowledge-sources entry | All | `grep -ci "knowledge.sources\|BL-086\|source.url.*newtype" CHANGELOG.md` | >= 1 |
| PC-026 | build | Full workspace builds with zero errors | AC-003.3 | `cargo build --workspace` | exit 0 |
| PC-027 | lint | Clippy passes with zero warnings | All | `cargo clippy --workspace -- -D warnings` | exit 0 |
| PC-028 | test | Full test suite passes | All | `cargo test --workspace` | All PASS |

### Coverage Check

| AC | Covering PC(s) |
|----|---------------|
| AC-001.1 | PC-012 |
| AC-001.2 | PC-015 |
| AC-001.3 | PC-012 |
| AC-002.1 | PC-010 |
| AC-002.2 | PC-011 |
| AC-002.3 | PC-013 |
| AC-003.1 | PC-001, PC-002 |
| AC-003.2 | PC-003, PC-004, PC-006 |
| AC-003.3 | PC-007, PC-008, PC-009, PC-014, PC-015, PC-026 |
| AC-003.4 | PC-005 |
| AC-004.1 | PC-016 |
| AC-004.2 | PC-017 |
| AC-004.3 | PC-018 |
| AC-004.4 | PC-019 |
| AC-005.1 | PC-020, PC-021 |
| AC-005.2 | PC-023 |
| AC-005.3 | PC-022, PC-024 |

All 17 ACs covered. Zero uncovered.

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | CLI → App (add) | ecc-cli sources | FileSystem | `ecc sources add` creates entry with correct date | ignored | add() signature or CLI wiring modified |
| 2 | CLI → App (list) | ecc-cli sources | FileSystem | `ecc sources list` outputs entries correctly | ignored | SourceEntry type or list format changed |
| 3 | CLI → App (reindex) | ecc-cli sources | FileSystem | `ecc sources reindex` moves inbox entries | ignored | Reindex logic or file format modified |

### E2E Activation Rules

All 3 boundaries activated for this implementation: SourceUrl type change cascades through all commands, add() signature change affects CLI. Integration tests PC-016 through PC-019 cover all three.

## Test Strategy

TDD order (dependency-first):

1. **Phase 1: SourceUrl newtype** (PC-001 → PC-009) — Foundation type change in domain layer. All 4 domain files changed together since partial compilation is impossible.
2. **Phase 2: Stale flag tests** (PC-010, PC-011) — Lock in correct parser/serializer behavior for bare `stale` flag.
3. **Phase 3: Date injection + stale test fix** (PC-012 → PC-014) — App layer: add `date: &str` param, fix test data, compile with SourceUrl.
4. **Phase 4: CLI wiring** (PC-015) — Pass `today_date()` to `add()`.
5. **Phase 5: Integration tests** (PC-016 → PC-019) — End-to-end validation with file system doubles.
6. **Phase 6: Audit commands** (PC-020 → PC-024) — Markdown-only changes to audit-evolution and audit-full.
7. **Phase 7: Doc updates + final gates** (PC-025 → PC-028) — CHANGELOG, build, lint, full test suite.

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/backlog/BL-086-*.md` | Metadata | Update `status: implemented` | Mark backlog item complete | All US |
| 2 | `CLAUDE.md` | Project | Update test count | Increase after integration tests added | US-004 |
| 3 | `docs/domain/bounded-contexts.md` | Reference | Add SourceUrl | Mention SourceUrl value object in Sources context | US-003 |
| 4 | `CHANGELOG.md` | Project | Add entry | BL-086 hardening: SourceUrl newtype, date injection fix, stale flag fix, integration tests, audit integrations | All US |

## SOLID Assessment

**Conditional PASS** — 1 MEDIUM, 1 LOW finding:
- **MEDIUM (DIP)**: `add()` accepts `date: &str` instead of a `Clock` port trait. Acceptable since it matches the pre-existing `check()` pattern. Backlog item recommended for Clock port.
- **LOW (CRP)**: Pre-existing date utility functions (`date_to_days`, `days_between`) in sources app module. Out of scope for this spec.

## Robert's Oath Check

**CLEAN** — No blocking warnings:
- No harmful code introduced
- Changes are clean and follow established patterns
- Test coverage well-planned (25 PCs, 100% on critical paths)
- Atomic commit cadence per TDD phase
- Minor note: `add()` reaches 9 parameters after adding `date`. Backlog item for builder pattern refactor.

## Security Notes

**CLEAR** — 2 LOW informational findings:
- **LOW**: Markdown injection via crafted title (local CLI, no browser rendering, no exploit vector)
- **LOW**: SSRF via curl in `check` (local machine, http/https only, --max-time 10)

## Rollback Plan

Reverse dependency order:
1. Revert `commands/audit-full.md` — remove Sources Re-interrogation section
2. Revert `commands/audit-evolution.md` — remove Sources Re-interrogation section
3. Delete `crates/ecc-integration-tests/tests/sources_flow.rs`
4. Revert `crates/ecc-cli/src/commands/sources.rs` — remove date param from add() call
5. Revert `crates/ecc-app/src/sources.rs` — restore hardcoded date, revert stale test data, remove SourceUrl usage
6. Revert `crates/ecc-domain/src/sources/registry.rs` — restore String-based find_by_url
7. Revert `crates/ecc-domain/src/sources/serializer.rs` — restore String-based url access
8. Revert `crates/ecc-domain/src/sources/parser.rs` — restore validate_url() usage
9. Revert `crates/ecc-domain/src/sources/entry.rs` — remove SourceUrl newtype, restore String url field

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | Conditional PASS | 2 (1 MEDIUM, 1 LOW — both acceptable/pre-existing) |
| Robert | CLEAN | 0 blocking (2 minor notes) |
| Security | CLEAR | 2 LOW (informational, local CLI) |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Coverage | 88 | PASS | 17/17 ACs covered after adding conditional-skip PC |
| Order | 95 | PASS | Correct left-to-right hexagonal layer ordering |
| Fragility | 82 | PASS | Grep-based command PCs acceptable for markdown docs |
| Rollback | 90 | PASS | Reverse dependency order specified, compilation enforces completeness |
| Architecture | 95 | PASS | Strict hexagonal compliance, domain purity maintained |
| Blast radius | 85 | PASS | 9 files proportionate for 5 user stories |
| Missing PCs | 78→resolved | PASS | Added 3 PCs: conditional skip, ftp rejection, CHANGELOG |
| Doc plan | 82 | PASS | CHANGELOG, backlog, bounded-contexts, CLAUDE.md covered |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `crates/ecc-domain/src/sources/entry.rs` | Modify | US-003 |
| 2 | `crates/ecc-domain/src/sources/parser.rs` | Modify | US-003 |
| 3 | `crates/ecc-domain/src/sources/serializer.rs` | Modify | US-003 |
| 4 | `crates/ecc-domain/src/sources/registry.rs` | Modify | US-003 |
| 5 | `crates/ecc-app/src/sources.rs` | Modify | US-001, US-002, US-003 |
| 6 | `crates/ecc-cli/src/commands/sources.rs` | Modify | US-001 |
| 7 | `crates/ecc-integration-tests/tests/sources_flow.rs` | Create | US-004 |
| 8 | `commands/audit-evolution.md` | Modify | US-005 |
| 9 | `commands/audit-full.md` | Modify | US-005 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-30-knowledge-sources-registry/design.md | Full design with phase summary |
