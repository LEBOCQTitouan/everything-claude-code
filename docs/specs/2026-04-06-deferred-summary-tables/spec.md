# Spec: Deferred Pipeline Summary Tables

## Problem Statement

Three pipeline summary tables were deferred from BL-048 to avoid violating the "instruction additions only" constraint. They require logic changes: coverage tooling integration, bounded context enumeration analysis, and subagent return schema changes. Without them, /implement lacks coverage visibility, /design doesn't enumerate bounded contexts, and TDD cycle results don't include individual test names.

## Research Summary

- `cargo llvm-cov` is already listed in CLAUDE.md as the project coverage tool
- tdd-executor output schema currently has `red_result` and `green_result` as description strings -- no structured test names
- /design's Plan Mode preview mentions bounded context changes but doesn't enumerate them programmatically

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | cargo llvm-cov for coverage delta | Already in CLAUDE.md as the project tool | No |
| 2 | Graceful skip if coverage tool missing | Coverage is informational, not a gate | No |
| 3 | Coverage runs once after TDD loop | 30-60s runtime; single run is sufficient | No |
| 4 | Per-test-name is additive with migration guidance | Existing implement-done.md files need notes | No |

## User Stories

### US-001: Coverage Delta Table

**As a** developer running /implement, **I want** a before/after coverage delta table in the Phase Summary, **so that** I can see how the implementation affected test coverage.

#### Acceptance Criteria

- AC-001.1: Given /implement completes the TDD loop, when coverage is measured, then it runs `cargo llvm-cov --workspace --json` at the before-snapshot and current HEAD (after). The before-snapshot is: the `wave-1-start` git tag if wave dispatch was used, otherwise the commit SHA recorded at TDD loop entry (Phase 3 start). If neither exists, skip coverage with "No before-snapshot available"
- AC-001.2: Given coverage data is available, when the Phase Summary is rendered, then it includes a Coverage Delta table with columns: Crate, Before %, After %, Delta
- AC-001.3: Given cargo llvm-cov is not installed or fails (including partial failure -- before succeeds but after fails), when coverage is attempted, then the table shows "Coverage data unavailable -- install cargo-llvm-cov" and the pipeline continues. Partial data is discarded.
- AC-001.4: Given implement-done.md is written, when coverage data exists, then a `## Coverage Delta` section is included with the before/after table

#### Dependencies
- Depends on: none

### US-002: Bounded Context Enumeration

**As a** developer running /design, **I want** a bounded contexts table in the design output, **so that** I can see which DDD contexts are affected by the design.

#### Acceptance Criteria

- AC-002.1: Given /design Phase 1 produces file changes, when the design output is generated, then it includes a `## Bounded Contexts Affected` table listing each affected context and its role
- AC-002.2: Given the file changes table references files in `crates/ecc-domain/src/`, when bounded contexts are enumerated, then only modules listed in `docs/domain/bounded-contexts.md` are reported. Utility files (ansi.rs, traits.rs, paths.rs, time.rs) and modules not in the bounded contexts registry are excluded. If a modified file's parent module is not in bounded-contexts.md, it is listed under "Other domain modules" separately
- AC-002.3: Given no domain files are modified, when bounded contexts are enumerated, then the table shows "No bounded contexts affected"

#### Dependencies
- Depends on: none

### US-003: Per-Test-Name Inventory

**As a** developer running /implement, **I want** the TDD Log to include individual test function names, **so that** I can see exactly which tests were written for each PC.

#### Acceptance Criteria

- AC-003.1: Given tdd-executor completes a RED-GREEN-REFACTOR cycle, when it returns structured results, then the output includes a `test_names` field containing a list of fully qualified test function names (e.g., `["metrics::event::tests::hook_execution_event", "metrics::aggregate::tests::aggregator_computes_rates"]`). Fully qualified means module path + test name to avoid collisions across modules
- AC-003.2: Given the parent orchestrator receives test_names, when implement-done.md is written, then the TDD Log table includes a Test Names column with the list
- AC-003.3: Given an older tdd-executor invocation that doesn't return test_names, when implement-done.md is written, then the Test Names column shows "--" (graceful degradation)
- AC-003.4: Given implement-done.md schema changes, when existing files are encountered, then a migration note in CLAUDE.md Gotchas documents: field name (`test_names`), type (list of strings), default when absent ("--"), backward compat behavior (column shows "--" for older implement-done.md files)
- AC-003.5: Given these are markdown behavioral changes, when verification is needed, then grep-based lint checks validate the required language is present in each file. Automated testing of rendered markdown output is out of scope (manual review).

#### Dependencies
- Depends on: none

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| `commands/implement.md` | Command | Add coverage delta measurement step + coverage table in Phase Summary + test names in TDD Log |
| `commands/design.md` | Command | Add bounded context enumeration in Phase 9 output |
| `agents/tdd-executor.md` | Agent | Add test_names to structured result output |

## Constraints

- Markdown-only changes for commands/design -- no Rust code
- tdd-executor.md schema change is additive (new field, not modified field)
- Coverage tool failure must never block the pipeline
- Existing implement-done.md files must degrade gracefully

## Non-Requirements

- Real-time coverage display during TDD loop
- Per-PC coverage delta (only before/after the full TDD loop)
- Automatic bounded context documentation generation
- Coverage thresholds or gates

## E2E Boundaries Affected

None -- markdown behavioral changes only.

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Schema change | CLAUDE.md | Gotchas | Add test_names field migration note |
| Glossary | docs/domain | glossary.md | Add coverage delta, bounded context enumeration, per-test-name inventory |
| Changelog | CHANGELOG.md | Add entry | all |

## Rollback Plan

1. Revert the 3 markdown file edits -- all changes are additive instruction text
2. No data migration, no config changes, no Rust code changes

## Open Questions

None -- all questions resolved during grill-me interview.
