# Design: Audit-Web Guided Profile & Self-Improvement (BL-107)

## Overview

Add a YAML-based profile system to `/audit-web` with typed domain types, CLI management commands, deterministic report validation, and command markdown updates for Phase 0 (guided setup) and Phase 5 (self-improvement). Follows the same hexagonal pattern as `ecc sources`: domain types with serde derives, app use cases through FileSystem port, CLI wiring via clap.

## File Changes

| # | File | Layer | Action | Spec Ref | Depends On |
|---|------|-------|--------|----------|------------|
| FC-01 | `crates/ecc-domain/src/audit_web/mod.rs` | Entity | New: module declaration for `profile`, `dimension`, `report_validation` | US-001 | -- |
| FC-02 | `crates/ecc-domain/src/audit_web/profile.rs` | Entity | New: `AuditWebProfile`, `DimensionThreshold`, `ImprovementSuggestion`, `ProfileError`, YAML parse/serialize, round-trip, version check | US-001: AC-001.1, AC-001.3, AC-001.4, AC-001.5, AC-001.7 | FC-01 |
| FC-03 | `crates/ecc-domain/src/audit_web/dimension.rs` | Entity | New: `AuditDimension`, standard dimension definitions, `validate_query_template` sanitization | US-001: AC-001.2, AC-001.6 | FC-01 |
| FC-04 | `crates/ecc-domain/src/audit_web/report_validation.rs` | Entity | New: `ReportValidationResult`, required section check, score range check, citation count warning | US-003: AC-003.1..AC-003.4 | FC-01 |
| FC-05 | `crates/ecc-domain/src/lib.rs` | Entity | Modify: add `pub mod audit_web;` | -- | FC-01 |
| FC-06 | `crates/ecc-app/src/audit_web.rs` | UseCase | New: `profile::init`, `profile::show`, `profile::validate`, `profile::reset`, `report::validate` use cases | US-002, US-003 | FC-02, FC-03, FC-04 |
| FC-07 | `crates/ecc-app/src/lib.rs` | UseCase | Modify: add `pub mod audit_web;` | -- | FC-06 |
| FC-08 | `crates/ecc-cli/src/commands/audit_web.rs` | Adapter | New: `AuditWebArgs`, `AuditWebAction::{Profile, ValidateReport}`, clap wiring, `run()` | US-002, US-003 | FC-06 |
| FC-09 | `crates/ecc-cli/src/commands/mod.rs` | Adapter | Modify: add `pub mod audit_web;` | -- | FC-08 |
| FC-10 | `crates/ecc-cli/src/main.rs` | Framework | Modify: add `AuditWeb` variant to `Command` enum, route to `commands::audit_web::run` | -- | FC-08, FC-09 |
| FC-11 | `commands/audit-web.md` | Framework | Modify: add Phase 0 (guided setup) and Phase 5 (self-improvement), renumber to 0-5 | US-004: AC-004.1..AC-004.8 | FC-02, FC-03 |

## Pass Conditions

| PC | Description | Bash Command | Spec Ref | Depends On |
|----|-------------|--------------|----------|------------|
| PC-001 | `AuditWebProfile` construction with dimensions, thresholds, history | `cargo test -p ecc-domain audit_web::profile::tests::profile_construction` | AC-001.1 | -- |
| PC-002 | YAML round-trip: serialize then parse produces identical struct | `cargo test -p ecc-domain audit_web::profile::tests::yaml_round_trip` | AC-001.4 | PC-001 |
| PC-003 | Corrupted YAML returns typed error with location | `cargo test -p ecc-domain audit_web::profile::tests::corrupted_yaml_error` | AC-001.5 | PC-001 |
| PC-004 | Unknown version returns error with upgrade message | `cargo test -p ecc-domain audit_web::profile::tests::unknown_version_error` | AC-001.7 | PC-001 |
| PC-005 | `AuditDimension` with valid query template passes sanitization | `cargo test -p ecc-domain audit_web::dimension::tests::valid_query_template` | AC-001.2 | -- |
| PC-006 | Query template with shell metacharacters rejected | `cargo test -p ecc-domain audit_web::dimension::tests::rejects_shell_metacharacters` | AC-001.6 | PC-005 |
| PC-007 | Query template allows alphanumeric, spaces, hyphens, underscores, dots, slashes, `{placeholder}` | `cargo test -p ecc-domain audit_web::dimension::tests::allows_safe_chars` | AC-001.6 | PC-005 |
| PC-008 | Standard 8 dimensions are defined and all valid | `cargo test -p ecc-domain audit_web::dimension::tests::standard_dimensions` | AC-001.1 | PC-005 |
| PC-009 | Valid report with all sections passes validation | `cargo test -p ecc-domain audit_web::report_validation::tests::valid_report_passes` | AC-003.1 | -- |
| PC-010 | Report missing required sections fails with specific errors | `cargo test -p ecc-domain audit_web::report_validation::tests::missing_sections_error` | AC-003.2 | PC-009 |
| PC-011 | Score outside 0-5 range fails validation | `cargo test -p ecc-domain audit_web::report_validation::tests::score_out_of_range` | AC-003.3 | PC-009 |
| PC-012 | Finding with <3 citations produces warning | `cargo test -p ecc-domain audit_web::report_validation::tests::low_citation_warning` | AC-003.4 | PC-009 |
| PC-013 | `profile::init` scans codebase and writes profile YAML | `cargo test -p ecc-app audit_web::tests::init_creates_profile` | AC-002.1 | PC-001, PC-005, PC-008 |
| PC-014 | `profile::init` errors if profile already exists | `cargo test -p ecc-app audit_web::tests::init_rejects_existing` | AC-002.5 | PC-013 |
| PC-015 | `profile::show` reads and returns profile content | `cargo test -p ecc-app audit_web::tests::show_reads_profile` | AC-002.2 | PC-013 |
| PC-016 | `profile::validate` passes for valid profile | `cargo test -p ecc-app audit_web::tests::validate_valid_profile` | AC-002.3 | PC-013 |
| PC-017 | `profile::reset` deletes profile file | `cargo test -p ecc-app audit_web::tests::reset_deletes_profile` | AC-002.4 | PC-013 |
| PC-018 | `report::validate` passes for well-formed report | `cargo test -p ecc-app audit_web::tests::validate_report_passes` | AC-003.1 | PC-009 |
| PC-019 | CLI `ecc audit-web profile init` routes to app use case | `cargo test -p ecc-cli commands::audit_web::tests::profile_init_routes` | AC-002.1 | PC-013 |
| PC-020 | CLI `ecc audit-web validate-report` routes to app use case | `cargo test -p ecc-cli commands::audit_web::tests::validate_report_routes` | AC-003.1 | PC-018 |
| PC-021 | `cargo clippy -p ecc-domain -- -D warnings` passes | `cargo clippy -p ecc-domain -- -D warnings` | -- | PC-001..PC-012 |
| PC-022 | `cargo clippy -p ecc-app -- -D warnings` passes | `cargo clippy -p ecc-app -- -D warnings` | -- | PC-013..PC-018 |
| PC-023 | `cargo clippy -p ecc-cli -- -D warnings` passes | `cargo clippy -p ecc-cli -- -D warnings` | -- | PC-019..PC-020 |
| PC-024 | Full workspace builds: `cargo build` | `cargo build` | -- | PC-021..PC-023 |
| PC-025 | Full test suite passes: `cargo test` | `cargo test` | -- | PC-024 |

## TDD Phase Order

### Phase 1: Domain — Profile types and YAML (Entity)

Layers: [Entity]

**Files**: FC-01, FC-02, FC-05

**RED**: Write tests for `AuditWebProfile` construction, YAML parse/serialize round-trip, corrupted YAML error, unknown version error.

**Test Targets**:
- `profile_construction` — create profile with standard dims, thresholds, empty history (PC-001)
- `yaml_round_trip` — serialize to YAML string, parse back, assert equality (PC-002)
- `corrupted_yaml_error` — feed broken YAML, assert `ProfileError::MalformedYaml` with location (PC-003)
- `unknown_version_error` — feed `version: 99`, assert error message contains upgrade instructions (PC-004)

**GREEN**: Implement `AuditWebProfile`, `DimensionThreshold`, `ImprovementSuggestion` structs with `Serialize`/`Deserialize` derives. Implement `parse_profile(yaml: &str)` and `serialize_profile(profile: &AuditWebProfile)`. Implement version checking in parse.

**REFACTOR**: Extract constants (e.g., `CURRENT_PROFILE_VERSION = 1`).

**Commit cadence**:
1. `test: add AuditWebProfile domain type tests (PC-001..PC-004)`
2. `feat: implement AuditWebProfile domain types with YAML serde`
3. `refactor: extract profile version constant`

---

### Phase 2: Domain — Dimension types and sanitization (Entity)

Layers: [Entity]

**Files**: FC-03

**RED**: Write tests for query template validation — safe chars, shell metachar rejection, standard dimension definitions.

**Test Targets**:
- `valid_query_template` — template `"rust {project} performance"` passes (PC-005)
- `rejects_shell_metacharacters` — templates with `;`, `|`, `$`, `` ` ``, `<`, `>`, `&`, `!`, `(`, `)`, `'`, `"` rejected (PC-006)
- `allows_safe_chars` — alphanumeric, spaces, `-`, `_`, `.`, `/`, `{placeholder}` all allowed (PC-007)
- `standard_dimensions` — 8 standard dimensions created, all have valid templates (PC-008)

**GREEN**: Implement `AuditDimension` struct, `validate_query_template` using regex `^[a-zA-Z0-9 \-_./{}]+$`, and `standard_dimensions()` returning the 8 default dimensions.

**REFACTOR**: Consider extracting allowed-character regex to a constant.

**Commit cadence**:
1. `test: add AuditDimension and query template sanitization tests (PC-005..PC-008)`
2. `feat: implement AuditDimension with query template sanitization`
3. `refactor: extract query template regex constant`

---

### Phase 3: Domain — Report validation (Entity)

Layers: [Entity]

**Files**: FC-04

**RED**: Write tests for report structure validation — valid report passes, missing sections, score range, citation warning.

**Test Targets**:
- `valid_report_passes` — report with all 5 required sections, valid scores, 3+ citations per finding (PC-009)
- `missing_sections_error` — report missing "Feature Opportunities", assert error lists it (PC-010)
- `score_out_of_range` — score of 6 or -1, assert `ScoreOutOfRange` error (PC-011)
- `low_citation_warning` — finding with 2 citations, assert warning (non-blocking) (PC-012)

**GREEN**: Implement `validate_report(content: &str) -> ReportValidationResult` that parses markdown sections, extracts scores, counts citations. `ReportValidationResult` contains `errors: Vec<ReportError>` and `warnings: Vec<ReportWarning>`.

**REFACTOR**: Extract required section names to a constant array.

**Commit cadence**:
1. `test: add report validation domain tests (PC-009..PC-012)`
2. `feat: implement deterministic report validation`
3. `refactor: extract required section names constant`

---

### Phase 4: App — Profile use cases (UseCase)

Layers: [UseCase]

**Files**: FC-06, FC-07

**RED**: Write tests for app-layer use cases using `InMemoryFileSystem` — init creates profile, init rejects existing, show reads, validate passes, reset deletes, report validate passes.

**Test Targets**:
- `init_creates_profile` — write Cargo.toml to in-memory FS, call `init`, assert profile file written at `docs/audits/audit-web-profile.yaml` (PC-013)
- `init_rejects_existing` — profile already exists, call `init`, assert error (PC-014)
- `show_reads_profile` — create profile file, call `show`, assert returns content (PC-015)
- `validate_valid_profile` — create valid profile, call `validate`, assert Ok (PC-016)
- `reset_deletes_profile` — create profile file, call `reset`, assert file gone (PC-017)
- `validate_report_passes` — create valid report, call `report::validate`, assert Ok (PC-018)

**GREEN**: Implement use case functions following `ecc_app::sources` pattern:
- `pub fn init(fs: &dyn FileSystem, project_dir: &Path) -> Result<(), AuditWebAppError>`
- `pub fn show(fs: &dyn FileSystem, profile_path: &Path) -> Result<String, AuditWebAppError>`
- `pub fn validate(fs: &dyn FileSystem, profile_path: &Path) -> Result<(), AuditWebAppError>`
- `pub fn reset(fs: &dyn FileSystem, profile_path: &Path) -> Result<(), AuditWebAppError>`
- `pub fn validate_report(fs: &dyn FileSystem, report_path: &Path) -> Result<(), AuditWebAppError>`

Define `AuditWebAppError` with `Domain(ProfileError)`, `Domain(ReportError)`, `Io(String)` variants.

**REFACTOR**: Boy Scout delta on nearby files.

**Commit cadence**:
1. `test: add audit-web profile and report use case tests (PC-013..PC-018)`
2. `feat: implement audit-web app use cases`
3. `refactor: improve audit-web app error messages`

---

### Phase 5: CLI — Subcommand wiring (Adapter)

Layers: [Adapter]

**Files**: FC-08, FC-09, FC-10

**RED**: Write tests for CLI routing — `profile init` routes to app, `validate-report` routes to app.

**Test Targets**:
- `profile_init_routes` — create temp dir with Cargo.toml, invoke `run(AuditWebArgs { action: Profile { action: Init }, .. })`, assert profile file created (PC-019)
- `validate_report_routes` — create temp file with valid report, invoke `run(AuditWebArgs { action: ValidateReport { path } })`, assert Ok (PC-020)

**GREEN**: Implement clap structs and `run()` function following `commands::sources` pattern:

```rust
#[derive(Args)]
pub struct AuditWebArgs {
    #[command(subcommand)]
    pub action: AuditWebAction,
}

#[derive(Subcommand)]
pub enum AuditWebAction {
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    ValidateReport {
        path: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum ProfileAction {
    Init,
    Show,
    Validate,
    Reset {
        #[arg(long)]
        force: bool,
    },
}
```

Wire into `main.rs` `Command` enum and match arm.

**REFACTOR**: Boy Scout delta on CLI module.

**Commit cadence**:
1. `test: add audit-web CLI routing tests (PC-019..PC-020)`
2. `feat: wire audit-web CLI subcommands`
3. `refactor: improve audit-web CLI output formatting`

---

### Phase 6: Command markdown — Phase 0 + Phase 5 (Framework)

Layers: [Framework]

**Files**: FC-11

**Action**: Modify `commands/audit-web.md` to:
1. Add Phase 0 (GUIDED SETUP) before current Phase 1
2. Add Phase 5 (SELF-IMPROVEMENT) after current Phase 4
3. Renumber all phases 0-5
4. Update TodoWrite items to include Phase 0 and Phase 5
5. Phase 0 loads profile or triggers interactive generation
6. Phase 5 analyzes findings for coverage gaps and suggests new dimensions

No Rust tests for this phase (markdown-only change). Validated by human review.

**Commit cadence**:
1. `feat: add Phase 0 (guided setup) and Phase 5 (self-improvement) to audit-web command`

---

### Phase 7: Final gates

**Commit cadence**: No commit (gate only).

| Gate | Command |
|------|---------|
| PC-021 | `cargo clippy -p ecc-domain -- -D warnings` |
| PC-022 | `cargo clippy -p ecc-app -- -D warnings` |
| PC-023 | `cargo clippy -p ecc-cli -- -D warnings` |
| PC-024 | `cargo build` |
| PC-025 | `cargo test` |

## E2E Assessment

- **Touches user-facing flows?** Yes -- new `ecc audit-web` CLI subcommands
- **Crosses 3+ modules end-to-end?** Yes -- domain, app, cli
- **New E2E tests needed?** No -- the CLI routing tests in Phase 5 serve as integration smoke tests using real filesystem via temp dirs. The command markdown (Phase 6) is tested by human usage. Existing `cargo test` suite covers the full stack.

## Risks & Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| YAML serde for profile types may conflict with domain zero-I/O rule | Medium | `serde_yml` is already used in `ecc-domain` for backlog entry parsing; it is a pure data transformation, not I/O |
| `ecc audit-web` subcommand collides with existing `ecc audit` command in clap | Medium | They are distinct: `Audit` vs `AuditWeb`. Clap handles this as two separate enum variants. Verify with `cargo test -p ecc-cli` |
| Query template regex too restrictive for legitimate templates | Low | Start conservative (alphanumeric + safe chars + `{placeholder}`); can relax later via feedback |
| Report validation breaks on future report format changes | Low | Required section names stored as constants; updating is a single-point change |

## Success Criteria

- [ ] `AuditWebProfile` round-trips through YAML without data loss
- [ ] Query templates with shell metacharacters are rejected
- [ ] Standard 8 dimensions are defined and included by default
- [ ] Report validation catches missing sections, bad scores, low citations
- [ ] `ecc audit-web profile init|show|validate|reset` all function correctly
- [ ] `ecc audit-web validate-report <path>` validates report structure
- [ ] Phase 0 and Phase 5 added to `commands/audit-web.md`
- [ ] `cargo clippy -- -D warnings` passes for all affected crates
- [ ] `cargo test` passes with no regressions
