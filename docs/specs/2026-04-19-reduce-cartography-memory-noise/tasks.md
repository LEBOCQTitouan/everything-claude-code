# Tasks: Reduce Cartography and Memory System Noise

## Pass Conditions

### Phase D — Domain pure functions (PR1)

- [ ] PC-001: is_noise_path classifies fixed prefixes as noise | `cargo test -p ecc-domain cartography::noise_filter::tests::classifies_fixed_prefixes_as_noise` | pending@2026-04-19T00:00:00Z
- [ ] PC-002: is_noise_path classifies exact matches as noise | `cargo test -p ecc-domain cartography::noise_filter::tests::classifies_exact_matches_as_noise` | pending@2026-04-19T00:00:00Z
- [ ] PC-003: is_noise_path — crate paths are signal | `cargo test -p ecc-domain cartography::noise_filter::tests::crate_paths_are_signal` | pending@2026-04-19T00:00:00Z
- [ ] PC-004: is_noise_path normalizes case + separators | `cargo test -p ecc-domain cartography::noise_filter::tests::normalizes_case_and_separators` | pending@2026-04-19T00:00:00Z
- [ ] PC-005: is_noise_path — symlink policy by path only | `cargo test -p ecc-domain cartography::noise_filter::tests::symlink_policy_by_path_only` | pending@2026-04-19T00:00:00Z
- [ ] PC-006: noise_filter module has no I/O imports | `cargo test -p ecc-domain cartography::noise_filter::tests::module_has_no_io_imports` | pending@2026-04-19T00:00:00Z
- [ ] PC-007: canonical_hash deterministic via serde_jcs | `cargo test -p ecc-domain cartography::dedupe::tests::hash_is_canonical_and_deterministic` | pending@2026-04-19T00:00:00Z
- [ ] PC-008: canonical_hash SHA-256 64-hex format | `cargo test -p ecc-domain cartography::dedupe::tests::hash_format_sha256_hex` | pending@2026-04-19T00:00:00Z
- [ ] PC-105: canonical_hash snapshot stability across serde_json versions | `cargo test -p ecc-domain cartography::dedupe::tests::hash_snapshot_stability` | pending@2026-04-19T00:00:00Z
- [ ] PC-110: ecc-domain has zero std::fs/net/io imports | `cargo test -p ecc-domain architecture::tests::no_io_imports_in_domain` | pending@2026-04-19T00:00:00Z
- [ ] PC-121: bare-string `.claude/workflow` exact match classified noise | `cargo test -p ecc-domain cartography::noise_filter::tests::bare_workflow_exact_noise` | pending@2026-04-19T00:00:00Z

### Phase E — Error enum

- [ ] PC-009: HookError::CartographyIo display + source chain | `cargo test -p ecc-app hook::errors::tests::cartography_io_display` | pending@2026-04-19T00:00:00Z

### Phase C — Cartography integration

- [ ] PC-010: filters workflow-only session | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::filters_workflow_only_session` | pending@2026-04-19T00:00:00Z
- [ ] PC-011: filters spec-only session | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::filters_spec_only_session` | pending@2026-04-19T00:00:00Z
- [ ] PC-012: filters backlog-only session | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::filters_backlog_only_session` | pending@2026-04-19T00:00:00Z
- [ ] PC-013: filters cartography self-ingestion | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::filters_cartography_output_self_ingestion` | pending@2026-04-19T00:00:00Z
- [ ] PC-014: filters Cargo.lock only | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::filters_cargo_lock_only` | pending@2026-04-19T00:00:00Z
- [ ] PC-015: mixed session retains signal only | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::mixed_session_retains_signal_only` | pending@2026-04-19T00:00:00Z
- [ ] PC-016: emits filter info on skip | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::emits_filter_info_on_skip` | pending@2026-04-19T00:00:00Z
- [ ] PC-017: clean-tree debug log | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::clean_tree_debug_log` | pending@2026-04-19T00:00:00Z
- [ ] PC-021: empty post-filter — no hash | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::empty_post_filter_no_hash` | pending@2026-04-19T00:00:00Z
- [ ] PC-018: duplicate payload skips write | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::dedupe_io::tests::duplicate_payload_skips_write` | pending@2026-04-19T00:00:00Z
- [ ] PC-019: reads last N through FileSystem port | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::dedupe_io::tests::reads_last_n_through_filesystem_port` | pending@2026-04-19T00:00:00Z
- [ ] PC-020: dedupe opt-out env var | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests::dedupe_opt_out_env` | pending@2026-04-19T00:00:00Z
- [ ] PC-022: flock timeout fail-open | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::dedupe_io::tests::flock_timeout_fail_open` | pending@2026-04-19T00:00:00Z
- [ ] PC-023: scans pending and processed desc | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::dedupe_io::tests::scans_pending_and_processed_desc` | pending@2026-04-19T00:00:00Z
- [ ] PC-024: window 0/1/100 boundaries | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::dedupe_io::tests::window_0_1_100_boundaries` | pending@2026-04-19T00:00:00Z
- [ ] PC-113: stale lock recoverable | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::dedupe_io::tests::stale_lock_recoverable` | pending@2026-04-19T00:00:00Z

### Phase C+ — Daily summary

- [ ] PC-025: daily skips noise-only | `cargo test -p ecc-app hook::handlers::tier3_session::daily::tests::skips_append_on_noise_only_session` | pending@2026-04-19T00:00:00Z
- [ ] PC-026: daily appends when signal present | `cargo test -p ecc-app hook::handlers::tier3_session::daily::tests::appends_when_signal_present` | pending@2026-04-19T00:00:00Z
- [ ] PC-027: reuses domain predicate | `cargo test -p ecc-app hook::handlers::tier3_session::daily::tests::reuses_domain_predicate` | pending@2026-04-19T00:00:00Z
- [ ] PC-117: daily env flag disabled → appends | `cargo test -p ecc-app hook::handlers::tier3_session::daily::tests::noise_filter_env_disabled_appends` | pending@2026-04-19T00:00:00Z

### Phase H — delta_helpers cleanup

- [ ] PC-028: no silent error suppressions | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::tests_helpers::no_silent_error_suppressions` | pending@2026-04-19T00:00:00Z
- [ ] PC-029: existing cartography tests still pass | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::delta_writer::tests` | pending@2026-04-19T00:00:00Z
- [ ] PC-030: clippy clean on ecc-app | `cargo clippy -p ecc-app --all-targets -- -D warnings` | pending@2026-04-19T00:00:00Z
- [ ] PC-031: no dead_code allow | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::tests_helpers::no_dead_code_allow` | pending@2026-04-19T00:00:00Z
- [ ] PC-032: cargo check workspace clean | `cargo check --workspace --all-targets` | pending@2026-04-19T00:00:00Z
- [ ] PC-033: no orphan tests | `cargo test -p ecc-app hook::handlers::tier3_session::cartography::tests_helpers::no_orphan_tests` | pending@2026-04-19T00:00:00Z

### Phase R — Corpus (PR1)

- [ ] PC-049: corpus fixture shape present | `cargo test -p ecc-app --test cartography_corpus fixtures_shape_present` | pending@2026-04-19T00:00:00Z
- [ ] PC-050: corpus outcomes match expected | `cargo test -p ecc-app --test cartography_corpus outcomes_match_expected` | pending@2026-04-19T00:00:00Z
- [ ] PC-051: corpus test not ignored | `cargo test -p ecc-app --test cartography_corpus` | pending@2026-04-19T00:00:00Z

### Phase D+ — SafePath (PR2 domain)

- [ ] PC-101: SafePath rejects traversal | `cargo test -p ecc-domain memory::safe_path::tests::rejects_traversal` | pending@2026-04-19T00:00:00Z
- [ ] PC-111: SafePath from_canonical rejects escape | `cargo test -p ecc-domain memory::safe_path::tests::rejects_escape_pure` | pending@2026-04-19T00:00:00Z
- [ ] PC-102: SafePath bounds check after canonicalize | `cargo test -p ecc-domain memory::safe_path::tests::bounds_check_after_canonicalize` | pending@2026-04-19T00:00:00Z

### Phase M — File-based memory prune

- [ ] PC-103: ECC_PROJECT_MEMORY_ROOT outside $HOME rejected | `cargo test -p ecc-app memory::paths::tests::rejects_root_outside_home` | pending@2026-04-19T00:00:00Z
- [ ] PC-034: resolve root env override | `cargo test -p ecc-app memory::paths::tests::resolve_root_env_override` | pending@2026-04-19T00:00:00Z
- [ ] PC-035: resolve root hash algorithm vectors | `cargo test -p ecc-app memory::paths::tests::resolve_root_hash_algorithm_vectors` | pending@2026-04-19T00:00:00Z
- [ ] PC-112: missing-root behavior cli vs hook | `cargo test -p ecc-app memory::paths::tests::missing_root_behaviour_cli_vs_hook` | pending@2026-04-19T00:00:00Z
- [ ] PC-041: prune is idempotent | `cargo test -p ecc-app memory::file_prune::tests::is_idempotent` | pending@2026-04-19T00:00:00Z
- [ ] PC-042: BL-ID regex collision safety | `cargo test -p ecc-app memory::file_prune::tests::bl_id_regex_collision_safety` | pending@2026-04-19T00:00:00Z
- [ ] PC-036: trashes and updates index | `cargo test -p ecc-app memory::file_prune::tests::trashes_and_updates_index` | pending@2026-04-19T00:00:00Z
- [ ] PC-114: MEMORY.md atomic rewrite | `cargo test -p ecc-app memory::file_prune::tests::memory_md_atomic_rewrite` | pending@2026-04-19T00:00:00Z
- [ ] PC-037: prune failure does not fail transition | `cargo test -p ecc-app backlog::tests::prune_failure_does_not_fail_transition` | pending@2026-04-19T00:00:00Z
- [ ] PC-038: migrate does NOT prune memory | `cargo test -p ecc-app backlog::tests::migrate_does_not_prune_memory` | pending@2026-04-19T00:00:00Z
- [ ] PC-115: migrate variant never fires hooks | `cargo test -p ecc-app backlog::tests::migrate_variant_never_fires_hooks` | pending@2026-04-19T00:00:00Z
- [ ] PC-116: non-implemented transitions skip prune | `cargo test -p ecc-app backlog::tests::non_implemented_transitions_skip_prune` | pending@2026-04-19T00:00:00Z
- [ ] PC-119: concurrent update_status idempotent | `cargo test -p ecc-app backlog::tests::concurrent_update_status_idempotent_prune` | pending@2026-04-19T00:00:00Z
- [ ] PC-120: write-guard allows memory root | `cargo test -p ecc-app hook::handlers::tier0_pretool::write_guard::tests::memory_root_allowed` | pending@2026-04-19T00:00:00Z

### Phase S — SQLite prune

- [ ] PC-044: prune_by_backlog returns count | `cargo test -p ecc-app memory::lifecycle::tests::prune_by_backlog_returns_count` | pending@2026-04-19T00:00:00Z
- [ ] PC-045: no new port methods | `cargo test -p ecc-app memory::lifecycle::tests::prune_by_backlog_no_new_port` | pending@2026-04-19T00:00:00Z
- [ ] PC-047: empty store returns Ok(0) | `cargo test -p ecc-app memory::lifecycle::tests::prune_by_backlog_empty_store` | pending@2026-04-19T00:00:00Z
- [ ] PC-046: implemented transition prunes file and SQLite | `cargo test -p ecc-app backlog::tests::implemented_transition_prunes_file_and_sqlite` | pending@2026-04-19T00:00:00Z
- [ ] PC-048: prune_by_backlog mixed corpus | `cargo test -p ecc-app memory::lifecycle::tests::prune_by_backlog_mixed_corpus` | pending@2026-04-19T00:00:00Z

### Phase L — CLI surface

- [ ] PC-039: CLI prune dry-run default | `cargo test -p ecc-cli commands::memory::tests::prune_orphaned_dry_run_default` | pending@2026-04-19T00:00:00Z
- [ ] PC-040: CLI prune --apply trashes | `cargo test -p ecc-cli commands::memory::tests::prune_orphaned_apply_trashes` | pending@2026-04-19T00:00:00Z
- [ ] PC-043: CLI restore lists and applies | `cargo test -p ecc-cli commands::memory::tests::restore_lists_and_applies` | pending@2026-04-19T00:00:00Z
- [ ] PC-104: CLI restore rejects non-ISO date | `cargo test -p ecc-cli commands::memory::tests::restore_rejects_non_iso_date` | pending@2026-04-19T00:00:00Z
- [ ] PC-106: file_prune uses SafePath only | `cargo test -p ecc-app memory::file_prune::tests::uses_safe_path_only` | pending@2026-04-19T00:00:00Z
- [ ] PC-107: CLI memory uses SafePath | `cargo test -p ecc-cli commands::memory::tests::uses_safe_path` | pending@2026-04-19T00:00:00Z
- [ ] PC-108: trash_gc by date only | `cargo test -p ecc-app memory::trash_gc::tests::gc_by_date_only` | pending@2026-04-19T00:00:00Z

### Phase Observability

- [ ] PC-118: ecc status --json exposes counters | `cargo test -p ecc-cli commands::status::tests::status_json_exposes_counters` | pending@2026-04-19T00:00:00Z

### Phase F — Final gates

- [ ] PC-109: ADR-0068 present + references AMV-L/SafePath/serde_jcs | `test -f docs/adr/0068-memory-prune-lifecycle.md && grep -q "SafePath\|AMV-L\|serde_jcs" docs/adr/0068-memory-prune-lifecycle.md` | pending@2026-04-19T00:00:00Z
- [ ] PC-122: no PR2 file imports cartography module | `cargo test -p ecc-app integration::tests::pr2_no_cartography_imports` | pending@2026-04-19T00:00:00Z
- [ ] PC-052: workspace clippy gate | `cargo clippy --workspace --all-targets -- -D warnings` | pending@2026-04-19T00:00:00Z
- [ ] PC-053: workspace release build | `cargo build --release` | pending@2026-04-19T00:00:00Z
- [ ] PC-054: corpus regression gate | `cargo test --test cartography_corpus` | pending@2026-04-19T00:00:00Z

## Post-TDD

- [ ] E2E tests | pending@2026-04-19T00:00:00Z
- [ ] Code review | pending@2026-04-19T00:00:00Z
- [ ] Doc updates | pending@2026-04-19T00:00:00Z
- [ ] Supplemental docs | pending@2026-04-19T00:00:00Z
- [ ] Write implement-done.md | pending@2026-04-19T00:00:00Z
