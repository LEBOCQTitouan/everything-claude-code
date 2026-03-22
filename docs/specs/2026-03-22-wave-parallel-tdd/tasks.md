# Tasks: Wave-based parallel TDD execution (BL-032)

## Pass Conditions

- [x] PC-001: Wave Analysis subsection | `bash tests/test-wave-parallel.sh test_wave_analysis_section` | done@2026-03-22T21:40:00Z
- [x] PC-002: Wave grouping algorithm | `bash tests/test-wave-parallel.sh test_wave_grouping_algorithm` | done@2026-03-22T21:40:00Z
- [x] PC-003: Max 4 cap | `bash tests/test-wave-parallel.sh test_max_concurrency_cap` | done@2026-03-22T21:40:00Z
- [x] PC-004: Wave plan display | `bash tests/test-wave-parallel.sh test_wave_plan_display` | done@2026-03-22T21:40:00Z
- [x] PC-005: Degenerate cases | `bash tests/test-wave-parallel.sh test_degenerate_cases` | done@2026-03-22T21:40:00Z
- [x] PC-006: Worktree dispatch | `bash tests/test-wave-parallel.sh test_worktree_dispatch` | done@2026-03-22T21:40:00Z
- [x] PC-007: Sequential merge | `bash tests/test-wave-parallel.sh test_sequential_merge` | done@2026-03-22T21:40:00Z
- [x] PC-008: Single-PC backward compat | `bash tests/test-wave-parallel.sh test_single_pc_backward_compat` | done@2026-03-22T21:40:00Z
- [x] PC-009: Prior results scoping | `bash tests/test-wave-parallel.sh test_prior_results_scoping` | done@2026-03-22T21:40:00Z
- [x] PC-010: Merge error handling | `bash tests/test-wave-parallel.sh test_merge_error_handling` | done@2026-03-22T21:40:00Z
- [x] PC-011: Wave regression | `bash tests/test-wave-parallel.sh test_wave_regression` | done@2026-03-22T21:40:00Z
- [x] PC-012: Failure semantics | `bash tests/test-wave-parallel.sh test_failure_semantics` | done@2026-03-22T21:40:00Z
- [x] PC-013: Re-entry wave aware | `bash tests/test-wave-parallel.sh test_reentry_wave_aware` | done@2026-03-22T21:40:00Z
- [x] PC-014: Git tags | `bash tests/test-wave-parallel.sh test_git_tags` | done@2026-03-22T21:40:00Z
- [x] PC-015: Wave tasks tracking | `bash tests/test-wave-parallel.sh test_wave_tasks_tracking` | done@2026-03-22T21:40:00Z
- [x] PC-016: Implement-done wave column | `bash tests/test-wave-parallel.sh test_implement_done_wave_column` | done@2026-03-22T21:40:00Z
- [x] PC-017: Line count | `bash tests/test-wave-parallel.sh test_line_count` | done@2026-03-22T21:40:00Z
- [x] PC-018: ADR 0012 | `bash tests/test-wave-parallel.sh test_adr_0012` | done@2026-03-22T21:40:00Z
- [x] PC-019: Glossary terms | `bash tests/test-wave-parallel.sh test_glossary_terms` | done@2026-03-22T21:40:00Z
- [x] PC-020: CHANGELOG entry | `bash tests/test-wave-parallel.sh test_changelog_entry` | done@2026-03-22T21:40:00Z
- [x] PC-021: Pipeline tests regression | `bash tests/test-pipeline-summaries.sh` | done@2026-03-22T21:40:00Z

## Post-TDD

- [x] E2E tests | done@2026-03-22T21:40:00Z (none required)
- [x] Code review | done@2026-03-22T21:42:00Z (PASS — pure markdown, 215/215 assertions)
- [x] Doc updates | done@2026-03-22T21:41:00Z (ADR 0012, glossary, CHANGELOG)
- [x] Write implement-done.md | done@2026-03-22T21:43:00Z
