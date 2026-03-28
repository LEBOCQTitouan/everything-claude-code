# Spec: Deterministic Spec/Design Artifact Validation

## Problem Statement

The spec-adversary and solution-adversary agents spend significant LLM tokens on mechanical structural validation: checking AC/PC numbering sequences, verifying table column counts, mapping AC-to-PC coverage, and validating dependency ordering. These are deterministic pattern-matching and graph operations perfectly suited for compiled code. Current LLM accuracy on these structural checks is ~85%, with false positives/negatives wasting adversary rounds and token budget. Adding two new `ecc validate` subcommands (`spec` and `design`) will replace this with binary pass/fail validation in <100ms.

## Research Summary

- Rust markdown parsing: pulldown-cmark (event-based) and comrak (full AST + GFM) are the dominant crates. For our structured AC/PC patterns, regex extraction is simpler and faster than full AST traversal.
- petgraph (2M+ downloads) provides `toposort` with cycle detection for the file-overlap dependency graph.
- BDD/Gherkin ecosystem is the closest prior art for spec-to-test coverage mapping, but no Rust-native spec validation tool exists.
- Deterministic acceptance criteria validation requires explicit, independently testable criteria with clear pass/fail outcomes.
- For hexagonal architecture: domain holds pure parsing/validation logic (str → value objects), app orchestrates I/O, CLI handles argument parsing.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | `ecc validate design` (not `solution`) | Matches filename (design.md) and /design command name | No |
| 2 | Regex over full markdown AST | AC/PC patterns are line-based with stable format; regex is simpler, faster, no new dependency needed | Yes |
| 3 | Separate `ecc-domain/src/spec/` module | Spec/design artifacts are a distinct bounded context from ECC config validation; mixing would violate CCP | Yes |
| 4 | Always JSON output | Designed for agent consumption; follows ecc-workflow structured output pattern | No |
| 5 | Phantom AC warnings | Flag PCs referencing ACs not in spec — catches copy-paste errors | No |
| 6 | Both US + sub-number sequential validation | Validate US numbers (001, 002) and sub-numbers within each US (AC-001.1, AC-001.2) for gaps/duplicates | No |

## User Stories

### US-001: Spec AC Extraction and Validation

**As a** spec-adversary agent or developer, **I want** `ecc validate spec <path>` to parse a spec file, extract all AC-NNN.N lines, and validate sequential numbering, **so that** structural validation is deterministic and token-free.

#### Acceptance Criteria

- AC-001.1: Given a spec file with ACs AC-001.1 through AC-003.4 with no gaps, when `ecc validate spec spec.md` runs, then exit code 0 and JSON output includes `valid: true` with all ACs listed
- AC-001.2: Given a spec file with a gap (AC-002.1, AC-002.3 missing AC-002.2), when the command runs, then exit code 1 and JSON `errors` array includes the gap description
- AC-001.3: Given a spec file with duplicate AC IDs, when the command runs, then exit code 1 and JSON reports the duplicate
- AC-001.4: Given a spec file with no AC- lines, when the command runs, then exit code 1 and JSON reports "no acceptance criteria found"
- AC-001.5: Given a valid spec, when the command runs, then JSON includes `ac_count` and `acs` array with `{id, description}` entries
- AC-001.6: Given a path that does not exist, when the command runs, then exit code 1 with error message
- AC-001.7: Given AC lines inside markdown code blocks, when the command parses, then those lines are ignored (not treated as real ACs)
- AC-001.8: Given US numbers with gaps (US-001, US-003 missing US-002), when the command runs, then exit code 1 and JSON reports the US gap
- AC-001.9: Given malformed AC IDs (e.g., "AC-ABC.1", "AC-001.0", "AC-000.1"), when the command parses, then they are ignored and a warning is added to the `warnings` array
- AC-001.10: Given a file that is not valid UTF-8, when the command runs, then exit code 1 with error message "file is not valid UTF-8"
- AC-001.11: Given AC references in prose (e.g., "See AC-001.1 for details"), when the command parses, then only lines starting with `- AC-NNN.N:` are treated as AC definitions

#### Dependencies

- None

### US-002: Design PC Table Structural Validation

**As a** solution-adversary agent, **I want** `ecc validate design <path>` to parse the Pass Conditions table and validate its structure, **so that** PC tables are structurally correct without LLM analysis.

#### Acceptance Criteria

- AC-002.1: Given a design file with a valid PC table (6 columns: ID, Type, Description, Verifies AC, Command, Expected), when `ecc validate design design.md` runs, then exit code 0 and JSON includes `pc_count` and `valid: true`
- AC-002.2: Given a PC table row with fewer than 6 columns, when the command runs, then exit code 1 and JSON reports the malformed row with row number
- AC-002.3: Given PC IDs not sequential (PC-001, PC-003 missing PC-002), when the command runs, then JSON reports the gap
- AC-002.4: Given duplicate PC IDs, when the command runs, then JSON reports the duplicates
- AC-002.5: Given PC rows with empty required fields (ID, Type, Description), when the command runs, then JSON reports which fields are missing
- AC-002.6: Given a design file with no Pass Conditions table, when the command runs, then exit code 1 with "no pass conditions table found"
- AC-002.7: Given table separator rows (---|---|...), when parsing, then they are correctly skipped

#### Dependencies

- None

### US-003: AC-to-PC Coverage Cross-Reference

**As a** solution-adversary agent, **I want** `ecc validate design <path> --spec <spec-path>` to cross-reference ACs against PCs and report gaps, **so that** coverage is verified deterministically.

#### Acceptance Criteria

- AC-003.1: Given all spec ACs appear in at least one PC's "Verifies AC" column, when the command runs with --spec, then `uncovered_acs` is empty
- AC-003.2: Given AC-003.2 appears in no PC, when the command runs with --spec, then `uncovered_acs` includes "AC-003.2"
- AC-003.3: Given --spec is NOT provided, when the command runs, then coverage check is skipped and `uncovered_acs` is null
- AC-003.4: Given --spec points to a nonexistent file, when the command runs, then exit code 1 with error message
- AC-003.5: Given a PC references AC-004.1 but spec only has ACs through AC-003.4, when the command runs, then `phantom_acs` includes "AC-004.1" as a warning (does NOT affect `valid` field or exit code — warnings are informational only)
- AC-003.6: Given multiple PCs cover the same AC, when the command runs, then no warning (valid)

#### Dependencies

- Depends on US-001, US-002

### US-004: PC Ordering Validation via File Overlap

**As a** solution-adversary agent, **I want** the design validator to flag PCs that modify the same file in wrong dependency order, **so that** TDD ordering is correct.

#### Acceptance Criteria

- AC-004.1: Given PCs modifying the same file are in ascending PC-ID order, when the command runs, then `ordering_violations` is empty
- AC-004.2: Given PC-005 and PC-002 both modify src/lib.rs and PC-005 appears after PC-002 in the table, when the command runs, then no violation (correct order)
- AC-004.3: Given a design with no File Changes table, when the command runs, then ordering check is skipped with a warning in JSON
- AC-004.4: Given an ordering violation, when the command runs, then `ordering_violations` includes `{pc, depends_on, reason}` with the file name in reason

#### Dependencies

- Depends on US-002

### US-005: JSON Output Format and Exit Codes

**As a** CI pipeline operator or agent, **I want** consistent JSON output and exit codes, **so that** downstream tools can programmatically consume results.

#### Acceptance Criteria

- AC-005.1: Given a successful validation, when the command completes, then exit code is 0 and stdout is valid JSON
- AC-005.2: Given validation errors, when the command completes, then exit code is 1 and stdout is valid JSON containing the errors
- AC-005.3: Given any validation, then human-readable warnings go to stderr, structured results go to stdout
- AC-005.4: Given `ecc validate spec`, then JSON schema includes: `{valid, ac_count, acs, errors}`
- AC-005.5: Given `ecc validate design`, then JSON schema includes: `{valid, pc_count, pcs, uncovered_acs, phantom_acs, ordering_violations, errors}`

#### Dependencies

- Depends on US-001, US-002

### US-006: Domain Types and Module Structure

**As a** maintainer, **I want** AcId, PcId, CoverageReport, and OrderingViolation as domain value objects in a new `spec/` module, **so that** the domain layer remains pure and well-organized.

#### Acceptance Criteria

- AC-006.1: Given `crates/ecc-domain/src/spec/mod.rs` exists, when domain tests run, then all AC parsing and validation logic has zero I/O imports
- AC-006.2: Given AcId value object, when constructed with valid format "AC-001.2", then it stores us_number=1 and sub_number=2
- AC-006.3: Given AcId value object, when constructed with invalid format "AC-1.2", then it returns an error
- AC-006.4: Given PcId value object, when constructed with "PC-003", then it stores number=3
- AC-006.5: Given CoverageReport, when all ACs covered, then `uncovered_acs` is empty and `phantom_acs` is empty
- AC-006.6: Given OrderingViolation, then it contains `pc: PcId`, `depends_on: PcId`, and `reason: String`

#### Dependencies

- None (can be done in parallel with US-001)

### US-007: Documentation and Registration

**As a** developer, **I want** the new subcommands documented in CLAUDE.md and the glossary, **so that** they are discoverable.

#### Acceptance Criteria

- AC-007.1: Given the subcommands exist, when I read CLAUDE.md, then `ecc validate spec` and `ecc validate design` appear in the CLI Commands section
- AC-007.2: Given the domain types exist, when I read the glossary, then AcId, PcId, CoverageReport, OrderingViolation have entries
- AC-007.3: Given BL-067 in the backlog, when the feature is implemented, then BL-067 is updated to `status: promoted`

#### Dependencies

- Depends on US-001

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `crates/ecc-domain/src/spec/` (new) | Domain | AC parsing, PC parsing, coverage mapping, ordering validation — pure functions, zero I/O |
| `crates/ecc-domain/src/lib.rs` | Domain | Add `pub mod spec;` |
| `crates/ecc-app/src/validate.rs` | App | Add Spec/Design variants to ValidateTarget, orchestrate file reads + domain calls |
| `crates/ecc-cli/src/commands/validate.rs` | CLI | Add Spec/Design clap subcommands with path args, map to app-level targets |
| `crates/ecc-integration-tests/` | Test | E2E scenarios for spec/design validation |
| `docs/adr/` | Documentation | ADR-0022 (regex over AST), ADR-0023 (separate spec module) |

## Constraints

- Domain module (`ecc-domain/src/spec/`) must have zero I/O imports — pure functions only
- Performance target: <100ms for a 200-AC spec file
- Output always JSON — no human-readable mode for these subcommands
- Must integrate with existing `ValidateTarget` enum dispatch pattern
- `ecc-app/src/validate.rs` may need refactoring into a module directory if it exceeds 800 lines

## Non-Requirements

- Wave ordering validation (complex, follow-up)
- Doc Update Plan table validation (secondary)
- Coverage Check table validation (redundant with AC-to-PC coverage)
- Adversary agent integration (separate follow-up BL)
- File Changes table independent structural validation
- Full markdown AST parsing (regex sufficient)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| FileSystem port | Existing | Used to read spec/design files — no new port methods |
| TerminalIO port | Existing | Used to write JSON output — no new port methods |
| CLI (clap) | Extended | New subcommands Spec and Design with path arguments |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New CLI commands | docs | CLAUDE.md | Add `ecc validate spec` and `ecc validate design` to CLI Commands |
| New domain terms | docs | docs/domain/glossary | Add AcId, PcId, CoverageReport, OrderingViolation |
| New ADR | docs | docs/adr/0022-regex-over-ast.md | Create |
| New ADR | docs | docs/adr/0023-separate-spec-module.md | Create |
| Backlog update | docs | docs/backlog/BL-067 | Status → promoted |
| CHANGELOG | docs | CHANGELOG.md | Add entry |

## Rollback Plan

This is an additive feature (new subcommands, new domain module). Rollback strategy:
1. Revert the merge commit — removes all code changes
2. ADRs 0022 and 0023 should be marked "Superseded" if rolled back
3. The regex patterns for AC/PC parsing are hardcoded to the current spec format; if the format evolves, the regex patterns in `ecc-domain/src/spec/` need updating (no runtime configuration — YAGNI)
4. No existing behavior is modified — rollback has zero impact on current `ecc validate` subcommands
5. Error accumulation: validation collects ALL errors before reporting (does not stop at first error)

## File Changes Table Format

For US-004 ordering validation, the expected File Changes table format is:
```
| # | File | Action | Rationale | Spec Ref |
```
5 columns, detected by header row containing "File" and "Action". PCs are linked to files via the "Spec Ref" column (matching US/AC references to PCs' "Verifies AC" column). If the table has a different structure, ordering validation is skipped with a warning.

## Open Questions

None — all resolved during grill-me.
