# Tasks: Interview-me skill + interviewer agent (BL-013)

## Pass Conditions

- [x] PC-001: foundation-models-on-device origin | `grep -q '^origin: ECC' skills/foundation-models-on-device/SKILL.md` | done@2026-03-22T14:20:00Z
- [x] PC-002: skill-stocktake name | `grep -q '^name: skill-stocktake' skills/skill-stocktake/SKILL.md` | done@2026-03-22T14:20:00Z
- [x] PC-003: swift-concurrency-6-2 origin | `grep -q '^origin: ECC' skills/swift-concurrency-6-2/SKILL.md` | done@2026-03-22T14:20:00Z
- [x] PC-004: swiftui-patterns origin | `grep -q '^origin: ECC' skills/swiftui-patterns/SKILL.md` | done@2026-03-22T14:20:00Z
- [x] PC-005: Skill frontmatter | `bash tests/test-interview-me.sh test_skill_frontmatter` | done@2026-03-22T14:25:00Z
- [x] PC-006: Skill word count | `bash tests/test-interview-me.sh test_skill_word_count` | done@2026-03-22T14:25:00Z
- [x] PC-007: Skill triggers | `bash tests/test-interview-me.sh test_skill_triggers` | done@2026-03-22T14:25:00Z
- [x] PC-008: Skill stages | `bash tests/test-interview-me.sh test_skill_stages` | done@2026-03-22T14:25:00Z
- [x] PC-009: Skill output format | `bash tests/test-interview-me.sh test_skill_output_format` | done@2026-03-22T14:25:00Z
- [x] PC-010: Skill negative examples | `bash tests/test-interview-me.sh test_skill_negative_examples` | done@2026-03-22T14:25:00Z
- [x] PC-011: Skill distinct from grill-me | `bash tests/test-interview-me.sh test_skill_distinct_from_grill_me` | done@2026-03-22T14:25:00Z
- [x] PC-012: Agent frontmatter | `bash tests/test-interview-me.sh test_agent_frontmatter` | done@2026-03-22T14:30:00Z
- [x] PC-013: Agent codebase exploration | `bash tests/test-interview-me.sh test_agent_codebase_exploration` | done@2026-03-22T14:30:00Z
- [x] PC-014: Agent skip known | `bash tests/test-interview-me.sh test_agent_skip_known` | done@2026-03-22T14:30:00Z
- [x] PC-015: Agent security gate | `bash tests/test-interview-me.sh test_agent_security_gate` | done@2026-03-22T14:30:00Z
- [x] PC-016: Agent output path | `bash tests/test-interview-me.sh test_agent_output_path` | done@2026-03-22T14:30:00Z
- [x] PC-017: Agent one question per turn | `bash tests/test-interview-me.sh test_agent_one_question_per_turn` | done@2026-03-22T14:30:00Z
- [x] PC-018: Agent TodoWrite | `bash tests/test-interview-me.sh test_agent_todowrite` | done@2026-03-22T14:30:00Z
- [x] PC-019: Agent early exit | `bash tests/test-interview-me.sh test_agent_early_exit` | done@2026-03-22T14:30:00Z
- [x] PC-020: Agent numeric suffix | `bash tests/test-interview-me.sh test_agent_numeric_suffix` | done@2026-03-22T14:30:00Z
- [x] PC-021: Validation missing name | `cargo test --package ecc-app skills_missing_name_field` | done@2026-03-22T14:35:00Z
- [x] PC-022: Validation missing description | `cargo test --package ecc-app skills_missing_description_field` | done@2026-03-22T14:35:00Z
- [x] PC-023: Validation missing origin | `cargo test --package ecc-app skills_missing_origin_field` | done@2026-03-22T14:35:00Z
- [x] PC-024: Validation all fields pass | `cargo test --package ecc-app skills_valid_frontmatter` | done@2026-03-22T14:35:00Z
- [x] PC-025: Validation warns model/tools | `cargo test --package ecc-app skills_warns_on_model_or_tools` | done@2026-03-22T14:35:00Z
- [x] PC-026: Existing test updated | `cargo test --package ecc-app skills_valid_dir` | done@2026-03-22T14:35:00Z
- [x] PC-027: Glossary entries | `bash tests/test-interview-me.sh test_glossary_entries` | done@2026-03-22T14:40:00Z
- [x] PC-028: CHANGELOG BL-013 | `bash tests/test-interview-me.sh test_changelog_entry` | done@2026-03-22T14:40:00Z
- [x] PC-029: ADR 0010 | `bash tests/test-interview-me.sh test_adr_exists` | done@2026-03-22T14:40:00Z
- [x] PC-030: cargo clippy | `cargo clippy -- -D warnings` | done@2026-03-22T14:42:00Z
- [x] PC-031: cargo build | `cargo build` | done@2026-03-22T14:42:00Z
- [x] PC-032: cargo test | `cargo test` | done@2026-03-22T14:42:00Z
- [x] PC-033: Full bash suite | `bash tests/test-interview-me.sh` | done@2026-03-22T14:42:00Z
- [x] PC-034: Markdown lint | `npx markdownlint-cli2 "skills/interview-me/**" "agents/interviewer.md" "docs/adr/0010*"` | done@2026-03-22T14:42:00Z
- [x] PC-035: No-frontmatter errors | `cargo test --package ecc-app skills_no_frontmatter` | done@2026-03-22T14:35:00Z
- [x] PC-036: Valid count accuracy | `cargo test --package ecc-app skills_valid_count_accuracy` | done@2026-03-22T14:35:00Z

## Post-TDD

- [x] E2E tests | done@2026-03-22T14:42:00Z (none required)
- [x] Code review | done@2026-03-22T14:45:00Z (2 HIGH fixed, PASS)
- [x] Doc updates | done@2026-03-22T14:40:00Z (glossary, CHANGELOG, ADR 0010)
- [x] Write implement-done.md | done@2026-03-22T14:47:00Z
