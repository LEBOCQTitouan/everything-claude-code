# Spec: Audit Findings Remediation — All 24 Smells

## Problem Statement

The full codebase audit (2026-03-28) graded the project at D, driven by 3 CRITICAL cross-correlations, 2 CRITICAL documentation findings, and 35 HIGH findings across 9 domains. The core issues are: (1) observability is effectively absent — 48 warn! calls are silent and 38 error-discard sites emit nothing; (2) the highest-churn code (ecc-workflow) has zero unit tests and bypasses the hexagonal architecture; (3) two documentation files are entirely from the TypeScript era. Secondary issues include security injection, error type inconsistency, file size violations, convention drift, and stale documentation. This refactoring addresses all 24 cataloged smells to raise the health grade from D toward B.

## Research Summary

- **env_logger default level**: Use `env_logger::Builder::from_env(Env::default().default_filter_or("warn"))` — allows RUST_LOG override while making warnings visible by default
- **Functional Core, Imperative Shell**: Extract pure functions (typed inputs -> Result<Output, Error>) from I/O-heavy handlers. CLI layer stays as thin orchestrator calling ports then pure logic. Domain crate owns the pure core.
- **thiserror migration**: Start from leaf crates inward. Use `#[derive(thiserror::Error)]` with `#[from]` for upstream conversion. Keep `anyhow` only in binary crates.
- **Avoid `#[error(transparent)]` overuse** — prefer explicit error messages with context
- **Domain error enums must not contain `std::io::Error`** — map infra errors to domain variants at port boundaries
- **Pitfall: `#[from]` on same source type** — two variants with `#[from]` on same type fails; use manual `From` impl for one
- **Incremental migration**: Run `cargo test` + `cargo clippy` after each crate conversion before proceeding

## Decisions Made

| # | Decision | Rationale | ADR? |
|---|----------|-----------|------|
| 1 | ecc-workflow keeps direct I/O, extract pure logic only | Standalone binary — full port refactor is overkill. Arch-reviewer and component-auditor agree. | Yes |
| 2 | Add traits to ecc-domain to improve abstractness (D=0.99->lower) | User requested. Will introduce behavioral traits (`Validatable`, `Transitionable`) for core domain types. | Yes |
| 3 | Error strategy: thiserror enums per module in ecc-app, anyhow only in binaries | Resolves 3 competing error strategies. Cross-crate convention for all future code. | Yes |
| 4 | Notification sanitization at app layer, not infra | ShellExecutor is a generic port — context-specific escaping violates SRP. | No |
| 5 | Frontmatter parsing consolidated to serde_yaml | Eliminates 3 competing implementations. Benchmark before removing manual parser. | No |
| 6 | Split oversized files before error migration | Reduces merge conflict surface. Error changes touch many lines. | No |
| 7 | Use generics (not dyn) for domain traits | Zero-cost abstraction — no runtime overhead for performance-sensitive paths. | No |
| 8 | Use log + env_logger, not tracing + tracing-subscriber | CLI tool with simple needs, no async spans, minimal overhead. tracing is overkill for synchronous CLI. | No |
| 9 | Stdin truncation at byte boundary with no trailing indicator | Hooks sending >1MB are already misbehaving; simplicity over precision. Log a warning on truncation. | No |
| 10 | Benchmark threshold for frontmatter parser: serde_yaml retained if within 3x of manual parser on p95 | Objective threshold eliminates subjective escape hatch. Benchmark runs during US-006 implementation. | No |

## User Stories

### US-001: Observable CLI Operations

**As a** developer using ECC, **I want** warnings and errors to be visible during normal operation, **so that** I can diagnose configuration problems without setting RUST_LOG manually.

#### Acceptance Criteria

- AC-001.1: Given default ecc invocation, when a warn! is emitted, then it appears on stderr
- AC-001.2: Given `--verbose` flag, when ecc runs, then debug-level output appears
- AC-001.3: Given ecc-workflow binary, when it runs with RUST_LOG=debug, then diagnostic output appears
- AC-001.4: Given 38 error-discard sites in ecc-app, when an Err occurs, then log::warn! is emitted including: the function name, the error's Display string, and any relevant input identifier (file path, hook ID, etc.)
- AC-001.5: Given install/dev failure, when exit(1) is called, then a line matching `Error: <description>` appears on stderr before exit

#### Dependencies

- None

### US-002: Testable Workflow Commands

**As a** developer maintaining ecc-workflow, **I want** pure logic extracted from command handlers into unit-testable functions, **so that** I can validate business logic without spawning the binary.

#### Acceptance Criteria

- AC-002.1: Given memory_write.rs, when pure logic is extracted, then format builders (build_action_entry, build_work_item_content, build_daily_content, build_memory_index_content) are testable without filesystem
- AC-002.2: Given extracted pure functions, when unit tests are added, then each function has >= 3 test cases covering happy path, edge cases, and error conditions
- AC-002.3: Given io.rs read_stdin, when input exceeds 1MB, then reading stops at byte boundary and returns the truncated content
- AC-002.3a: Given stdin input of exactly 1MB, when read, then the full content is returned without truncation
- AC-002.3b: Given truncated stdin (>1MB), when returned, then log::warn! is emitted indicating truncation occurred with the original byte count
- AC-002.4: Given integration.rs (2750 lines), when split into per-command files, then each file is < 400 lines with shared helpers in tests/common/mod.rs
- AC-002.5: Given ecc-workflow, when log + env_logger dependencies are added, then RUST_LOG=debug produces diagnostic output

#### Dependencies

- None

### US-003: Secure Notification Hooks

**As a** user running ECC hooks, **I want** notification strings to be sanitized before shell execution, **so that** special characters in tool names or session IDs cannot inject commands.

#### Acceptance Criteria

- AC-003.1: Given a message containing `'` or `"`, when passed to osascript builder, then characters are escaped
- AC-003.2: Given a message containing `'`, when passed to PowerShell builder, then characters are escaped
- AC-003.3: Given adversarial input (e.g., `'; rm -rf /; echo '`), when used as notification title, then no command injection occurs
- AC-003.4: Given the sanitization functions, when unit tests run, then at least 5 adversarial inputs are tested per platform

#### Dependencies

- None

### US-004: Typed Error Handling in ecc-app

**As a** developer working on ecc-app, **I want** all public functions to return typed error enums instead of String or anyhow, **so that** callers can match on error variants and provide targeted user messages.

#### Acceptance Criteria

- AC-004.1: Given ecc-app/claw/, when ClawError enum is created, then all 5 Result<T, String> + 3 anyhow::Result functions use it
- AC-004.2: Given ecc-app/merge/, when MergeError enum is created, then all 4 Result<T, String> functions use it
- AC-004.3: Given ecc-app/config/, when ConfigAppError enum is created, then all 3 Result<T, String> functions use it
- AC-004.4: Given ecc-app/install/ + hook/helpers, when local error types are added, then remaining Result<T, String> functions use typed errors
- AC-004.5: Given anyhow dependency in ecc-app, when migration is complete, then anyhow is removed from ecc-app/Cargo.toml
- AC-004.6: Given ecc-cli command handlers, when they receive typed errors, then each error variant maps to a message containing the operation name and a one-sentence remediation hint (verified by unit tests)
- AC-004.7: Given each error enum migration applied to one module, when cargo check runs, then it passes before proceeding to the next module

#### Dependencies

- Depends on: US-007 (file splits reduce merge conflict surface for error migration)

### US-005: Accurate Documentation

**As a** contributor reading project documentation, **I want** docs to reflect the current Rust codebase, **so that** I build correct mental models when onboarding.

#### Acceptance Criteria

- AC-005.1: Given DEPENDENCY-GRAPH.md, when rewritten, then it shows the 9 Rust crate dependency graph with correct edges
- AC-005.2: Given glossary.md, when updated, then all 27 .ts file references are replaced with correct Rust file paths
- AC-005.3: Given ARCHITECTURE.md, when regenerated, then agent count = actual, command count = actual, skill count = actual
- AC-005.4: Given bounded-contexts.md, when updated, then backlog and workflow modules are documented
- AC-005.5: Given MODULE-SUMMARIES.md, when updated, then ecc-integration-tests and ecc-workflow are included
- AC-005.6: Given commands-reference.md, when updated, then spec pipeline commands (/spec, /spec-dev, /spec-fix, /spec-refactor, /design, /implement) are listed
- AC-005.7: Given module-dependency-graph diagram, when updated, then all 9 crate nodes are present
- AC-005.8: Given getting-started.md, when updated, then repo tree shows all 9 crates and skill count is correct
- AC-005.0: All documentation ACs (005.1-005.8) are verified after all code stories (US-001 through US-004, US-006 through US-008) are complete

#### Dependencies

- Depends on: US-004, US-006, US-007, US-008 (documentation must reflect final code state)

### US-006: Convention Consistency

**As a** developer maintaining ECC, **I want** convention violations fixed, **so that** the codebase follows its own rules consistently.

#### Acceptance Criteria

- AC-006.1: Given 3 frontmatter parsing implementations, when consolidated, then serde_yaml is the single parser (manual fallback retained only if serde_yaml is >3x slower on p95 parse time for the project's largest frontmatter file)
- AC-006.2: Given duplicate is_claude_available, when deduplicated, then one canonical function exists in claw/claude_runner.rs
- AC-006.3: Given Completion.phase as String, when changed to Phase enum, then existing state.json files deserialize correctly via serde
- AC-006.3a: Given state.json with an unrecognized phase string (e.g., manually edited), when deserialized, then a fallback "Unknown" variant is used and a warning is logged (not a crash)
- AC-006.3b: Given the Phase enum, when serialized to JSON, then the output is a plain lowercase string identical to the previous format (backward-compatible for older binaries)
- AC-006.4: Given WorkflowState concern/started_at as String, when typed (Concern enum, Timestamp newtype), then serialization is backward-compatible
- AC-006.5: Given port trait methods, when doc comments added, then every method on FileSystem, ShellExecutor, Environment, TerminalIO has /// docs

#### Dependencies

- Depends on: US-004, US-007 (error types may change during convention fixes; file splits must complete before frontmatter consolidation touches validate.rs)

### US-007: File Size Compliance

**As a** developer, **I want** all files under the 800-line limit, **so that** the codebase follows its own file organization rules.

#### Acceptance Criteria

- AC-007.1: Given validate.rs (1240 lines), when split into validate/{agents,commands,hooks,skills,rules,paths,statusline}.rs, then each file < 400 lines
- AC-007.2: Given dev.rs (1065 lines), when split into dev/{switch,status,toggle,format}.rs, then each file < 400 lines
- AC-007.3: Given merge/helpers.rs (923 lines), when test code is extracted, then production + test files are each < 500 lines
- AC-007.4: Given install/global.rs (863 lines), when split by install step, then each file < 400 lines
- AC-007.5: Given all splits, when cargo test runs, then all 1404+ tests pass identically

#### Dependencies

- Complete before US-004 (error migration touches these files)

### US-008: Domain Model Improvement

**As a** architect, **I want** ecc-domain to have behavioral traits reducing its Zone of Pain score, **so that** the most stable crate is also appropriately abstract.

#### Acceptance Criteria

- AC-008.1: Given ecc-domain (D=0.99), when Validatable trait is added, then config/hook/agent validation types implement it
- AC-008.2: Given ecc-domain, when Transitionable trait is added, then WorkflowState implements it with transition_to() method
- AC-008.3: Given ecc-domain, when traits are added using generics (not dyn), then zero runtime overhead is introduced
- AC-008.4: Given the new traits, when D is recomputed using D = |A + I - 1| where A = (public traits + trait methods) / (total public items) and I = Ce / (Ca + Ce) from Cargo.toml, then D < 0.80
- AC-008.5: Given corrupt aliases.json (smell #23), when loaded, then log::warn! is emitted (not silent reset)
- AC-008.6: Given dev_switch rollback (smell #24), when remove_file fails, then log::error! is emitted with the path

#### Dependencies

- US-001 (logging must work first for AC-008.5/6)

## Affected Modules

| Module | Layer | Change Nature |
|--------|-------|---------------|
| ecc-cli/src/main.rs | CLI | env_logger default, failure banners |
| ecc-workflow/src/commands/memory_write.rs | Binary | Extract pure logic |
| ecc-workflow/src/io.rs | Binary | Bound stdin |
| ecc-workflow/src/main.rs | Binary | Add logging |
| ecc-workflow/tests/ | Test | Split integration.rs |
| ecc-app/src/hook/handlers/tier2_notify.rs | App | Sanitize shell strings |
| ecc-app/src/claw/ | App | ClawError enum |
| ecc-app/src/merge/ | App | MergeError enum |
| ecc-app/src/config/ | App | ConfigAppError enum |
| ecc-app/src/install/ | App | InstallError enum, split global.rs |
| ecc-app/src/validate.rs | App | Split into submodules |
| ecc-app/src/dev.rs | App | Split into submodules |
| ecc-app/src/session/aliases.rs | App | Warn on corrupt JSON |
| ecc-domain/src/ | Domain | Add Validatable, Transitionable traits |
| ecc-domain/src/workflow/state.rs | Domain | Type Completion.phase, concern, started_at |
| ecc-domain/src/config/validate.rs | Domain | Consolidate frontmatter parsing |
| ecc-ports/src/ | Ports | Add doc comments to all trait methods |
| docs/ | Docs | Rewrite DEPENDENCY-GRAPH, glossary, ARCHITECTURE, etc. |

## Constraints

- All refactoring steps must be behavior-preserving (tests pass identically before and after)
- Build must pass after each phase (cargo test + cargo clippy -- -D warnings)
- Existing state.json files must deserialize correctly after type changes (backward compatibility)
- File splits are structural only — no behavioral changes during splits
- Benchmark frontmatter parsing before removing manual parser

## Non-Requirements

- No new feature development beyond what the 24 smells require
- No crate restructuring beyond file splits (no new crates, no crate merges)
- No dependency upgrades beyond thiserror addition to ecc-app
- No performance optimization beyond benchmark gating for frontmatter parser
- No public API changes to ecc-cli beyond error message improvements
- Domain traits are behavioral contracts only — no migration of existing validation logic into trait implementations in this work item

## Rollback Strategy

Each US is implemented on a feature branch and merged independently. If a US fails review, prior US merges are not reverted.

| Story | Rollback Complexity | Notes |
|-------|-------------------|-------|
| US-001 (observability) | Trivial | Additive only — revert env_logger default + remove warn! calls |
| US-002 (workflow testing) | Trivial | Pure logic extraction is additive; integration.rs split is structural |
| US-003 (security) | Trivial | Revert sanitization functions |
| US-004 (error types) | Medium | Each module migrated independently; partial migration is safe (cargo check gates each step) |
| US-005 (docs) | Trivial | Git revert on doc files |
| US-006 (conventions) | Medium | Serialization changes (Phase enum) use backward-compatible format; revert produces identical JSON |
| US-007 (file splits) | Low | Structural only; file splits are permanent improvements not reverted if downstream stories fail |
| US-008 (domain model) | Low | Trait additions are additive; implementations can be removed without breaking dependents |

**Phase isolation**: US-001, US-002, US-003 have zero dependencies and can execute in any order. US-007 must complete before US-004 and US-006. US-005 executes last. US-008 depends on US-001 for logging.

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|---|---|---|
| TerminalIO (stderr) | New warn output, failure banners | E2E tests asserting on stderr may need updating |
| ShellExecutor (osascript) | Escaped strings in notification commands | Notifications with special chars now display correctly |
| FileSystem | Doc comments only (additive) | No E2E impact |
| stdin (ecc-workflow) | 1MB cap on read | Hooks with >1MB payloads will be truncated |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|---|---|---|---|
| Rewrite | Major | docs/DEPENDENCY-GRAPH.md | Replace TypeScript content with Rust crate graph |
| Update | Major | docs/domain/glossary.md | Replace 27 .ts references with .rs paths |
| Regenerate | Major | docs/ARCHITECTURE.md | Update all counts (agents, commands, skills) |
| Add entries | Medium | docs/domain/bounded-contexts.md | Add backlog, workflow modules |
| Add entries | Medium | docs/MODULE-SUMMARIES.md | Add ecc-integration-tests, ecc-workflow |
| Add entries | Medium | docs/commands-reference.md | Add 6 spec pipeline commands |
| Update | Medium | docs/diagrams/module-dependency-graph.md | Add 2 missing crate nodes |
| Update | Medium | docs/getting-started.md | Fix repo tree, skill count |
| New | Minor | docs/adr/NNN-workflow-direct-io.md | ADR: ecc-workflow keeps direct I/O |
| New | Minor | docs/adr/NNN-domain-traits.md | ADR: Domain abstractness via traits |
| New | Minor | docs/adr/NNN-error-type-strategy.md | ADR: thiserror per module, anyhow in binaries only |
| Update | Minor | CLAUDE.md | Test count update after new tests added |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Smell triage — which 24 smells to address? | All 24 in scope | User |
| 2 | Target architecture — functional core or full port refactor for ecc-workflow? | Functional Core, Imperative Shell (extract pure logic, keep direct I/O) | Recommended |
| 3 | Step independence — grouping? | 7 groups: independent (1,4,6,3), ordered (5→8/9, 7→2), batch docs, batch conventions | Recommended |
| 4 | Downstream dependencies — ecc-domain abstractness? | Attempt improvement (add traits), not just document as trade-off | User |
| 5 | Rename vs behavioral — classification? | 5 pure structural, 5 additive, 4 behavioral, 4 type-level, 1 investigation | Recommended |
| 6 | Performance budget — hot paths? | Use generics not dyn for domain traits; benchmark frontmatter parser before switching | Recommended |
| 7 | ADR decisions — which warrant ADRs? | 3 ADRs: workflow direct I/O, domain traits, error type strategy | Recommended |
| 8 | Test safety net — sufficient coverage? | Yes for most areas; add unit tests during workflow extraction and adversarial tests for notification fix | Recommended |

**Smells addressed**: All 24 (silent logging, workflow untested, stale docs, osascript injection, error types, unbounded stdin, integration.rs monolith, validate.rs, dev.rs, ARCHITECTURE.md counts, bounded-contexts, workflow logging, frontmatter parsing, duplicate function, Completion.phase, domain abstractness, merge/helpers.rs, install/global.rs, MODULE-SUMMARIES, commands-reference, temporal coupling, port docs, corrupt aliases, rollback errors)

**Smells deferred**: None

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Observable CLI Operations | 5 | None |
| US-002 | Testable Workflow Commands | 7 | None |
| US-003 | Secure Notification Hooks | 4 | None |
| US-004 | Typed Error Handling in ecc-app | 7 | US-007 |
| US-005 | Accurate Documentation | 9 | US-004, US-006, US-007, US-008 |
| US-006 | Convention Consistency | 7 | US-004, US-007 |
| US-007 | File Size Compliance | 5 | None (but must complete before US-004) |
| US-008 | Domain Model Improvement | 6 | US-001 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | warn! appears on stderr by default | US-001 |
| AC-001.2 | --verbose produces debug output | US-001 |
| AC-001.3 | ecc-workflow supports RUST_LOG | US-001 |
| AC-001.4 | 38 error-discard sites emit warn! with function name + Display + input ID | US-001 |
| AC-001.5 | Failure banner `Error: <desc>` on stderr before exit(1) | US-001 |
| AC-002.1 | memory_write.rs pure logic extracted | US-002 |
| AC-002.2 | >= 3 unit tests per extracted function | US-002 |
| AC-002.3 | stdin bounded at 1MB (byte boundary) | US-002 |
| AC-002.3a | Exactly 1MB: no truncation | US-002 |
| AC-002.3b | Truncation logged with byte count | US-002 |
| AC-002.4 | integration.rs split, each < 400 lines | US-002 |
| AC-002.5 | ecc-workflow gets log + env_logger | US-002 |
| AC-003.1 | osascript escapes ' and " | US-003 |
| AC-003.2 | PowerShell escapes ' | US-003 |
| AC-003.3 | Adversarial injection blocked | US-003 |
| AC-003.4 | >= 5 adversarial inputs tested per platform | US-003 |
| AC-004.1 | ClawError enum for claw/ (8 functions) | US-004 |
| AC-004.2 | MergeError enum for merge/ (4 functions) | US-004 |
| AC-004.3 | ConfigAppError enum for config/ (3 functions) | US-004 |
| AC-004.4 | Typed errors for install/ + hook/helpers | US-004 |
| AC-004.5 | anyhow removed from ecc-app/Cargo.toml | US-004 |
| AC-004.6 | Error messages include operation name + remediation hint | US-004 |
| AC-004.7 | cargo check passes after each module migration | US-004 |
| AC-005.0 | Docs verified after all code stories complete | US-005 |
| AC-005.1 | DEPENDENCY-GRAPH.md rewritten for 9 Rust crates | US-005 |
| AC-005.2 | glossary.md: 27 .ts refs replaced with .rs | US-005 |
| AC-005.3 | ARCHITECTURE.md counts match actual | US-005 |
| AC-005.4 | bounded-contexts.md adds backlog + workflow | US-005 |
| AC-005.5 | MODULE-SUMMARIES.md adds 2 missing crates | US-005 |
| AC-005.6 | commands-reference.md adds 6 pipeline commands | US-005 |
| AC-005.7 | Module dependency diagram shows 9 nodes | US-005 |
| AC-005.8 | getting-started.md updated | US-005 |
| AC-006.1 | Frontmatter parsing consolidated (3x p95 threshold) | US-006 |
| AC-006.2 | Duplicate is_claude_available deduplicated | US-006 |
| AC-006.3 | Completion.phase String -> Phase enum | US-006 |
| AC-006.3a | Unknown phase string -> fallback variant + warning | US-006 |
| AC-006.3b | Phase serializes as plain lowercase string (backward compat) | US-006 |
| AC-006.4 | WorkflowState concern/started_at typed | US-006 |
| AC-006.5 | Port trait methods get /// doc comments | US-006 |
| AC-007.1 | validate.rs split into submodules (< 400 lines each) | US-007 |
| AC-007.2 | dev.rs split into submodules (< 400 lines each) | US-007 |
| AC-007.3 | merge/helpers.rs tests extracted (< 500 lines each) | US-007 |
| AC-007.4 | install/global.rs split (< 400 lines each) | US-007 |
| AC-007.5 | All 1404+ tests pass after splits | US-007 |
| AC-008.1 | Validatable trait added, types implement it | US-008 |
| AC-008.2 | Transitionable trait added, WorkflowState implements it | US-008 |
| AC-008.3 | Traits use generics (no dyn dispatch) | US-008 |
| AC-008.4 | D < 0.80 (formula: \|A + I - 1\|) | US-008 |
| AC-008.5 | Corrupt aliases.json emits log::warn! | US-008 |
| AC-008.6 | dev_switch rollback emits log::error! | US-008 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Ambiguity | 78 | PASS | ACs tightened: context defined, failure banner formatted, benchmark threshold set |
| Edge Cases | 72 | PASS | Unknown phase fallback, stdin boundary, truncation warning added |
| Scope | 80 | PASS | Non-Requirements section added with 6 explicit boundaries |
| Dependencies | 82 | PASS | Graph fixed: US-004->US-007, US-005->all, US-006->US-004+US-007 |
| Testability | 75 | PASS | D formula explicit, debug output testable, doc verification scripted |
| Decisions | 78 | PASS | 3 new decisions added (log vs tracing, stdin truncation, benchmark threshold) |
| Rollback | 70 | PASS | Per-US rollback table, phase isolation defined, backward-compat serialization |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-28-audit-findings-remediation/spec.md | Full spec + Phase Summary |
| docs/audits/full-2026-03-28.md | Source audit report |
| .claude/workflow/spec-adversary-report.md | Adversary reports (rounds 1-2) |
