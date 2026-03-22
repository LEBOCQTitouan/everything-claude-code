# Tasks: Explanatory narrative audit (BL-051)

## Pass Conditions

- [x] PC-001: Skill frontmatter | `bash tests/test-narrative-audit.sh test_skill_frontmatter` | done@2026-03-22T21:05:00Z
- [x] PC-002: Skill content + word count | `bash tests/test-narrative-audit.sh test_skill_content` | done@2026-03-22T21:05:00Z
- [x] PC-003: spec-dev narrative | `bash tests/test-narrative-audit.sh test_specdev_narrative` | done@2026-03-22T21:10:00Z
- [x] PC-004: spec-fix narrative | `bash tests/test-narrative-audit.sh test_specfix_narrative` | done@2026-03-22T21:10:00Z
- [x] PC-005: spec-refactor narrative | `bash tests/test-narrative-audit.sh test_specrefactor_narrative` | done@2026-03-22T21:10:00Z
- [x] PC-006: design narrative | `bash tests/test-narrative-audit.sh test_design_narrative` | done@2026-03-22T21:12:00Z
- [x] PC-007: implement narrative | `bash tests/test-narrative-audit.sh test_implement_narrative` | done@2026-03-22T21:12:00Z
- [x] PC-008: audit-full narrative | `bash tests/test-narrative-audit.sh test_audit_full_narrative` | done@2026-03-22T21:15:00Z
- [x] PC-009: domain audit narrative | `bash tests/test-narrative-audit.sh test_audit_domain_narrative` | done@2026-03-22T21:15:00Z
- [x] PC-010: verify narrative | `bash tests/test-narrative-audit.sh test_verify_narrative` | done@2026-03-22T21:15:00Z
- [x] PC-011: build-fix narrative | `bash tests/test-narrative-audit.sh test_buildfix_narrative` | done@2026-03-22T21:15:00Z
- [x] PC-012: review narrative | `bash tests/test-narrative-audit.sh test_review_narrative` | done@2026-03-22T21:15:00Z
- [x] PC-013: catchup narrative | `bash tests/test-narrative-audit.sh test_catchup_narrative` | done@2026-03-22T21:15:00Z
- [x] PC-014: utility narrative | `bash tests/test-narrative-audit.sh test_utility_narrative` | done@2026-03-22T21:15:00Z
- [x] PC-015: ADR 0011 | `bash tests/test-narrative-audit.sh test_adr_0011` | done@2026-03-22T21:18:00Z
- [x] PC-016: CHANGELOG BL-051 | `bash tests/test-narrative-audit.sh test_changelog_bl051` | done@2026-03-22T21:18:00Z
- [x] PC-017: narrative-audit.md | `bash tests/test-narrative-audit.sh test_audit_doc` | done@2026-03-22T21:18:00Z
- [x] PC-018: Line counts | `bash tests/test-narrative-audit.sh test_line_counts` | done@2026-03-22T21:20:00Z
- [x] PC-019: Skill ref consistency | `bash tests/test-narrative-audit.sh test_skill_ref_consistency` | done@2026-03-22T21:20:00Z
- [x] PC-020: Full bash suite | `bash tests/test-narrative-audit.sh` | done@2026-03-22T21:20:00Z
- [x] PC-021: cargo clippy | `cargo clippy -- -D warnings` | done@2026-03-22T21:20:00Z
- [x] PC-022: cargo build | `cargo build` | done@2026-03-22T21:20:00Z
- [x] PC-023: cargo test | `cargo test` | done@2026-03-22T21:20:00Z
- [x] PC-024: Markdown lint | `npx markdownlint-cli2` | done@2026-03-22T21:20:00Z

## Post-TDD

- [x] E2E tests | done@2026-03-22T21:20:00Z (none required)
- [x] Code review | done@2026-03-22T21:20:00Z (PASS — pure markdown, 118/118 assertions)
- [x] Doc updates | done@2026-03-22T21:18:00Z (ADR 0011, CHANGELOG, narrative-audit.md)
- [x] Write implement-done.md | done@2026-03-22T21:22:00Z
