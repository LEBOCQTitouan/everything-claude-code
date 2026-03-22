# Tasks: Pipeline output summaries + DRY cleanup (BL-048)

## Pass Conditions

- [x] PC-001: Create test file | `test -f tests/test-pipeline-summaries.sh` | done@2026-03-22T13:30:00Z
- [x] PC-002: Skill frontmatter | `bash tests/test-pipeline-summaries.sh test_skill_frontmatter` | done@2026-03-22T13:31:00Z
- [x] PC-003: Skill Project Detection | `bash tests/test-pipeline-summaries.sh test_skill_project_detection` | done@2026-03-22T13:31:00Z
- [x] PC-004: Skill Grill-Me Rules | `bash tests/test-pipeline-summaries.sh test_skill_grillme_rules` | done@2026-03-22T13:31:00Z
- [x] PC-005: Skill Adversarial + Schema | `bash tests/test-pipeline-summaries.sh test_skill_adversarial_schema` | done@2026-03-22T13:31:00Z
- [x] PC-006: spec-dev DRY ref | `bash tests/test-pipeline-summaries.sh test_specdev_dry_ref` | done@2026-03-22T13:32:00Z
- [x] PC-007: spec-fix DRY ref | `bash tests/test-pipeline-summaries.sh test_specfix_dry_ref` | done@2026-03-22T13:32:00Z
- [x] PC-008: spec-refactor DRY ref | `bash tests/test-pipeline-summaries.sh test_specrefactor_dry_ref` | done@2026-03-22T13:32:00Z
- [x] PC-009: spec-dev grill-me accumulator | `bash tests/test-pipeline-summaries.sh test_specdev_grillme_accumulator` | done@2026-03-22T13:32:00Z
- [x] PC-010: spec-dev Grill-Me Decisions table | `bash tests/test-pipeline-summaries.sh test_specdev_grillme_table` | done@2026-03-22T13:32:00Z
- [x] PC-011: spec-dev US + AC tables | `bash tests/test-pipeline-summaries.sh test_specdev_us_ac_tables` | done@2026-03-22T13:32:00Z
- [x] PC-012: spec-dev Adversary table | `bash tests/test-pipeline-summaries.sh test_specdev_adversary_table` | done@2026-03-22T13:32:00Z
- [x] PC-013: spec-dev Artifacts table | `bash tests/test-pipeline-summaries.sh test_specdev_artifacts_table` | done@2026-03-22T13:32:00Z
- [x] PC-014: spec-dev Phase Summary persist | `bash tests/test-pipeline-summaries.sh test_specdev_phase_summary` | done@2026-03-22T13:32:00Z
- [x] PC-015: spec-fix core + root cause | `bash tests/test-pipeline-summaries.sh test_specfix_variant_tables` | done@2026-03-22T13:33:00Z
- [x] PC-016: spec-refactor core + smells | `bash tests/test-pipeline-summaries.sh test_specrefactor_variant_tables` | done@2026-03-22T13:33:00Z
- [x] PC-017: design Design Reviews table | `bash tests/test-pipeline-summaries.sh test_design_reviews_table` | done@2026-03-22T13:33:00Z
- [x] PC-018: design Adversary table | `bash tests/test-pipeline-summaries.sh test_design_adversary_table` | done@2026-03-22T13:33:00Z
- [x] PC-019: design File Changes table | `bash tests/test-pipeline-summaries.sh test_design_filechanges_table` | done@2026-03-22T13:33:00Z
- [x] PC-020: design Artifacts table | `bash tests/test-pipeline-summaries.sh test_design_artifacts_table` | done@2026-03-22T13:33:00Z
- [x] PC-021: design Phase Summary persist | `bash tests/test-pipeline-summaries.sh test_design_phase_summary` | done@2026-03-22T13:33:00Z
- [x] PC-022: implement Tasks Executed table | `bash tests/test-pipeline-summaries.sh test_implement_tasks_table` | done@2026-03-22T13:34:00Z
- [x] PC-023: implement Commits Made table | `bash tests/test-pipeline-summaries.sh test_implement_commits_table` | done@2026-03-22T13:34:00Z
- [x] PC-024: implement Docs Updated table | `bash tests/test-pipeline-summaries.sh test_implement_docs_table` | done@2026-03-22T13:34:00Z
- [x] PC-025: implement Artifacts table | `bash tests/test-pipeline-summaries.sh test_implement_artifacts_table` | done@2026-03-22T13:34:00Z
- [x] PC-026: implement commit accumulator | `bash tests/test-pipeline-summaries.sh test_implement_commit_accumulator` | done@2026-03-22T13:34:00Z
- [x] PC-027: implement Phase Summary persist | `bash tests/test-pipeline-summaries.sh test_implement_phase_summary` | done@2026-03-22T13:34:00Z
- [x] PC-028: ADR 0009 | `bash tests/test-pipeline-summaries.sh test_adr_0009` | done@2026-03-22T13:35:00Z
- [x] PC-029: CHANGELOG BL-048 | `bash tests/test-pipeline-summaries.sh test_changelog_bl048` | done@2026-03-22T13:35:00Z
- [x] PC-030: Deferred items | `bash tests/test-pipeline-summaries.sh test_deferred_items` | done@2026-03-22T13:35:00Z
- [x] PC-031: Line counts | `bash tests/test-pipeline-summaries.sh test_line_counts` | done@2026-03-22T13:36:00Z
- [x] PC-032: Idempotent overwrite | `bash tests/test-pipeline-summaries.sh test_idempotent_overwrite` | done@2026-03-22T13:36:00Z
- [x] PC-033: cargo clippy | `cargo clippy -- -D warnings` | done@2026-03-22T13:36:00Z
- [x] PC-034: cargo build | `cargo build` | done@2026-03-22T13:36:00Z
- [x] PC-035: cargo test | `cargo test` | done@2026-03-22T13:36:00Z

## Post-TDD

- [x] E2E tests | done@2026-03-22T13:36:00Z (none required)
- [x] Code review | done@2026-03-22T13:37:00Z (PASS — 0 CRITICAL, 0 HIGH)
- [x] Doc updates | done@2026-03-22T13:37:00Z (ADR 0009 + CHANGELOG + ADR index)
- [x] Write implement-done.md | done@2026-03-22T13:38:00Z
