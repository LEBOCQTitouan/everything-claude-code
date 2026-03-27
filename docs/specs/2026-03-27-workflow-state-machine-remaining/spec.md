# Spec: Workflow State Machine — Status, Artifact, Reset Commands + Hook Rewiring

## Problem Statement

The ecc-workflow binary has domain models and transition logic but lacks visibility (no `status` command), artifact resolution (no `artifact` command), and safe reset (no `reset` command). Additionally, hooks still call shell scripts instead of the Rust binary, missing the reliability benefits of typed phase enforcement.

## Research Summary

Web research skipped: extending existing internal CLI with 3 commands + hook config change. No external patterns needed.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Extend ecc-workflow binary (not ecc-cli) | Binary already has I/O layer, state reading, dispatch | No |
| 2 | Rust-only hooks, no shell fallback | Binary is always built from source in dev workflow | No |
| 3 | `reset` requires `--force` flag (no interactive confirmation) | CLI tool used by agents; interactive prompts break automation | No |

## User Stories

### US-001: Workflow status visibility

**As a** developer or agent, **I want** `ecc-workflow status` to show the current phase, feature, and artifact paths, **so that** I can quickly understand workflow state without reading JSON.

#### Acceptance Criteria

- AC-001.1: Given an active workflow, when `ecc-workflow status` runs, then it prints phase, concern, feature, started_at, and artifact paths
- AC-001.2: Given no state.json exists, when `ecc-workflow status` runs, then it prints "No active workflow"
- AC-001.3: Output is human-readable (not JSON) with labeled fields

#### Dependencies

- Depends on: none

### US-002: Artifact path resolution

**As a** command or agent, **I want** `ecc-workflow artifact <type>` to resolve and validate an artifact path, **so that** I can locate spec/design/tasks files without parsing JSON.

#### Acceptance Criteria

- AC-002.1: Given `ecc-workflow artifact spec` with a valid spec_path in state, when it runs, then it prints the absolute path to stdout
- AC-002.2: Given `ecc-workflow artifact spec` with spec_path set but file missing, when it runs, then it exits with error "artifact file not found"
- AC-002.3: Given `ecc-workflow artifact spec` with spec_path null, when it runs, then it exits with error "no spec artifact registered"
- AC-002.4: Supported types: `spec`, `design`, `tasks`, `campaign`

#### Dependencies

- Depends on: none

### US-003: Safe workflow reset

**As a** developer, **I want** `ecc-workflow reset --force` to reset the workflow to idle state, **so that** I can start a new workflow without manually editing state.json.

#### Acceptance Criteria

- AC-003.1: Given `ecc-workflow reset --force`, when it runs, then state.json is deleted
- AC-003.2: Given `ecc-workflow reset` without `--force`, when it runs, then it exits with error requiring --force
- AC-003.3: Given no state.json exists, when `ecc-workflow reset --force` runs, then it exits cleanly (no error)

#### Dependencies

- Depends on: none

### US-004: Hook rewiring to Rust binary

**As a** developer, **I want** hooks to call the ecc-workflow binary instead of shell scripts, **so that** phase gates use typed Rust logic instead of fragile shell parsing.

#### Acceptance Criteria

- AC-004.1: hooks.json phase-gate hook command uses `ecc-workflow phase-gate` instead of `bash .claude/hooks/phase-gate.sh`
- AC-004.2: hooks.json phase-transition references are updated to use `ecc-workflow transition`
- AC-004.3: The Rust phase-gate produces identical exit codes (0 for pass/warn, 2 for block) as the shell version

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `ecc-workflow/src/commands/status.rs` (new) | CLI | Status display command |
| `ecc-workflow/src/commands/artifact.rs` (new) | CLI | Artifact resolution command |
| `ecc-workflow/src/commands/reset.rs` (new) | CLI | Reset command |
| `ecc-workflow/src/main.rs` | CLI | Add 3 new command variants |
| `hooks/hooks.json` | Config | Update hook commands to use Rust binary |

## Constraints

- Must not change state.json format (backwards compatible)
- Must not change ecc-domain workflow types (already correct)
- Exit codes must match shell versions (0 pass, 2 block)
- `reset` must require `--force` (no interactive prompts)

## Non-Requirements

- Not moving ecc-workflow into ecc-cli (separate binary stays)
- Not changing the domain models or transition rules
- Not adding new phase values
- Not modifying phase-gate allowed path list

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| FileSystem (via io.rs) | Read existing | status + artifact read state.json |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CHANGELOG | CHANGELOG.md | Project | Add entry |

## Open Questions

None — all questions resolved in grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Scope | Rescope BL-068 to missing pieces: status, artifact, reset, hook rewiring | User |
| 2 | Architecture | Extend ecc-workflow binary (not ecc-cli) | Recommended |
| 3 | Hook rewiring | Rewire to Rust binary, no shell fallback | Recommended |
| 4 | Breaking changes | Rust-only, no fallback — binary always built from source | User |
| 5 | Tests + security + ADR | Tests for new commands, no security/ADR/domain concerns | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Workflow status visibility | 3 | none |
| US-002 | Artifact path resolution | 4 | none |
| US-003 | Safe workflow reset | 3 | none |
| US-004 | Hook rewiring to Rust binary | 3 | none |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Status prints phase, concern, feature, artifacts | US-001 |
| AC-001.2 | Status prints "No active workflow" when no state | US-001 |
| AC-001.3 | Human-readable labeled output | US-001 |
| AC-002.1 | Artifact prints path for valid artifact | US-002 |
| AC-002.2 | Artifact errors when file missing | US-002 |
| AC-002.3 | Artifact errors when path null | US-002 |
| AC-002.4 | Supports spec, design, tasks, campaign | US-002 |
| AC-003.1 | Reset --force deletes state.json | US-003 |
| AC-003.2 | Reset without --force errors | US-003 |
| AC-003.3 | Reset on no state exits cleanly | US-003 |
| AC-004.1 | hooks.json uses ecc-workflow phase-gate | US-004 |
| AC-004.2 | hooks.json uses ecc-workflow transition | US-004 |
| AC-004.3 | Exit codes match shell version | US-004 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-27-workflow-state-machine-remaining/spec.md | Full spec |
