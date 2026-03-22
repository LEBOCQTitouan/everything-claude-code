# Solution: Interview-me skill + interviewer agent (BL-013)

## Spec Reference
Concern: dev, Feature: Create interview-me skill — collaborative requirements interview (BL-013)

## File Changes (dependency order)
| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | skills/foundation-models-on-device/SKILL.md | modify | Add missing origin: ECC | AC-004.1 |
| 2 | skills/skill-stocktake/SKILL.md | modify | Add missing name: skill-stocktake | AC-004.2 |
| 3 | skills/swift-concurrency-6-2/SKILL.md | modify | Add missing origin: ECC | AC-004.3 |
| 4 | skills/swiftui-patterns/SKILL.md | modify | Add missing origin: ECC | AC-004.4 |
| 5 | tests/test-interview-me.sh | create | Bash test suite for content validation | US-001, US-002, US-005 |
| 6 | skills/interview-me/SKILL.md | create | Collaborative interview methodology | US-001 |
| 7 | agents/interviewer.md | create | Orchestration agent with codebase exploration | US-002 |
| 8 | crates/ecc-app/src/validate.rs | modify | Add frontmatter field validation to validate_skills | US-003 |
| 9 | docs/domain/glossary.md | modify | Add Interview Me + Interviewer definitions | AC-005.1 |
| 10 | CHANGELOG.md | modify | Add BL-013 entry | AC-005.2 |
| 11 | docs/adr/0010-skill-frontmatter-validation.md | create | ADR for validation enhancement | AC-005.3 |

## Pass Conditions
| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | lint | foundation-models-on-device has origin | AC-004.1 | `grep -q '^origin: ECC' skills/foundation-models-on-device/SKILL.md` | exit 0 |
| PC-002 | lint | skill-stocktake has name | AC-004.2 | `grep -q '^name: skill-stocktake' skills/skill-stocktake/SKILL.md` | exit 0 |
| PC-003 | lint | swift-concurrency-6-2 has origin | AC-004.3 | `grep -q '^origin: ECC' skills/swift-concurrency-6-2/SKILL.md` | exit 0 |
| PC-004 | lint | swiftui-patterns has origin | AC-004.4 | `grep -q '^origin: ECC' skills/swiftui-patterns/SKILL.md` | exit 0 |
| PC-005 | integration | Skill frontmatter correct | AC-001.1 | `bash tests/test-interview-me.sh test_skill_frontmatter` | exit 0 |
| PC-006 | integration | Skill under 500 words | AC-001.2 | `bash tests/test-interview-me.sh test_skill_word_count` | exit 0 |
| PC-007 | integration | Skill has trigger phrases | AC-001.3 | `bash tests/test-interview-me.sh test_skill_triggers` | exit 0 |
| PC-008 | integration | Skill has 8 interview stages | AC-001.3 | `bash tests/test-interview-me.sh test_skill_stages` | exit 0 |
| PC-009 | integration | Skill has output format | AC-001.3 | `bash tests/test-interview-me.sh test_skill_output_format` | exit 0 |
| PC-010 | integration | Skill has negative examples | AC-001.3 | `bash tests/test-interview-me.sh test_skill_negative_examples` | exit 0 |
| PC-011 | integration | Skill distinct from grill-me | AC-001.4 | `bash tests/test-interview-me.sh test_skill_distinct_from_grill_me` | exit 0 |
| PC-012 | integration | Agent frontmatter correct | AC-002.1 | `bash tests/test-interview-me.sh test_agent_frontmatter` | exit 0 |
| PC-013 | integration | Agent has codebase exploration | AC-002.2 | `bash tests/test-interview-me.sh test_agent_codebase_exploration` | exit 0 |
| PC-014 | integration | Agent skips known answers | AC-002.3 | `bash tests/test-interview-me.sh test_agent_skip_known` | exit 0 |
| PC-015 | integration | Agent has security gate | AC-002.4 | `bash tests/test-interview-me.sh test_agent_security_gate` | exit 0 |
| PC-016 | integration | Agent outputs to docs/interviews/ | AC-002.5 | `bash tests/test-interview-me.sh test_agent_output_path` | exit 0 |
| PC-017 | integration | Agent one question per turn | AC-002.6 | `bash tests/test-interview-me.sh test_agent_one_question_per_turn` | exit 0 |
| PC-018 | integration | Agent has TodoWrite | AC-002.7 | `bash tests/test-interview-me.sh test_agent_todowrite` | exit 0 |
| PC-019 | integration | Agent handles early exit | AC-002.8 | `bash tests/test-interview-me.sh test_agent_early_exit` | exit 0 |
| PC-020 | integration | Agent numeric suffix on collision | AC-002.5 | `bash tests/test-interview-me.sh test_agent_numeric_suffix` | exit 0 |
| PC-021 | unit | Validation errors on missing name | AC-003.1 | `cargo test --package ecc-app skills_missing_name_field` | PASS |
| PC-022 | unit | Validation errors on missing description | AC-003.2 | `cargo test --package ecc-app skills_missing_description_field` | PASS |
| PC-023 | unit | Validation errors on missing origin | AC-003.3 | `cargo test --package ecc-app skills_missing_origin_field` | PASS |
| PC-024 | unit | Validation passes with all fields | AC-003.4 | `cargo test --package ecc-app skills_valid_frontmatter` | PASS |
| PC-025 | unit | Validation warns on model/tools | AC-003.5 | `cargo test --package ecc-app skills_warns_on_model_or_tools` | PASS |
| PC-026 | unit | Existing test updated + passes | AC-003.6 | `cargo test --package ecc-app skills_valid_dir` | PASS |
| PC-027 | integration | Glossary has entries | AC-005.1 | `bash tests/test-interview-me.sh test_glossary_entries` | exit 0 |
| PC-028 | integration | CHANGELOG has BL-013 | AC-005.2 | `bash tests/test-interview-me.sh test_changelog_entry` | exit 0 |
| PC-029 | integration | ADR 0010 exists | AC-005.3 | `bash tests/test-interview-me.sh test_adr_exists` | exit 0 |
| PC-030 | lint | Clippy passes | — | `cargo clippy -- -D warnings` | exit 0 |
| PC-031 | build | Cargo builds | — | `cargo build` | exit 0 |
| PC-032 | build | All Rust tests pass | AC-004.5 | `cargo test` | PASS |
| PC-033 | integration | Full bash suite passes | — | `bash tests/test-interview-me.sh` | exit 0 |
| PC-034 | lint | Markdown lint | — | `npx markdownlint-cli2 "skills/interview-me/**" "agents/interviewer.md" "docs/adr/0010*"` | exit 0 |
| PC-035 | unit | No-frontmatter skill errors | AC-003.1 | `cargo test --package ecc-app skills_no_frontmatter` | PASS |
| PC-036 | unit | Valid count accuracy | AC-003.6 | `cargo test --package ecc-app skills_valid_count_accuracy` | PASS |

### Coverage Check
All 27 ACs covered. Zero uncovered ACs.

### E2E Test Plan
| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | ecc validate skills | FileSystem | FileSystem trait | Frontmatter validation | ignored | validate.rs modified |

### E2E Activation Rules
No E2E tests activated — unit tests in ecc-app cover the validation logic.

## Test Strategy
TDD order: fix malformed skills (PC-001–004) → skill (PC-005–011) → agent (PC-012–020) → Rust validation (PC-021–026, PC-035–036) → docs (PC-027–029) → build/lint (PC-030–034).

## Doc Update Plan
| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | docs/domain/glossary.md | domain | Add entries | Interview Me + Interviewer definitions | AC-005.1 |
| 2 | CHANGELOG.md | project | Add entry | BL-013 feature entry | AC-005.2 |
| 3 | docs/adr/0010-skill-frontmatter-validation.md | architecture | Create | ADR with third-party breaking change note in Consequences | AC-005.3 |

## SOLID Assessment
PASS

## Robert's Oath Check
CLEAN

## Security Notes
CLEAR

## Rollback Plan
Revert CHANGELOG, revert glossary, delete ADR 0010, revert validate.rs, delete agents/interviewer.md, delete skills/interview-me, delete tests/test-interview-me.sh, revert 4 skill frontmatter fixes.

## Key Implementation Notes
- `skills_valid_dir` test fixture must be updated to: `"---\nname: tdd\ndescription: TDD skill\norigin: ECC\n---\n# TDD Skill"`
- Skills with no frontmatter delimiters at all → error (not silently pass)
- ADR 0010 Consequences must note: "Third-party skills without required frontmatter fields will fail validation after this change"
