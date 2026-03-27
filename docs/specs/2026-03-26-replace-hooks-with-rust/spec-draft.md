# Spec: BL-052 Replace Shell Hooks with Compiled Rust Binaries

## Problem Statement

ECC's workflow state machine is implemented in 13 shell scripts under `.claude/hooks/` that depend on `jq` and a POSIX shell. This makes ECC unusable on Windows and any environment without these tools. The scripts bypass the hexagonal architecture entirely — reading/writing `state.json` directly with no port abstraction or domain modeling.

## Research Summary

- Rust type-state patterns can enforce valid workflow transitions at compile time
- DDD aggregates in Rust benefit from immutable getters + mutable domain logic with compile-time enforcement
- Claude Code hooks receive JSON on stdin, exit codes 0/2 control pass/block behavior
- Prior audit flagged 5000+ lines of churn in hook handler subsystem and zero logging in hook dispatch
- The `statig` crate provides hierarchical state machines for Rust, though a simpler enum-based approach is likely sufficient

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Separate `ecc-workflow` crate (not extend ecc-hook) | Must be buildable independently of main ecc binary for any configuration | Yes |
| 2 | Single binary with subcommand dispatch | One `ecc-workflow` binary dispatching by first arg, not 13 separate binaries | No |
| 3 | Depends on ecc-domain for WorkflowState types | Share domain model, avoid logic duplication | No |
| 4 | Dual interface: CLI args + stdin JSON | CLI args when called from commands (`!bash`); stdin JSON when triggered by hooks.json. Auto-detect by checking stdin. Structured JSON output in both modes | No |
| 6 | Build trigger: bundled in workspace | `ecc-workflow` compiles as part of `cargo build` alongside other crates — resolves backlog open question | No |
| 5 | Audit-first: remove dead scripts before porting | Avoid wasting effort porting superseded code | No |

## User Stories

### US-001: Audit and classify hook scripts

**As a** ECC maintainer, **I want** to audit all 13 shell scripts to identify which are still actively called, **so that** I only port living code.

#### Acceptance Criteria

- AC-001.1: Given the 13 scripts in `.claude/hooks/` (workflow-init.sh, phase-transition.sh, toolchain-persist.sh, memory-writer.sh, phase-gate.sh, stop-gate.sh, grill-me-gate.sh, tdd-enforcement.sh, scope-check.sh, doc-enforcement.sh, doc-level-check.sh, pass-condition-check.sh, e2e-boundary-check.sh), when audited via grep across commands/, skills/, hooks/, then each is classified as "active" or "dead"
- AC-001.2: Given dead scripts are identified, when the audit completes, then dead scripts are deleted with a commit
- AC-001.3: Given the audit results, when documented, then the active script inventory is recorded in the spec

#### Dependencies
- Depends on: none

### US-002: Model WorkflowState domain aggregate

**As a** ECC developer, **I want** a WorkflowState aggregate in ecc-domain with Phase value object and transition rules, **so that** workflow logic has compile-time enforcement.

#### Acceptance Criteria

- AC-002.1: Given a WorkflowState aggregate, when created, then it contains phase, concern, feature, started_at, toolchain, artifacts, and completed fields
- AC-002.2: Given a Phase value object (Plan, Solution, Implement, Done), when a transition is requested, then only legal transitions are allowed: plan→solution, solution→implement, implement→done. Phase aliases (spec→design maps to plan→solution, design→implement maps to solution→implement) are also accepted
- AC-002.3: Given an illegal transition, when attempted, then a domain error is returned (not a panic)
- AC-002.4: Given a WorkflowState, when serialized/deserialized, then it round-trips to/from the existing state.json format
- AC-002.5: Given a re-entry transition (e.g., solution→solution), when the current phase matches the target, then the transition is accepted idempotently (no error)
- AC-002.6: Given corrupted or invalid JSON in state.json, when deserialized, then a clear domain error is returned with the parse failure reason

#### Dependencies
- Depends on: none

### US-003: Create ecc-workflow crate with single binary

**As a** ECC user on any OS, **I want** a compiled `ecc-workflow` binary that replaces shell scripts, **so that** hooks work without a POSIX shell or jq.

#### Acceptance Criteria

- AC-003.1: Given the Cargo workspace, when `cargo build` runs, then `ecc-workflow` binary is produced
- AC-003.2: Given `ecc-workflow init <concern> <feature>`, when called, then state.json is created with correct initial state
- AC-003.3: Given `ecc-workflow transition <target-phase> [artifact-path]`, when called with valid args, then state.json is updated atomically
- AC-003.4: Given `ecc-workflow transition` with an illegal phase, when called, then exit code is non-zero and JSON error is output
- AC-003.5: Given no state.json exists, when any subcommand runs, then exit 0 with a JSON warning message to stderr
- AC-003.6: Given each subcommand, when it completes, then output is structured JSON: `{"status": "pass"|"block"|"warn", "message": "..."}`
- AC-003.7: Given the binary, when built on macOS/Linux/Windows, then it compiles and runs without a POSIX shell
- AC-003.8: Given stdin contains JSON (hooks.json invocation), when the binary runs, then it reads context from stdin and processes it. Given no stdin (CLI invocation), when CLI args are provided, then it uses args instead

#### Dependencies
- Depends on: US-002

### US-004: Port active workflow scripts to ecc-workflow subcommands

**As a** ECC maintainer, **I want** each active workflow script ported as a subcommand in ecc-workflow, **so that** all shell scripts can be removed.

#### Acceptance Criteria

- AC-004.1: Given `workflow-init.sh`, when ported, then `ecc-workflow init` produces identical state.json output
- AC-004.2: Given `phase-transition.sh`, when ported, then `ecc-workflow transition` handles all phase transitions and artifact stamping
- AC-004.3: Given `toolchain-persist.sh`, when ported, then `ecc-workflow toolchain-persist` writes toolchain to state.json
- AC-004.4: Given `memory-writer.sh`, when ported, then `ecc-workflow memory-write` produces identical memory files
- AC-004.5: Given each script classified as "active" in US-001 audit (beyond the 4 named above), when ported, then it has a corresponding ecc-workflow subcommand
- AC-004.6: Given all scripts are ported, when the original .sh files are deleted, then no command or skill references them

#### Dependencies
- Depends on: US-001, US-003

### US-005: Update commands and skills to use ecc-workflow

**As a** ECC user, **I want** all commands and skills to call `ecc-workflow` instead of `bash .claude/hooks/*.sh`, **so that** the shell dependency is fully eliminated.

#### Acceptance Criteria

- AC-005.1: Given every command in `commands/`, when searched for `bash .claude/hooks/`, then zero matches are found
- AC-005.2: Given every skill in `skills/`, when searched for `bash .claude/hooks/`, then zero matches are found
- AC-005.3: Given `/spec-dev` calls `ecc-workflow init` and `ecc-workflow transition`, when run, then the workflow state machine behaves identically
- AC-005.4: Given `hooks.json`, when updated, then any hook entries reference ecc-workflow instead of shell scripts

#### Dependencies
- Depends on: US-004

### US-006: Delete shell scripts and update docs

**As a** ECC maintainer, **I want** all `.claude/hooks/*.sh` files deleted and docs updated, **so that** the codebase has no dead shell scripts.

#### Acceptance Criteria

- AC-006.1: Given all ports are verified, when shell scripts are deleted, then `.claude/hooks/` contains only non-.sh files
- AC-006.2: Given CLAUDE.md, when updated, then it references ecc-workflow instead of shell scripts
- AC-006.3: Given the glossary, when updated, then WorkflowState and Phase are defined
- AC-006.4: Given the test count in CLAUDE.md, when updated, then it reflects new test additions

#### Dependencies
- Depends on: US-005

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `ecc-domain` | Domain | Add WorkflowState aggregate, Phase VO, transition rules |
| `ecc-workflow` (new crate) | CLI/Infrastructure | New crate with single binary, subcommand dispatch, JSON I/O |
| `commands/` | Documentation | Update all command files to call ecc-workflow |
| `skills/` | Documentation | Update skill files to call ecc-workflow |
| `hooks/hooks.json` | Infrastructure | Update hook entries |
| `.claude/hooks/` | Infrastructure | Delete all .sh files |

## Constraints

- ecc-workflow crate MUST depend on ecc-domain but NOT on ecc-app or ecc-cli
- ecc-workflow binary MUST compile on macOS, Linux, and Windows
- ecc-workflow MUST use atomic writes (mktemp + rename) for state.json mutations
- ecc-domain MUST have zero I/O imports (existing constraint — WorkflowState is pure domain)
- ECC_WORKFLOW_BYPASS=1 env var check must be preserved in the Rust binary
- Output MUST be structured JSON: `{"status": "pass"|"block"|"warn", "message": "..."}`
- Phase transition logic at 100% test coverage; passive checks at 80%
- ecc-workflow MUST read `CLAUDE_PROJECT_DIR` and `CLAUDE_SESSION_ID` env vars (provided by Claude Code runtime)
- ecc-workflow MUST support dual invocation: CLI args (from `!bash` callers) AND stdin JSON (from hooks.json runtime)
- Cross-script coupling: `phase-transition.sh` calls `memory-writer.sh` internally — in Rust, this becomes an internal function call within the binary, not a subprocess
- Timestamp format: ISO 8601 UTC (`YYYY-MM-DDTHH:MM:SSZ`) matching existing `date -u` output
- "Identical output" means semantically equivalent JSON (same keys, same values) — not byte-identical (key ordering may differ)

## Rollback Plan

1. Shell scripts are preserved in git history — can be restored via `git checkout`
2. US-006 (delete scripts) is the final step, gated on US-005 (all commands updated)
3. During transition, both invocation paths can coexist (commands can call either `bash .claude/hooks/*.sh` or `ecc-workflow`)
4. If rollback needed post-merge: revert US-006 + US-005 commits to restore shell scripts and command references

## Non-Requirements

- Porting scripts in `skills/`, `scripts/`, `statusline/`, `bin/` — not part of hook runtime
- Adding new hook functionality beyond faithful port
- Modifying the hooks runtime or hooks.json schema (beyond updating paths)
- Changing ecc-hook dispatcher — this is a separate binary
- Latency targets — just faster than shell is sufficient

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| WorkflowState (new) | New domain aggregate | Need integration tests for state.json round-trip |
| ecc-workflow binary | New CLI entry point | Need smoke tests for each subcommand |
| Commands/*.md | Reference updates | Need to verify workflow still functions end-to-end |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New crate | Medium | ARCHITECTURE.md | Add ecc-workflow to crate diagram |
| New domain types | Medium | docs/domain/glossary.md | Add WorkflowState, Phase |
| New binary | Low | CLAUDE.md | Update test count, mention ecc-workflow |
| Architecture decision | Medium | docs/adr/ | ADR for separate crate approach |

## Open Questions

None — all resolved during grill-me interview.
