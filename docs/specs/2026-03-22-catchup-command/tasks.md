# Tasks: Create /catchup session resumption command (BL-017)

## Pass Conditions

- [x] PC-001: Test file exists | `test -f tests/hooks/test-catchup.sh` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:20:00Z
- [x] PC-002: Test script runs successfully | `bash tests/hooks/test-catchup.sh` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:20:00Z
- [x] PC-003: Active workflow shows phase/feature/concern/started_at/artifacts | `bash tests/hooks/test-catchup.sh test_workflow_active_state` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:25:00Z
- [x] PC-004: Tasks progress displayed | `bash tests/hooks/test-catchup.sh test_tasks_progress` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:30:00Z
- [x] PC-005: Missing tasks.md reports path | `bash tests/hooks/test-catchup.sh test_tasks_missing` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:35:00Z
- [x] PC-006: No active workflow message | `bash tests/hooks/test-catchup.sh test_no_workflow` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:35:00Z
- [x] PC-007: Done phase shows completion | `bash tests/hooks/test-catchup.sh test_workflow_done` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:35:00Z
- [x] PC-008: Spec/design paths shown | `bash tests/hooks/test-catchup.sh test_spec_design_paths` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:35:00Z
- [x] PC-009: Malformed JSON warns and continues | `bash tests/hooks/test-catchup.sh test_malformed_json` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:35:00Z
- [x] PC-010: Uncommitted changes counted | `bash tests/hooks/test-catchup.sh test_git_uncommitted` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:40:00Z
- [x] PC-011: Stashes listed | `bash tests/hooks/test-catchup.sh test_git_stashes` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:40:00Z
- [x] PC-012: Multiple worktrees listed | `bash tests/hooks/test-catchup.sh test_git_worktrees` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:40:00Z
- [x] PC-013: Clean git state reported | `bash tests/hooks/test-catchup.sh test_git_clean` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:40:00Z
- [x] PC-014: Zero commits handled | `bash tests/hooks/test-catchup.sh test_git_zero_commits` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:40:00Z
- [x] PC-015: Stale workflow flagged STALE | `bash tests/hooks/test-catchup.sh test_stale_detection` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:45:00Z
- [x] PC-016: Reset archives then deletes state.json | `bash tests/hooks/test-catchup.sh test_stale_reset` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:45:00Z
- [x] PC-017: Resume continues normally | `bash tests/hooks/test-catchup.sh test_stale_resume` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:45:00Z
- [x] PC-018: Recent commit no staleness | `bash tests/hooks/test-catchup.sh test_not_stale` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:45:00Z
- [x] PC-019: Today's daily memory shown | `bash tests/hooks/test-catchup.sh test_memory_today` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:50:00Z
- [x] PC-020: Previous day's memory with label | `bash tests/hooks/test-catchup.sh test_memory_previous` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:50:00Z
- [x] PC-021: No daily files message | `bash tests/hooks/test-catchup.sh test_memory_none` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:50:00Z
- [x] PC-022: Valid YAML frontmatter | `bash tests/hooks/test-catchup.sh test_frontmatter_valid` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:55:00Z
- [x] PC-023: Allowed-tools read-only only | `bash tests/hooks/test-catchup.sh test_allowed_tools` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:55:00Z
- [x] PC-024: Section headers present | `bash tests/hooks/test-catchup.sh test_section_headers` | pending@2026-03-22T11:15:00Z | done@2026-03-22T11:55:00Z
- [x] PC-025: Glossary has Catchup entry | `grep -q "Catchup" docs/domain/glossary.md` | pending@2026-03-22T11:15:00Z | done@2026-03-22T12:00:00Z
- [x] PC-026: CLAUDE.md has /catchup | `grep -q '/catchup' CLAUDE.md` | pending@2026-03-22T11:15:00Z | done@2026-03-22T12:00:00Z
- [x] PC-027: commands-reference has /catchup | `grep -q '/catchup' docs/commands-reference.md` | pending@2026-03-22T11:15:00Z | done@2026-03-22T12:00:00Z
- [x] PC-028: CHANGELOG has BL-017 | `grep -q 'BL-017' CHANGELOG.md` | pending@2026-03-22T11:15:00Z | done@2026-03-22T12:00:00Z
- [x] PC-029: Lint passes | `cargo clippy -- -D warnings` | pending@2026-03-22T11:15:00Z | done@2026-03-22T12:05:00Z
- [x] PC-030: Build passes | `cargo build` | pending@2026-03-22T11:15:00Z | done@2026-03-22T12:05:00Z
- [x] PC-031: Full test suite passes | `bash tests/hooks/test-catchup.sh` | pending@2026-03-22T11:15:00Z | done@2026-03-22T12:05:00Z

## Post-TDD

- [x] E2E tests | pending@2026-03-22T11:15:00Z | done@2026-03-22T12:05:00Z
- [x] Code review | pending@2026-03-22T11:15:00Z | done@2026-03-22T12:10:00Z
- [x] Doc updates | pending@2026-03-22T11:15:00Z | done@2026-03-22T12:10:00Z
- [x] Write implement-done.md | pending@2026-03-22T11:15:00Z | done@2026-03-22T12:15:00Z
