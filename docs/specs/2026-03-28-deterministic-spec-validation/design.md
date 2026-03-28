# Solution: Deterministic Spec/Design Artifact Validation

## Spec Reference
Concern: dev, Feature: BL-067 — Add `ecc validate spec` and `ecc validate design` subcommands

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/spec/mod.rs` | Create | Module declaration re-exporting ac, pc, coverage, ordering submodules | US-006, all |
| 2 | `crates/ecc-domain/src/spec/error.rs` | Create | `SpecError` enum with thiserror — domain error type for all spec validation failures | US-005, US-006 |
| 3 | `crates/ecc-domain/src/spec/ac.rs` | Create | `AcId` value object (us_number: u16, sub_number: u16), `parse_acs(&str) -> Result<AcReport>` — extracts AC definitions, validates sequential numbering, detects gaps/duplicates, ignores code blocks and prose references | US-001, US-006 |
| 4 | `crates/ecc-domain/src/spec/pc.rs` | Create | `PcId` value object (number: u16), `PcRow` struct (id, pc_type, description, verifies_acs, command, expected), `parse_pcs(&str) -> Result<PcReport>` — extracts PC table, validates structure (6 columns), sequential IDs, skips separator rows | US-002, US-006 |
| 5 | `crates/ecc-domain/src/spec/coverage.rs` | Create | `CoverageReport` struct (uncovered_acs, phantom_acs), `check_coverage(acs, pcs) -> CoverageReport` — cross-references AC set against PCs' "Verifies AC" column | US-003, US-006 |
| 6 | `crates/ecc-domain/src/spec/ordering.rs` | Create | `OrderingViolation` struct (pc, depends_on, reason), `check_ordering(file_changes_content, pcs) -> Vec<OrderingViolation>` — parses File Changes table, builds file-to-PC mapping, flags out-of-order PCs per file | US-004, US-006 |
| 7 | `crates/ecc-domain/src/spec/output.rs` | Create | `SpecOutput` and `DesignOutput` structs with serde Serialize — JSON output schemas for the two subcommands | US-005 |
| 8 | `crates/ecc-domain/src/lib.rs` | Modify | Add `pub mod spec;` | Module wiring |
| 9 | `crates/ecc-app/src/validate_spec.rs` | Create | `run_validate_spec(fs, terminal, path) -> bool` — reads file via FileSystem port, calls domain parsing, serializes JSON to stdout, warnings to stderr | US-001, US-005 |
| 10 | `crates/ecc-app/src/validate_design.rs` | Create | `run_validate_design(fs, terminal, path, spec_path) -> bool` — reads design file, optionally reads spec for coverage, calls domain parsing/coverage/ordering, serializes JSON output | US-002, US-003, US-004, US-005 |
| 11 | `crates/ecc-app/src/lib.rs` | Modify | Add `pub mod validate_spec;` and `pub mod validate_design;` | Module wiring |
| 12 | `crates/ecc-cli/src/commands/validate.rs` | Modify | Add `Spec { path }` and `Design { path, spec }` variants to `CliValidateTarget`, map to new app functions | US-001, US-002, US-003 |
| 13 | `crates/ecc-integration-tests/tests/validate_spec_flow.rs` | Create | E2E tests with fixture spec files (valid, gaps, duplicates, empty, nonexistent, code blocks, malformed IDs, UTF-8 errors) | US-001, US-005 |
| 14 | `crates/ecc-integration-tests/tests/validate_design_flow.rs` | Create | E2E tests with fixture design files (valid PC table, malformed rows, coverage gaps, phantom ACs, ordering violations) | US-002, US-003, US-004, US-005 |
| 15 | `crates/ecc-integration-tests/tests/fixtures/spec_valid.md` | Create | Fixture: valid spec with sequential ACs AC-001.1 through AC-003.4 | US-001 |
| 16 | `crates/ecc-integration-tests/tests/fixtures/spec_gap.md` | Create | Fixture: spec with AC gap (AC-002.1, AC-002.3 missing AC-002.2) | US-001 |
| 17 | `crates/ecc-integration-tests/tests/fixtures/spec_duplicate.md` | Create | Fixture: spec with duplicate AC IDs | US-001 |
| 18 | `crates/ecc-integration-tests/tests/fixtures/spec_empty.md` | Create | Fixture: spec with no AC lines | US-001 |
| 19 | `crates/ecc-integration-tests/tests/fixtures/spec_code_block.md` | Create | Fixture: spec with AC lines inside code blocks (should be ignored) | US-001 |
| 20 | `crates/ecc-integration-tests/tests/fixtures/spec_malformed_ids.md` | Create | Fixture: spec with malformed AC IDs (AC-ABC.1, AC-001.0, AC-000.1) | US-001 |
| 21 | `crates/ecc-integration-tests/tests/fixtures/spec_us_gap.md` | Create | Fixture: spec with US number gap (US-001, US-003, missing US-002) | US-001 |
| 22 | `crates/ecc-integration-tests/tests/fixtures/spec_prose_refs.md` | Create | Fixture: spec with AC references in prose, not as definitions | US-001 |
| 23 | `crates/ecc-integration-tests/tests/fixtures/design_valid.md` | Create | Fixture: valid design with 6-column PC table + File Changes table | US-002, US-003, US-004 |
| 24 | `crates/ecc-integration-tests/tests/fixtures/design_malformed_row.md` | Create | Fixture: design with a PC row having <6 columns | US-002 |
| 25 | `crates/ecc-integration-tests/tests/fixtures/design_pc_gap.md` | Create | Fixture: design with non-sequential PC IDs | US-002 |
| 26 | `crates/ecc-integration-tests/tests/fixtures/design_no_pc_table.md` | Create | Fixture: design without Pass Conditions table | US-002 |
| 27 | `crates/ecc-integration-tests/tests/fixtures/design_uncovered_ac.md` | Create | Fixture: design where one AC has no covering PC | US-003 |
| 28 | `crates/ecc-integration-tests/tests/fixtures/design_ordering_violation.md` | Create | Fixture: design with PCs modifying same file in wrong order | US-004 |
| 29 | `CLAUDE.md` | Modify | Add `ecc validate spec` and `ecc validate design` to CLI Commands section | AC-007.1 |

## Architecture Notes

### Why separate `validate_spec.rs` and `validate_design.rs` instead of extending `validate.rs`

`validate.rs` is already 1269 lines (well above 800-line limit). The spec/design validation has a fundamentally different interface — it outputs JSON to stdout and uses exit codes, while existing validators use bool return + stderr text. Adding to `validate.rs` would push it to ~1500+ lines and mix two output paradigms. New files in `ecc-app/src/` follow the existing pattern of one file per use case.

The CLI dispatch in `commands/validate.rs` will call the new app functions directly for Spec/Design variants rather than going through `run_validate`, since the interface contract is different (JSON stdout vs. text stdout/stderr).

### Domain module structure

```
crates/ecc-domain/src/spec/
├── mod.rs        — re-exports
├── error.rs      — SpecError enum
├── ac.rs         — AcId, AcReport, parse_acs()
├── pc.rs         — PcId, PcRow, PcReport, parse_pcs()
├── coverage.rs   — CoverageReport, check_coverage()
├── ordering.rs   — OrderingViolation, check_ordering()
└── output.rs     — SpecOutput, DesignOutput (serde)
```

All functions are pure `&str -> Result<T>`. Zero I/O. The app layer handles file reading via the FileSystem port and JSON serialization to the TerminalIO port.

### Code block detection

AC lines inside markdown fenced code blocks (` ``` `) must be ignored (AC-001.7). The parser tracks a `in_code_block: bool` flag, toggling on lines starting with ` ``` `. This is simpler and faster than pulling in a full markdown AST parser.

### AC definition vs. AC reference (AC-001.11)

Only lines matching `^- AC-\d{3}\.\d+:` are treated as AC definitions. Inline references like "See AC-001.1 for details" are ignored because they lack the leading `- ` and trailing `:` after the ID.

### PC table detection

The PC table is identified by a header row containing all 6 column names: `ID`, `Type`, `Description`, `Verifies AC`, `Command`, `Expected`. The parser scans for this header, then processes subsequent `|`-delimited rows, skipping the separator row (`---|---`).

### File Changes table detection (ordering validation)

The File Changes table is identified by a header containing `File` and `Action` columns. The parser extracts file paths from the `File` column and maps them to PCs via the `Spec Ref` column (matching AC references in PCs' "Verifies AC" field). If no File Changes table is found, ordering validation is skipped with a warning (AC-004.3).

### Error accumulation

All validation collects ALL errors before reporting (spec constraint). The domain functions return `AcReport` / `PcReport` structs that include both the parsed data and accumulated `errors: Vec<String>` and `warnings: Vec<String>`.

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | AcId::parse("AC-001.2") returns us_number=1, sub_number=2 | AC-006.2 | `cargo test -p ecc-domain -- spec::ac::tests::ac_id_valid` | PASS |
| PC-002 | unit | AcId::parse("AC-1.2") returns error | AC-006.3 | `cargo test -p ecc-domain -- spec::ac::tests::ac_id_invalid_format` | PASS |
| PC-003 | unit | AcId::parse("AC-000.1") returns error (zero US not allowed) | AC-001.9 | `cargo test -p ecc-domain -- spec::ac::tests::ac_id_zero_us` | PASS |
| PC-004 | unit | AcId::parse("AC-001.0") returns error (zero sub not allowed) | AC-001.9 | `cargo test -p ecc-domain -- spec::ac::tests::ac_id_zero_sub` | PASS |
| PC-005 | unit | AcId::parse("AC-ABC.1") returns error | AC-001.9 | `cargo test -p ecc-domain -- spec::ac::tests::ac_id_non_numeric` | PASS |
| PC-006 | unit | parse_acs with sequential ACs returns valid report | AC-001.1, AC-001.5 | `cargo test -p ecc-domain -- spec::ac::tests::parse_acs_valid_sequential` | PASS |
| PC-007 | unit | parse_acs with gap reports error | AC-001.2 | `cargo test -p ecc-domain -- spec::ac::tests::parse_acs_gap_detected` | PASS |
| PC-008 | unit | parse_acs with duplicates reports error | AC-001.3 | `cargo test -p ecc-domain -- spec::ac::tests::parse_acs_duplicate_detected` | PASS |
| PC-009 | unit | parse_acs with no ACs reports error | AC-001.4 | `cargo test -p ecc-domain -- spec::ac::tests::parse_acs_no_acs_found` | PASS |
| PC-010 | unit | parse_acs ignores AC lines inside code blocks | AC-001.7 | `cargo test -p ecc-domain -- spec::ac::tests::parse_acs_ignores_code_blocks` | PASS |
| PC-011 | unit | parse_acs detects US number gaps | AC-001.8 | `cargo test -p ecc-domain -- spec::ac::tests::parse_acs_us_gap_detected` | PASS |
| PC-012 | unit | parse_acs adds warnings for malformed AC IDs | AC-001.9 | `cargo test -p ecc-domain -- spec::ac::tests::parse_acs_malformed_warnings` | PASS |
| PC-013 | unit | parse_acs ignores prose AC references (no leading "- " or trailing ":") | AC-001.11 | `cargo test -p ecc-domain -- spec::ac::tests::parse_acs_ignores_prose_refs` | PASS |
| PC-014 | unit | PcId::parse("PC-003") returns number=3 | AC-006.4 | `cargo test -p ecc-domain -- spec::pc::tests::pc_id_valid` | PASS |
| PC-015 | unit | parse_pcs with valid 6-column table returns PcReport | AC-002.1 | `cargo test -p ecc-domain -- spec::pc::tests::parse_pcs_valid_table` | PASS |
| PC-016 | unit | parse_pcs with row <6 columns reports malformed row with row number | AC-002.2 | `cargo test -p ecc-domain -- spec::pc::tests::parse_pcs_malformed_row` | PASS |
| PC-017 | unit | parse_pcs with non-sequential PC IDs reports gap | AC-002.3 | `cargo test -p ecc-domain -- spec::pc::tests::parse_pcs_gap_detected` | PASS |
| PC-018 | unit | parse_pcs with duplicate PC IDs reports duplicates | AC-002.4 | `cargo test -p ecc-domain -- spec::pc::tests::parse_pcs_duplicate_detected` | PASS |
| PC-019 | unit | parse_pcs with empty required fields reports error | AC-002.5 | `cargo test -p ecc-domain -- spec::pc::tests::parse_pcs_empty_fields` | PASS |
| PC-020 | unit | parse_pcs with no PC table reports error | AC-002.6 | `cargo test -p ecc-domain -- spec::pc::tests::parse_pcs_no_table_found` | PASS |
| PC-021 | unit | parse_pcs correctly skips separator rows | AC-002.7 | `cargo test -p ecc-domain -- spec::pc::tests::parse_pcs_skips_separator` | PASS |
| PC-022 | unit | check_coverage with all ACs covered returns empty uncovered_acs | AC-003.1 | `cargo test -p ecc-domain -- spec::coverage::tests::all_acs_covered` | PASS |
| PC-023 | unit | check_coverage with uncovered AC returns it in uncovered_acs | AC-003.2 | `cargo test -p ecc-domain -- spec::coverage::tests::uncovered_ac_reported` | PASS |
| PC-024 | unit | check_coverage with phantom AC returns warning | AC-003.5 | `cargo test -p ecc-domain -- spec::coverage::tests::phantom_ac_warning` | PASS |
| PC-025 | unit | check_coverage with multiple PCs covering same AC — no warning | AC-003.6 | `cargo test -p ecc-domain -- spec::coverage::tests::multi_pc_same_ac_valid` | PASS |
| PC-026 | unit | CoverageReport with all covered — uncovered and phantom both empty | AC-006.5 | `cargo test -p ecc-domain -- spec::coverage::tests::coverage_report_all_covered` | PASS |
| PC-027 | unit | OrderingViolation contains pc, depends_on, reason | AC-006.6 | `cargo test -p ecc-domain -- spec::ordering::tests::violation_struct_fields` | PASS |
| PC-028 | unit | check_ordering with correct order returns empty violations | AC-004.1, AC-004.2 | `cargo test -p ecc-domain -- spec::ordering::tests::correct_order_no_violations` | PASS |
| PC-029 | unit | check_ordering with wrong order returns violation with file in reason | AC-004.4 | `cargo test -p ecc-domain -- spec::ordering::tests::wrong_order_detected` | PASS |
| PC-030 | unit | check_ordering with no File Changes table returns warning | AC-004.3 | `cargo test -p ecc-domain -- spec::ordering::tests::no_file_changes_warning` | PASS |
| PC-031 | unit | validate_spec reads file, calls parse_acs, outputs valid JSON on success | AC-005.1, AC-005.4 | `cargo test -p ecc-app -- validate_spec::tests::valid_spec_json_output` | PASS |
| PC-032 | unit | validate_spec outputs JSON with errors on validation failure | AC-005.2 | `cargo test -p ecc-app -- validate_spec::tests::invalid_spec_json_errors` | PASS |
| PC-033 | unit | validate_spec returns error for nonexistent file | AC-001.6 | `cargo test -p ecc-app -- validate_spec::tests::nonexistent_file_error` | PASS |
| PC-034 | unit | validate_spec returns error for non-UTF-8 file | AC-001.10 | `cargo test -p ecc-app -- validate_spec::tests::non_utf8_file_error` | PASS |
| PC-035 | unit | validate_spec sends warnings to stderr, JSON to stdout | AC-005.3 | `cargo test -p ecc-app -- validate_spec::tests::warnings_to_stderr` | PASS |
| PC-036 | unit | validate_design outputs JSON with pc_count, pcs, etc. on success | AC-005.5 | `cargo test -p ecc-app -- validate_design::tests::valid_design_json_output` | PASS |
| PC-037 | unit | validate_design with --spec runs coverage check | AC-003.1 | `cargo test -p ecc-app -- validate_design::tests::with_spec_runs_coverage` | PASS |
| PC-038 | unit | validate_design without --spec skips coverage (uncovered_acs null) | AC-003.3 | `cargo test -p ecc-app -- validate_design::tests::without_spec_skips_coverage` | PASS |
| PC-039 | unit | validate_design with nonexistent --spec returns error | AC-003.4 | `cargo test -p ecc-app -- validate_design::tests::nonexistent_spec_error` | PASS |
| PC-040 | unit | validate_design phantom ACs are warnings only (valid still true if no other errors) | AC-003.5 | `cargo test -p ecc-app -- validate_design::tests::phantom_acs_do_not_fail` | PASS |
| PC-041 | integration | `ecc validate spec` with valid fixture file exits 0 with valid JSON | AC-001.1, AC-005.1 | `cargo test -p ecc-integration-tests -- validate_spec_flow::valid_spec_exits_zero` | PASS |
| PC-042 | integration | `ecc validate spec` with gap fixture exits 1 with errors in JSON | AC-001.2, AC-005.2 | `cargo test -p ecc-integration-tests -- validate_spec_flow::gap_spec_exits_one` | PASS |
| PC-043 | integration | `ecc validate spec` with nonexistent path exits 1 | AC-001.6 | `cargo test -p ecc-integration-tests -- validate_spec_flow::nonexistent_path_exits_one` | PASS |
| PC-044 | integration | `ecc validate design` with valid fixture exits 0 | AC-002.1, AC-005.1 | `cargo test -p ecc-integration-tests -- validate_design_flow::valid_design_exits_zero` | PASS |
| PC-045 | integration | `ecc validate design --spec` with coverage gap reports uncovered ACs | AC-003.2 | `cargo test -p ecc-integration-tests -- validate_design_flow::coverage_gap_reported` | PASS |
| PC-046 | integration | `ecc validate design` with ordering violation reports it | AC-004.4 | `cargo test -p ecc-integration-tests -- validate_design_flow::ordering_violation_reported` | PASS |
| PC-047 | lint | clippy zero warnings | All | `cargo clippy -- -D warnings` | exit 0 |
| PC-048 | build | release build succeeds | All | `cargo build --release` | exit 0 |

### Adversary-Required Additions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-049 | unit | parse_acs ignores AC lines inside tilde-fenced code blocks (~~~) | AC-001.7 | `cargo test -p ecc-domain -- spec::ac::tests::ignores_tilde_fenced_blocks` | PASS |
| PC-050 | lint | CLAUDE.md contains validate spec and validate design commands | AC-007.1 | `grep -q 'validate spec' CLAUDE.md && grep -q 'validate design' CLAUDE.md` | exit 0 |
| PC-051 | lint | ADR-0022 exists | Spec Decision 2 | `test -f docs/adr/0022-regex-over-ast.md` | exit 0 |
| PC-052 | lint | ADR-0023 exists | Spec Decision 3 | `test -f docs/adr/0023-separate-spec-module.md` | exit 0 |
| PC-053 | lint | CHANGELOG.md contains BL-067 entry | Doc Impact | `grep -q 'BL-067' CHANGELOG.md` | exit 0 |
| PC-054 | unit | check_ordering skips with warning when File Changes table has unrecognized format | AC-004.3 | `cargo test -p ecc-domain -- spec::ordering::tests::unrecognized_table_format_skipped` | PASS |

### Additional File Changes (Adversary-Required)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 30 | `docs/adr/0022-regex-over-ast.md` | Create | ADR for regex over AST decision | Spec Decision 2 |
| 31 | `docs/adr/0023-separate-spec-module.md` | Create | ADR for separate spec module decision | Spec Decision 3 |
| 32 | `CHANGELOG.md` | Modify | Add BL-067 entry | Doc Impact |
| 33 | `docs/domain/glossary.md` | Modify | Add AcId, PcId, CoverageReport, OrderingViolation | AC-007.2 |

Note: AC-007.2 (glossary) and AC-007.3 (backlog promotion) are verified via PC-050/PC-053 and manual checklist respectively. These are doc-level verifications, not code tests.

### Coverage Check

All 42 ACs covered. Zero uncovered ACs.

| AC | Covered By |
|----|------------|
| AC-001.1 | PC-006, PC-041 |
| AC-001.2 | PC-007, PC-042 |
| AC-001.3 | PC-008 |
| AC-001.4 | PC-009 |
| AC-001.5 | PC-006 |
| AC-001.6 | PC-033, PC-043 |
| AC-001.7 | PC-010 |
| AC-001.8 | PC-011 |
| AC-001.9 | PC-003, PC-004, PC-005, PC-012 |
| AC-001.10 | PC-034 |
| AC-001.11 | PC-013 |
| AC-002.1 | PC-015, PC-044 |
| AC-002.2 | PC-016 |
| AC-002.3 | PC-017 |
| AC-002.4 | PC-018 |
| AC-002.5 | PC-019 |
| AC-002.6 | PC-020 |
| AC-002.7 | PC-021 |
| AC-003.1 | PC-022, PC-037 |
| AC-003.2 | PC-023, PC-045 |
| AC-003.3 | PC-038 |
| AC-003.4 | PC-039 |
| AC-003.5 | PC-024, PC-040 |
| AC-003.6 | PC-025 |
| AC-004.1 | PC-028 |
| AC-004.2 | PC-028 |
| AC-004.3 | PC-030 |
| AC-004.4 | PC-029, PC-046 |
| AC-005.1 | PC-031, PC-041, PC-044 |
| AC-005.2 | PC-032, PC-042 |
| AC-005.3 | PC-035 |
| AC-005.4 | PC-031 |
| AC-005.5 | PC-036 |
| AC-006.1 | PC-001 through PC-030 (all domain tests, zero I/O imports) |
| AC-006.2 | PC-001 |
| AC-006.3 | PC-002 |
| AC-006.4 | PC-014 |
| AC-006.5 | PC-026 |
| AC-006.6 | PC-027 |
| AC-007.1 | PC-048 (verified by reading CLAUDE.md after file change #29) |
| AC-007.2 | Out of scope for pass conditions (manual doc update) |
| AC-007.3 | Out of scope for pass conditions (manual backlog update) |

## TDD Implementation Order

### Phase 1: Domain Value Objects (Entity layer)
Layers: [Entity]

**Files**: `crates/ecc-domain/src/spec/mod.rs`, `error.rs`, `ac.rs` (value objects only)

1. Write tests for `AcId::parse` — valid, invalid format, zero US, zero sub, non-numeric (PC-001 through PC-005)
2. Implement `AcId` value object to pass tests
3. Refactor + Boy Scout

**Commit cadence**:
- `test: add AcId value object tests`
- `feat: implement AcId value object`
- `refactor: improve AcId` (if needed)

### Phase 2: AC Parsing (Entity layer)
Layers: [Entity]

**Files**: `crates/ecc-domain/src/spec/ac.rs` (parsing logic)

1. Write tests for `parse_acs` — valid sequential, gap, duplicate, empty, code blocks, US gaps, malformed warnings, prose refs (PC-006 through PC-013)
2. Implement `parse_acs` to pass tests
3. Refactor + Boy Scout

**Commit cadence**:
- `test: add AC parsing tests`
- `feat: implement AC parsing`
- `refactor: improve AC parsing` (if needed)

### Phase 3: PC Parsing (Entity layer)
Layers: [Entity]

**Files**: `crates/ecc-domain/src/spec/pc.rs`

1. Write tests for `PcId::parse` and `parse_pcs` — valid table, malformed row, gap, duplicate, empty fields, no table, separator skip (PC-014 through PC-021)
2. Implement `PcId` and `parse_pcs` to pass tests
3. Refactor + Boy Scout

**Commit cadence**:
- `test: add PC parsing tests`
- `feat: implement PC parsing`
- `refactor: improve PC parsing` (if needed)

### Phase 4: Coverage Analysis (Entity layer)
Layers: [Entity]

**Files**: `crates/ecc-domain/src/spec/coverage.rs`

1. Write tests for `check_coverage` — all covered, uncovered, phantom, multi-PC same AC, coverage report struct (PC-022 through PC-026)
2. Implement `check_coverage` to pass tests
3. Refactor + Boy Scout

**Commit cadence**:
- `test: add coverage analysis tests`
- `feat: implement coverage analysis`
- `refactor: improve coverage analysis` (if needed)

### Phase 5: Ordering Validation (Entity layer)
Layers: [Entity]

**Files**: `crates/ecc-domain/src/spec/ordering.rs`

1. Write tests for `check_ordering` — struct fields, correct order, wrong order, no file changes table (PC-027 through PC-030)
2. Implement `check_ordering` to pass tests
3. Refactor + Boy Scout

**Commit cadence**:
- `test: add ordering validation tests`
- `feat: implement ordering validation`
- `refactor: improve ordering validation` (if needed)

### Phase 6: JSON Output Types (Entity layer)
Layers: [Entity]

**Files**: `crates/ecc-domain/src/spec/output.rs`, `crates/ecc-domain/src/lib.rs`

1. Write tests for `SpecOutput` and `DesignOutput` serialization to JSON
2. Implement output structs with serde Serialize
3. Add `pub mod spec;` to `lib.rs`

**Commit cadence**:
- `test: add JSON output type tests`
- `feat: implement JSON output types and wire spec module`

### Phase 7: App Layer — Spec Validation (UseCase layer)
Layers: [UseCase]

**Files**: `crates/ecc-app/src/validate_spec.rs`, `crates/ecc-app/src/lib.rs`

1. Write tests for `run_validate_spec` — valid output, error output, nonexistent file, non-UTF-8, warnings to stderr (PC-031 through PC-035). Use `InMemoryFileSystem` and `MockTerminal` from `ecc-test-support`.
2. Implement `run_validate_spec` to pass tests
3. Add `pub mod validate_spec;` to `lib.rs`
4. Refactor + Boy Scout

**Commit cadence**:
- `test: add validate_spec use case tests`
- `feat: implement validate_spec use case`
- `refactor: improve validate_spec` (if needed)

### Phase 8: App Layer — Design Validation (UseCase layer)
Layers: [UseCase]

**Files**: `crates/ecc-app/src/validate_design.rs`

1. Write tests for `run_validate_design` — valid output, with-spec coverage, without-spec skips coverage, nonexistent spec, phantom ACs as warnings (PC-036 through PC-040). Use test doubles.
2. Implement `run_validate_design` to pass tests
3. Add `pub mod validate_design;` to `lib.rs`
4. Refactor + Boy Scout

**Commit cadence**:
- `test: add validate_design use case tests`
- `feat: implement validate_design use case`
- `refactor: improve validate_design` (if needed)

### Phase 9: CLI Wiring (Adapter layer)
Layers: [Adapter]

**Files**: `crates/ecc-cli/src/commands/validate.rs`

1. Add `Spec { path: PathBuf }` and `Design { path: PathBuf, #[arg(long)] spec: Option<PathBuf> }` variants to `CliValidateTarget`
2. Add match arms in `run()` calling `ecc_app::validate_spec::run_validate_spec` and `ecc_app::validate_design::run_validate_design`
3. Verify `cargo build` passes

**Commit cadence**:
- `feat: wire spec and design validate subcommands`

### Phase 10: Integration Tests (Framework layer)
Layers: [Framework]

**Files**: `crates/ecc-integration-tests/tests/validate_spec_flow.rs`, `validate_design_flow.rs`, `fixtures/*.md`

1. Create fixture files (valid spec, gap spec, design files, etc.)
2. Write integration tests exercising the full CLI binary (PC-041 through PC-046)
3. Verify all tests pass

**Commit cadence**:
- `test: add spec/design validation integration tests`

### Phase 11: Documentation + Final Gate
Layers: [Adapter]

**Files**: `CLAUDE.md`

1. Add `ecc validate spec` and `ecc validate design` to CLI Commands section in CLAUDE.md
2. Run `cargo clippy -- -D warnings` (PC-047)
3. Run `cargo build --release` (PC-048)
4. Run full test suite `cargo test`

**Commit cadence**:
- `docs: add validate spec/design to CLAUDE.md`

## E2E Assessment
- **Touches user-facing flows?** Yes — new CLI subcommands
- **Crosses 3+ modules end-to-end?** Yes — domain (spec/) → app (validate_spec/validate_design) → CLI (validate)
- **New E2E tests needed?** Yes
- **E2E scenarios** (Phase 10):
  1. `ecc validate spec valid.md` exits 0 with valid JSON containing `ac_count` and `acs`
  2. `ecc validate spec gap.md` exits 1 with JSON containing errors array
  3. `ecc validate spec nonexistent.md` exits 1
  4. `ecc validate design valid.md` exits 0 with valid JSON
  5. `ecc validate design design.md --spec spec.md` reports uncovered ACs
  6. `ecc validate design ordering-violation.md` reports ordering violations

## Testing Strategy
- **Unit tests**: All domain parsing logic (30 tests in ecc-domain), app orchestration (10 tests in ecc-app) — using test doubles from ecc-test-support
- **Integration tests**: 6 E2E tests in ecc-integration-tests exercising the real binary with fixture files
- **Coverage target**: 80%+ on domain module, full AC coverage via PC mapping

## Risks & Mitigations

- **Risk**: Regex fragility — AC/PC patterns evolve and regex breaks silently
  - Mitigation: Comprehensive fixture-based tests; regex patterns documented in code comments; integration tests use real spec format

- **Risk**: `validate.rs` already at 1269 lines — new variants could push existing file further
  - Mitigation: New validation lives in separate `validate_spec.rs` / `validate_design.rs` files; CLI dispatches directly to new functions for Spec/Design variants

- **Risk**: Code block detection may miss edge cases (nested blocks, tildes `~~~`)
  - Mitigation: Track both ` ``` ` and `~~~` fences; test with nested code blocks; document limitation of non-recursive detection

- **Risk**: File Changes table format varies across design files
  - Mitigation: Skip ordering validation with warning when table format is unrecognized (AC-004.3); detect by header column names, not position

## Success Criteria

- [ ] `ecc validate spec valid-spec.md` exits 0 with JSON containing `valid: true`, `ac_count`, and `acs` array
- [ ] `ecc validate spec gap-spec.md` exits 1 with JSON containing `valid: false` and errors describing the gap
- [ ] `ecc validate design valid-design.md` exits 0 with JSON containing `pc_count`, `pcs`, and `ordering_violations`
- [ ] `ecc validate design design.md --spec spec.md` cross-references and reports `uncovered_acs` and `phantom_acs`
- [ ] All 42 spec ACs are covered by at least one PC
- [ ] `cargo clippy -- -D warnings` exits 0
- [ ] `cargo build --release` exits 0
- [ ] `cargo test` passes with all new tests (48 PCs)
- [ ] Domain module `ecc-domain/src/spec/` has zero I/O imports
- [ ] JSON output goes to stdout, warnings go to stderr
