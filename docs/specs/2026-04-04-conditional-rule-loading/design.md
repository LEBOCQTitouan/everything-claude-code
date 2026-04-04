# Implementation Plan: BL-079 Conditional Rule Loading

## Overview

Add `applies-to` frontmatter to rules for stack-based conditional loading during `ecc install`. The domain layer parses and evaluates applicability conditions (languages, frameworks, sentinel files) with OR semantics. The app layer detects the project stack at install time and filters rules before merging. Existing rules without `applies-to` remain universally installed (backwards compatible). A `--all-rules` CLI flag overrides filtering entirely.

## Requirements

- Parse `applies-to: { languages: [...], frameworks: [...], files: [...] }` from rule frontmatter
- Evaluate with OR semantics: any matching condition means the rule applies
- No `applies-to` = universally applicable (backwards compat)
- Detect project stack at install time using existing `ecc-domain/src/detection/` infra
- Filter rules before merge step; `--all-rules` overrides filtering
- Fail-open: zero stacks detected or detection error installs ALL rules with a warning
- `ecc validate rules` warns on invalid `applies-to` values
- Annotate ~75 language-specific rule files with `applies-to`

## Architecture Changes

| File | Layer | Change |
|------|-------|--------|
| `crates/ecc-domain/src/config/applies_to.rs` (NEW) | Entity | `AppliesTo` struct, parser, evaluator, validator |
| `crates/ecc-domain/src/config/mod.rs` | Entity | Re-export `applies_to` module |
| `crates/ecc-app/src/install/helpers/stack_detect.rs` (NEW) | UseCase | Project stack detection using domain detectors + FileSystem port |
| `crates/ecc-app/src/install/helpers/rule_filter.rs` (NEW) | UseCase | Filter rule files by applicability against detected stack |
| `crates/ecc-app/src/install/helpers/mod.rs` | UseCase | Re-export new modules |
| `crates/ecc-app/src/install/mod.rs` | UseCase | Add `all_rules: bool` to `InstallOptions` |
| `crates/ecc-app/src/install/global/steps.rs` | UseCase | Wire detection + filtering into `step_merge_artifacts` |
| `crates/ecc-app/src/validate/rules.rs` | UseCase | Add `applies-to` validation warnings |
| `crates/ecc-cli/src/commands/install.rs` | Adapter | Add `--all-rules` flag |
| `rules/**/*.md` (~75 files) | Content | Add `applies-to` frontmatter |

## Implementation Steps

### Phase 1: Domain — AppliesTo Type and Parser

**Layers: [Entity]**

**Files:**
- `crates/ecc-domain/src/config/applies_to.rs` (NEW)
- `crates/ecc-domain/src/config/mod.rs` (modify)

**Actions:**
1. Create `AppliesTo` struct with `languages: Vec<String>`, `frameworks: Vec<String>`, `files: Vec<String>` fields. All optional/empty-able. Derive `Debug, Clone, PartialEq, Eq, Default`.
2. Create `parse_applies_to(frontmatter: &HashMap<String, String>) -> Option<AppliesTo>` that extracts from the existing frontmatter map. Returns `None` when no `applies-to` key exists (universal rule). Parse the YAML-ish inline format: `{ languages: [rust, python], frameworks: [django] }`.
3. Create `AppliesTo::is_empty(&self) -> bool` — returns true when all fields are empty vectors.
4. Re-export module from `config/mod.rs`.

**Why:** Domain types first. Pure logic, zero I/O. Everything else depends on this.

**Dependencies:** None

**Risk:** Low. Pure function, well-bounded.

**ACs:** AC-001.1, AC-001.2, AC-001.3, AC-001.4

#### Test Targets for Phase 1
- **Unit tests:**
  - `parse_applies_to` with `languages: [rust]` returns `Some(AppliesTo { languages: ["rust"], .. })`
  - `parse_applies_to` with `frameworks: [django]` returns frameworks
  - `parse_applies_to` with `files: ["manage.py"]` returns files
  - `parse_applies_to` with no `applies-to` key returns `None`
  - `parse_applies_to` with empty `applies-to: {}` returns `Some(AppliesTo::default())`
  - `parse_applies_to` with multi-condition returns all conditions
  - `is_empty` on default vs populated
- **Edge cases:** Whitespace variants, single-element lists, mixed quotes
- **Expected test file:** inline `#[cfg(test)] mod tests` in `applies_to.rs`

**Pass Conditions:**
```bash
cargo test -p ecc-domain -- applies_to
```

**Commit cadence:**
1. `test: add AppliesTo parser unit tests` (RED)
2. `feat: add AppliesTo type and parser` (GREEN)
3. `refactor: improve AppliesTo parser` (REFACTOR, if needed)

---

### Phase 2: Domain — Applicability Evaluator

**Layers: [Entity]**

**Files:**
- `crates/ecc-domain/src/config/applies_to.rs` (extend)

**Actions:**
1. Create `DetectedStack` struct with `languages: Vec<String>`, `frameworks: Vec<String>`, `files: Vec<String>` fields. Derive `Debug, Clone, Default`.
2. Create `evaluate_applicability(applies_to: &Option<AppliesTo>, stack: &DetectedStack) -> bool`:
   - `None` (no applies-to) returns `true` (universal)
   - `Some(empty)` returns `true` (empty conditions = universal)
   - OR semantics: if ANY language matches OR ANY framework matches OR ANY file matches, return `true`
   - All conditions present but none match: return `false`
3. Create `VALID_LANGUAGES` and `VALID_FRAMEWORKS` const arrays derived from existing detection rules (reference `LANGUAGE_RULES` and `FRAMEWORK_RULES` for the canonical list).
4. Create `validate_applies_to(applies_to: &AppliesTo) -> Vec<String>` that returns warnings for unknown language/framework values.

**Why:** Evaluation logic is pure domain. Separating parse from evaluate enables independent testing and reuse.

**Dependencies:** Phase 1

**Risk:** Low. Pure logic.

**ACs:** AC-001.5, AC-001.6

#### Test Targets for Phase 2
- **Unit tests:**
  - `evaluate_applicability(None, any_stack)` returns `true`
  - `evaluate_applicability(Some(empty), any_stack)` returns `true`
  - `evaluate_applicability(Some({languages: [rust]}), {languages: [rust]})` returns `true`
  - `evaluate_applicability(Some({languages: [python]}), {languages: [rust]})` returns `false`
  - OR semantics: `{languages: [python], frameworks: [actix]}` with stack `{frameworks: [actix]}` returns `true`
  - `validate_applies_to` with `languages: [bogus]` returns warning
  - `validate_applies_to` with `languages: [rust]` returns empty
- **Edge cases:** Empty stack, empty applies-to fields, case sensitivity
- **Expected test file:** inline in `applies_to.rs`

**Pass Conditions:**
```bash
cargo test -p ecc-domain -- applies_to
```

**Commit cadence:**
1. `test: add applicability evaluator tests` (RED)
2. `feat: add applicability evaluator and validator` (GREEN)
3. `refactor: improve evaluator` (REFACTOR, if needed)

---

### Phase 3: App — Project Stack Detection

**Layers: [UseCase]**

**Files:**
- `crates/ecc-app/src/install/helpers/stack_detect.rs` (NEW)
- `crates/ecc-app/src/install/helpers/mod.rs` (modify)

**Actions:**
1. Create `detect_project_stack(fs: &dyn FileSystem, project_dir: &Path) -> DetectedStack`:
   - Walk `project_dir` (and immediate subdirectories for monorepo support) looking for marker files from `LANGUAGE_RULES` and `FRAMEWORK_RULES`.
   - For each `LanguageRule`, check if any marker file exists at `project_dir/{marker}` or `project_dir/{subdir}/{marker}`.
   - For each `FrameworkRule`, check if any marker file exists similarly.
   - Populate `DetectedStack.files` with sentinel files found at project root (for `files:` condition matching).
   - Return `DetectedStack` with deduplicated, sorted lists.
2. Handle errors gracefully: if `fs.read_dir` fails, log warning and return empty `DetectedStack` (caller will fail-open).
3. Re-export from `helpers/mod.rs`.

**Why:** This bridges domain detection rules with the filesystem port. Must be in app layer because it uses `FileSystem` port trait.

**Dependencies:** Phase 2 (needs `DetectedStack` type)

**Risk:** Medium. Monorepo detection needs careful testing with nested structures.

**ACs:** AC-002.1 (partial), AC-002.2, AC-002.5 (partial), AC-002.6 (partial)

#### Test Targets for Phase 3
- **Unit tests (using InMemoryFileSystem):**
  - Rust project (Cargo.toml present) detects `languages: ["rust"]`
  - Python+Django project (requirements.txt + manage.py) detects both language and framework
  - Monorepo with `frontend/package.json` + `backend/Cargo.toml` detects both languages
  - Empty directory returns empty `DetectedStack`
  - Permission error returns empty `DetectedStack` (fail-open)
- **Edge cases:** Deeply nested markers (should NOT match beyond 1 level), duplicate detection
- **Expected test file:** inline in `stack_detect.rs`

**Pass Conditions:**
```bash
cargo test -p ecc-app -- stack_detect
```

**Commit cadence:**
1. `test: add project stack detection tests` (RED)
2. `feat: add project stack detection for install` (GREEN)
3. `refactor: improve stack detection` (REFACTOR, if needed)

---

### Phase 4: App — Rule Filtering

**Layers: [UseCase]**

**Files:**
- `crates/ecc-app/src/install/helpers/rule_filter.rs` (NEW)
- `crates/ecc-app/src/install/helpers/mod.rs` (modify)

**Actions:**
1. Create `RuleFilterResult` struct: `included: Vec<String>`, `skipped: Vec<String>`.
2. Create `filter_rules_by_stack(fs: &dyn FileSystem, rules_dir: &Path, groups: &[String], stack: &DetectedStack) -> RuleFilterResult`:
   - For each group, for each `.md` file in the group directory:
     - Read file content, call `extract_frontmatter`, call `parse_applies_to`.
     - Call `evaluate_applicability` against the detected stack.
     - Classify as included or skipped.
   - Return the result.
3. Re-export from `helpers/mod.rs`.

**Why:** Connects domain parsing/evaluation to the actual rule files on disk. Separated from stack detection for single-responsibility.

**Dependencies:** Phases 1-3

**Risk:** Low. Straightforward composition of existing functions.

**ACs:** AC-002.1, AC-002.4 (provides data for output)

#### Test Targets for Phase 4
- **Unit tests (using InMemoryFileSystem):**
  - Rule with `applies-to: { languages: [rust] }` + Rust stack -> included
  - Rule with `applies-to: { languages: [python] }` + Rust stack -> skipped
  - Rule without frontmatter -> included (universal)
  - Rule with empty `applies-to: {}` -> included
  - Common group rules (no applies-to) -> all included
  - Mixed group with some matching, some not
- **Edge cases:** Unreadable file (skip with warning, don't crash), empty group directory
- **Expected test file:** inline in `rule_filter.rs`

**Pass Conditions:**
```bash
cargo test -p ecc-app -- rule_filter
```

**Commit cadence:**
1. `test: add rule filtering tests` (RED)
2. `feat: add rule filtering by stack` (GREEN)
3. `refactor: improve rule filter` (REFACTOR, if needed)

---

### Phase 5: App + CLI — Wire Filtering into Install Pipeline

**Layers: [UseCase, Adapter]**

**Files:**
- `crates/ecc-app/src/install/mod.rs` (modify — add `all_rules` field)
- `crates/ecc-app/src/install/global/steps.rs` (modify — wire detection + filtering)
- `crates/ecc-cli/src/commands/install.rs` (modify — add `--all-rules` flag)

**Actions:**
1. Add `all_rules: bool` field to `InstallOptions` (default `false`).
2. In `step_merge_artifacts`:
   - Before the `collect_rule_groups` call, if `!options.all_rules`:
     - Call `detect_project_stack(ctx.fs, project_dir)` where `project_dir` is the current working directory (passed through context or resolved).
     - If detection returns empty stack, emit warning "No stack detected, installing all rules" to stderr and proceed with all rules.
     - Otherwise emit "Detected: [lang1, lang2, ...]" to stderr.
     - After collecting rule groups, call `filter_rules_by_stack` to determine which individual rule files to include.
     - Emit "Skipped N rules (not matching detected stack)" to stderr.
   - If `options.all_rules`, skip detection and filtering entirely.
3. Modify `merge_rules` to accept an optional skip list, or restructure to only merge included files.
4. Add `--all-rules` flag to `InstallArgs` in CLI.
5. Pass `all_rules` through to `InstallOptions`.

**Why:** This is the integration point. Must touch multiple layers because it wires CLI flag through app to domain.

**Dependencies:** Phases 1-4

**Risk:** Medium. Changing the install pipeline is the most sensitive part. Must preserve backwards compatibility when `--all-rules` or no `applies-to` content exists.

**ACs:** AC-002.1, AC-002.2, AC-002.3, AC-002.4, AC-002.5, AC-002.6

#### Test Targets for Phase 5
- **Integration tests (using InMemoryFileSystem):**
  - Install in Rust project: only rust + common rules merged
  - Install with `--all-rules`: all rules merged regardless of stack
  - Install in empty project: all rules merged (fail-open) with warning
  - Install in monorepo (Rust + TS): both language rules merged
  - Output contains "Detected:" and "Skipped N rules" lines
- **Edge cases:** Detection error (filesystem failure) -> fail-open, all rules installed
- **Expected test file:** `crates/ecc-app/src/install/global/steps.rs` inline tests + `crates/ecc-integration-tests/tests/`

**Pass Conditions:**
```bash
cargo test -p ecc-app -- step_merge
cargo test -p ecc-integration-tests
```

**Commit cadence:**
1. `test: add install pipeline filtering tests` (RED)
2. `feat: wire stack detection and rule filtering into install` (GREEN)
3. `refactor: improve install pipeline` (REFACTOR, if needed)

---

### Phase 6: Validation — applies-to Warnings

**Layers: [UseCase]**

**Files:**
- `crates/ecc-app/src/validate/rules.rs` (modify)

**Actions:**
1. In `validate_rules`, after checking for empty files, add a new pass:
   - For each `.md` file with frontmatter, call `parse_applies_to`.
   - If `Some(applies_to)`, call `validate_applies_to` from domain.
   - Emit warnings (not errors) for invalid values via stderr.
2. Validation still passes overall (warnings only, per AC-001.5).

**Why:** Users should get feedback when writing `applies-to` with typos.

**Dependencies:** Phase 2 (needs `validate_applies_to`)

**Risk:** Low. Additive change to existing validation.

**ACs:** AC-001.5

#### Test Targets for Phase 6
- **Unit tests:**
  - Rule with valid `applies-to: { languages: [rust] }` -> no warnings
  - Rule with `applies-to: { languages: [bogus] }` -> warning emitted
  - Rule without `applies-to` -> no warnings
  - Validation still passes (warnings, not errors)
- **Expected test file:** inline in `validate/rules.rs`

**Pass Conditions:**
```bash
cargo test -p ecc-app -- validate::rules
```

**Commit cadence:**
1. `test: add applies-to validation warning tests` (RED)
2. `feat: add applies-to validation warnings` (GREEN)

---

### Phase 7: Content — Annotate Rule Files

**Layers: [Content]**

**Files:** ~75 rule files across `rules/python/`, `rules/rust/`, `rules/golang/`, `rules/java/`, `rules/kotlin/`, `rules/csharp/`, `rules/swift/`, `rules/cpp/`, `rules/php/`, `rules/shell/`, `rules/yaml/`, `rules/json/`, `rules/typescript/`, `rules/perl/`, `rules/ecc/`

**Actions:**
1. For each language-specific rule directory, add `applies-to: { languages: [<lang>] }` to the YAML frontmatter.
   - Rules that already have `---` frontmatter: add `applies-to` line inside the existing block.
   - Rules that have NO frontmatter (like `rules/ecc/*.md`): add frontmatter block with `applies-to`.
2. `rules/common/*.md` must NOT get `applies-to` (universal). Verify no accidental annotation.
3. `rules/ecc/*.md` get `applies-to: { files: ["crates/", ".claude/"] }` since they're ECC-project-specific. Actually, per spec AC-003.4, ecc rules get "appropriate" annotations. Since ecc rules are about ECC development conventions, they can remain universal or use file markers. Decision: ecc rules use `applies-to: { files: ["agents/", "commands/", "skills/"] }` to match ECC-like project structures.

**Mapping by directory:**
| Directory | `applies-to` |
|-----------|-------------|
| `rules/common/` | NONE (universal) |
| `rules/python/` | `{ languages: [python] }` |
| `rules/rust/` | `{ languages: [rust] }` |
| `rules/typescript/` | `{ languages: [typescript] }` |
| `rules/golang/` | `{ languages: [golang] }` |
| `rules/java/` | `{ languages: [java] }` |
| `rules/kotlin/` | `{ languages: [kotlin] }` |
| `rules/csharp/` | `{ languages: [csharp] }` |
| `rules/swift/` | `{ languages: [swift] }` |
| `rules/cpp/` | `{ languages: [cpp] }` |
| `rules/php/` | `{ languages: [php] }` |
| `rules/perl/` | `{ languages: [perl] }` |
| `rules/shell/` | `{ languages: [shell] }` |
| `rules/yaml/` | `{ languages: [yaml] }` |
| `rules/json/` | `{ languages: [json] }` |
| `rules/ecc/` | `{ files: [".claude/agents"] }` |

**Why:** Content annotations make the filtering meaningful. Without this, all rules remain universal.

**Dependencies:** Phase 1 (schema must be defined first)

**Risk:** Low. Content-only changes, but must be careful not to break existing frontmatter.

**ACs:** AC-003.1, AC-003.2, AC-003.3, AC-003.4

#### Test Targets for Phase 7
- **Validation test:** Run `ecc validate rules` and verify it passes
- **Spot-check:** Grep all `rules/python/*.md` for `applies-to.*python`, all `rules/common/*.md` for absence of `applies-to`

**Pass Conditions:**
```bash
cargo test -p ecc-integration-tests -- validate_rules
# Plus manual grep verification:
grep -l "applies-to" rules/python/*.md | wc -l  # Should equal total python rules
grep -l "applies-to" rules/common/*.md | wc -l  # Should be 0
```

**Commit cadence:**
1. `feat: annotate language-specific rules with applies-to frontmatter`

---

### Phase 8: Build Gate and Final Verification

**Layers: [All]**

**Actions:**
1. Run full build: `cargo build --release`
2. Run clippy: `cargo clippy -- -D warnings`
3. Run fmt check: `cargo fmt -- --check`
4. Run full test suite: `cargo nextest run`
5. Run integration tests: `cargo test -p ecc-integration-tests`
6. Run validation: `cargo run -- validate rules`

**Dependencies:** All previous phases

**Risk:** Low. Verification only.

**Pass Conditions:**
```bash
cargo clippy -- -D warnings
cargo fmt -- --check
cargo nextest run
cargo test -p ecc-integration-tests
```

---

## E2E Assessment

- **Touches user-facing flows?** Yes -- `ecc install` behavior changes, `ecc validate rules` gets new warnings
- **Crosses 3+ modules end-to-end?** Yes -- domain (parse/evaluate) -> app (detect/filter/merge) -> CLI (--all-rules flag)
- **New E2E tests needed?** Yes
- **E2E scenarios** (after Phase 5):
  1. `ecc install` in a Rust project: only rust + common rules installed, output shows "Detected: [rust]" and "Skipped N rules"
  2. `ecc install --all-rules` in a Rust project: all rules installed regardless
  3. `ecc install` in empty directory: all rules installed with "No stack detected" warning
  4. `ecc validate rules`: passes with no new errors after annotation

## Testing Strategy

- **Unit tests:** AppliesTo parsing, evaluation, validation (domain); stack detection, rule filtering (app)
- **Integration tests:** Full install pipeline with InMemoryFileSystem; validate flow
- **E2E tests:** Install in mock project directories with real binary

## Risks & Mitigations

- **Risk:** `extract_frontmatter` is a flat key-value parser and `applies-to` uses nested YAML (`{ languages: [rust] }`)
  - Mitigation: The value after `applies-to:` will be the raw string `{ languages: [rust] }`. Write a purpose-built parser for this inline format in `parse_applies_to`, similar to how `parse_tool_list` handles bracket-delimited lists. Do NOT pull in a full YAML parser.

- **Risk:** Monorepo detection finds too many languages, defeating the purpose of filtering
  - Mitigation: Only scan 1 level deep for markers. Document this behavior. Users can use `--all-rules` as escape hatch.

- **Risk:** Breaking `ecc validate rules` for existing rules
  - Mitigation: Phase 6 adds warnings only, not errors. Existing rules without `applies-to` pass validation unchanged. Run `validate_rules` integration test after every phase.

- **Risk:** `collect_rule_groups` currently filters by directory name matching `--languages`. New filtering is per-file within groups. These could conflict.
  - Mitigation: When `applies-to` filtering is active, it subsumes the directory-level `--languages` filter. The `--languages` flag remains as a coarse pre-filter (which directories to scan), and `applies-to` is the fine-grained per-file filter within those directories.

## Boy Scout Deltas

- Phase 1: Remove unused import or TODO in `validate.rs` if found
- Phase 3: Rename `detect` module's vague `result` variable in `detect.rs`
- Phase 5: Extract magic strings in `steps.rs` (e.g., "Agents", "Commands") into constants

## Success Criteria

- [ ] `AppliesTo` struct parses all three condition types from frontmatter (AC-001.1, AC-001.2, AC-001.3)
- [ ] Rules without `applies-to` are universally applicable (AC-001.4)
- [ ] Invalid `applies-to` values produce validation warnings (AC-001.5)
- [ ] Multi-condition `applies-to` uses OR semantics (AC-001.6)
- [ ] Rust project install filters out non-Rust rules (AC-002.1)
- [ ] Monorepo installs rules for all detected languages (AC-002.2)
- [ ] `--all-rules` overrides filtering (AC-002.3)
- [ ] Output shows "Detected:" and "Skipped N rules" lines (AC-002.4)
- [ ] Empty project falls back to all rules (AC-002.5)
- [ ] Detection error falls back to all rules (AC-002.6)
- [ ] All python rules annotated (AC-003.1)
- [ ] All rust rules annotated (AC-003.2)
- [ ] Common rules have no `applies-to` (AC-003.3)
- [ ] Shell/yaml/ecc rules appropriately annotated (AC-003.4)
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo nextest run` passes (all tests green)
- [ ] `ecc validate rules` passes

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS | 0 |
| Robert | CLEAN | 0 |
| Security | CLEAR | 0 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Coverage | 88 | PASS | All 16 ACs mapped |
| Order | 92 | PASS | Clean phase dependencies |
| Fragility | 64 | CONDITIONAL | Hand-rolled parser needs explicit error handling (fail-open) |
| Rollback | 71 | PASS | --all-rules escape hatch |
| Architecture | 85 | PASS | Hexagonal boundaries respected |
| Blast Radius | 78 | PASS | Gated behind !all_rules |
| Missing PCs | 68 | CONDITIONAL | --languages interaction, case sensitivity, MODULE-SUMMARIES |
| Doc Plan | 80 | PASS | ADR + CLAUDE.md + CHANGELOG |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `crates/ecc-domain/src/config/applies_to.rs` | Create | US-001 |
| 2 | `crates/ecc-domain/src/config/mod.rs` | Modify | US-001 |
| 3 | `crates/ecc-app/src/install/helpers/stack_detect.rs` | Create | US-002 |
| 4 | `crates/ecc-app/src/install/helpers/rule_filter.rs` | Create | US-002 |
| 5 | `crates/ecc-app/src/install/helpers/mod.rs` | Modify | US-002 |
| 6 | `crates/ecc-app/src/install/global/steps.rs` | Modify | US-002 |
| 7 | `crates/ecc-app/src/validate/rules.rs` | Modify | US-001 |
| 8 | `crates/ecc-cli/src/commands/install.rs` | Modify | US-002 |
| 9 | `rules/**/*.md` (~75 files) | Modify | US-003 |
| 10 | `crates/ecc-app/src/install/mod.rs` | Modify | US-002 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-04-conditional-rule-loading/design.md` | Full design + Phase Summary |
