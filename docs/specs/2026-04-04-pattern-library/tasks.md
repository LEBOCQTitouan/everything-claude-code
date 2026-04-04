# Tasks: Pattern Library — Phase 1 Foundation

## Pass Conditions

- [ ] PC-001: VALID_PATTERN_LANGUAGES contains 10 languages + "all" | `cargo test -p ecc-domain --lib -- config::validate::tests::valid_pattern_languages`
- [ ] PC-002: VALID_PATTERN_DIFFICULTIES contains 3 values | `cargo test -p ecc-domain --lib -- config::validate::tests::valid_pattern_difficulties`
- [ ] PC-003: UNSAFE_CODE_PATTERNS is non-empty | `cargo test -p ecc-domain --lib -- config::validate::tests::unsafe_code_patterns_non_empty`
- [ ] PC-004: REQUIRED_PATTERN_SECTIONS contains 9 sections | `cargo test -p ecc-domain --lib -- config::validate::tests::required_pattern_sections`
- [ ] PC-005: patterns field defaults to empty vec | `cargo test -p ecc-domain --lib -- config::manifest::tests::patterns_field_defaults_empty`
- [ ] PC-006: is_ecc_managed recognizes "patterns" | `cargo test -p ecc-domain --lib -- config::manifest::tests::is_ecc_managed_patterns`
- [ ] PC-007: no patterns dir returns true | `cargo test -p ecc-app --lib -- validate::patterns::tests::no_patterns_dir_succeeds`
- [ ] PC-008: empty dir succeeds with 0 count | `cargo test -p ecc-app --lib -- validate::patterns::tests::empty_dir_succeeds`
- [ ] PC-009: valid pattern passes full quality gate | `cargo test -p ecc-app --lib -- validate::patterns::tests::valid_pattern_passes`
- [ ] PC-010: missing category field errors | `cargo test -p ecc-app --lib -- validate::patterns::tests::missing_category_field_errors`
- [ ] PC-011: category-dir mismatch errors | `cargo test -p ecc-app --lib -- validate::patterns::tests::category_dir_mismatch_errors`
- [ ] PC-012: missing section errors | `cargo test -p ecc-app --lib -- validate::patterns::tests::missing_section_errors`
- [ ] PC-013: empty section body errors | `cargo test -p ecc-app --lib -- validate::patterns::tests::empty_section_body_errors`
- [ ] PC-014: invalid cross-ref errors | `cargo test -p ecc-app --lib -- validate::patterns::tests::invalid_cross_ref_errors`
- [ ] PC-015: lang impl mismatch errors | `cargo test -p ecc-app --lib -- validate::patterns::tests::lang_impl_mismatch_errors`
- [ ] PC-016: languages all skips impl check | `cargo test -p ecc-app --lib -- validate::patterns::tests::languages_all_skips_impl_check`
- [ ] PC-017: empty languages errors | `cargo test -p ecc-app --lib -- validate::patterns::tests::empty_languages_errors`
- [ ] PC-018: invalid language errors | `cargo test -p ecc-app --lib -- validate::patterns::tests::invalid_language_errors`
- [ ] PC-019: invalid difficulty errors | `cargo test -p ecc-app --lib -- validate::patterns::tests::invalid_difficulty_errors`
- [ ] PC-020: unsafe code warns | `cargo test -p ecc-app --lib -- validate::patterns::tests::unsafe_code_warns`
- [ ] PC-021: unsafe-examples suppresses | `cargo test -p ecc-app --lib -- validate::patterns::tests::unsafe_examples_suppresses`
- [ ] PC-022: self-reference warns | `cargo test -p ecc-app --lib -- validate::patterns::tests::self_reference_warns`
- [ ] PC-023: root-level file warns | `cargo test -p ecc-app --lib -- validate::patterns::tests::root_level_file_warns`
- [ ] PC-024: YAML flow list syntax | `cargo test -p ecc-app --lib -- validate::patterns::tests::yaml_flow_list_syntax`
- [ ] PC-025: missing from index errors | `cargo test -p ecc-app --lib -- validate::patterns::tests::missing_from_index_errors`
- [ ] PC-026: success message counts | `cargo test -p ecc-app --lib -- validate::patterns::tests::success_message_counts`
- [ ] PC-027: agent patterns invalid category warns | `cargo test -p ecc-app --lib -- validate::agents::tests::agent_patterns_invalid_category_warns`
- [ ] PC-028: agent no patterns field ok | `cargo test -p ecc-app --lib -- validate::agents::tests::agent_no_patterns_field_ok`
- [ ] PC-029: collect_artifacts includes patterns | `cargo test -p ecc-app --lib -- install::helpers::artifacts::tests::collect_artifacts_includes_patterns`
- [ ] PC-030: check_pattern_count reports | `cargo test -p ecc-app --lib -- config::audit::checks::content::tests::check_pattern_count_reports`
- [ ] PC-031: ecc validate patterns passes | `cargo test -p ecc-integration-tests -- validate_flow::validate_patterns_passes`
- [ ] PC-032: ecc validate agents passes | `cargo test -p ecc-integration-tests -- validate_flow::validate_agents_passes`
- [ ] PC-033: merge_patterns copies correctly | `cargo test -p ecc-app --lib -- install::merge::tests::merge_patterns_copies`
- [ ] PC-034: Language Implementations section required | `cargo test -p ecc-app --lib -- validate::patterns::tests::language_implementations_section_required`
- [ ] PC-035: clippy clean | `cargo clippy -- -D warnings`
- [ ] PC-036: build succeeds | `cargo build`

## Post-TDD

- [ ] E2E tests
- [ ] Code review
- [ ] Doc updates
- [ ] Supplemental docs
- [ ] Write implement-done.md

## Status Trail
