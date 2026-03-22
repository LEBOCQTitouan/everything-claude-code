# Tasks: Explanatory narrative audit (BL-051)

## Pass Conditions

- [ ] PC-001: Skill frontmatter | `bash tests/test-narrative-audit.sh test_skill_frontmatter` | pending@2026-03-22T21:00:00Z
- [ ] PC-002: Skill content + word count | `bash tests/test-narrative-audit.sh test_skill_content` | pending@2026-03-22T21:00:00Z
- [ ] PC-003: spec-dev narrative | `bash tests/test-narrative-audit.sh test_specdev_narrative` | pending@2026-03-22T21:00:00Z
- [ ] PC-004: spec-fix narrative | `bash tests/test-narrative-audit.sh test_specfix_narrative` | pending@2026-03-22T21:00:00Z
- [ ] PC-005: spec-refactor narrative | `bash tests/test-narrative-audit.sh test_specrefactor_narrative` | pending@2026-03-22T21:00:00Z
- [ ] PC-006: design narrative | `bash tests/test-narrative-audit.sh test_design_narrative` | pending@2026-03-22T21:00:00Z
- [ ] PC-007: implement narrative | `bash tests/test-narrative-audit.sh test_implement_narrative` | pending@2026-03-22T21:00:00Z
- [ ] PC-008: audit-full narrative | `bash tests/test-narrative-audit.sh test_audit_full_narrative` | pending@2026-03-22T21:00:00Z
- [ ] PC-009: domain audit narrative | `bash tests/test-narrative-audit.sh test_audit_domain_narrative` | pending@2026-03-22T21:00:00Z
- [ ] PC-010: verify narrative | `bash tests/test-narrative-audit.sh test_verify_narrative` | pending@2026-03-22T21:00:00Z
- [ ] PC-011: build-fix narrative | `bash tests/test-narrative-audit.sh test_buildfix_narrative` | pending@2026-03-22T21:00:00Z
- [ ] PC-012: review narrative | `bash tests/test-narrative-audit.sh test_review_narrative` | pending@2026-03-22T21:00:00Z
- [ ] PC-013: catchup narrative | `bash tests/test-narrative-audit.sh test_catchup_narrative` | pending@2026-03-22T21:00:00Z
- [ ] PC-014: utility narrative | `bash tests/test-narrative-audit.sh test_utility_narrative` | pending@2026-03-22T21:00:00Z
- [ ] PC-015: ADR 0011 | `bash tests/test-narrative-audit.sh test_adr_0011` | pending@2026-03-22T21:00:00Z
- [ ] PC-016: CHANGELOG BL-051 | `bash tests/test-narrative-audit.sh test_changelog_bl051` | pending@2026-03-22T21:00:00Z
- [ ] PC-017: narrative-audit.md | `bash tests/test-narrative-audit.sh test_audit_doc` | pending@2026-03-22T21:00:00Z
- [ ] PC-018: Line counts | `bash tests/test-narrative-audit.sh test_line_counts` | pending@2026-03-22T21:00:00Z
- [ ] PC-019: Skill ref consistency | `bash tests/test-narrative-audit.sh test_skill_ref_consistency` | pending@2026-03-22T21:00:00Z
- [ ] PC-020: Full bash suite | `bash tests/test-narrative-audit.sh` | pending@2026-03-22T21:00:00Z
- [ ] PC-021: cargo clippy | `cargo clippy -- -D warnings` | pending@2026-03-22T21:00:00Z
- [ ] PC-022: cargo build | `cargo build` | pending@2026-03-22T21:00:00Z
- [ ] PC-023: cargo test | `cargo test` | pending@2026-03-22T21:00:00Z
- [ ] PC-024: Markdown lint | `npx markdownlint-cli2 "skills/narrative-conventions/**" "docs/adr/0011*" "docs/narrative-audit.md"` | pending@2026-03-22T21:00:00Z

## Post-TDD

- [ ] E2E tests | pending@2026-03-22T21:00:00Z
- [ ] Code review | pending@2026-03-22T21:00:00Z
- [ ] Doc updates | pending@2026-03-22T21:00:00Z
- [ ] Write implement-done.md | pending@2026-03-22T21:00:00Z
