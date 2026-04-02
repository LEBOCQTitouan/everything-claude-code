# Spec: Deterministic Hook System Redesign

## Problem Statement

The ECC workflow enforcement system suffers from a fundamental design flaw: workflow state transitions are driven by Bash commands (`!ecc-workflow init`, `!ecc-workflow transition`) embedded in Markdown command files that Claude must faithfully execute. Claude is an LLM -- it can skip steps, get interrupted mid-command, have its context compacted, or simply fail to execute a transition call. When this happens, the workflow state gets stuck at a previous phase with no automatic detection or recovery. Additionally, the hook system is split across two separate binaries (`ecc-hook` for hook dispatch, `ecc-workflow` for state management) that share no process context, the state file is not worktree-scoped (causing state corruption when Claude enters a worktree), the `ECC_WORKFLOW_BYPASS` environment variable silently disables ALL enforcement, and the phase gate has a path traversal vulnerability.

## Research Summary

- **Type-state pattern for workflow phase enforcement**: Encode each workflow phase as a distinct Rust type. Transition methods consume `self` and return the next state type, making invalid transitions impossible at compile time. Trade-off: keep the number of states small since the graph must be known at compile time.
- **Hook registration as typed trait objects**: Define a `Hook` trait in Rust with `pre_transition()` and `post_transition()` methods. Register hooks as trait objects rather than shelling out to bash. This removes the bypass vector -- hooks run in-process and their execution order is deterministic.
- **Persist-and-recover pattern**: Write a durable state file after every successful transition. On startup, read the file to determine current phase. Combine with advisory file lock (`flock`) to prevent concurrent advances.
- **Worktree-scoped state via `git rev-parse --git-dir`**: Store all tool-specific state relative to the git dir path, not the working tree root. This makes state automatically worktree-isolated.
- **Deterministic simulation testing (DST)**: Workflow engine and hook runner should be pure functions of `(current_state, event) -> (next_state, effects)` with all I/O behind port traits, enabling fully deterministic replay testing.
- **Pitfall: over-engineering type-state**: Use runtime enums for dynamic/user-driven branching within a phase; reserve compile-time type-state only for the phase-level state machine.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Unify ecc-hook, ecc-workflow, and ecc CLI into a single binary | Eliminates dual-binary divergence, shared process context, simpler deployment | Yes |
| 2 | Scope workflow state to `git rev-parse --git-dir` | Automatic worktree isolation without special branching logic | Yes |
| 3 | Replace command-embedded Bash calls with transition-triggered hooks | Hooks fire automatically on state transitions as Rust trait objects, not relying on Claude to execute shell commands | Yes |
| 4 | Keep old binaries as thin wrappers during migration, remove in cleanup PR after Group C lands | Preserves downstream compatibility; bounded timeline (cleanup PR is mandatory, not indefinite) | No |
| 5 | Write characterization tests before refactoring | Session hooks and typed hook merge have zero tests; need safety net | No |
| 6 | 3 atomic PR groups with explicit US mapping (see PR Group Mapping) | Each group independently shippable with own tests, manageable blast radius | No |
| 7 | Profile hook dispatch latency (time from `ecc hook` invocation to first useful work) after merge; optimize only if >10ms p99 | Avoid premature optimization; hooks currently ~5ms measured via `hyperfine` | No |
| 8 | All 22 ecc-workflow subcommands are migrated to `ecc workflow <name>` with identical exit codes, stdout, and stderr for the same inputs | No subcommands are dropped or left behind; characterization tests from US-008 define "identical" | No |
| 9 | State format versioning: add `"version": 1` field to state.json now | Cheap addition; enables future migrations without breaking old readers (unknown fields are ignored by serde) | No |
| 10 | Bypass hardening (granular ECC_WORKFLOW_BYPASS) is deferred to a follow-up spec | It is net-new feature work, not refactoring; including it risks scope creep | No |
| 11 | Path canonicalization uses lexical normalization (strip `..` without filesystem access), not `std::fs::canonicalize` | Lexical normalization works on non-existent paths (the path may not exist yet when the phase gate runs) | No |

## PR Group Mapping

| PR Group | User Stories | Description |
|----------|-------------|-------------|
| **Group A** | US-001, US-002, US-008 | Worktree-scoped state, path canonicalization, characterization tests |
| **Group B** | US-003, US-005, US-006 | Merge ecc-workflow into ecc CLI, phase verification, stuck-state recovery |
| **Group C** | US-004 | Merge ecc-hook into ecc CLI, hooks.json migration |
| **Cleanup** | (none -- removes thin wrappers) | Remove ecc-hook and ecc-workflow crates after Group C is verified stable for 1 week |

### PR Group Compatibility Matrix

| Merged Groups | System State | Status |
|---------------|-------------|--------|
| None | Original system, all 3 binaries | Working |
| A only | Worktree-scoped state, old binaries | Working -- ecc-workflow reads from new location, old binary uses thin wrapper |
| A + B | Workflow in ecc CLI, ecc-workflow is thin wrapper, ecc-hook still standalone | Working -- hooks.json still references ecc-hook, workflow commands reference ecc workflow |
| A + B + C | Full unification, both old binaries are thin wrappers | Working -- all references updated |
| A + B + C + Cleanup | Single binary, old crates removed | Working -- final state |

## User Stories

### US-001: Worktree-Scoped Workflow State

**As a** developer using ECC with worktrees, **I want** the workflow state to be automatically scoped to the current worktree, **so that** entering a worktree doesn't corrupt or read stale state from the main tree.

#### Acceptance Criteria

- AC-001.1: Given a git worktree, when `ecc workflow init` is called, then state.json is written under the worktree's git-dir (not the main repo's `.git/`)
- AC-001.2: Given a main repo with active workflow state, when Claude enters a worktree, then the worktree has its own independent state.json
- AC-001.3: Given a worktree with state.json, when `ecc workflow status` is called from the worktree, then it reads the worktree-scoped state
- AC-001.4: Given `CLAUDE_PROJECT_DIR` pointing to a worktree, when any workflow command runs, then it resolves state relative to that worktree's git-dir
- AC-001.5: Given a directory that is not a git repository, when any workflow command runs, then it falls back to `.claude/workflow/state.json` relative to `CLAUDE_PROJECT_DIR` (or `.`) and logs a warning: "Not a git repository -- state is not worktree-isolated"
- AC-001.6: Given an existing state.json at the old location (`.claude/workflow/state.json` relative to project root) and no state.json at the new worktree-scoped location, when any workflow command runs, then it reads from the old location (backward-compatible fallback) and logs a warning: "Migrating state to worktree-scoped location"
- AC-001.7: Given a bare git repository, when any workflow command runs, then it stores state under the bare repo's git-dir (which is the repo itself)

#### Dependencies

- Depends on: none

### US-002: Path Canonicalization in Phase Gate

**As a** developer relying on the phase gate, **I want** path traversal attacks to be blocked, **so that** `../../src/evil.rs` cannot bypass the allowed-path check during gated phases.

#### Acceptance Criteria

- AC-002.1: Given a gated phase and a path containing `..` components, when the phase gate evaluates it, then the path is lexically normalized (all `..` and `.` components resolved without filesystem access) before checking against allowed prefixes
- AC-002.2: Given a gated phase and an absolute path outside allowed prefixes, when the phase gate evaluates it, then it is blocked
- AC-002.3: Given a gated phase and a path like `docs/specs/../../src/evil.rs`, when the phase gate evaluates it, then lexical normalization produces `src/evil.rs` which is blocked

#### Dependencies

- Depends on: none

### US-003: Unified Binary (ecc-workflow into ecc CLI)

**As a** developer, **I want** the workflow state machine to be part of the `ecc` CLI binary, **so that** there's a single binary for all ECC functionality with shared process context.

#### Acceptance Criteria

- AC-003.1: Given the `ecc` binary, all 22 ecc-workflow subcommands are available under `ecc workflow <subcommand>` with identical exit codes, stdout content, and stderr content for the same inputs and state file. "Identical" is defined by the characterization tests from US-008.
- AC-003.2: Given the `ecc-workflow` binary after migration, when any subcommand is called, then it delegates to `ecc workflow <subcommand>` via `exec` (thin wrapper). The wrapper preserves all arguments, stdin, env vars, and exit codes.
- AC-003.3: Given the `ecc` binary, when any workflow command uses file I/O, then it uses port traits (FileSystem, Environment) not raw `std::fs`. The workflow commands are testable with InMemoryFileSystem.
- AC-003.4: Given the `ecc` binary, when `ecc workflow` is called with `--verbose`, then it emits tracing output identical to `ecc-workflow -v`.
- AC-003.5: Given state.json, the format includes a `"version": 1` field. Readers ignore unknown fields (forward-compatible). Writers always include the version field.

#### Full Subcommand Migration Table

| ecc-workflow subcommand | ecc workflow equivalent |
|------------------------|------------------------|
| init | workflow init |
| transition | workflow transition |
| toolchain-persist | workflow toolchain-persist |
| memory-write | workflow memory-write |
| phase-gate | workflow phase-gate |
| stop-gate | workflow stop-gate |
| grill-me-gate | workflow grill-me-gate |
| tdd-enforcement | workflow tdd-enforcement |
| status | workflow status |
| artifact | workflow artifact |
| reset | workflow reset |
| scope-check | workflow scope-check |
| doc-enforcement | workflow doc-enforcement |
| doc-level-check | workflow doc-level-check |
| pass-condition-check | workflow pass-condition-check |
| e2e-boundary-check | workflow e2e-boundary-check |
| worktree-name | workflow worktree-name |
| wave-plan | workflow wave-plan |
| merge | workflow merge |
| backlog add-entry | workflow backlog add-entry |
| tasks sync | workflow tasks sync |
| tasks update | workflow tasks update |
| tasks init | workflow tasks init |

#### Dependencies

- Depends on: US-001 (worktree-scoped state must be in place before merging)

### US-004: Unified Binary (ecc-hook into ecc CLI)

**As a** developer, **I want** hook dispatch to be part of the `ecc` CLI binary, **so that** hooks share process context with the workflow state machine.

#### Acceptance Criteria

- AC-004.1: Given the `ecc` binary, when `ecc hook <id>` is called, then it produces the same exit code, stdout, and stderr as the current `ecc-hook <id>` for the same inputs. Defined by characterization tests.
- AC-004.2: Given the `ecc-hook` binary after migration, when any hook is dispatched, then it delegates to `ecc hook <id>` via `exec` (thin wrapper). If `ecc hook` fails with exit code > 2 (unexpected error, not a hook block), the wrapper falls back to the original built-in implementation for safety.
- AC-004.3: Given hooks.json contains `ecc-hook` references, when `ecc install` is run after Group C lands, then hooks.json is updated to reference `ecc hook`. The migration is idempotent -- running it again produces no changes.
- AC-004.4: Given hooks.json contains custom user hooks alongside ecc-hook references, when the migration runs, then only ecc-hook references are updated and custom hooks are preserved unchanged.
- AC-004.5: The hooks.json migration MUST NOT run until `ecc hook <id>` is verified to work (the binary is on PATH and responds to `ecc hook check:hook:enabled`).

#### Dependencies

- Depends on: US-003 (workflow must be merged first so hooks can access workflow state in-process)

### US-005: Phase Verification in Commands

**As a** developer running spec/design/implement commands, **I want** the command to verify the current workflow phase before proceeding, **so that** running a command out of order fails fast instead of doing work then failing at transition time.

Note: This is a **user-facing behavior change** -- commands that previously proceeded silently and failed at transition time will now fail fast with a clear error message. This is intentional and improves the user experience.

#### Acceptance Criteria

- AC-005.1: Given a workflow in phase=Idle, when `/design` is invoked, then it fails immediately with "Expected phase: Solution, current: Idle. Run /spec first."
- AC-005.2: Given a workflow in phase=Plan, when `/implement` is invoked, then it fails immediately with "Expected phase: Implement, current: Plan. Run /design first."
- AC-005.3: Given a workflow in phase=Solution, when `/design` is invoked, then it proceeds normally
- AC-005.4: Given no state.json (no active workflow), when `/spec-*` is invoked, then it initializes normally
- AC-005.5: Phase verification is implemented as a pure Rust function `verify_phase(current: Phase, expected: Phase) -> Result<(), PhaseError>` callable from unit tests without requiring a Claude Code session.

#### Dependencies

- Depends on: US-003 (phase verification uses the unified workflow commands)

### US-006: Stuck-State Recovery

**As a** developer whose previous session died mid-workflow, **I want** automatic detection and recovery of stuck workflow states, **so that** I don't have to manually run `ecc-workflow reset --force`.

#### Acceptance Criteria

- AC-006.1: Given a state.json with `started_at` older than the configurable staleness threshold (default: 4 hours, chosen because ECC sessions rarely exceed 3 hours), when a new session starts (SessionStart hook), then a warning is emitted: "Workflow stuck at phase X since <timestamp>. Run `ecc workflow reset --force` to reset, or resume with the appropriate command."
- AC-006.2: Given a state.json with a `started_at` timestamp, the staleness detection logic accepts an injectable clock source (a `fn now() -> Timestamp` parameter or `Clock` port trait) so that staleness can be tested without real wall-clock delays.
- AC-006.3: Given a stuck workflow, when `ecc workflow recover` is called, then it archives the current state to `.claude/workflow/archive/state-<timestamp>.json` (same format as existing `archive_state` function) and resets to Idle. If archival fails (disk full, permissions), the error is returned and the reset does NOT happen (no data loss).
- AC-006.4: Given `ecc workflow status` is called on a state with `started_at` older than the staleness threshold, then the output includes "STALE" alongside the phase.

#### Dependencies

- Depends on: US-001 (state must be worktree-scoped for correct staleness detection)

### US-008: Characterization Tests

**As a** developer about to refactor the hook system, **I want** characterization tests for the current behavior, **so that** the refactoring doesn't silently change behavior.

#### Acceptance Criteria

- AC-008.1: Given the current session hook handlers (session_start, session_end), then characterization tests exist that capture their exit codes and stderr/stdout for representative inputs
- AC-008.2: Given the current typed hook merge logic (merge_hooks_typed, remove_legacy_hooks_typed), then characterization tests exist covering: add new hooks, update existing hooks, remove legacy hooks, preserve user customizations
- AC-008.3: Given the full workflow lifecycle (init -> plan -> solution -> implement -> done), then an E2E integration test exercises the complete cycle using the ecc-workflow binary, verifying exit codes and state.json content at each step
- AC-008.4: Given a worktree scenario, then a test creates a worktree, runs workflow commands from both the main repo and the worktree, and verifies state isolation

#### Dependencies

- Depends on: none (must be done FIRST, before any refactoring)

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `ecc-domain/src/workflow/` | Domain | Add staleness detection, recovery logic, state version field |
| `ecc-ports` | Port | Add `GitInfo` port trait for `git rev-parse --git-dir`, add `Clock` port trait |
| `ecc-infra` | Adapter | Implement `OsGitInfo` and `SystemClock` adapters |
| `ecc-app` | App | Add workflow orchestration use cases, absorb ecc-workflow command logic |
| `ecc-cli/src/commands/` | CLI | Add `workflow` and `hook` subcommand groups |
| `ecc-workflow/` | Standalone | Convert to thin wrapper delegating to `ecc workflow` |
| `ecc-workflow/src/commands/phase_gate.rs` | Workflow | Add lexical path normalization |
| `hooks/hooks.json` | Config | Update binary references from `ecc-hook` to `ecc hook` (Group C) |
| `commands/*.md` | Commands | Update `!ecc-workflow` to `!ecc workflow` (Group B), add phase verification (Group B) |

## Constraints

- All refactoring steps must be behavior-preserving (characterization tests pass after each PR group)
- Test suite must stay green after each atomic group
- Old binaries (ecc-hook, ecc-workflow) must remain functional as thin wrappers until cleanup PR (1 week after Group C)
- hooks.json format must remain compatible with Claude Code's hook schema
- State file format (JSON) must remain backward-compatible; new `version` field is additive
- Shell scripts called BY hooks remain as-is; only the dispatch mechanism (how hooks are invoked) changes

## Non-Requirements

- **Not in scope**: Compile-time type-state pattern for phases (runtime enum is sufficient; type-state adds complexity without proportional benefit for 5 phases)
- **Not in scope**: Replacing shell-based quality hooks (format, typecheck, console-warn) with Rust implementations -- shell scripts called by hooks are unchanged; only the dispatch entry point changes
- **Not in scope**: Granular ECC_WORKFLOW_BYPASS (deferred to follow-up spec -- this is net-new feature work, not refactoring)
- **Not in scope**: Full hexagonal compliance for all migrated workflow commands (moving code is in scope; rewriting every command to use port traits is a stretch goal for Group B, not a hard requirement)

## Rollback Strategy

| PR Group | Rollback Mechanism |
|----------|-------------------|
| Group A | Revert the PR. State falls back to project-dir-scoped location. No data loss (old location files untouched). |
| Group B | Revert the PR. ecc-workflow thin wrapper reverts to standalone binary. Commands revert to `!ecc-workflow`. No state format changes (version field is additive, ignored by old readers). |
| Group C | Revert the PR. ecc-hook thin wrapper reverts to standalone binary. hooks.json reverts to `ecc-hook` references. Since the thin wrapper has a fallback (AC-004.2), even a partial deployment is safe. |

### In-Flight State Migration

Developers mid-workflow during upgrade: AC-001.6 ensures backward-compatible fallback. If a state.json exists at the old location but not at the new worktree-scoped location, the system reads from the old location and logs a migration warning. The next write will create state at the new location. The old file is left untouched (no data loss).

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| FileSystem (state.json location) | Path change | E2E tests must use worktree-scoped paths |
| ShellExecutor (hook dispatch) | Process change | E2E tests must invoke `ecc hook` instead of `ecc-hook` |
| Environment (CLAUDE_PROJECT_DIR) | Interpretation change | E2E tests must verify git-dir resolution |
| GitInfo (new port) | New | E2E tests must cover worktree, non-worktree, non-git, and bare repo scenarios |
| Clock (new port) | New | Staleness tests use injectable clock |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Binary names | CLAUDE.md | `## CLI Commands` section | Update ecc-workflow refs to ecc workflow |
| Architecture | docs/ARCHITECTURE.md | Crate diagram | Remove ecc-workflow as standalone, add to ecc-cli |
| ADR | docs/adr/ | New files | 3 ADRs: binary unification, worktree state, transition hooks |
| Commands | commands/*.md | All spec/design/implement commands | Update !ecc-workflow to !ecc workflow |
| hooks.json | hooks/hooks.json | binary references | Update ecc-hook to ecc hook |
| Gotchas | CLAUDE.md | `## Gotchas` | Add worktree-scoped state behavior |

## Open Questions

None -- all questions resolved in grill-me interview and adversarial review.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Smell triage | Address all 10 smells in one pass | User |
| 2 | Target architecture | Move everything into ecc CLI (single binary) | User |
| 3 | Step independence | 3 atomic groups (A: state, B: workflow, C: hooks) | Recommended |
| 4 | Downstream dependencies | Thin wrapper migration during transition | Recommended |
| 5 | Rename vs behavioral | A=behavioral, B=structural+behavioral, C=behavioral | Recommended |
| 6 | Performance budget | Profile and optimize if >10ms after merge | Recommended |
| 7 | ADR decisions | All 3 ADRs (binary unification, worktree state, transition hooks) | Recommended |
| 8 | Test safety net | Write characterization tests before refactoring | Recommended |

**Smells addressed**: #1 soft enforcement, #2 dual-binary split, #3 bypass, #4 worktree state, #5 path traversal, #6 hex bypass, #7 session hook tests, #8 match dispatch, #9 phase verification, #10 stuck-state recovery.

**Smells deferred**: Granular bypass hardening (moved to follow-up spec as net-new feature work).

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Worktree-Scoped Workflow State | 7 | none |
| US-002 | Path Canonicalization in Phase Gate | 3 | none |
| US-003 | Unified Binary (ecc-workflow into ecc CLI) | 5 | US-001 |
| US-004 | Unified Binary (ecc-hook into ecc CLI) | 5 | US-003 |
| US-005 | Phase Verification in Commands | 5 | US-003 |
| US-006 | Stuck-State Recovery | 4 | US-001 |
| US-008 | Characterization Tests | 4 | none |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Worktree git-dir state write | US-001 |
| AC-001.2 | Independent worktree state | US-001 |
| AC-001.3 | Worktree-scoped status read | US-001 |
| AC-001.4 | CLAUDE_PROJECT_DIR git-dir resolution | US-001 |
| AC-001.5 | Non-git fallback with warning | US-001 |
| AC-001.6 | Old-location backward-compatible fallback | US-001 |
| AC-001.7 | Bare repo support | US-001 |
| AC-002.1 | Lexical path normalization for `..` | US-002 |
| AC-002.2 | Absolute path outside prefix blocked | US-002 |
| AC-002.3 | Normalized traversal path blocked | US-002 |
| AC-003.1 | All 22 subcommands migrated identically | US-003 |
| AC-003.2 | Thin wrapper delegation via exec | US-003 |
| AC-003.3 | Port traits for file I/O | US-003 |
| AC-003.4 | Verbose/tracing parity | US-003 |
| AC-003.5 | State format version field | US-003 |
| AC-004.1 | ecc hook identical behavior | US-004 |
| AC-004.2 | Thin wrapper with fallback | US-004 |
| AC-004.3 | Idempotent hooks.json migration | US-004 |
| AC-004.4 | Custom hooks preserved | US-004 |
| AC-004.5 | Migration safety gate | US-004 |
| AC-005.1 | Design fails on wrong phase | US-005 |
| AC-005.2 | Implement fails on wrong phase | US-005 |
| AC-005.3 | Correct phase proceeds | US-005 |
| AC-005.4 | No state initializes normally | US-005 |
| AC-005.5 | Pure function testable | US-005 |
| AC-006.1 | Staleness warning with configurable threshold | US-006 |
| AC-006.2 | Injectable clock for testing | US-006 |
| AC-006.3 | Recover archives then resets | US-006 |
| AC-006.4 | Status shows STALE | US-006 |
| AC-008.1 | Session hook characterization tests | US-008 |
| AC-008.2 | Typed hook merge characterization tests | US-008 |
| AC-008.3 | Full lifecycle E2E test | US-008 |
| AC-008.4 | Worktree isolation test | US-008 |

### Adversary Findings

| Dimension | Score (R1 -> R2) | Verdict | Key Rationale |
|-----------|-----------------|---------|---------------|
| Ambiguity | 72 -> 85 | PASS | "Behaves identically" now defined by characterization tests; staleness threshold justified |
| Edge Cases | 45 -> 78 | PASS | All 22 subcommands in migration table; non-git, bare repo, old-location fallback ACs added |
| Scope Creep Risk | 55 -> 90 | PASS | US-007 removed to follow-up spec; Non-Requirements clarified |
| Dependency Gaps | 65 -> 82 | PASS | PR Group Mapping table and compatibility matrix added |
| Testability | 70 -> 85 | PASS | Injectable clock, pure function phase verification, characterization tests |
| Decision Completeness | 50 -> 78 | PASS | 11 decisions covering all subcommands, state versioning, wrapper timeline |
| Rollback & Failure | 42 -> 75 | PASS | Per-group rollback table, in-flight migration strategy, wrapper fallback |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-04-01-deterministic-hook-system-redesign/spec.md | Full spec with Phase Summary |
