# Spec: Consolidate Bypass to Baked-In Auditable System

## Problem Statement

ECC has two coexisting bypass mechanisms: the deprecated `ECC_WORKFLOW_BYPASS=1` env-var kill-switch (binary, no audit trail, checked in 3 code paths) and the newer auditable bypass token system (granular, session-scoped, with `BypassStore` port). ADR-0055 introduced the new system and ADR-0056 deprecated the old one, but removal was deferred. The old mechanism persists in `.envrc`, `dispatch()`, `ecc-workflow main.rs`, 50+ documentation/test references, and drives all developer workflows via the `.envrc` default. Meanwhile, the new token bypass has its own architectural smells: inline filesystem reads in `dispatch()` bypassing the port layer, `bypass_store: None` boilerplate in 41 test sites, and the domain layer at D=1.00 (Zone of Pain) with zero abstractness.

## Research Summary

- Use Rust's type system to enforce removal — deleting a bypass enum variant or env-var check causes compiler errors at all call sites
- Maintain backward compatibility during transitions via deprecation warnings, then remove in a version bump (Docker's pattern)
- Replace binary kill-switches with granular, auditable tokens per the Write-Audit-Publish pattern
- Centralize authorization checks rather than distributing bypass gates across handlers
- Bypass tokens should be session-scoped or time-scoped, not persistent env vars
- Use `cargo-unused-features` style detection to find dead bypass code
- Removing a default bypass is a breaking change — requires minor version bump minimum

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Remove ECC_WORKFLOW_BYPASS entirely | ADR-0056 planned this; deprecation period complete | No (update ADR-0056 to Completed) |
| 2 | Add `check_token()` to BypassStore port | Token resolution currently bypasses port layer | No |
| 3 | Introduce `HookPorts::test_default()` | Eliminates 41 `bypass_store: None` boilerplate sites | No |
| 4 | Extract BypassInterceptor from dispatch() | dispatch() accumulates 170 lines of bypass orchestration | No |
| 5 | Add trait to ecc-domain/bypass | Domain types at D=1.00 (Zone of Pain), need abstractness | No |
| 6 | Write tests before refactoring | User requested additional coverage for token-bypass path | No |
| 7 | Independent steps, each ships alone | Each step leaves test suite green | No |
| 8 | Non-breaking change for external consumers | No external consumers depend on ECC_WORKFLOW_BYPASS; internal-only env var | No |
| 9 | Delete `.envrc` entirely | Its sole content is `export ECC_WORKFLOW_BYPASS=1`; empty file has no purpose | No |
| 10 | Update hook convention in rules/ecc/development.md | Current rule mandates `ECC_WORKFLOW_BYPASS` check in all hooks — generates broken code | No |
| 11 | Rewrite `/ecc-test-mode` command | Entire workflow predicated on `ECC_WORKFLOW_BYPASS=0 claude`; needs token-based alternative | No |
| 12 | hook CLI wires SqliteBypassStore when DB path resolves, None otherwise | Graceful degradation for environments without bypass DB | No |

## User Stories

### US-001: Remove Legacy Bypass Env Var

**As a** developer, **I want** the deprecated `ECC_WORKFLOW_BYPASS` env var removed from all code paths, **so that** there is a single, auditable bypass mechanism.

#### Acceptance Criteria

- AC-001.1: Given `ECC_WORKFLOW_BYPASS=1` in environment, when any hook is dispatched, then the env var is ignored (no passthrough, no deprecation warning)
- AC-001.2: Given `ecc-workflow` binary is invoked with `ECC_WORKFLOW_BYPASS=1`, when any command runs, then it executes normally (env var ignored)
- AC-001.3: Given `.envrc` file, when reviewed, then it contains no `ECC_WORKFLOW_BYPASS` reference
- AC-001.4: Given a repository-wide grep for `ECC_WORKFLOW_BYPASS`, when executed, then matches exist only in historical locations: `docs/specs/*/`, `docs/adr/*/`, `docs/backlog/`, `CHANGELOG.md`, and `.claude/workflow/` reports
- AC-001.5: Given ADR-0056, when read, then its status is "Completed"
- AC-001.6: Given ecc-workflow and integration test files containing `env_remove("ECC_WORKFLOW_BYPASS")`, when the env var check is removed from production code, then these defensive removal calls are also removed
- AC-001.7: Given `tests/hooks/test-phase-gate.sh` `test_bypass()` function, when the env var is removed, then the test is rewritten to test token-based bypass or removed
- AC-001.8: Given the ecc-workflow binary invoked with `ECC_WORKFLOW_BYPASS=1` still in environment, when any subcommand runs, then it executes normally without early exit, producing correct state transitions
- AC-001.9: Given developer documentation (CLAUDE.md Gotchas, docs/getting-started.md), when reviewed, then the default bypass workflow description references token-based bypass via `ecc bypass grant`, not the removed env var
- AC-001.10: Given `.envrc` file contains only `export ECC_WORKFLOW_BYPASS=1`, when the line is removed, then the file is deleted (not left empty)
- AC-001.11: Given integration test files in `ecc-integration-tests/` that set `env("ECC_WORKFLOW_BYPASS", "0")`, when the env var check is removed from production code, then these env-set calls are also removed
- AC-001.12: Given `commands/ecc-test-mode.md`, `commands/create-component.md`, `rules/ecc/development.md`, `skills/ecc-component-authoring/SKILL.md`, and `patterns/agentic/guardrails.md`, when the env var is removed, then all instructional/template references to `ECC_WORKFLOW_BYPASS` are updated or removed
- AC-001.13: Given developer documentation, when reviewed, then it includes a note about running `direnv revoke` or removing cached `.envrc` approvals after deletion

#### Dependencies

- Depends on: US-006

### US-002: Extract Token Resolution to BypassStore Port

**As a** maintainer, **I want** bypass token lookup abstracted behind the `BypassStore` port trait, **so that** the application layer doesn't directly read the filesystem for token files.

#### Acceptance Criteria

- AC-002.1: Given `BypassStore` port trait, when reviewed, then it has a `check_token(hook_id, session_id) -> Option<BypassToken>` method
- AC-002.2: Given `SqliteBypassStore` adapter, when `check_token()` is called, then it reads from the token directory (filesystem-based, matching current behavior)
- AC-002.3: Given `InMemoryBypassStore` test double, when `check_token()` is called, then it returns pre-configured tokens
- AC-002.4: Given `dispatch()` in hook/mod.rs, when a hook blocks (exit code 2), then it calls `bypass_store.check_token()` instead of directly reading the filesystem
- AC-002.5: Given a corrupt or malformed token JSON file on disk, when `check_token()` is called, then it returns `None` and does not propagate an error
- AC-002.6: Given a token file with mismatched hook_id or session_id, when `check_token()` is called, then it returns `None`
- AC-002.7: Given no HOME environment variable, when `check_token()` is called, then it returns `None`

#### Dependencies

- Depends on: US-006

### US-003: Eliminate Test Boilerplate

**As a** developer, **I want** a `HookPorts::test_default()` constructor, **so that** test files don't need to specify `bypass_store: None` and other optional ports manually.

#### Acceptance Criteria

- AC-003.1: Given `HookPorts` struct, when `test_default()` is called with required ports (fs, shell, env, terminal), then optional ports (cost_store, bypass_store, metrics_store) default to None
- AC-003.2: Given all handler test files with `bypass_store: None` (26 handler files + hook/mod.rs dispatch tests + ecc-cli/commands/hook.rs), when updated, then they use `test_default()` instead of manual struct construction
- AC-003.3: Given ecc-cli/src/commands/hook.rs, when reviewed, then it wires `SqliteBypassStore` when the database path is resolvable, or `None` when it is not (with `tracing::debug` log)

#### Dependencies

- Depends on: US-002, US-006

### US-004: Extract BypassInterceptor

**As a** maintainer, **I want** bypass orchestration logic extracted from `dispatch()` into a `BypassInterceptor`, **so that** `dispatch()` is focused on routing hooks to handlers.

#### Acceptance Criteria

- AC-004.1: Given a new `bypass_interceptor` function in ecc-app/src/hook/, when a hook returns exit code 2, then the function handles token checking via `bypass_store.check_token()` and audit logging via `bypass_store.record()`
- AC-004.2: Given `dispatch()`, when reviewed, then it delegates to `BypassInterceptor` for all bypass logic (deprecation check removed, token check delegated)
- AC-004.3: Given `bypass_interceptor` function, when it has unit tests, then they cover: token found (exit 0 passthrough), token not found (exit 2 block), no session ID (exit 2 block), audit recording (InMemoryBypassStore contains one Applied record)

#### Dependencies

- Depends on: US-001, US-002, US-006

### US-005: Add Domain Abstractness

**As a** architect, **I want** a trait in `ecc-domain/bypass` to reduce the Zone of Pain metric (D=1.00 -> D<0.50), **so that** the domain layer is extensible.

#### Acceptance Criteria

- AC-005.1: Given `ecc-domain/bypass`, when reviewed, then it defines a `BypassPolicy` trait with method `fn should_bypass(&self, hook_id: &str, session_id: &str) -> bool` that abstracts bypass decision logic
- AC-005.2: Given `ecc-domain/bypass`, when reviewed, then it defines at least one public trait, and at least one struct in the module implements it or an external module provides an implementation

#### Dependencies

- Depends on: US-006

### US-006: Pre-Refactor Test Coverage

**As a** developer, **I want** additional test coverage for the token-bypass path before refactoring, **so that** behavior is locked down and regressions are caught.

#### Acceptance Criteria

- AC-006.1: Given dispatch() tests in hook/mod.rs, when reviewed, then there is a test for: bypass token found -> passthrough
- AC-006.2: Given dispatch() tests, when reviewed, then there is a test for: bypass token not found -> block with hint
- AC-006.3: Given dispatch() tests, when reviewed, then there is a test for: no CLAUDE_SESSION_ID -> block
- AC-006.4: Given ecc-workflow integration tests, when the binary is invoked with `ECC_WORKFLOW_BYPASS=1` in environment, then it executes normally (the binary ignores the variable)

#### Dependencies

- Depends on: none (must be done FIRST)

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| ecc-domain/src/hook_runtime/bypass.rs | Domain | Add BypassPolicy trait |
| ecc-ports/src/bypass_store.rs | Port | Add check_token() method |
| ecc-infra/src/sqlite_bypass_store.rs | Adapter | Implement check_token() |
| ecc-test-support/src/in_memory_bypass_store.rs | Test | Implement check_token() |
| ecc-app/src/hook/mod.rs | App | Extract BypassInterceptor, remove env-var check, use port for tokens |
| ecc-app/src/hook/handlers/ (28 files) | App | Replace manual HookPorts with test_default() |
| ecc-cli/src/commands/hook.rs | CLI | Wire SqliteBypassStore |
| ecc-workflow/src/main.rs | Binary | Remove ECC_WORKFLOW_BYPASS check |
| .envrc | Config | Remove ECC_WORKFLOW_BYPASS=1 |
| CLAUDE.md | Doc | Remove bypass env-var references |
| docs/adr/0056-*.md | Doc | Status -> Completed |
| rules/ecc/development.md | Rule | Remove ECC_WORKFLOW_BYPASS hook convention |
| commands/ecc-test-mode.md | Command | Rewrite to use token-based bypass |
| commands/create-component.md | Command | Update hook template, remove bypass check |
| skills/ecc-component-authoring/SKILL.md | Skill | Update hook authoring instructions |
| patterns/agentic/guardrails.md | Pattern | Remove bypass env var reference |
| ecc-integration-tests/tests/ (4 files) | Test | Remove env("ECC_WORKFLOW_BYPASS", "0") calls |
| tests/hooks/test-phase-gate.sh | Test | Rewrite or remove test_bypass() function |

## Constraints

- All refactoring steps must be behavior-preserving (except deliberate removal of deprecated env-var passthrough)
- Test suite must stay green after each step
- Each step ships independently
- Pre-refactor tests (US-006) must be written FIRST

## Non-Requirements

- Bus factor improvement (smell #9) — not addressable via code
- Replacing `serde_json` for token parsing — current approach is fine
- Adding a migration CLI from old bypass to new — not needed since `.envrc` is a local file

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| BypassStore | Method added | New integration tests for check_token() |
| HookPorts | Constructor added | No E2E impact (test infrastructure) |
| dispatch() | Logic extracted | Existing E2E tests cover hook dispatch |
| ecc-workflow binary | Env check removed | Integration tests must stop using ECC_WORKFLOW_BYPASS=1 |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Env var removal | CLAUDE.md | CLAUDE.md | Remove ECC_WORKFLOW_BYPASS references from Gotchas |
| ADR completion | ADR | docs/adr/0056 | Update status to Completed |
| New port method | Module docs | ecc-ports/bypass_store.rs | Add check_token() doc comments |
| Test pattern | Dev guide | crates/CLAUDE.md | Document HookPorts::test_default() convention |
| CHANGELOG | Release notes | CHANGELOG.md | Add entry for bypass consolidation |

## Rollback Strategy

Each commit is independently revertable via `git revert`. The env var removal commit (US-001) can be reverted without affecting the port extraction (US-002) or test boilerplate (US-003) work. Developers who still have `ECC_WORKFLOW_BYPASS=1` in personal shell configs will experience no breakage — after removal, the variable is simply ignored (AC-001.1, AC-001.8).

Note: `.envrc` is gitignored and not tracked. If rollback is needed, manually recreate with `echo 'export ECC_WORKFLOW_BYPASS=1' > .envrc`. After US-001, developers bypass hooks via `ecc bypass grant`, which is fully functional today (ADR-0055). Setting the env var after removal is harmless (the variable is ignored, not rejected) — no CI breakage from incremental merge order.

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Smell triage | Address all smells 1-8 (skip bus factor) | User |
| 2 | Target architecture | Full port extraction: add check_token() to BypassStore, wire SqliteBypassStore in prod CLI, HookPorts::test_default() helper | Recommended |
| 3 | Step independence | Independent steps — each ships alone, test suite stays green | Recommended |
| 4 | Downstream dependencies | HookPorts::test_default() for 28 handler files, manual integration test update, doc sweep last | Recommended |
| 5 | Rename vs behavioral change | Pure removal (env var), behavioral additive (port method), behavioral restructure (interceptor), mechanical (test boilerplate) | Recommended |
| 6 | Performance budget | Negligible impact — vtable indirection only, no new I/O on hot path | Recommended |
| 7 | ADR decisions | Update ADR-0056 status to Completed, no new ADRs | Recommended |
| 8 | Test safety net | Need more coverage first — add tests for check_token() and token-bypass end-to-end path | User |

**Smells addressed:** #1 (dual bypass), #2 (inline token validation), #3 (bypass_store: None boilerplate), #4 (ecc-workflow main.rs), #5 (D=1.00 Zone of Pain), #6 (.envrc), #7 (dispatch bloat), #8 (50-file reference cleanup)
**Smells deferred:** #9 (bus factor — not addressable via code)

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Remove Legacy Bypass Env Var | 13 | US-006 |
| US-002 | Extract Token Resolution to BypassStore Port | 7 | US-006 |
| US-003 | Eliminate Test Boilerplate | 3 | US-002, US-006 |
| US-004 | Extract BypassInterceptor | 3 | US-001, US-002, US-006 |
| US-005 | Add Domain Abstractness | 2 | US-006 |
| US-006 | Pre-Refactor Test Coverage | 4 | none |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | ECC_WORKFLOW_BYPASS=1 ignored by hook dispatch | US-001 |
| AC-001.2 | ecc-workflow binary ignores env var | US-001 |
| AC-001.3 | .envrc has no ECC_WORKFLOW_BYPASS reference | US-001 |
| AC-001.4 | Repo-wide grep matches only in docs/specs/ and docs/adr/ | US-001 |
| AC-001.5 | ADR-0056 status is Completed | US-001 |
| AC-001.6 | env_remove("ECC_WORKFLOW_BYPASS") calls removed from tests | US-001 |
| AC-001.7 | test-phase-gate.sh test_bypass() rewritten or removed | US-001 |
| AC-001.8 | ecc-workflow binary executes normally with env var set | US-001 |
| AC-001.9 | Dev docs reference ecc bypass grant, not env var | US-001 |
| AC-001.10 | .envrc deleted (not left empty) | US-001 |
| AC-001.11 | Integration test env-set calls removed | US-001 |
| AC-001.12 | Command/rule/skill/pattern templates updated | US-001 |
| AC-001.13 | direnv revoke note in documentation | US-001 |
| AC-002.1 | BypassStore has check_token() method | US-002 |
| AC-002.2 | SqliteBypassStore implements check_token() | US-002 |
| AC-002.3 | InMemoryBypassStore implements check_token() | US-002 |
| AC-002.4 | dispatch() uses bypass_store.check_token() | US-002 |
| AC-002.5 | Malformed token JSON returns None | US-002 |
| AC-002.6 | Mismatched token returns None | US-002 |
| AC-002.7 | No HOME env var returns None | US-002 |
| AC-003.1 | HookPorts::test_default() with optional ports defaulting to None | US-003 |
| AC-003.2 | All handler test files use test_default() | US-003 |
| AC-003.3 | hook CLI wires SqliteBypassStore or None with debug log | US-003 |
| AC-004.1 | bypass_interceptor function handles token checking + audit | US-004 |
| AC-004.2 | dispatch() delegates to bypass_interceptor | US-004 |
| AC-004.3 | Unit tests with concrete exit codes and assertions | US-004 |
| AC-005.1 | BypassPolicy trait with should_bypass method | US-005 |
| AC-005.2 | At least one public trait with implementation | US-005 |
| AC-006.1 | Test: bypass token found -> passthrough | US-006 |
| AC-006.2 | Test: bypass token not found -> block with hint | US-006 |
| AC-006.3 | Test: no CLAUDE_SESSION_ID -> block | US-006 |
| AC-006.4 | ecc-workflow binary ignores env var in tests | US-006 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Ambiguity | 80 | PASS | ACs are concrete with grep-testable criteria |
| Edge Cases | 75 | PASS | Malformed tokens, mismatched IDs, missing HOME covered |
| Scope Creep | 90 | PASS | Well-bounded non-requirements, US-005 flagged as optional |
| Dependencies | 88 | PASS | US-006 dependency added to all stories |
| Testability | 85 | PASS | Exit codes specified, structural criteria for domain trait |
| Decisions | 72 | PASS | 12 decisions including hook convention, test-mode rewrite |
| Rollback | 85 | PASS | Independent revert, .envrc recreation, CI safety noted |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-07-consolidate-bypass-to-baked-in/spec.md | Full spec |
| docs/specs/2026-04-07-consolidate-bypass-to-baked-in/campaign.md | Grill-me decisions |
| .claude/workflow/spec-adversary-report.md | Adversary report (3 rounds) |
