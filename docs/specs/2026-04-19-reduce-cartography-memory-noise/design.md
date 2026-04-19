# Solution: Reduce Cartography and Memory System Noise

## Spec Reference

- **Concern**: fix
- **Feature**: Reduce cartography and memory system noise — write-time filters, self-referential guard, and lifecycle-driven cleanup
- **Spec**: `docs/specs/2026-04-19-reduce-cartography-memory-noise/spec.md`
- **Delivery**: two sequential PRs (PR1 = cartography, PR2 = memory) sharing this single design

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/cartography/noise_filter.rs` | create | Pure `is_noise_path(&str) -> bool` — ASCII-lowercase prefix match + exact match over fixed noise set | US-001 / AC-001.1–11 |
| 2 | `crates/ecc-domain/src/cartography/mod.rs` | modify | Register `noise_filter` module + re-export | US-001 |
| 3 | `crates/ecc-domain/src/cartography/dedupe.rs` | create | `canonical_hash(&SessionDelta) -> String` via SHA-256 of `serde_jcs`-canonicalized JSON | US-002 / AC-002.2 |
| 4 | `crates/ecc-domain/Cargo.toml` | modify | Add `sha2 = "0.10"` and `serde_jcs = "0.1"` deps | US-002 |
| 5 | `crates/ecc-app/src/hook/errors.rs` | create | New `HookError` enum with `CartographyIo { operation, path, source }` via thiserror | US-004 / AC-004.2 |
| 6 | `crates/ecc-app/src/hook/mod.rs` | modify | Register `errors` submodule | US-004 |
| 7 | `crates/ecc-app/src/hook/handlers/tier3_session/cartography/dedupe_io.rs` | create | `should_dedupe(ports, dir, delta, window) -> DedupeOutcome` — scans pending + processed with `ecc-flock` 500 ms timeout, fail-open on LockBusy | US-002 / AC-002.3, AC-002.6, AC-002.7 |
| 8 | `crates/ecc-app/src/hook/handlers/tier3_session/cartography/delta_writer.rs` | modify | Replace `.starts_with(".claude/")` at line 58 with `is_noise_path`; add `tracing::info!` skip logs; call dedupe_io; honor `ECC_CARTOGRAPHY_DEDUPE`, `ECC_CARTOGRAPHY_DEDUPE_WINDOW` | US-001, US-002 |
| 9 | `crates/ecc-app/src/hook/handlers/tier3_session/cartography/delta_helpers.rs` | modify | ERR-002: convert 23 `let _ =` to `?` / `if let Err` with `HookError::CartographyIo`; remove `#[allow(dead_code)]` and ~500 LOC dead code | US-004, US-005 |
| 10 | `crates/ecc-app/src/hook/handlers/tier3_session/cartography/tests_helpers.rs` | modify | Prune orphaned tests for removed dead-code fns | US-005 / AC-005.3 |
| 11 | `crates/ecc-app/src/hook/handlers/tier3_session/daily.rs` | modify | Reuse `is_noise_path` before `create_dir_all`; env gate via `ECC_DAILY_SUMMARY_NOISE_FILTER` | US-003 / AC-003.1–3 |
| 12 | `crates/ecc-app/tests/fixtures/cartography-corpus/*.json` (10 files) | create | 5 noise-only + 3 mixed + 2 pure-code fixtures in SessionDelta shape | US-008 / AC-008.1 |
| 13 | `crates/ecc-app/tests/fixtures/cartography-corpus/expected.yaml` | create | Sidecar outcome metadata per fixture | US-008 / AC-008.1 |
| 14 | `crates/ecc-app/tests/fixtures/cartography-corpus/README.md` | create | Fixture authoring + signal/noise definition | US-008 / AC-008.4 |
| 15 | `crates/ecc-app/tests/cartography_corpus.rs` | create | Corpus runner; not `#[ignore]` | US-008 / AC-008.2, AC-008.3 |
| **[PR1 MERGE POINT — files 16-27 in PR2]** | | | | |
| 16 | `crates/ecc-domain/src/memory/safe_path.rs` | create | **Pure** `SafePath` newtype: takes pre-canonicalized `(root: PathBuf, child: PathBuf)` and performs string-prefix bounds check + regex validation only. Zero I/O. Canonicalization happens at the app-layer boundary (file 18) | US-006 (security) |
| 17 | `crates/ecc-domain/src/memory/mod.rs` | modify | Register `safe_path` module | US-006 |
| 18 | `crates/ecc-app/src/memory/paths.rs` | create | `resolve_project_memory_root(env, fs) -> Result<SafePath, PathError>` — performs `fs::canonicalize` at the app boundary, then constructs `SafePath` via pure domain newtype; env override + HOME fallback | US-006 / AC-006.0 |
| 19 | `crates/ecc-app/src/memory/file_prune.rs` | create | BL-driven deletion: regex match + trash move + atomic MEMORY.md rewrite | US-006 / AC-006.1–8 |
| 20 | `crates/ecc-app/src/memory/trash_gc.rs` | create | Time-based trash retention (7 days); split per SOLID-001 | US-006 / AC-006.1 |
| 21 | `crates/ecc-app/src/memory/lifecycle.rs` | modify | Add `prune_by_backlog(&dyn MemoryStore, &str) -> Result<u32>` | US-007 / AC-007.1–5 |
| 22 | `crates/ecc-app/src/memory/mod.rs` | modify | Register `paths`, `file_prune`, `trash_gc` submodules | US-006, US-007 |
| 23 | `crates/ecc-app/src/backlog.rs` | modify | Fire-and-forget `PostTransitionHooks` on `update_status` success with `status == "implemented"`; NOT fired from `migrate` | US-006 / AC-006.4, US-007 / AC-007.3 |
| 24 | `crates/ecc-cli/src/commands/memory.rs` | modify | Add `Prune { orphaned_backlogs, apply }` (dry-run default) + `Restore { trash_date, apply }` subcommands | US-006 / AC-006.5, AC-006.6, AC-006.9 |
| 25a | `docs/adr/0068-memory-prune-lifecycle.md` | create | ADR covering memory prune, SafePath (pure-newtype split), env bounds, trash retention, deferred `ECC_CARTOGRAPHY_NOISE_EXTRA` ref | Decision #2 |
| 25b | `docs/adr/0069-cartography-noise-filter-architecture.md` | create | ADR covering cartography Decisions #1, #11, #14, #15 (Stop trigger preserved, N=20 window, flock fail-open, tracing observability) | Decision #1, #11, #14, #15 |
| 26a | `crates/ecc-app/src/status/cartography_metrics.rs` | create | Counter source for `ecc status --json` — surfaces `cartography.skipped_deltas_24h` + `cartography.dedupe_enabled` + `cartography.dedupe_window` + `daily_summary.noise_filter_enabled` via SQLite log_store query over last 24h | Spec Observability + Rollback sections |
| 26b | `crates/ecc-app/src/status/memory_metrics.rs` | create | Counter source — `memory.pruned_files_24h` via log_store + `memory.prune_enabled` flag surface | Spec Observability |
| 26c | `crates/ecc-cli/src/commands/status.rs` | modify | Wire new cartography + memory counter sources into `ecc status --json` payload | Spec Observability |
| 26 | `CHANGELOG.md` | modify | Changed/Removed/Fixed entries under Unreleased (PR1 + PR2 sections) | Doc Impact |
| 27 | `CLAUDE.md`, `docs/commands-reference.md`, `docs/cartography/elements/cartography-system.md`, `docs/MODULE-SUMMARIES.md`, `docs/domain/bounded-contexts.md` | modify | Gotchas + CLI ref + write-time filter section + module summaries + bounded contexts | Doc Impact |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | `is_noise_path` true for each fixed prefix | AC-001.1–4, AC-001.8 | `cargo test -p ecc-domain cartography::noise_filter::tests::classifies_fixed_prefixes_as_noise` | pass |
| PC-002 | unit | Exact matches (`Cargo.lock`, `.claude/workflow`) | AC-001.5, AC-001.8 | `cargo test -p ecc-domain cartography::noise_filter::tests::classifies_exact_matches_as_noise` | pass |
| PC-003 | unit | Crate paths are signal | AC-001.6 | `cargo test -p ecc-domain cartography::noise_filter::tests::crate_paths_are_signal` | pass |
| PC-004 | unit | Case + separator normalization | AC-001.8 | `cargo test -p ecc-domain cartography::noise_filter::tests::normalizes_case_and_separators` | pass |
| PC-005 | unit | Symlink by path only | AC-001.9 | `cargo test -p ecc-domain cartography::noise_filter::tests::symlink_policy_by_path_only` | pass |
| PC-006 | unit | Module is pure (no I/O imports) | AC-001.7 | `cargo test -p ecc-domain cartography::noise_filter::tests::module_has_no_io_imports` | pass |
| PC-007 | unit | `canonical_hash` deterministic across field reorder (serde_jcs) | AC-002.2 | `cargo test -p ecc-domain cartography::dedupe::tests::hash_is_canonical_and_deterministic` | pass |
| PC-008 | unit | SHA-256 64-hex format | AC-002.2 | `cargo test -p ecc-domain cartography::dedupe::tests::hash_format_sha256_hex` | pass |
| PC-009 | unit | `HookError::CartographyIo` display + source chain | AC-004.2 | `cargo test -p ecc-app hook::errors::tests::cartography_io_display` | pass |
| PC-010 | unit | delta_writer skips workflow-only session | AC-001.1 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::filters_workflow_only_session` | pass |
| PC-011 | unit | delta_writer skips spec-only | AC-001.2 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::filters_spec_only_session` | pass |
| PC-012 | unit | delta_writer skips backlog-only | AC-001.3 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::filters_backlog_only_session` | pass |
| PC-013 | unit | delta_writer skips cartography self-ingestion | AC-001.4 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::filters_cartography_output_self_ingestion` | pass |
| PC-014 | unit | delta_writer skips Cargo.lock only | AC-001.5 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::filters_cargo_lock_only` | pass |
| PC-015 | unit | Mixed session retains signal only | AC-001.6 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::mixed_session_retains_signal_only` | pass |
| PC-016 | unit | `tracing::info!` on full-noise skip | AC-001.10 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::emits_filter_info_on_skip` | pass |
| PC-017 | unit | `tracing::debug!` on clean-tree skip | AC-001.11 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::clean_tree_debug_log` | pass |
| PC-018 | unit | Dedupe skips duplicate payloads | AC-002.1 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::dedupe_io::tests::duplicate_payload_skips_write` | pass |
| PC-019 | unit | Dedupe reads via FileSystem port, no new port | AC-002.3 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::dedupe_io::tests::reads_last_n_through_filesystem_port` | pass |
| PC-020 | unit | `ECC_CARTOGRAPHY_DEDUPE=0` disables | AC-002.4 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::dedupe_opt_out_env` | pass |
| PC-021 | unit | Empty post-filter — no hash computed | AC-002.5 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::empty_post_filter_no_hash` | pass |
| PC-022 | unit | ecc-flock timeout → fail-open write | AC-002.6 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::dedupe_io::tests::flock_timeout_fail_open` | pass |
| PC-023 | unit | Scans pending + processed lexdesc | AC-002.7 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::dedupe_io::tests::scans_pending_and_processed_desc` | pass |
| PC-024 | unit | Window=0/1/100 boundaries | AC-002.8 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::dedupe_io::tests::window_0_1_100_boundaries` | pass |
| PC-025 | unit | daily_summary skips noise-only | AC-003.1 | `cargo test -p ecc-app hook::handlers::tier3_session::daily::tests::skips_append_on_noise_only_session` | pass |
| PC-026 | unit | daily_summary appends on signal | AC-003.2 | `cargo test -p ecc-app hook::handlers::tier3_session::daily::tests::appends_when_signal_present` | pass |
| PC-027 | unit | daily_summary reuses domain predicate | AC-003.3 | `cargo test -p ecc-app hook::handlers::tier3_session::daily::tests::reuses_domain_predicate` | pass |
| PC-028 | unit | Zero `let _ =` in delta_helpers.rs | AC-004.1 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::tests_helpers::no_silent_error_suppressions` | pass |
| PC-029 | unit | Existing cartography tests still pass | AC-004.3 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests` | pass |
| PC-030 | lint | Clippy clean on delta_helpers.rs | AC-004.4 | `cargo clippy -p ecc-app --all-targets -- -D warnings` | zero warnings |
| PC-031 | unit | `#[allow(dead_code)]` removed | AC-005.1 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::tests_helpers::no_dead_code_allow` | pass |
| PC-032 | build | cargo check clean after deletion | AC-005.2 | `cargo check --workspace --all-targets` | exit 0 |
| PC-033 | unit | No orphan tests | AC-005.3 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::tests_helpers::no_orphan_tests` | pass |
| PC-034 | unit | `resolve_project_memory_root` env override | AC-006.0 | `cargo test -p ecc-app memory::paths::tests::resolve_root_env_override` | pass |
| PC-035 | unit | Root hash algorithm vectors match memory_write | AC-006.0 | `cargo test -p ecc-app memory::paths::tests::resolve_root_hash_algorithm_vectors` | pass |
| PC-036 | unit | Prune trashes files + updates index atomically | AC-006.1, AC-006.2 | `cargo test -p ecc-app memory::file_prune::tests::trashes_and_updates_index` | pass |
| PC-037 | unit | Prune failure logs warn, does not fail transition | AC-006.3 | `cargo test -p ecc-app backlog::tests::prune_failure_does_not_fail_transition` | pass |
| PC-038 | unit | migrate does NOT prune | AC-006.4 | `cargo test -p ecc-app backlog::tests::migrate_does_not_prune_memory` | pass |
| PC-039 | unit | CLI prune dry-run default | AC-006.5, AC-006.6 | `cargo test -p ecc-cli commands::memory::tests::prune_orphaned_dry_run_default` | pass |
| PC-040 | unit | CLI prune --apply trashes | AC-006.5 | `cargo test -p ecc-cli commands::memory::tests::prune_orphaned_apply_trashes` | pass |
| PC-041 | unit | Prune idempotent | AC-006.7 | `cargo test -p ecc-app memory::file_prune::tests::is_idempotent` | pass |
| PC-042 | unit | BL-ID regex collision safety (BL-10 vs BL-100) | AC-006.8 | `cargo test -p ecc-app memory::file_prune::tests::bl_id_regex_collision_safety` | pass |
| PC-043 | unit | CLI restore lists + applies | AC-006.9 | `cargo test -p ecc-cli commands::memory::tests::restore_lists_and_applies` | pass |
| PC-044 | unit | `prune_by_backlog` returns count | AC-007.1 | `cargo test -p ecc-app memory::lifecycle::tests::prune_by_backlog_returns_count` | pass |
| PC-045 | unit | No new port methods added | AC-007.2 | `cargo test -p ecc-app memory::lifecycle::tests::prune_by_backlog_no_new_port` | pass |
| PC-046 | integration | Hook fires BOTH file + SQLite prune | AC-007.3 | `cargo test -p ecc-app backlog::tests::implemented_transition_prunes_file_and_sqlite` | pass |
| PC-047 | unit | Empty store returns Ok(0) | AC-007.4 | `cargo test -p ecc-app memory::lifecycle::tests::prune_by_backlog_empty_store` | pass |
| PC-048 | integration | Mixed-corpus prune | AC-007.5 | `cargo test -p ecc-app memory::lifecycle::tests::prune_by_backlog_mixed_corpus` | pass |
| PC-049 | integration | Fixture shape present | AC-008.1, AC-008.4 | `cargo test -p ecc-app --test cartography_corpus fixtures_shape_present` | pass |
| PC-050 | integration | Corpus outcomes match expected | AC-008.2 | `cargo test -p ecc-app --test cartography_corpus outcomes_match_expected` | pass |
| PC-051 | integration | Corpus test not ignored | AC-008.3 | `cargo test -p ecc-app --test cartography_corpus` | pass, not ignored |
| PC-052 | lint | Workspace clippy gate | Final | `cargo clippy --workspace --all-targets -- -D warnings` | zero warnings |
| PC-053 | build | Workspace release build | Final | `cargo build --release` | exit 0 |
| PC-054 | regression | Corpus regression gate | Final | `cargo test --test cartography_corpus` | pass |
| PC-101 | unit | SafePath rejects path traversal `../../etc` | SEC-001 | `cargo test -p ecc-domain memory::safe_path::tests::rejects_traversal` | pass |
| PC-102 | unit | SafePath enforces root bounds after canonicalize | SEC-002 | `cargo test -p ecc-domain memory::safe_path::tests::bounds_check_after_canonicalize` | pass |
| PC-103 | unit | ECC_PROJECT_MEMORY_ROOT outside $HOME rejected | SEC-002 | `cargo test -p ecc-app memory::paths::tests::rejects_root_outside_home` | pass |
| PC-104 | unit | CLI restore --trash rejects non-ISO-date | SEC-003 | `cargo test -p ecc-cli commands::memory::tests::restore_rejects_non_iso_date` | pass |
| PC-105 | integration | canonical_hash stable across serde_json versions (snapshot) | SEC-004 | `cargo test -p ecc-domain cartography::dedupe::tests::hash_snapshot_stability` | pass |
| PC-106 | unit | file_prune uses SafePath exclusively (no raw Path/&str) | SEC-001/002 | `cargo test -p ecc-app memory::file_prune::tests::uses_safe_path_only` | pass |
| PC-107 | unit | CLI memory commands use SafePath | SEC-001/002 | `cargo test -p ecc-cli commands::memory::tests::uses_safe_path` | pass |
| PC-108 | unit | trash_gc split is separately testable (GC by date, not BL-ID) | SOLID-001 | `cargo test -p ecc-app memory::trash_gc::tests::gc_by_date_only` | pass |
| PC-109 | unit | ADR-0068 present + references AMV-L + documents SafePath + serde_jcs | Decision #2 | `test -f docs/adr/0068-memory-prune-lifecycle.md && grep -q "SafePath\|AMV-L\|serde_jcs" docs/adr/0068-memory-prune-lifecycle.md` | exit 0 |
| PC-110 | unit | `ecc-domain` has zero `std::fs`, `std::net`, `std::io` imports (purity guard) | SOLID-domain | `cargo test -p ecc-domain architecture::tests::no_io_imports_in_domain` | pass |
| PC-111 | unit | `SafePath::from_canonical(root, child)` rejects `child` that doesn't start with `root` (pure string-prefix check) | SEC-002 | `cargo test -p ecc-domain memory::safe_path::tests::rejects_escape_pure` | pass |
| PC-112 | unit | `resolve_project_memory_root` on non-existent path returns `Err` in CLI context, `None` in hook context | Fragility #2 | `cargo test -p ecc-app memory::paths::tests::missing_root_behaviour_cli_vs_hook` | pass |
| PC-113 | unit | Stale `.dedupe.lock` file (no holder process) — next `should_dedupe` acquires successfully | Fragility #1 | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::dedupe_io::tests::stale_lock_recoverable` | pass |
| PC-114 | unit | `MEMORY.md` atomic rewrite: temp file exists during write, final rename is atomic; partial read impossible | Fragility #3 | `cargo test -p ecc-app memory::file_prune::tests::memory_md_atomic_rewrite` | pass |
| PC-115 | unit | `update_status` with explicit `call_hooks: false` (migrate variant) does not fire prune for ANY status transition (table-driven: open→in-progress, open→archived, open→promoted, in-progress→implemented via migrate) | Fragility #4 | `cargo test -p ecc-app backlog::tests::migrate_variant_never_fires_hooks` | pass |
| PC-116 | unit | Non-`implemented` status transitions from `update_status` (with hooks enabled) do NOT fire prune (`open→in-progress`, `in-progress→archived`, `→promoted`) | Fragility #4 | `cargo test -p ecc-app backlog::tests::non_implemented_transitions_skip_prune` | pass |
| PC-117 | integration | `ECC_DAILY_SUMMARY_NOISE_FILTER=0` + noise-only session → daily entry IS appended (soft-rollback knob verified) | Rollback soft-flag | `cargo test -p ecc-app hook::handlers::tier3_session::daily::tests::noise_filter_env_disabled_appends` | pass |
| PC-118 | unit | `ecc status --json` payload contains `cartography.skipped_deltas_24h`, `cartography.dedupe_enabled`, `cartography.dedupe_window`, `memory.pruned_files_24h`, `daily_summary.noise_filter_enabled` | Spec Observability | `cargo test -p ecc-cli commands::status::tests::status_json_exposes_counters` | pass |
| PC-119 | integration | Concurrent `update_status(BL-X, implemented)` × 2 — only one triggers prune (idempotent under race) | Fragility concurrency | `cargo test -p ecc-app backlog::tests::concurrent_update_status_idempotent_prune` | pass |
| PC-120 | unit | Memory-prune write-guard compatibility: paths under `~/.claude/projects/<hash>/memory/` are explicitly allowed by write-guard hook (path allow-listed) | Fragility #6 | `cargo test -p ecc-app hook::handlers::tier0_pretool::write_guard::tests::memory_root_allowed` | pass |
| PC-121 | unit | AC-001.8 exact match: `.claude/workflow` bare string (no trailing slash) is classified noise | AC-001.8 (full) | `cargo test -p ecc-domain cartography::noise_filter::tests::bare_workflow_exact_noise` | pass |

**Total: 68 PCs** (48 original ACs covered + 5 security + 12 fragility/observability/coverage additions from adversary round 1 and round 2).

### Coverage Check

All 48 spec ACs mapped to ≥ 1 PC:

- AC-001.1 → PC-010; AC-001.2 → PC-011; AC-001.3 → PC-012; AC-001.4 → PC-013; AC-001.5 → PC-014; AC-001.6 → PC-015; AC-001.7 → PC-006; AC-001.8 → PC-001, PC-002, PC-004; AC-001.9 → PC-005; AC-001.10 → PC-016; AC-001.11 → PC-017
- AC-002.1 → PC-018; AC-002.2 → PC-007, PC-008; AC-002.3 → PC-019; AC-002.4 → PC-020; AC-002.5 → PC-021; AC-002.6 → PC-022; AC-002.7 → PC-023; AC-002.8 → PC-024
- AC-003.1 → PC-025; AC-003.2 → PC-026; AC-003.3 → PC-027
- AC-004.1 → PC-028; AC-004.2 → PC-009; AC-004.3 → PC-029; AC-004.4 → PC-030
- AC-005.1 → PC-031; AC-005.2 → PC-032; AC-005.3 → PC-033
- AC-006.0 → PC-034, PC-035; AC-006.1 → PC-036; AC-006.2 → PC-036; AC-006.3 → PC-037; AC-006.4 → PC-038; AC-006.5 → PC-039, PC-040; AC-006.6 → PC-039; AC-006.7 → PC-041; AC-006.8 → PC-042; AC-006.9 → PC-043
- AC-007.1 → PC-044; AC-007.2 → PC-045; AC-007.3 → PC-046; AC-007.4 → PC-047; AC-007.5 → PC-048
- AC-008.1 → PC-049; AC-008.2 → PC-050; AC-008.3 → PC-051; AC-008.4 → PC-049

**All 48 ACs covered. Zero uncovered.**

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| E2E-1 | `FileSystem` (dedupe read) | `FsFileSystem` | `FileSystem` | Stop-cartography reads last-N from real FS without new port | ignored | Filter or dedupe logic modified |
| E2E-2 | `MemoryStore` (prune loop) | `SqliteMemoryStore` | `MemoryStore` | `prune_by_backlog` on real SQLite deletes tagged entries | ignored | Memory lifecycle modified |
| E2E-3 | CLI `memory prune` | binary | — | `ecc memory prune --orphaned-backlogs --dry-run` lists expected files | ignored | CLI memory surface modified |
| E2E-4 | CLI `memory restore` | binary | — | `ecc memory restore --trash <date> --apply` restores + updates MEMORY.md | ignored | CLI restore surface modified |
| E2E-5 | Backlog transition hook | app integration | — | `ecc backlog update-status BL-X implemented` fires both prunes | ignored | Backlog hook or memory prune modified |

### E2E Activation Rules

All 5 E2E tests are **un-ignored** during this implementation because every row matches at least one file change. Integration PCs (PC-046, PC-048, PC-050) use in-memory test doubles to cover the same flows at unit-test speed; E2E tests run in nightly/CI scope only.

## Test Strategy

TDD order (dependency-respecting):

**PR1 sequence**:
1. PC-001 → PC-006 (noise_filter domain foundation, RED→GREEN→REFACTOR)
2. PC-007 → PC-008, PC-105 (dedupe domain + serde_jcs stability)
3. PC-009 (HookError enum — blocks ERR-002)
4. PC-010 → PC-017 (delta_writer filter integration + observability)
5. PC-021 (empty post-filter)
6. PC-018 → PC-020, PC-022 → PC-024 (dedupe_io integration + env + flock)
7. PC-025 → PC-027 (daily-summary)
8. PC-028 → PC-030 (ERR-002 conversion)
9. PC-031 → PC-033 (dead-code removal — strictly after PC-030)
10. PC-049 → PC-051 (corpus)
11. PC-052 → PC-054 (PR1 final gates)
**→ PR1 MERGE**

**PR2 sequence** (new worktree after PR1 merged):
12. PC-101 → PC-102 (SafePath domain)
13. PC-034 → PC-035, PC-103 (paths + env bounds)
14. PC-041 → PC-042 (idempotence + regex)
15. PC-036 (trash + atomic index)
16. PC-037 → PC-038 (hook integration, migrate-excluded)
17. PC-044 → PC-045, PC-047 (prune_by_backlog)
18. PC-046, PC-048 (cross-source integration)
19. PC-039 → PC-040, PC-043, PC-104, PC-106 → PC-107 (CLI + SafePath integration)
20. PC-108 (trash_gc separation)
21. PC-109 (ADR-0068 presence)
22. Final gates (clippy, build)

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1a | `docs/adr/0068-memory-prune-lifecycle.md` | HIGH | create | ADR: file + SQLite prune, SafePath (pure-newtype split), trash 7-day retention, serde_jcs choice, AMV-L ref, `ECC_CARTOGRAPHY_NOISE_EXTRA` deferred-extension pointer | Decision #2 |
| 1b | `docs/adr/0069-cartography-noise-filter-architecture.md` | HIGH | create | ADR: Stop trigger preserved (not post-commit), noise-set enumeration, N=20 window rationale, flock fail-open trade-off, tracing observability targets | Decisions #1, #11, #14, #15 |
| 2 | `docs/cartography/elements/cartography-system.md` | HIGH | modify | Add "Write-Time Noise Filter" section + Mermaid Diagram 1 | Doc Impact #1 |
| 3 | `CHANGELOG.md` | MEDIUM | modify | Unreleased Changed/Removed/Fixed entries for PR1 (cartography + ERR-002 + dead-code) and PR2 (memory prune + CLI) | Doc Impact #4 |
| 4 | `CLAUDE.md` | MEDIUM | modify | Gotchas: filter path list, env vars, memory prune on backlog transition | Doc Impact #5 |
| 5 | `docs/commands-reference.md` | MEDIUM | modify | Add `ecc memory prune --orphaned-backlogs [--apply]` and `ecc memory restore --trash <date> [--apply]` | Doc Impact #3 |
| 6 | `docs/MODULE-SUMMARIES.md` | MEDIUM | modify | Add 6 new module summary entries (noise_filter, dedupe, safe_path, file_prune, trash_gc, paths) | Doc Impact |
| 7 | `docs/domain/bounded-contexts.md` | MEDIUM | modify | Register cartography (noise_filter, dedupe) + memory (safe_path) as domain services | Doc Impact |
| 8 | `docs/ARCHITECTURE.md` | MEDIUM | modify | Add hexagonal-layer notes for cartography filter, memory prune, SafePath boundary | Doc Impact |
| 9 | `crates/ecc-app/tests/fixtures/cartography-corpus/README.md` | LOW | create | Fixture authoring + signal/noise definition | Doc Impact #7 |
| 10 | Release notes (auto) | LOW | — | Note ~500 LOC legacy removal | Doc Impact #6 |

**Mandatory**: CHANGELOG.md entry (row 3) ✓. ADR entry for Decision #2 ✓ (row 1).

## SOLID Assessment

(from composite design-reviewer)

- [SOLID-001] MEDIUM — `file_prune.rs` had 4 concerns → **addressed by design**: split into `paths.rs` (resolve), `file_prune.rs` (BL-driven delete), `trash_gc.rs` (time-based GC).
- [SOLID-002] MEDIUM — Fixed noise set closed against new categories → **deferred**: domain constants remain; `ECC_CARTOGRAPHY_NOISE_EXTRA` extension deferred to follow-up BL to keep domain pure. Documented in ADR-0068.
- [SOLID-003] LOW — DIP on `PostTransitionHooks` → **addressed**: struct lives in app layer; horizontal coupling between `backlog` and `memory` submodules documented as deliberate in ADR-0068 (single subscriber, no port justified).

**`sha2` + `serde_jcs` in `ecc-domain`**: CLEAN — both are pure compute, no I/O. Same category as accepted `serde`. Comment at import.

**Verdict: PASS** with 2 MEDIUM addressed + 1 LOW acknowledged.

## Robert's Oath Check

- Oath 1 (harmful code): CLEAN — trash-not-delete + dry-run-default + atomic status transition before hook fires.
- Oath 2 (no mess): CLEAN — 500 LOC dead-code removal + 23 `let _ =` → `?` conversion.
- Oath 3 (proof): CLEAN — 68 PCs / 48 ACs = 1.42:1 ratio including 5 security + 12 fragility/observability/coverage PCs from two adversary rounds.
- Oath 4 (small releases): WARNING → **addressed**: split into PR1 (11 files) + PR2 (11 files) sequential.
- Oath 5 (fearless improvement): CLEAN — 3 scouted consts/renames bounded.
- Oath 8 (honest estimates): WARNING → **addressed**: dead-code list enumerated in spec's US-005 acceptance criteria (process_cartography, collect_pending_deltas, clean_corrupt_deltas and their helpers).

**Verdict: CLEAN** after addressing the two WARNING items.

## Security Notes

(from composite design-reviewer — all HIGH findings addressed in design)

- [SEC-001] HIGH path traversal via BL-ID → **addressed**: `SafePath` newtype in `ecc-domain::memory::safe_path` enforces regex + canonicalize + bounds-check. PC-101, PC-106.
- [SEC-002] HIGH `ECC_PROJECT_MEMORY_ROOT` escape → **addressed**: `resolve_project_memory_root` rejects canonical paths outside `$HOME`. PC-102, PC-103.
- [SEC-003] HIGH CLI `--trash <date>` injection → **addressed**: regex `^\d{4}-\d{2}-\d{2}$` validation before `PathBuf::join`. PC-104.
- [SEC-004] MEDIUM serde_json canonicalization drift → **addressed**: use `serde_jcs` (RFC 8785) with pinned version. PC-105 snapshot test.
- [SEC-005] MEDIUM flock fail-open DoS → **accepted trade-off**: local single-user context; documented in ADR-0068 § Security.

**Verdict: CLEAR** after SafePath + serde_jcs integration.

## Rollback Plan

Reverse dependency order (PR2 first if both merged, else PR1 alone):

**PR2 rollback**:
1. `git revert` PR2 merge commit → restores backlog.rs, memory/* to pre-PR2 state
2. No data migration — trashed files remain in `<memory_root>/.trash/<date>/`; user recovers manually or leaves for 7-day GC
3. SQLite MemoryStore entries pruned by PR2 are permanently gone unless restored from backup (acceptable: they were stale by definition)

**PR1 rollback**:
1. `git revert` PR1 merge → restores delta_writer/helpers/daily/errors to pre-PR1 state
2. No data migration — 158 archived deltas unchanged; new filter is write-time only
3. Feature flag escape hatches (`ECC_CARTOGRAPHY_DEDUPE=0`, `ECC_DAILY_SUMMARY_NOISE_FILTER=0`) allow soft-rollback without revert if the filter misbehaves in production

**Cross-PR rollback policy** (rewritten per adversary clarification):

- **PR2 revert alone**: **SAFE**. PR2 adds no consumer of PR1 cartography code; PR2 memory code touches zero cartography types. PR1 continues operating unchanged. Integration PC verifies no cross-PR symbol reference (PC-NEW-ISO, see below).
- **PR1 revert alone**: **SAFE**. PR2 memory code does not import `is_noise_path`, `canonical_hash`, or cartography types. Memory prune operates independently.
- **Both reverted**: trivial; return to pre-fix state.
- **Trash orphaning after PR2 revert**: trashed files under `<memory_root>/.trash/<date>/` persist because `trash_gc.rs` was deleted by the revert. **Operator runbook**: manually `rm -rf <memory_root>/.trash/` or cherry-pick only `trash_gc.rs` and the gc call site back. Document this escape hatch in CHANGELOG's rollback note.

**PC-122**: grep assertion that no PR2 file imports any cartography module:
```
cargo test -p ecc-app integration::tests::pr2_no_cartography_imports
```
Expected: pass (no imports).

## Bounded Contexts Affected

| Context | Role | Files Modified |
|---------|------|----------------|
| cartography | domain service (new pure predicates) | `crates/ecc-domain/src/cartography/noise_filter.rs`, `dedupe.rs`, `mod.rs` |
| memory | domain service (new SafePath value object) | `crates/ecc-domain/src/memory/safe_path.rs`, `mod.rs` |

Other domain modules (not registered as bounded contexts):
- `ecc-app::memory::{paths, file_prune, trash_gc, lifecycle}`: app-layer use cases orchestrating the domain + MemoryStore port
- `ecc-app::hook::handlers::tier3_session::cartography::dedupe_io`: app-layer orchestration
- `ecc-app::hook::errors`: app-layer error taxonomy
- `ecc-app::backlog`: app-layer workflow orchestration (hook site)
- `ecc-cli::commands::memory`: CLI surface

## Security Considerations

Added per composite review. Full rationale in ADR-0068.

- **SafePath boundary newtype** — closes path traversal, env escape, CLI injection at the type level; all memory-prune functions take `SafePath`, not `&str` / `&Path`. Constructors perform regex validation + `fs::canonicalize` + `starts_with(root)` bounds check; returns `SafePathError::{Invalid, Escape, Io}` on failure.
- **`ECC_PROJECT_MEMORY_ROOT` bounds** — resolved path must canonicalize under `$HOME` (or an explicit `$CLAUDE_PROJECT_DIR/.claude` allow-list); symlinks resolved; reject on escape.
- **CLI `restore --trash <date>` input sanitization** — regex-validate ISO date format before path construction; use `PathBuf::join` (not string concat).
- **`serde_jcs` canonical JSON** — RFC 8785 deterministic serialization for SHA-256 input; pinned version avoids hash drift across `serde_json` minor bumps.
- **Flock fail-open** — documented trade-off: 500 ms timeout prefers benign duplicate over lost write; local single-user context; integration test demonstrates idempotent downstream consumption.

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS (2 MEDIUM addressed, 1 LOW acknowledged) | 3 |
| Robert (Oath) | CLEAN (2 WARNINGS addressed: Oath 4 via PR split, Oath 8 via enumerated dead-code list) | 2 |
| Security | CLEAR (3 HIGH addressed via SafePath + env bounds + regex sanitization; 2 MEDIUM addressed via serde_jcs + documented flock trade-off) | 5 |

### Adversary Findings

| Dimension | R1 | R2 | Key Rationale |
|-----------|----|----|---------------|
| AC Coverage | 82 | 95 | AC-001.8 bare-string covered by PC-121 |
| Execution Order | 78 | 88 | PR1/PR2 split preserves acyclic TDD deps |
| Fragility | 55 | 85 | PC-112..117, PC-119..120 close all round-1 gaps (missing-root, stale lock, atomic rewrite, migrate variant, concurrent races, write-guard allow-list) |
| Rollback | 60 | 85 | PC-122 enforces PR2→PR1 isolation; trash-orphan runbook; soft-rollback env flag PC-117 |
| Architecture | 58 | 92 | SafePath refactored to pure newtype; `fs::canonicalize` moved to app boundary; PC-110 asserts zero I/O in domain |
| Blast Radius | 75 | 80 | PR1 (15 files) + PR2 (~12 files); cross-crate justified |
| Missing PCs | 62 | 90 | 12 new PCs added across fragility/observability/coverage |
| Doc Plan | 80 | 82 | ADR-0068 (memory) + ADR-0069 (cartography) split |
| **Average** | **68.75** | **87.0** | **Verdict: PASS** |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `ecc-domain/src/cartography/noise_filter.rs` | create | US-001 |
| 2 | `ecc-domain/src/cartography/mod.rs` | modify | US-001 |
| 3 | `ecc-domain/src/cartography/dedupe.rs` | create | US-002 |
| 4 | `ecc-domain/Cargo.toml` | modify | US-002 |
| 5 | `ecc-app/src/hook/errors.rs` | create | US-004 |
| 6 | `ecc-app/src/hook/mod.rs` | modify | US-004 |
| 7 | `ecc-app/src/hook/handlers/tier3_session/cartography/dedupe_io.rs` | create | US-002 |
| 8 | `ecc-app/src/hook/handlers/tier3_session/cartography/delta_writer.rs` | modify | US-001, US-002 |
| 9 | `ecc-app/src/hook/handlers/tier3_session/cartography/delta_helpers.rs` | modify | US-004, US-005 |
| 10 | `ecc-app/src/hook/handlers/tier3_session/cartography/tests_helpers.rs` | modify | US-005 |
| 11 | `ecc-app/src/hook/handlers/tier3_session/daily.rs` | modify | US-003 |
| 12-14 | `ecc-app/tests/fixtures/cartography-corpus/` (10 JSON + yaml + README) | create | US-008 |
| 15 | `ecc-app/tests/cartography_corpus.rs` | create | US-008 |
| 16 | `ecc-domain/src/memory/safe_path.rs` | create | US-006 (SEC) |
| 17 | `ecc-domain/src/memory/mod.rs` | modify | US-006 |
| 18 | `ecc-app/src/memory/paths.rs` | create | US-006 |
| 19 | `ecc-app/src/memory/file_prune.rs` | create | US-006 |
| 20 | `ecc-app/src/memory/trash_gc.rs` | create | US-006 |
| 21 | `ecc-app/src/memory/lifecycle.rs` | modify | US-007 |
| 22 | `ecc-app/src/memory/mod.rs` | modify | US-006, US-007 |
| 23 | `ecc-app/src/backlog.rs` | modify | US-006, US-007 |
| 24 | `ecc-cli/src/commands/memory.rs` | modify | US-006 |
| 25a | `docs/adr/0068-memory-prune-lifecycle.md` | create | Decision #2 |
| 25b | `docs/adr/0069-cartography-noise-filter-architecture.md` | create | Decisions #1, #11, #14, #15 |
| 26a | `ecc-app/src/status/cartography_metrics.rs` | create | Observability |
| 26b | `ecc-app/src/status/memory_metrics.rs` | create | Observability |
| 26c | `ecc-cli/src/commands/status.rs` | modify | Observability |
| 27 | `CHANGELOG.md`, `CLAUDE.md`, `docs/commands-reference.md`, `docs/cartography/elements/cartography-system.md`, `docs/MODULE-SUMMARIES.md`, `docs/domain/bounded-contexts.md`, `docs/ARCHITECTURE.md` | modify | Doc Impact |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-19-reduce-cartography-memory-noise/design.md` | Full design (12 sections + Phase Summary) |
| `docs/specs/2026-04-19-reduce-cartography-memory-noise/campaign.md` | 6 grill-me decisions + PR-split decision |
| `<git-dir>/ecc-workflow/state.json` | Phase: implement; design_path persisted |
