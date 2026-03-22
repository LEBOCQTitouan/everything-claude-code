# Tasks: Pipeline output summaries + DRY cleanup (BL-048)

## Pass Conditions

- [ ] PC-001: Create test file | `test -f tests/test-pipeline-summaries.sh` | pending@2026-03-22T13:30:00Z
- [ ] PC-002: Skill frontmatter | `bash tests/test-pipeline-summaries.sh test_skill_frontmatter` | pending@2026-03-22T13:30:00Z
- [ ] PC-003: Skill Project Detection | `bash tests/test-pipeline-summaries.sh test_skill_project_detection` | pending@2026-03-22T13:30:00Z
- [ ] PC-004: Skill Grill-Me Rules | `bash tests/test-pipeline-summaries.sh test_skill_grillme_rules` | pending@2026-03-22T13:30:00Z
- [ ] PC-005: Skill Adversarial + Schema | `bash tests/test-pipeline-summaries.sh test_skill_adversarial_schema` | pending@2026-03-22T13:30:00Z
- [ ] PC-006: spec-dev DRY ref | `bash tests/test-pipeline-summaries.sh test_specdev_dry_ref` | pending@2026-03-22T13:30:00Z
- [ ] PC-007: spec-fix DRY ref | `bash tests/test-pipeline-summaries.sh test_specfix_dry_ref` | pending@2026-03-22T13:30:00Z
- [ ] PC-008: spec-refactor DRY ref | `bash tests/test-pipeline-summaries.sh test_specrefactor_dry_ref` | pending@2026-03-22T13:30:00Z
- [ ] PC-009: spec-dev grill-me accumulator | `bash tests/test-pipeline-summaries.sh test_specdev_grillme_accumulator` | pending@2026-03-22T13:30:00Z
- [ ] PC-010: spec-dev Grill-Me Decisions table | `bash tests/test-pipeline-summaries.sh test_specdev_grillme_table` | pending@2026-03-22T13:30:00Z
- [ ] PC-011: spec-dev US + AC tables | `bash tests/test-pipeline-summaries.sh test_specdev_us_ac_tables` | pending@2026-03-22T13:30:00Z
- [ ] PC-012: spec-dev Adversary table | `bash tests/test-pipeline-summaries.sh test_specdev_adversary_table` | pending@2026-03-22T13:30:00Z
- [ ] PC-013: spec-dev Artifacts table | `bash tests/test-pipeline-summaries.sh test_specdev_artifacts_table` | pending@2026-03-22T13:30:00Z
- [ ] PC-014: spec-dev Phase Summary persist | `bash tests/test-pipeline-summaries.sh test_specdev_phase_summary` | pending@2026-03-22T13:30:00Z
- [ ] PC-015: spec-fix core + root cause | `bash tests/test-pipeline-summaries.sh test_specfix_variant_tables` | pending@2026-03-22T13:30:00Z
- [ ] PC-016: spec-refactor core + smells | `bash tests/test-pipeline-summaries.sh test_specrefactor_variant_tables` | pending@2026-03-22T13:30:00Z
- [ ] PC-017: design Design Reviews table | `bash tests/test-pipeline-summaries.sh test_design_reviews_table` | pending@2026-03-22T13:30:00Z
- [ ] PC-018: design Adversary table | `bash tests/test-pipeline-summaries.sh test_design_adversary_table` | pending@2026-03-22T13:30:00Z
- [ ] PC-019: design File Changes table | `bash tests/test-pipeline-summaries.sh test_design_filechanges_table` | pending@2026-03-22T13:30:00Z
- [ ] PC-020: design Artifacts table | `bash tests/test-pipeline-summaries.sh test_design_artifacts_table` | pending@2026-03-22T13:30:00Z
- [ ] PC-021: design Phase Summary persist | `bash tests/test-pipeline-summaries.sh test_design_phase_summary` | pending@2026-03-22T13:30:00Z
- [ ] PC-022: implement Tasks Executed table | `bash tests/test-pipeline-summaries.sh test_implement_tasks_table` | pending@2026-03-22T13:30:00Z
- [ ] PC-023: implement Commits Made table | `bash tests/test-pipeline-summaries.sh test_implement_commits_table` | pending@2026-03-22T13:30:00Z
- [ ] PC-024: implement Docs Updated table | `bash tests/test-pipeline-summaries.sh test_implement_docs_table` | pending@2026-03-22T13:30:00Z
- [ ] PC-025: implement Artifacts table | `bash tests/test-pipeline-summaries.sh test_implement_artifacts_table` | pending@2026-03-22T13:30:00Z
- [ ] PC-026: implement commit accumulator | `bash tests/test-pipeline-summaries.sh test_implement_commit_accumulator` | pending@2026-03-22T13:30:00Z
- [ ] PC-027: implement Phase Summary persist | `bash tests/test-pipeline-summaries.sh test_implement_phase_summary` | pending@2026-03-22T13:30:00Z
- [ ] PC-028: ADR 0009 | `bash tests/test-pipeline-summaries.sh test_adr_0009` | pending@2026-03-22T13:30:00Z
- [ ] PC-029: CHANGELOG BL-048 | `bash tests/test-pipeline-summaries.sh test_changelog_bl048` | pending@2026-03-22T13:30:00Z
- [ ] PC-030: Deferred items | `bash tests/test-pipeline-summaries.sh test_deferred_items` | pending@2026-03-22T13:30:00Z
- [ ] PC-031: Line counts | `bash tests/test-pipeline-summaries.sh test_line_counts` | pending@2026-03-22T13:30:00Z
- [ ] PC-032: Idempotent overwrite | `bash tests/test-pipeline-summaries.sh test_idempotent_overwrite` | pending@2026-03-22T13:30:00Z
- [ ] PC-033: npm run lint | `npm run lint` | pending@2026-03-22T13:30:00Z
- [ ] PC-034: cargo build | `cargo build` | pending@2026-03-22T13:30:00Z
- [ ] PC-035: cargo test | `cargo test` | pending@2026-03-22T13:30:00Z

## Post-TDD

- [ ] E2E tests | pending@2026-03-22T13:30:00Z
- [ ] Code review | pending@2026-03-22T13:30:00Z
- [ ] Doc updates | pending@2026-03-22T13:30:00Z
- [ ] Write implement-done.md | pending@2026-03-22T13:30:00Z
