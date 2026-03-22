# Tasks: Interview-me skill + interviewer agent (BL-013)

## Pass Conditions

- [ ] PC-001: foundation-models-on-device origin | `grep -q '^origin: ECC' skills/foundation-models-on-device/SKILL.md` | pending@2026-03-22T14:15:00Z
- [ ] PC-002: skill-stocktake name | `grep -q '^name: skill-stocktake' skills/skill-stocktake/SKILL.md` | pending@2026-03-22T14:15:00Z
- [ ] PC-003: swift-concurrency-6-2 origin | `grep -q '^origin: ECC' skills/swift-concurrency-6-2/SKILL.md` | pending@2026-03-22T14:15:00Z
- [ ] PC-004: swiftui-patterns origin | `grep -q '^origin: ECC' skills/swiftui-patterns/SKILL.md` | pending@2026-03-22T14:15:00Z
- [ ] PC-005: Skill frontmatter | `bash tests/test-interview-me.sh test_skill_frontmatter` | pending@2026-03-22T14:15:00Z
- [ ] PC-006: Skill word count | `bash tests/test-interview-me.sh test_skill_word_count` | pending@2026-03-22T14:15:00Z
- [ ] PC-007: Skill triggers | `bash tests/test-interview-me.sh test_skill_triggers` | pending@2026-03-22T14:15:00Z
- [ ] PC-008: Skill stages | `bash tests/test-interview-me.sh test_skill_stages` | pending@2026-03-22T14:15:00Z
- [ ] PC-009: Skill output format | `bash tests/test-interview-me.sh test_skill_output_format` | pending@2026-03-22T14:15:00Z
- [ ] PC-010: Skill negative examples | `bash tests/test-interview-me.sh test_skill_negative_examples` | pending@2026-03-22T14:15:00Z
- [ ] PC-011: Skill distinct from grill-me | `bash tests/test-interview-me.sh test_skill_distinct_from_grill_me` | pending@2026-03-22T14:15:00Z
- [ ] PC-012: Agent frontmatter | `bash tests/test-interview-me.sh test_agent_frontmatter` | pending@2026-03-22T14:15:00Z
- [ ] PC-013: Agent codebase exploration | `bash tests/test-interview-me.sh test_agent_codebase_exploration` | pending@2026-03-22T14:15:00Z
- [ ] PC-014: Agent skip known | `bash tests/test-interview-me.sh test_agent_skip_known` | pending@2026-03-22T14:15:00Z
- [ ] PC-015: Agent security gate | `bash tests/test-interview-me.sh test_agent_security_gate` | pending@2026-03-22T14:15:00Z
- [ ] PC-016: Agent output path | `bash tests/test-interview-me.sh test_agent_output_path` | pending@2026-03-22T14:15:00Z
- [ ] PC-017: Agent one question per turn | `bash tests/test-interview-me.sh test_agent_one_question_per_turn` | pending@2026-03-22T14:15:00Z
- [ ] PC-018: Agent TodoWrite | `bash tests/test-interview-me.sh test_agent_todowrite` | pending@2026-03-22T14:15:00Z
- [ ] PC-019: Agent early exit | `bash tests/test-interview-me.sh test_agent_early_exit` | pending@2026-03-22T14:15:00Z
- [ ] PC-020: Agent numeric suffix | `bash tests/test-interview-me.sh test_agent_numeric_suffix` | pending@2026-03-22T14:15:00Z
- [ ] PC-021: Validation missing name | `cargo test --package ecc-app skills_missing_name_field` | pending@2026-03-22T14:15:00Z
- [ ] PC-022: Validation missing description | `cargo test --package ecc-app skills_missing_description_field` | pending@2026-03-22T14:15:00Z
- [ ] PC-023: Validation missing origin | `cargo test --package ecc-app skills_missing_origin_field` | pending@2026-03-22T14:15:00Z
- [ ] PC-024: Validation all fields pass | `cargo test --package ecc-app skills_valid_frontmatter` | pending@2026-03-22T14:15:00Z
- [ ] PC-025: Validation warns model/tools | `cargo test --package ecc-app skills_warns_on_model_or_tools` | pending@2026-03-22T14:15:00Z
- [ ] PC-026: Existing test updated | `cargo test --package ecc-app skills_valid_dir` | pending@2026-03-22T14:15:00Z
- [ ] PC-027: Glossary entries | `bash tests/test-interview-me.sh test_glossary_entries` | pending@2026-03-22T14:15:00Z
- [ ] PC-028: CHANGELOG BL-013 | `bash tests/test-interview-me.sh test_changelog_entry` | pending@2026-03-22T14:15:00Z
- [ ] PC-029: ADR 0010 | `bash tests/test-interview-me.sh test_adr_exists` | pending@2026-03-22T14:15:00Z
- [ ] PC-030: cargo clippy | `cargo clippy -- -D warnings` | pending@2026-03-22T14:15:00Z
- [ ] PC-031: cargo build | `cargo build` | pending@2026-03-22T14:15:00Z
- [ ] PC-032: cargo test | `cargo test` | pending@2026-03-22T14:15:00Z
- [ ] PC-033: Full bash suite | `bash tests/test-interview-me.sh` | pending@2026-03-22T14:15:00Z
- [ ] PC-034: Markdown lint | `npx markdownlint-cli2 "skills/interview-me/**" "agents/interviewer.md" "docs/adr/0010*"` | pending@2026-03-22T14:15:00Z
- [ ] PC-035: No-frontmatter errors | `cargo test --package ecc-app skills_no_frontmatter` | pending@2026-03-22T14:15:00Z
- [ ] PC-036: Valid count accuracy | `cargo test --package ecc-app skills_valid_count_accuracy` | pending@2026-03-22T14:15:00Z

## Post-TDD

- [ ] E2E tests | pending@2026-03-22T14:15:00Z
- [ ] Code review | pending@2026-03-22T14:15:00Z
- [ ] Doc updates | pending@2026-03-22T14:15:00Z
- [ ] Write implement-done.md | pending@2026-03-22T14:15:00Z
