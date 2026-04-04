# Solution: Pattern Library — Phase 1 Foundation

## Spec Reference
Concern: dev, Feature: Pattern Library for Agent-Assisted Development
Spec: `docs/specs/2026-04-04-pattern-library/spec.md`

---

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/config/validate.rs` | Modify | Add `VALID_PATTERN_LANGUAGES`, `VALID_PATTERN_DIFFICULTIES`, `UNSAFE_CODE_PATTERNS` constants and `REQUIRED_PATTERN_SECTIONS` constant. Pure domain knowledge, no I/O. | AC-001.5, AC-001.6, AC-002.5, AC-002.8 |
| 2 | `crates/ecc-domain/src/config/manifest.rs` | Modify | Add `patterns: Vec<String>` field with `#[serde(default)]` to `Artifacts` struct. Extend `is_ecc_managed()` to handle `"patterns"` artifact type. | AC-005.3, AC-005.5 |
| 3 | `Cargo.toml` (workspace root) | Modify | Add `rayon = "1"` to `[workspace.dependencies]`. | AC-002.10 |
| 4 | `crates/ecc-app/Cargo.toml` | Modify | Add `rayon = { workspace = true }` to `[dependencies]`. | AC-002.10 |
| 5 | `crates/ecc-app/src/validate/patterns.rs` | Create | New validation module: `validate_patterns(root, fs, terminal) -> bool`. Full quality gate: frontmatter parsing, required fields, category-directory match, required sections, section non-empty, cross-reference resolution, language-implementation matching, unsafe-code scanning, index coverage, root-file warnings, self-reference warnings. Uses `rayon::prelude::*` for parallel file collection. ~300 LOC. | AC-002.1 through AC-002.17 |
| 6 | `crates/ecc-app/src/validate/mod.rs` | Modify | Add `mod patterns;` declaration, `Patterns` variant to `ValidateTarget` enum, dispatch arm in `run_validate()`. | AC-002.1 |
| 7 | `crates/ecc-cli/src/commands/validate.rs` | Modify | Add `Patterns` variant to `CliValidateTarget` enum, mapping in `map_target()`. | AC-002.1 |
| 8 | `crates/ecc-app/src/validate/agents.rs` | Modify | Add optional `patterns` frontmatter field validation: parse as list, warn on non-existent category directories under `root.join("patterns")`. No error on missing field. | AC-006.1, AC-006.2, AC-006.3 |
| 9 | `crates/ecc-app/src/install/helpers/artifacts.rs` | Modify | Add `list_pattern_categories()` helper. Extend `collect_installed_artifacts()` to populate `artifacts.patterns` by listing subdirectories of `claude_dir.join("patterns")`. | AC-005.1, AC-005.3 |
| 10 | `crates/ecc-app/src/install/global/steps.rs` | Modify | Add `merge_patterns()` call for patterns directory (source: `ecc_root.join("patterns")`, dest: `claude_dir.join("patterns")`). Dedicated function handles two-level nesting. | AC-005.1, AC-005.2 |
| 10b | `crates/ecc-app/src/merge/mod.rs` | Modify | Add `merge_patterns()` function for recursive category-directory copy with "Patterns" label. | AC-005.1, AC-005.2 |
| 11 | `crates/ecc-app/src/config/audit/checks/content.rs` | Modify | Add `check_pattern_count()` function. Wire into `run_all_checks()` in `mod.rs`. Reports pattern file count and any schema violations. | AC-005.4 |
| 12 | `crates/ecc-app/src/config/audit/mod.rs` | Modify | Add `check_pattern_count()` call to `run_all_checks()` checks vec. | AC-005.4 |
| 13 | `patterns/index.md` | Create | Master index listing populated categories with pattern counts and language coverage summary. | AC-001.4 |
| 14 | `patterns/creational/factory-method.md` | Create | Factory Method pattern with full schema. Languages: rust, go, python, typescript. | AC-003.1, AC-003.2 |
| 15 | `patterns/creational/abstract-factory.md` | Create | Abstract Factory pattern. Languages: rust, go, python, typescript. | AC-003.1, AC-003.2 |
| 16 | `patterns/creational/builder.md` | Create | Builder pattern. Rust impl uses typestate pattern. Languages: rust, go, python, typescript. | AC-003.1, AC-003.2, AC-003.3 |
| 17 | `patterns/creational/prototype.md` | Create | Prototype pattern. Languages: rust, go, python, typescript. | AC-003.1, AC-003.2 |
| 18 | `patterns/creational/singleton.md` | Create | Singleton pattern. Anti-Patterns section warns about testability + recommends DI. Languages: rust, go, python, typescript. | AC-003.1, AC-003.2, AC-003.4 |
| 19 | `patterns/architecture/hexagonal.md` | Create | Hexagonal Architecture. Cross-references ECC crates. Languages: all. | AC-004.1, AC-004.2 |
| 20 | `patterns/architecture/clean-architecture.md` | Create | Clean Architecture. Structural examples. Languages: all. | AC-004.1, AC-004.3 |
| 21 | `patterns/architecture/cqrs.md` | Create | CQRS. Structural examples. Languages: all. | AC-004.1, AC-004.3 |
| 22 | `patterns/structural/` | Create | Empty placeholder directory (with .gitkeep). | AC-001.2 |
| 23 | `patterns/behavioral/` | Create | Empty placeholder directory. | AC-001.2 |
| 24 | `patterns/concurrency/` | Create | Empty placeholder directory. | AC-001.2 |
| 25 | `patterns/error-handling/` | Create | Empty placeholder directory. | AC-001.2 |
| 26 | `patterns/resilience/` | Create | Empty placeholder directory. | AC-001.2 |
| 27 | `patterns/testing/` | Create | Empty placeholder directory. | AC-001.2 |
| 28 | `patterns/ddd/` | Create | Empty placeholder directory. | AC-001.2 |
| 29 | `patterns/api-design/` | Create | Empty placeholder directory. | AC-001.2 |
| 30 | `patterns/security/` | Create | Empty placeholder directory. | AC-001.2 |
| 31 | `patterns/observability/` | Create | Empty placeholder directory. | AC-001.2 |
| 32 | `patterns/cicd/` | Create | Empty placeholder directory. | AC-001.2 |
| 33 | `patterns/agentic/` | Create | Empty placeholder directory. | AC-001.2 |
| 34 | `patterns/functional/` | Create | Empty placeholder directory. | AC-001.2 |
| 35 | `patterns/data-access/` | Create | Empty placeholder directory. | AC-001.2 |
| 36 | `patterns/idioms/` | Create | Empty placeholder directory. | AC-001.2 |
| 37 | `agents/architect.md` | Modify | Add `patterns: ["architecture"]` to frontmatter. | AC-006.4 |
| 38 | `.github/workflows/ci.yml` | Modify | Add `./target/release/ecc validate patterns` to "Validate components" step. | AC-002.9 |
| 39 | `CLAUDE.md` | Modify | Add `ecc validate patterns` to CLI command list. | Doc impact |
| 40 | `docs/domain/glossary.md` | Modify | Add 4 glossary terms: pattern library, pattern schema, pattern category, language matrix. | AC-001.5 (Decision 8) |
| 41 | `docs/adr/0045-patterns-as-content-type.md` | Create | ADR: Patterns as separate content type (Decision 1). | Spec Decision 1 |
| 42 | `docs/adr/0046-pattern-file-schema.md` | Create | ADR: Pattern file schema design (Decision 2). | Spec Decision 2 |

---

## Architecture Decisions

### Pattern Validation Function Signature

Follows existing convention from `skills.rs`:

```rust
pub(super) fn validate_patterns(root: &Path, fs: &dyn FileSystem, terminal: &dyn TerminalIO) -> bool
```

### Rayon Integration

Rayon is used for parallel file reading and validation. The pattern:

```rust
use rayon::prelude::*;

let results: Vec<PatternResult> = pattern_files
    .par_iter()
    .map(|path| validate_single_pattern(path, fs, &all_pattern_names))
    .collect();
```

Note: `FileSystem` trait is `Send + Sync` (required by port trait bounds), so passing `&dyn FileSystem` into rayon closures is safe. The `TerminalIO` trait is also `Send + Sync`. Error collection aggregates results after parallel phase.

### YAML Frontmatter Parsing

Reuses existing `extract_frontmatter()` from `ecc-domain`. For list-valued fields (`tags`, `languages`), the validator must parse both YAML flow syntax (`[a, b]`) and detect block syntax. Since `extract_frontmatter()` returns `HashMap<String, String>`, list fields arrive as raw strings. The validator uses `parse_tool_list()` (already in domain validate module) adapted for general list parsing -- the function already handles `["a", "b"]`, `[a, b]`, and bare strings.

### Pattern File Discovery

Pattern files are discovered by:
1. `fs.read_dir(patterns_dir)` to get category directories
2. For each category directory, `fs.read_dir(category_dir)` to get `.md` files
3. Files at root level (not in a subdirectory) are warned and skipped (AC-002.17)
4. `index.md` at root is excluded from pattern validation but checked for completeness

### Install Integration

Patterns require a dedicated `merge_patterns` function (NOT reusing `merge_skills`). `merge_skills` is hardcoded for single-level nesting (`skills/<name>/SKILL.md`) and uses the label "Skills" in output. Patterns have two-level nesting (`patterns/<category>/<pattern>.md`) with multiple `.md` files per category. `merge_patterns` copies each category directory recursively, reports with "Patterns" label, and handles the add/update/clean lifecycle. The `collect_installed_artifacts` function is extended to list pattern category directories.

### Manifest Backward Compatibility

The `patterns` field uses `#[serde(default)]` on `Vec<String>`, so old manifests without the field deserialize to an empty vec. The `is_ecc_managed()` function adds a `"patterns"` match arm.

---

## Constants (in `ecc-domain/src/config/validate.rs`)

```rust
/// Valid language identifiers for pattern frontmatter.
pub const VALID_PATTERN_LANGUAGES: &[&str] = &[
    "rust", "go", "python", "typescript", "java",
    "kotlin", "csharp", "cpp", "swift", "shell", "all",
];

/// Valid difficulty levels for pattern frontmatter.
pub const VALID_PATTERN_DIFFICULTIES: &[&str] = &[
    "beginner", "intermediate", "advanced",
];

/// Required sections in pattern files (checked as markdown headings).
pub const REQUIRED_PATTERN_SECTIONS: &[&str] = &[
    "Intent",
    "Problem",
    "Solution",
    "Language Implementations",
    "When to Use",
    "When NOT to Use",
    "Anti-Patterns",
    "Related Patterns",
    "References",
];

/// Known-unsafe code patterns to warn about in code blocks.
pub const UNSAFE_CODE_PATTERNS: &[&str] = &[
    "eval(", "eval ",
    "exec(", "exec ",
    "system(",
    "innerHTML",
    "f\"SELECT", "f\"INSERT", "f\"UPDATE", "f\"DELETE",
    "f'SELECT", "f'INSERT", "f'UPDATE", "f'DELETE",
];
```

---

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Domain constants: VALID_PATTERN_LANGUAGES contains 10 languages + "all" | AC-001.5 | `cargo test -p ecc-domain --lib -- config::validate::tests::valid_pattern_languages` | pass |
| PC-002 | unit | Domain constants: VALID_PATTERN_DIFFICULTIES contains 3 values | AC-001.6 | `cargo test -p ecc-domain --lib -- config::validate::tests::valid_pattern_difficulties` | pass |
| PC-003 | unit | Domain constants: UNSAFE_CODE_PATTERNS is non-empty | AC-002.8 | `cargo test -p ecc-domain --lib -- config::validate::tests::unsafe_code_patterns_non_empty` | pass |
| PC-004 | unit | Domain constants: REQUIRED_PATTERN_SECTIONS contains 9 sections | AC-002.5 | `cargo test -p ecc-domain --lib -- config::validate::tests::required_pattern_sections` | pass |
| PC-005 | unit | Manifest: patterns field defaults to empty vec on deserialization of old manifest | AC-005.5 | `cargo test -p ecc-domain --lib -- config::manifest::tests::patterns_field_defaults_empty` | pass |
| PC-006 | unit | Manifest: is_ecc_managed recognizes "patterns" artifact type | AC-005.3 | `cargo test -p ecc-domain --lib -- config::manifest::tests::is_ecc_managed_patterns` | pass |
| PC-007 | unit | Validate patterns: no directory returns true with skip message | AC-002.11 | `cargo test -p ecc-app --lib -- validate::patterns::tests::no_patterns_dir_succeeds` | pass |
| PC-008 | unit | Validate patterns: empty directory returns true with "0 pattern files across 0 categories" | AC-002.11 | `cargo test -p ecc-app --lib -- validate::patterns::tests::empty_dir_succeeds` | pass |
| PC-009 | unit | Validate patterns: valid pattern file passes full quality gate | AC-002.1, AC-002.4 | `cargo test -p ecc-app --lib -- validate::patterns::tests::valid_pattern_passes` | pass |
| PC-010 | unit | Validate patterns: missing required frontmatter field produces ERROR + exit 1 | AC-002.2 | `cargo test -p ecc-app --lib -- validate::patterns::tests::missing_category_field_errors` | pass |
| PC-011 | unit | Validate patterns: category-directory mismatch produces error | AC-002.3 | `cargo test -p ecc-app --lib -- validate::patterns::tests::category_dir_mismatch_errors` | pass |
| PC-012 | unit | Validate patterns: missing required section produces error | AC-002.5 | `cargo test -p ecc-app --lib -- validate::patterns::tests::missing_section_errors` | pass |
| PC-013 | unit | Validate patterns: empty section body produces error | AC-002.12 | `cargo test -p ecc-app --lib -- validate::patterns::tests::empty_section_body_errors` | pass |
| PC-014 | unit | Validate patterns: cross-reference to non-existent pattern produces error | AC-002.6 | `cargo test -p ecc-app --lib -- validate::patterns::tests::invalid_cross_ref_errors` | pass |
| PC-015 | unit | Validate patterns: language implementation not in frontmatter produces error | AC-002.7 | `cargo test -p ecc-app --lib -- validate::patterns::tests::lang_impl_mismatch_errors` | pass |
| PC-016 | unit | Validate patterns: languages: ["all"] skips implementation matching | AC-002.14 | `cargo test -p ecc-app --lib -- validate::patterns::tests::languages_all_skips_impl_check` | pass |
| PC-017 | unit | Validate patterns: empty languages list produces error | AC-002.15 | `cargo test -p ecc-app --lib -- validate::patterns::tests::empty_languages_errors` | pass |
| PC-018 | unit | Validate patterns: unrecognized language identifier produces error | AC-001.5 | `cargo test -p ecc-app --lib -- validate::patterns::tests::invalid_language_errors` | pass |
| PC-019 | unit | Validate patterns: unrecognized difficulty produces error | AC-001.6 | `cargo test -p ecc-app --lib -- validate::patterns::tests::invalid_difficulty_errors` | pass |
| PC-020 | unit | Validate patterns: unsafe code in examples without unsafe-examples: true emits warning | AC-002.8 | `cargo test -p ecc-app --lib -- validate::patterns::tests::unsafe_code_warns` | pass |
| PC-021 | unit | Validate patterns: unsafe code with unsafe-examples: true suppresses warning | AC-002.8 | `cargo test -p ecc-app --lib -- validate::patterns::tests::unsafe_examples_suppresses` | pass |
| PC-022 | unit | Validate patterns: self-reference in related-patterns emits warning | AC-002.13 | `cargo test -p ecc-app --lib -- validate::patterns::tests::self_reference_warns` | pass |
| PC-023 | unit | Validate patterns: root-level .md file (not in category) emits warning and is skipped | AC-002.17 | `cargo test -p ecc-app --lib -- validate::patterns::tests::root_level_file_warns` | pass |
| PC-024 | unit | Validate patterns: YAML flow-style list syntax parsed correctly | AC-002.16 | `cargo test -p ecc-app --lib -- validate::patterns::tests::yaml_flow_list_syntax` | pass |
| PC-025 | unit | Validate patterns: pattern not listed in index.md produces error | AC-001.4 | `cargo test -p ecc-app --lib -- validate::patterns::tests::missing_from_index_errors` | pass |
| PC-026 | unit | Validate patterns: success message shows correct counts | AC-002.4 | `cargo test -p ecc-app --lib -- validate::patterns::tests::success_message_counts` | pass |
| PC-027 | unit | Agent validation: patterns field validated against existing directories (warn on non-existent) | AC-006.1, AC-006.3 | `cargo test -p ecc-app --lib -- validate::agents::tests::agent_patterns_invalid_category_warns` | pass |
| PC-028 | unit | Agent validation: missing patterns field is backward compatible (no warning) | AC-006.2 | `cargo test -p ecc-app --lib -- validate::agents::tests::agent_no_patterns_field_ok` | pass |
| PC-029 | unit | Install artifacts: collect_installed_artifacts populates patterns field | AC-005.3 | `cargo test -p ecc-app --lib -- install::helpers::artifacts::tests::collect_artifacts_includes_patterns` | pass |
| PC-030 | unit | Audit: check_pattern_count reports pattern count | AC-005.4 | `cargo test -p ecc-app --lib -- config::audit::checks::content::tests::check_pattern_count_reports` | pass |
| PC-031 | integration | `ecc validate patterns` succeeds against real workspace patterns/ | AC-003.5, AC-004.4 | `cargo test -p ecc-integration-tests -- validate_flow::validate_patterns_passes` | pass |
| PC-032 | integration | `ecc validate agents` still passes (backward compat with patterns field) | AC-006.2 | `cargo test -p ecc-integration-tests -- validate_flow::validate_agents_passes` | pass |
| PC-033 | unit | Install: merge_patterns copies patterns from source to dest with correct nesting | AC-005.1, AC-005.2 | `cargo test -p ecc-app --lib -- install::merge::tests::merge_patterns_copies` | pass |
| PC-034 | unit | Validate patterns: Language Implementations section required and checked | AC-001.3, AC-003.2 | `cargo test -p ecc-app --lib -- validate::patterns::tests::language_implementations_section_required` | pass |
| PC-035 | lint | Clippy clean | All | `cargo clippy -- -D warnings` | 0 warnings |
| PC-036 | build | Full build succeeds | All | `cargo build` | exit 0 |

---

## TDD Order

1. **PC-001, PC-002, PC-003, PC-004** -- Domain constants first. These are leaf dependencies with zero coupling. Write the constants and their tests in `ecc-domain`. Foundation for all validation logic.

2. **PC-005, PC-006** -- Manifest changes. Add `patterns` field to `Artifacts` struct and extend `is_ecc_managed()`. Must come before install/audit tests that depend on manifest shape.

3. **PC-007, PC-008** -- Pattern validation scaffolding: no-dir and empty-dir cases. Creates `patterns.rs` with the public function signature and basic directory-existence checks. Establishes the module wiring (mod.rs, CLI mapping).

4. **PC-009** -- First valid-pattern test. Drives the happy path: frontmatter parsing, field validation, section checking. The largest single implementation step.

5. **PC-010, PC-011** -- Frontmatter error paths: missing fields, category mismatch. Exercises the error-reporting code paths.

6. **PC-012, PC-013** -- Section validation: missing sections, empty section bodies. Builds on the section-parsing logic from PC-009.

7. **PC-014** -- Cross-reference resolution. Requires multiple pattern files in the in-memory FS to test resolution.

8. **PC-015, PC-016, PC-017, PC-018, PC-019** -- Language and difficulty validation cluster. Tests language-implementation matching, languages:all skip, empty languages, invalid language, invalid difficulty. All related validation branches.

9. **PC-020, PC-021** -- Unsafe code scanning. Tests the UNSAFE_CODE_PATTERNS deny-list with and without suppression.

10. **PC-022, PC-023** -- Warning paths: self-reference and root-level files.

11. **PC-024** -- YAML list syntax parsing (flow vs block). Edge case for frontmatter parsing.

12. **PC-025** -- Index coverage validation. Requires index.md parsing logic.

13. **PC-026** -- Success message with correct counts. Final happy-path verification.

14. **PC-027, PC-028** -- Agent validation extension for patterns field. Modifies existing `agents.rs`.

15. **PC-029** -- Install artifacts collection for patterns. Extends `collect_installed_artifacts`.

16. **PC-030** -- Audit check for pattern count.

17. **PC-031, PC-032** -- Integration tests. Run after all seed pattern content files are created (content files created between PC-026 and PC-031).

18. **PC-034, PC-035** -- Lint and build gates. Always last.

---

## Phase Breakdown

### Phase 1: Domain Constants
Layers: [Entity]
Files: `crates/ecc-domain/src/config/validate.rs`
PCs: PC-001, PC-002, PC-003, PC-004
Commits: `test: add pattern domain constant tests` then `feat: add VALID_PATTERN_LANGUAGES, VALID_PATTERN_DIFFICULTIES, UNSAFE_CODE_PATTERNS, REQUIRED_PATTERN_SECTIONS`

### Phase 2: Manifest Extension
Layers: [Entity]
Files: `crates/ecc-domain/src/config/manifest.rs`
PCs: PC-005, PC-006
Commits: `test: add manifest patterns field tests` then `feat: add patterns field to Artifacts struct`

### Phase 3: Validation Scaffolding + Wiring
Layers: [UseCase, Adapter]
Files: `crates/ecc-app/src/validate/patterns.rs`, `crates/ecc-app/src/validate/mod.rs`, `crates/ecc-cli/src/commands/validate.rs`, `Cargo.toml`, `crates/ecc-app/Cargo.toml`
PCs: PC-007, PC-008
Commits: `test: add pattern validation scaffolding tests` then `feat: wire validate patterns command with rayon`

### Phase 4a: Core Validation — Frontmatter
Layers: [UseCase]
Files: `crates/ecc-app/src/validate/patterns.rs`
PCs: PC-009, PC-010, PC-011
Commits: `test: add pattern frontmatter validation tests` then `feat: implement pattern frontmatter validation`

### Phase 4b: Core Validation — Sections
PCs: PC-012, PC-013, PC-034
Commits: `test: add pattern section validation tests` then `feat: implement pattern section validation`

### Phase 4c: Core Validation — Cross-refs + Language Matching
PCs: PC-014, PC-015, PC-016, PC-017, PC-018, PC-019
Commits: `test: add cross-ref and language validation tests` then `feat: implement cross-ref and language validation`

### Phase 4d: Core Validation — Unsafe Scanning + Warnings
PCs: PC-020, PC-021, PC-022, PC-023
Commits: `test: add unsafe code and warning path tests` then `feat: implement unsafe scanning and warning paths`

### Phase 4e: Core Validation — Index + Counts
PCs: PC-024, PC-025, PC-026
Commits: `test: add index coverage and count tests` then `feat: implement index validation and success message`

### Phase 5: Agent Discovery Extension
Layers: [UseCase]
Files: `crates/ecc-app/src/validate/agents.rs`
PCs: PC-027, PC-028
Commits: `test: add agent patterns field validation tests` then `feat: validate agent patterns frontmatter field`

### Phase 6: Install + Audit Integration
Layers: [UseCase]
Files: `crates/ecc-app/src/install/helpers/artifacts.rs`, `crates/ecc-app/src/install/global/steps.rs`, `crates/ecc-app/src/config/audit/checks/content.rs`, `crates/ecc-app/src/config/audit/mod.rs`
PCs: PC-029, PC-030
Commits: `test: add pattern install and audit tests` then `feat: integrate patterns into install and audit`

### Phase 7: Seed Pattern Content
Layers: [Framework] (content files, not Rust code)
Files: All `patterns/**/*.md` files (8 patterns + index + 15 placeholder dirs)
PCs: (content validated by PC-031)
Commits: `feat: add 8 seed patterns and 17 category directories`

### Phase 8: Agent Frontmatter Update + CI
Layers: [Adapter, Framework]
Files: `agents/architect.md`, `.github/workflows/ci.yml`
PCs: PC-031, PC-032
Commits: `feat: add patterns field to architect agent` then `ci: add ecc validate patterns to CI`

### Phase 9: Documentation
Layers: [Framework]
Files: `CLAUDE.md`, `docs/domain/glossary.md`, `docs/adr/0045-patterns-as-content-type.md`, `docs/adr/0046-pattern-file-schema.md`
PCs: (documentation, no test PCs)
Commits: `docs: add pattern library ADRs, glossary terms, and CLAUDE.md update`

### Phase 10: Final Gates
PCs: PC-035, PC-036
Commits: (no new commits -- these are verification gates)
Command: `cargo clippy -- -D warnings && cargo build`

---

## AC Coverage Matrix

| AC | Covered By PCs |
|----|----------------|
| AC-001.1 | PC-009 (valid pattern passes with all required fields and sections) |
| AC-001.2 | PC-031 (integration test against real patterns/ with 17 directories) |
| AC-001.3 | PC-015, PC-016, PC-034 |
| AC-001.4 | PC-025 |
| AC-001.5 | PC-001, PC-018 |
| AC-001.6 | PC-002, PC-019 |
| AC-002.1 | PC-009, PC-010 |
| AC-002.2 | PC-010 |
| AC-002.3 | PC-011 |
| AC-002.4 | PC-026 |
| AC-002.5 | PC-004, PC-012 |
| AC-002.6 | PC-014 |
| AC-002.7 | PC-015 |
| AC-002.8 | PC-003, PC-020, PC-021 |
| AC-002.9 | PC-031 (CI runs after content exists) |
| AC-002.10 | PC-007 (rayon in dep, used in implementation) |
| AC-002.11 | PC-007, PC-008 |
| AC-002.12 | PC-013 |
| AC-002.13 | PC-022 |
| AC-002.14 | PC-016 |
| AC-002.15 | PC-017 |
| AC-002.16 | PC-024 |
| AC-002.17 | PC-023 |
| AC-003.1 | PC-031 |
| AC-003.2 | PC-031, PC-034 |
| AC-003.3 | Content review (not machine-verifiable) |
| AC-003.4 | Content review (not machine-verifiable) |
| AC-003.5 | PC-031 |
| AC-004.1 | PC-031 |
| AC-004.2 | Content review (not machine-verifiable) |
| AC-004.3 | Content review (not machine-verifiable) |
| AC-004.4 | PC-031 |
| AC-005.1 | PC-029, PC-033 |
| AC-005.2 | PC-033 (dedicated merge_patterns function) |
| AC-005.3 | PC-006, PC-029 |
| AC-005.4 | PC-030 |
| AC-005.5 | PC-005 |
| AC-006.1 | PC-027 |
| AC-006.2 | PC-028, PC-032 |
| AC-006.3 | PC-027 |
| AC-006.4 | PC-032 (integration test validates agents including architect) |

---

## Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| rayon adds transitive deps (crossbeam-*) that could fail cargo deny | Medium | rayon is MIT-licensed. deny.toml allows MIT. Run `cargo deny check` after adding dependency. |
| `extract_frontmatter()` returns flat HashMap -- list fields need parsing | Medium | Reuse `parse_tool_list()` for flow-style lists `[a, b]`. Block-style YAML (`- a\n- b`) NOT supported in Phase 1 — all seed patterns use flow syntax. AC-002.16 scoped to flow-style only; block-style support deferred to Phase 2 with a proper YAML parser upgrade. |
| Pattern content quality (AC-003.3, AC-003.4, AC-004.2, AC-004.3) not machine-verifiable | Low | Explicitly scoped as "content-review criteria" in spec. Structural correctness covered by validation. |
| InMemoryFileSystem read_dir ordering may vary | Low | Tests assert on presence/absence of messages, not ordering. Rayon results collected then iterated sequentially. |

---

## E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| 1 | FileSystem (read) | OsFileSystem | FileSystem | Pattern files read during validation | ignored | patterns.rs modified |
| 2 | TerminalIO (write) | StdTerminal | TerminalIO | Validation output formatting | ignored | Output format changed |
| 3 | CLI validate | validate.rs | N/A | ecc validate patterns subcommand | ignored | CLI mapping changed |
| 4 | Install pipeline | install/steps.rs | FileSystem | Patterns installed to target dir | ignored | Install logic modified |

### E2E Activation Rules
Tests #1 and #3 un-ignored: both validation logic and CLI mapping are new code. PC-031 integration test covers full stack.

## Test Strategy
TDD order (dependency-driven):
1. PC-001 to PC-004: Domain constants (zero coupling, leaf layer)
2. PC-005 to PC-006: Manifest extension (Artifacts struct)
3. PC-007 to PC-008: Validation scaffolding + wiring
4. PC-009: First valid pattern (happy path — largest step)
5. PC-010 to PC-011: Frontmatter error paths
6. PC-012 to PC-013: Section validation
7. PC-014: Cross-reference resolution
8. PC-015 to PC-019: Language + difficulty validation cluster
9. PC-020 to PC-021: Unsafe code scanning
10. PC-022 to PC-023: Warning paths
11. PC-024: YAML list syntax edge case
12. PC-025: Index coverage validation
13. PC-026: Success message with counts
14. PC-027 to PC-028: Agent discovery extension
15. PC-029 to PC-030: Install + audit integration
16. PC-031 to PC-032: Integration tests (after seed content)
17. PC-034 to PC-035: Lint + build gates

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CLAUDE.md | Project | Modify | Add `ecc validate patterns` to CLI command list | Doc Impact |
| 2 | docs/domain/glossary.md | Domain | Modify | Add 4 terms: pattern library, pattern schema, pattern category, language matrix | Decision 8 |
| 3 | docs/adr/0045-patterns-as-content-type.md | ADR | Create | Patterns as separate content type decision | Decision 1 |
| 4 | docs/adr/0046-pattern-file-schema.md | ADR | Create | Pattern file schema design decision | Decision 2 |
| 5 | CHANGELOG.md | Project | Modify | Add Pattern Library Phase 1 entry | Required |
| 6 | docs/ARCHITECTURE.md | Architecture | Modify | Note patterns/ in project structure | Doc Impact |
| 7 | docs/commands-reference.md | Reference | Modify | Add ecc validate patterns command | Doc Impact |

## SOLID Assessment
PASS with 2 LOW findings:
1. LOW: ValidateTarget enum variant count (10 now) — monitor if approaching 15+, consider registry pattern
2. LOW: rayon pulled for single consumer (patterns.rs) — consider feature flag if dep cost matters

## Robert's Oath Check
CLEAN — no findings. Tests planned for every behavior (35 PCs for 41 ACs). Small releases (10 phases in dependency order). Boy Scout Rule satisfied (glossary, ADRs, CI gate added).

## Security Notes
CLEAR — no findings. Path traversal mitigated (fixed root, no user-supplied paths). YAML deserialization safe (flat HashMap, no polymorphic types). Rayon safe (Send+Sync traits, no shared mutable state).

## Rollback Plan
Reverse dependency order:
1. Revert `.github/workflows/ci.yml` (remove patterns validate step)
2. Revert `agents/architect.md` (remove patterns field)
3. Delete `patterns/` directory entirely
4. Revert `crates/ecc-app/src/config/audit/` changes
5. Revert `crates/ecc-app/src/install/` changes
6. Revert `crates/ecc-app/src/validate/agents.rs` (remove patterns field logic)
7. Delete `crates/ecc-app/src/validate/patterns.rs`
8. Revert `crates/ecc-app/src/validate/mod.rs` (remove Patterns variant)
9. Revert `crates/ecc-cli/src/commands/validate.rs` (remove Patterns mapping)
10. Revert `crates/ecc-domain/src/config/manifest.rs` (remove patterns field)
11. Revert `crates/ecc-domain/src/config/validate.rs` (remove constants)
12. Remove rayon from `Cargo.toml` and `crates/ecc-app/Cargo.toml`
13. Revert docs (CLAUDE.md, glossary, ARCHITECTURE.md, commands-reference.md, delete ADRs, revert CHANGELOG)
