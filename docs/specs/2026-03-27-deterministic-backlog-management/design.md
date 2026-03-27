# Solution: Deterministic Backlog Management CLI

## Spec Reference
Concern: dev, Feature: BL-066 — Add ecc backlog next-id, check-duplicates, reindex subcommands

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `Cargo.toml` (workspace root) | Modify | Add `serde_yaml = "0.9"` to `[workspace.dependencies]` | All (frontmatter prerequisite) |
| 2 | `crates/ecc-domain/Cargo.toml` | Modify | Add `serde_yaml = { workspace = true }` | Decision #6 |
| 3 | `crates/ecc-domain/src/backlog/mod.rs` | Create | Module declaration re-exporting entry, similarity, index | US-001,002,003 |
| 4 | `crates/ecc-domain/src/backlog/entry.rs` | Create | `BacklogEntry` struct, `parse_frontmatter`, `extract_id_from_filename` | AC-001.1-4, AC-002.4-6, AC-003.1-3,8 |
| 5 | `crates/ecc-domain/src/backlog/similarity.rs` | Create | `levenshtein_distance`, `normalized_levenshtein_similarity`, `keyword_jaccard`, `composite_score`, `DuplicateCandidate` | AC-002.1-2,7 |
| 6 | `crates/ecc-domain/src/backlog/index.rs` | Create | `generate_index_table`, `generate_stats`, `extract_dependency_graph` | AC-003.1-5,9 |
| 7 | `crates/ecc-domain/src/lib.rs` | Modify | Add `pub mod backlog;` | Module wiring |
| 8 | `crates/ecc-ports/src/fs.rs` | Modify | Add `fn rename(&self, from: &Path, to: &Path) -> Result<(), FsError>` | AC-003.7 |
| 9 | `crates/ecc-infra/src/os_fs.rs` | Modify | Implement `rename` using `std::fs::rename` | AC-003.7 |
| 10 | `crates/ecc-test-support/src/in_memory_fs.rs` | Modify | Implement `rename` (move BTreeMap entry) | AC-003.7 |
| 11 | `crates/ecc-app/src/backlog.rs` | Create | `next_id`, `check_duplicates`, `reindex` use cases | US-001,002,003 |
| 12 | `crates/ecc-app/src/lib.rs` | Modify | Add `pub mod backlog;` | Module wiring |
| 13 | `crates/ecc-cli/src/commands/backlog.rs` | Create | `BacklogArgs`, `BacklogAction` enum, `run()` handler | All US/AC |
| 14 | `crates/ecc-cli/src/commands/mod.rs` | Modify | Add `pub mod backlog;` | Module wiring |
| 15 | `crates/ecc-cli/src/main.rs` | Modify | Add `Backlog` variant + match arm | All US/AC |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | extract_id_from_filename parses "BL-075-title.md" → Some(75) | AC-001.1 | `cargo test -p ecc-domain -- backlog::entry::tests::extract_id_from_filename_valid` | PASS |
| PC-002 | unit | extract_id_from_filename returns None for "README.md" | AC-001.4 | `cargo test -p ecc-domain -- backlog::entry::tests::extract_id_from_filename_non_bl` | PASS |
| PC-003 | unit | parse_frontmatter extracts all fields from valid YAML | AC-003.1, AC-003.2 | `cargo test -p ecc-domain -- backlog::entry::tests::parse_frontmatter_valid` | PASS |
| PC-004 | unit | parse_frontmatter returns Err for malformed YAML | AC-002.6, AC-003.8 | `cargo test -p ecc-domain -- backlog::entry::tests::parse_frontmatter_malformed` | PASS |
| PC-005 | unit | parse_frontmatter handles missing optional fields | AC-003.2 | `cargo test -p ecc-domain -- backlog::entry::tests::parse_frontmatter_optional_fields_missing` | PASS |
| PC-006 | unit | levenshtein_distance("kitten","sitting") = 3 | AC-002.7 | `cargo test -p ecc-domain -- backlog::similarity::tests::levenshtein_known_pair` | PASS |
| PC-007 | unit | levenshtein_distance("","abc") = 3 | AC-002.7 | `cargo test -p ecc-domain -- backlog::similarity::tests::levenshtein_empty_string` | PASS |
| PC-008 | unit | normalized_levenshtein_similarity identical = 1.0 | AC-002.7 | `cargo test -p ecc-domain -- backlog::similarity::tests::normalized_levenshtein_identical` | PASS |
| PC-009 | unit | keyword_jaccard partial overlap returns correct ratio | AC-002.7 | `cargo test -p ecc-domain -- backlog::similarity::tests::keyword_jaccard_partial_overlap` | PASS |
| PC-010 | unit | keyword_jaccard disjoint sets = 0.0 | AC-002.3 | `cargo test -p ecc-domain -- backlog::similarity::tests::keyword_jaccard_no_overlap` | PASS |
| PC-011 | unit | composite_score tag boost capped at +0.3 | AC-002.2, AC-002.7 | `cargo test -p ecc-domain -- backlog::similarity::tests::composite_score_tag_boost_capped` | PASS |
| PC-012 | unit | composite_score similar titles >= 0.6 | AC-002.1 | `cargo test -p ecc-domain -- backlog::similarity::tests::composite_score_similar_titles` | PASS |
| PC-013 | unit | generate_index_table produces sorted markdown table | AC-003.1, AC-003.2, AC-003.3 | `cargo test -p ecc-domain -- backlog::index::tests::generate_index_table_sorted` | PASS |
| PC-014 | unit | generate_stats produces correct status counts | AC-003.4 | `cargo test -p ecc-domain -- backlog::index::tests::generate_stats_counts` | PASS |
| PC-015 | unit | extract_dependency_graph finds existing section | AC-003.5 | `cargo test -p ecc-domain -- backlog::index::tests::extract_dependency_graph_present` | PASS |
| PC-016 | unit | extract_dependency_graph returns None when absent | AC-003.9 | `cargo test -p ecc-domain -- backlog::index::tests::extract_dependency_graph_absent` | PASS |
| PC-017 | unit | next_id with BL-001..075 returns "BL-076" | AC-001.1 | `cargo test -p ecc-app -- backlog::tests::next_id_sequential` | PASS |
| PC-018 | unit | next_id with empty dir returns "BL-001" | AC-001.2 | `cargo test -p ecc-app -- backlog::tests::next_id_empty_dir` | PASS |
| PC-019 | unit | next_id with gaps returns max+1 | AC-001.3 | `cargo test -p ecc-app -- backlog::tests::next_id_with_gaps` | PASS |
| PC-020 | unit | next_id ignores non-BL files | AC-001.4 | `cargo test -p ecc-app -- backlog::tests::next_id_ignores_non_bl` | PASS |
| PC-021 | unit | next_id errors on missing dir | AC-001.5 | `cargo test -p ecc-app -- backlog::tests::next_id_missing_dir` | PASS |
| PC-022 | unit | check_duplicates finds similar entry >= 0.6 | AC-002.1 | `cargo test -p ecc-app -- backlog::tests::check_duplicates_finds_similar` | PASS |
| PC-023 | unit | check_duplicates applies tag boost | AC-002.2 | `cargo test -p ecc-app -- backlog::tests::check_duplicates_tag_boost` | PASS |
| PC-024 | unit | check_duplicates returns empty for no matches | AC-002.3 | `cargo test -p ecc-app -- backlog::tests::check_duplicates_no_matches` | PASS |
| PC-025 | unit | check_duplicates filters to open/in-progress | AC-002.4 | `cargo test -p ecc-app -- backlog::tests::check_duplicates_status_filter` | PASS |
| PC-026 | unit | check_duplicates sorted by score descending | AC-002.5 | `cargo test -p ecc-app -- backlog::tests::check_duplicates_sorted_desc` | PASS |
| PC-027 | unit | check_duplicates skips malformed with warning | AC-002.6 | `cargo test -p ecc-app -- backlog::tests::check_duplicates_skips_malformed` | PASS |
| PC-028 | unit | check_duplicates errors on empty query | AC-002.8 | `cargo test -p ecc-app -- backlog::tests::check_duplicates_empty_query` | PASS |
| PC-029 | unit | reindex generates full BACKLOG.md | AC-003.1, AC-003.2, AC-003.3, AC-003.4, AC-003.5 | `cargo test -p ecc-app -- backlog::tests::reindex_full` | PASS |
| PC-030 | unit | reindex dry-run prints without writing | AC-003.6 | `cargo test -p ecc-app -- backlog::tests::reindex_dry_run` | PASS |
| PC-031 | unit | reindex uses atomic write (temp + rename) | AC-003.7 | `cargo test -p ecc-app -- backlog::tests::reindex_atomic_write` | PASS |
| PC-032 | unit | reindex skips malformed with warning | AC-003.8 | `cargo test -p ecc-app -- backlog::tests::reindex_skips_malformed` | PASS |
| PC-033 | unit | reindex creates new BACKLOG.md when absent | AC-003.9 | `cargo test -p ecc-app -- backlog::tests::reindex_creates_new_file` | PASS |
| PC-034 | lint | clippy passes with zero warnings | All | `cargo clippy -p ecc-domain -p ecc-app -p ecc-cli -- -D warnings` | exit 0 |
| PC-035 | build | full workspace builds | All | `cargo build --workspace` | exit 0 |
| PC-036 | lint | formatting passes | All | `cargo fmt --all -- --check` | exit 0 |

### Coverage Check

All 22 ACs covered. Zero uncovered ACs.

| AC | Covered By |
|----|------------|
| AC-001.1 | PC-001, PC-017 |
| AC-001.2 | PC-018 |
| AC-001.3 | PC-019 |
| AC-001.4 | PC-002, PC-020 |
| AC-001.5 | PC-021 |
| AC-002.1 | PC-012, PC-022 |
| AC-002.2 | PC-011, PC-023 |
| AC-002.3 | PC-010, PC-024 |
| AC-002.4 | PC-025 |
| AC-002.5 | PC-026 |
| AC-002.6 | PC-004, PC-027 |
| AC-002.7 | PC-006..012 |
| AC-002.8 | PC-028 |
| AC-003.1 | PC-003, PC-013, PC-029 |
| AC-003.2 | PC-003, PC-005, PC-013 |
| AC-003.3 | PC-013, PC-029 |
| AC-003.4 | PC-014, PC-029 |
| AC-003.5 | PC-015, PC-029 |
| AC-003.6 | PC-030 |
| AC-003.7 | PC-031 |
| AC-003.8 | PC-004, PC-032 |
| AC-003.9 | PC-016, PC-033 |

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | FileSystem.rename | OsFileSystem | FileSystem | Rename temp file to target | ignored | FileSystem adapter modified |

### E2E Activation Rules

FileSystem.rename E2E test should be un-ignored during this implementation since we're adding the `rename` method.

## Test Strategy

TDD order (bottom-up):
1. PC-001..002: extract_id_from_filename (simplest pure function)
2. PC-003..005: parse_frontmatter (core parsing, requires serde_yaml)
3. PC-006..012: similarity functions (levenshtein → jaccard → composite)
4. PC-013..016: index generation (table, stats, dependency graph)
5. PC-017..021: next_id app use case
6. PC-022..028: check_duplicates app use case
7. PC-029..033: reindex app use case
8. PC-034..036: lint, build, fmt

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CLAUDE.md | Project | Modify | Add `ecc backlog next-id\|check-duplicates\|reindex` to CLI Commands | US-001,002,003 |
| 2 | CHANGELOG.md | Project | Modify | "feat: add deterministic backlog management CLI" | All |

## SOLID Assessment

PASS — 0 findings.
- SRP: entry/similarity/index are separate concerns in separate files
- OCP: new backlog module extends functionality without modifying existing code
- LSP: InMemoryFileSystem correctly substitutes for OsFileSystem via trait
- ISP: FileSystem trait is already appropriately sized; `rename` is a justified addition
- DIP: app layer depends on `&dyn FileSystem` trait, not concrete adapter

## Robert's Oath Check

CLEAN — 0 warnings.
- No harmful code, no mess, proof planned (36 PCs), small additive release
- Pure domain functions, immutable data patterns, comprehensive error handling

## Security Notes

CLEAR — 0 findings.
- No auth, no user data, no network, no secrets
- Pure local filesystem on markdown within bounded directory

## Rollback Plan

Reverse dependency order:
1. Remove `Backlog` variant from `main.rs` Command enum
2. Remove `pub mod backlog;` from `commands/mod.rs`
3. Delete `commands/backlog.rs`
4. Remove `pub mod backlog;` from `ecc-app/src/lib.rs`
5. Delete `ecc-app/src/backlog.rs`
6. Remove `rename` from `ecc-ports/fs.rs`, `os_fs.rs`, `in_memory_fs.rs`
7. Remove `pub mod backlog;` from `ecc-domain/src/lib.rs`
8. Delete `ecc-domain/src/backlog/` directory
9. Remove `serde_yaml` from workspace and domain Cargo.toml
