# Solution: Workflow State Machine — Status, Artifact, Reset + Hook Rewiring

## Spec Reference
Concern: dev, Feature: BL-068 — status, artifact, reset commands + hook rewiring

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `ecc-workflow/src/commands/status.rs` | Create | Status display command showing phase/concern/feature/artifacts | US-001 AC-001.1-3 |
| 2 | `ecc-workflow/src/commands/artifact.rs` | Create | Artifact path resolution and validation | US-002 AC-002.1-4 |
| 3 | `ecc-workflow/src/commands/reset.rs` | Create | Safe reset with --force requirement | US-003 AC-003.1-3 |
| 4 | `ecc-workflow/src/commands/mod.rs` | Modify | Add `pub mod status; pub mod artifact; pub mod reset;` | Module wiring |
| 5 | `ecc-workflow/src/main.rs` | Modify | Add Status, Artifact, Reset command variants + dispatch | CLI wiring |
| 6 | `hooks/hooks.json` | Modify | Update hook commands to use ecc-workflow binary | US-004 AC-004.1-2 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Status shows phase/concern/feature for active workflow | AC-001.1 | `cargo test -p ecc-workflow -- commands::status::tests::status_active_workflow` | PASS |
| PC-002 | unit | Status prints "No active workflow" when no state | AC-001.2 | `cargo test -p ecc-workflow -- commands::status::tests::status_no_workflow` | PASS |
| PC-003 | unit | Status output has labeled fields | AC-001.3 | `cargo test -p ecc-workflow -- commands::status::tests::status_labeled_output` | PASS |
| PC-004 | unit | Artifact prints valid spec_path | AC-002.1 | `cargo test -p ecc-workflow -- commands::artifact::tests::artifact_valid_spec` | PASS |
| PC-005 | unit | Artifact errors when file missing | AC-002.2 | `cargo test -p ecc-workflow -- commands::artifact::tests::artifact_file_missing` | PASS |
| PC-006 | unit | Artifact errors when path null | AC-002.3 | `cargo test -p ecc-workflow -- commands::artifact::tests::artifact_path_null` | PASS |
| PC-007 | unit | Artifact supports spec/design/tasks/campaign | AC-002.4 | `cargo test -p ecc-workflow -- commands::artifact::tests::artifact_all_types` | PASS |
| PC-008 | unit | Reset --force deletes state.json | AC-003.1 | `cargo test -p ecc-workflow -- commands::reset::tests::reset_force_deletes` | PASS |
| PC-009 | unit | Reset without --force errors | AC-003.2 | `cargo test -p ecc-workflow -- commands::reset::tests::reset_no_force_errors` | PASS |
| PC-010 | unit | Reset on no state exits cleanly | AC-003.3 | `cargo test -p ecc-workflow -- commands::reset::tests::reset_no_state_clean` | PASS |
| PC-011 | integration | hooks.json uses ecc-workflow phase-gate | AC-004.1 | `grep -q 'ecc-workflow phase-gate' hooks/hooks.json` | exit 0 |
| PC-012 | integration | hooks.json uses ecc-workflow transition | AC-004.2 | `grep -q 'ecc-workflow transition' hooks/hooks.json` | exit 0 |
| PC-013 | unit | Phase-gate exit codes: 0 pass, 2 block | AC-004.3 | `cargo test -p ecc-workflow -- commands::phase_gate::tests` | PASS |
| PC-014 | lint | Clippy zero warnings | All | `cargo clippy -- -D warnings` | exit 0 |
| PC-015 | build | Workspace builds | All | `cargo build --workspace` | exit 0 |

### Coverage Check

All 13 ACs covered:

| AC | Covered By |
|----|------------|
| AC-001.1 | PC-001 |
| AC-001.2 | PC-002 |
| AC-001.3 | PC-003 |
| AC-002.1 | PC-004 |
| AC-002.2 | PC-005 |
| AC-002.3 | PC-006 |
| AC-002.4 | PC-007 |
| AC-003.1 | PC-008 |
| AC-003.2 | PC-009 |
| AC-003.3 | PC-010 |
| AC-004.1 | PC-011 |
| AC-004.2 | PC-012 |
| AC-004.3 | PC-013 |

### E2E Test Plan

No E2E tests — all commands use existing io.rs functions on local state.json.

### E2E Activation Rules

None.

## Test Strategy

TDD order:
1. PC-001..003: status command (reads state, formats output)
2. PC-004..007: artifact command (resolves paths, validates existence)
3. PC-008..010: reset command (deletes state, requires --force)
4. PC-011..012: hooks.json update (config change)
5. PC-013..015: existing phase-gate tests + lint + build

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | CHANGELOG.md | Project | Modify | "feat: add workflow status/artifact/reset and rewire hooks to Rust" | All |

## SOLID Assessment

PASS — 0 findings. Each command is a single-responsibility module. All use the same I/O abstraction.

## Robert's Oath Check

CLEAN — adding visibility (status), validation (artifact), and safety (reset --force). Proof via 10 new tests.

## Security Notes

CLEAR — no auth, secrets, or network. State.json is local. Reset requires --force to prevent accidental data loss.

## Rollback Plan

1. Revert hooks/hooks.json to shell script commands
2. Remove Reset, Artifact, Status variants from main.rs
3. Remove `pub mod` entries from commands/mod.rs
4. Delete reset.rs, artifact.rs, status.rs
