# Tasks: Deterministic Hook System Redesign

## Pass Conditions

- [ ] PC-028: Session start hook characterization | `cargo test -p ecc-integration-tests characterization_session_hooks::session_start_characterization` | pending
- [ ] PC-029: Session end hook characterization | `cargo test -p ecc-integration-tests characterization_session_hooks::session_end_characterization` | pending
- [ ] PC-030: merge_hooks_typed adds new hooks | `cargo test -p ecc-integration-tests characterization_typed_merge::add_new_hooks` | pending
- [ ] PC-031: merge_hooks_typed updates existing | `cargo test -p ecc-integration-tests characterization_typed_merge::update_existing_hooks` | pending
- [ ] PC-032: remove_legacy_hooks_typed removes legacy | `cargo test -p ecc-integration-tests characterization_typed_merge::remove_legacy_hooks` | pending
- [ ] PC-033: merge_hooks_typed preserves customizations | `cargo test -p ecc-integration-tests characterization_typed_merge::preserve_user_customizations` | pending
- [ ] PC-034: Full workflow lifecycle E2E | `cargo test -p ecc-integration-tests characterization_workflow_lifecycle` | pending
- [ ] PC-035: Worktree isolation test | `cargo test -p ecc-integration-tests characterization_worktree_isolation -- --ignored` | pending
- [ ] PC-001: normalize_path strips `..` | `cargo test -p ecc-domain workflow::path::tests` | pending
- [ ] PC-002: normalize_path strips `.` | `cargo test -p ecc-domain workflow::path::tests` | pending
- [ ] PC-003: normalize_path preserves absolute | `cargo test -p ecc-domain workflow::path::tests` | pending
- [ ] PC-004: normalize_path complex traversal | `cargo test -p ecc-domain workflow::path::tests` | pending
- [ ] PC-005: Phase gate blocks traversal | `cargo test -p ecc-workflow phase_gate::tests::phase_gate_blocks_traversal_attack` | pending
- [ ] PC-006: Phase gate blocks absolute outside | `cargo test -p ecc-workflow phase_gate::tests::phase_gate_blocks_absolute_outside` | pending
- [ ] PC-013: WorkflowState version default | `cargo test -p ecc-domain workflow::state::tests::version_field_default` | pending
- [ ] PC-014: WorkflowState version serialized | `cargo test -p ecc-domain workflow::state::tests::version_field_serialized` | pending
- [ ] PC-015: WorkflowState ignores unknown | `cargo test -p ecc-domain workflow::state::tests::ignores_unknown_fields` | pending
- [ ] PC-007: is_stale true when exceeded | `cargo test -p ecc-domain workflow::staleness::tests` | pending
- [ ] PC-008: is_stale false within threshold | `cargo test -p ecc-domain workflow::staleness::tests` | pending
- [ ] PC-009: verify_phase rejects Idle/Solution | `cargo test -p ecc-domain workflow::phase_verify::tests` | pending
- [ ] PC-010: verify_phase rejects Plan/Implement | `cargo test -p ecc-domain workflow::phase_verify::tests` | pending
- [ ] PC-011: verify_phase accepts correct | `cargo test -p ecc-domain workflow::phase_verify::tests` | pending
- [ ] PC-012: verify_phase allows None for init | `cargo test -p ecc-domain workflow::phase_verify::tests` | pending
- [ ] PC-016: resolve_state_dir worktree path | `cargo test -p ecc-app workflow::state_resolver::tests::worktree_returns_git_dir` | pending
- [ ] PC-017: resolve_state_dir independent | `cargo test -p ecc-app workflow::state_resolver::tests::worktree_independent_from_main` | pending
- [ ] PC-018: resolve_state_dir CLAUDE_PROJECT_DIR | `cargo test -p ecc-app workflow::state_resolver::tests::uses_claude_project_dir` | pending
- [ ] PC-019: resolve_state_dir non-git fallback | `cargo test -p ecc-app workflow::state_resolver::tests::non_git_fallback` | pending
- [ ] PC-020: resolve_state_dir old location | `cargo test -p ecc-app workflow::state_resolver::tests::old_location_fallback` | pending
- [ ] PC-021: resolve_state_dir bare repo | `cargo test -p ecc-app workflow::state_resolver::tests::bare_repo_support` | pending
- [ ] PC-022: recover archives and resets | `cargo test -p ecc-app workflow::recover::tests::recover_archives_and_resets` | pending
- [ ] PC-023: recover fails if archive fails | `cargo test -p ecc-app workflow::recover::tests::recover_fails_if_archive_fails` | pending
- [ ] PC-046: detect_staleness mock clock | `cargo test -p ecc-app workflow::recover::tests::staleness_with_mock_clock` | pending
- [ ] PC-024: status shows STALE | `cargo test -p ecc-workflow status::tests::status_shows_stale` | pending
- [ ] PC-036: ecc workflow init | `cargo test -p ecc-integration-tests workflow_cli::init_succeeds` | pending
- [ ] PC-037: ecc workflow status parity | `cargo test -p ecc-integration-tests workflow_cli::status_parity` | pending
- [ ] PC-038: ecc workflow transition | `cargo test -p ecc-integration-tests workflow_cli::transition_parity` | pending
- [ ] PC-039: All 22 subcommands | `cargo test -p ecc-integration-tests workflow_cli::all_subcommands_exist` | pending
- [ ] PC-041: Port traits usage | `cargo test -p ecc-app workflow::state_resolver::tests` | pending
- [ ] PC-042: ecc workflow --verbose | `cargo test -p ecc-integration-tests workflow_cli::verbose_tracing` | pending
- [ ] PC-040: Thin wrapper delegation | `cargo test -p ecc-integration-tests workflow_cli::thin_wrapper_delegation` | pending
- [ ] PC-043: ecc hook parity | `cargo test -p ecc-integration-tests hook_parity::check_hook_enabled_parity` | pending
- [ ] PC-025: migrate_hooks_json replaces | `cargo test -p ecc-app install::hooks_migration::tests::replaces_ecc_hook` | pending
- [ ] PC-026: migrate_hooks_json idempotent | `cargo test -p ecc-app install::hooks_migration::tests::idempotent` | pending
- [ ] PC-027: migrate_hooks_json preserves | `cargo test -p ecc-app install::hooks_migration::tests::preserves_custom_hooks` | pending
- [ ] PC-045: Migration safety gate | `cargo test -p ecc-app install::hooks_migration::tests::safety_gate` | pending
- [ ] PC-044: ecc-hook thin wrapper | `cargo test -p ecc-integration-tests hook_parity::thin_wrapper_fallback` | pending
- [ ] PC-047: Clippy gate | `cargo clippy --workspace -- -D warnings` | pending
- [ ] PC-048: Build gate | `cargo build --workspace` | pending
- [ ] PC-049: Regression gate | `cargo test --workspace` | pending
- [ ] PC-050: Format gate | `cargo fmt --workspace -- --check` | pending

## Post-TDD

- [ ] E2E tests | pending
- [ ] Code review | pending
- [ ] Doc updates | pending
- [ ] Supplemental docs | pending
- [ ] Write implement-done.md | pending
