# Spec: Deterministic convention linting (BL-069)

## Problem Statement

Convention enforcement in ECC relies on LLM-based agents (convention-auditor) for checks that are mechanical and deterministic. `ecc validate` partially covers frontmatter presence and model values, but lacks naming consistency (filename vs frontmatter name), tool name validation against a known registry, placement checks (orphaned dirs, misplaced files), and cross-file reference validation. This leads to convention drift that agents detect inconsistently, wasting tokens on checks that should be deterministic and run in milliseconds.

## Research Summary

- CodeRabbit: "40+ built-in linters for concrete violations; LLM for semantic review" — validates the split between deterministic and LLM checks
- `VALID_MODELS` already exists in `ecc-domain::config::validate` as domain constants — reuse this pattern for `VALID_TOOLS`
- Existing `extract_frontmatter()` in ecc-domain handles YAML frontmatter extraction — reuse directly
- `Validatable<E>` trait pattern already established for `AgentFrontmatter` — extend for convention checks
- Convention-auditor agent can be simplified once mechanical checks are deterministic

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | New subcommand `ecc validate conventions` | Clean separation from existing per-target validators | No |
| 2 | ERROR severity for any invalid entry in list fields | Strict enforcement — mixed valid/invalid fails the whole file | No |
| 3 | Fix existing violations in same PR | Ensures meta-test passes immediately | No |
| 4 | Include cross-file reference validation (tool names) | User extended scope to validate tool names reference real Claude Code tools | No |
| 5 | Add VALID_TOOLS constant to ecc-domain | Reuses VALID_MODELS pattern. Values: `Read`, `Write`, `Edit`, `MultiEdit`, `Bash`, `Glob`, `Grep`, `Agent`, `Task`, `WebSearch`, `TodoWrite`, `TodoRead`, `AskUserQuestion` | No |
| 6 | Meta-test validates ECC repo itself | Catches real violations, serves as regression guard | No |
| 7 | No overlap with existing validators | `validate conventions` only adds NEW checks (naming, tool values). Model validation stays in `validate agents`. Placement checks (AC-003.x) are NEW — `validate skills` checks SKILL.md frontmatter fields, not orphaned directories. | No |
| 8 | Tools/allowed-tools parsed as bracket-delimited list | `extract_frontmatter` returns `HashMap<String, String>` — values like `["Read", "Write"]` parsed by stripping `[]`, splitting on `,`, trimming quotes/whitespace. Empty list `[]` is valid (no error). Bare string treated as single-element list. Same parsing for both `tools` and `allowed-tools`. | No |
| 9 | Kebab-case defined as `^[a-z][a-z0-9]*(-[a-z0-9]+)*$` | Standard kebab-case regex — lowercase alphanumeric segments separated by hyphens | No |
| 10 | Exit code: 0 if no ERRORs (WARNs allowed), 1 if any ERROR | WARNs are informational, ERRORs are blocking | No |

## User Stories

### US-001: Naming consistency checks

**As a** developer adding ECC components, **I want** `ecc validate conventions` to verify filenames match frontmatter names and follow kebab-case, **so that** naming drift is caught before it reaches LLM prompts.

#### Acceptance Criteria

- AC-001.1: Given an agent file where filename (sans .md) differs from frontmatter `name`, when conventions are validated, then an ERROR is reported with both names
- AC-001.2: Given an agent file with underscores or camelCase in the filename (not matching `^[a-z][a-z0-9]*(-[a-z0-9]+)*$`), when conventions are validated, then an ERROR is reported suggesting kebab-case
- AC-001.3: Given a skill directory whose name differs from SKILL.md frontmatter `name`, when conventions are validated, then an ERROR is reported
- AC-001.4: Given all files follow conventions, when conventions are validated, then output shows OK count with total files checked

#### Dependencies

- Depends on: none

### US-002: Value validation checks

**As a** developer, **I want** `ecc validate conventions` to verify frontmatter field values against known registries, **so that** typos and invalid values are caught deterministically.

#### Acceptance Criteria

- AC-002.1: Given an agent with `tools` field as a bare string (not bracket-delimited list), when validated, then it is parsed as a single-element list
- AC-002.2: Given an agent with a `tools` entry not in VALID_TOOLS, when validated, then ERROR for the whole file listing the invalid tool
- AC-002.3: Given a command with `allowed-tools` entries not in VALID_TOOLS, when validated, then ERROR reported
- AC-002.4: Given an agent with missing `name` field, when naming check runs, then WARN emitted and naming check skipped for that file
- AC-002.5: Given an agent with empty tools list `tools: []`, when validated, then no error (empty list is valid)
- AC-002.6: Given the command exits, when there are only WARNs (no ERRORs), then exit code is 0; when any ERROR exists, exit code is 1

#### Dependencies

- Depends on: none

### US-003: Placement checks

**As a** developer, **I want** `ecc validate conventions` to detect orphaned directories and misplaced files, **so that** the project structure stays clean.

#### Acceptance Criteria

- AC-003.1: Given a subdirectory under `skills/` that contains no `.md` files at all, when validated, then WARN for empty directory (distinct from `validate skills` which checks SKILL.md frontmatter)
- AC-003.2: Given all skill directories contain at least one `.md` file, when validated, then no placement warnings

#### Dependencies

- Depends on: none

### US-004: Meta-test and existing violation fixes

**As a** developer, **I want** the ECC repo itself to pass convention validation, **so that** the linter is proven correct and no existing violations remain.

#### Acceptance Criteria

- AC-004.1: Given the ECC repo, when `ecc validate conventions` runs in integration test, then exit code is 0
- AC-004.2: Given any existing violations found during implementation, when the PR ships, then they are fixed in the same PR

#### Dependencies

- Depends on: US-001, US-002, US-003

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `crates/ecc-domain/src/config/validate.rs` | Domain | Add VALID_TOOLS constant, convention check pure functions (naming, kebab-case, value, placement) |
| `crates/ecc-app/src/validate/conventions.rs` | App (new) | Orchestrates convention checks via FileSystem port |
| `crates/ecc-app/src/validate/mod.rs` | App | Add Conventions variant to ValidateTarget dispatch |
| `crates/ecc-cli/src/commands/validate.rs` | CLI | Add `conventions` subcommand to ValidateArgs enum |
| `crates/ecc-integration-tests/` | Test | Convention validation integration tests + meta-test |

## Constraints

- Must run in < 200ms for full scan (~100 markdown files)
- Must follow hexagonal pattern: pure domain logic in ecc-domain, orchestration in ecc-app via ports
- Must reuse existing `extract_frontmatter()` and `VALID_MODELS` from ecc-domain
- ERROR severity for any invalid entry in list fields (strict mode, whole-file error)
- Fix all existing violations found — meta-test must pass

## Non-Requirements

- Semantic validation ("is this description good enough?")
- Hook command syntax validation (shell/Python parsing)
- AC/PC format enforcement (handled by `ecc validate spec/design`)
- Automated fix suggestions (report only, human fixes)
- Validating rules/ file content structure

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| CLI validate subcommand | New subcommand added | CLI output format tested via integration tests |
| FileSystem port | Read-only usage (existing) | No new adapter needed |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New CLI command | CLAUDE.md | CLAUDE.md | Add `ecc validate conventions` to CLI Commands |
| Changelog | Project | CHANGELOG.md | Add entry under ### Added |

## Open Questions

None — all resolved during grill-me interview.
