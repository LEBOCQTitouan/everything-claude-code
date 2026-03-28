# Design: BL-068 — Deterministic Workflow State Machine (Remaining Gaps)

## Overview

Close five gaps in the workflow state machine: add Idle phase with `is_gated()`, replace loose JSON parsing in phase-gate with typed deserialization, archive state on reset instead of deleting, surface memory write errors in transition, and add an E2E lifecycle integration test. Implementation order: US-001 -> US-002 -> US-005 -> US-004 -> US-003.

## File Changes Table

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `crates/ecc-domain/src/workflow/phase.rs` | Modify | Add `Idle` variant to Phase enum, update Display/FromStr/serde; add `is_gated()` method | AC-001.1, AC-001.6 |
| 2 | `crates/ecc-domain/src/workflow/transition.rs` | Modify | Add Idle transition rules: Done->Idle, Idle->Plan, Idle->Idle (idempotent), Idle->other (illegal) | AC-001.2, AC-001.3, AC-001.4, AC-001.5 |
| 3 | `crates/ecc-workflow/src/commands/phase_gate.rs` | Modify | Replace `read_phase` manual JSON parsing with `WorkflowState::from_json()`; use `Phase::is_gated()` instead of string comparisons; on deserialization failure return `WorkflowOutput::warn` | AC-001.7, AC-002.1, AC-002.2, AC-002.3, AC-002.4, AC-002.5 |
| 4 | `crates/ecc-workflow/src/io.rs` | Modify | Extract `archive_state` as a public function (shared by init and reset); takes `workflow_dir`, `include_done: bool` params | Decision 5 |
| 5 | `crates/ecc-workflow/src/commands/init.rs` | Modify | Replace inline `archive_stale_state` with call to `io::archive_state(workflow_dir, false)` (false = skip done states) | Decision 5 |
| 6 | `crates/ecc-workflow/src/commands/reset.rs` | Rewrite | Archive via `io::archive_state(workflow_dir, true)` (true = include done), write Idle state instead of deleting, acquire state lock, handle no-state gracefully | AC-005.1 through AC-005.6, Decision 7, Decision 8 |
| 7 | `crates/ecc-workflow/src/commands/transition.rs` | Modify | Replace `let _ =` on memory writes with individual error capture; collect warnings into output | AC-004.1 through AC-004.4 |
| 8 | `crates/ecc-workflow/src/commands/scope_check.rs` | Modify | Add `Phase::Idle` arm to exhaustive `match state.phase` (pass through ungated, same as Implement/Done) | AC-001.1 |
| 9 | `crates/ecc-workflow/src/commands/grill_me_gate.rs` | Modify | Add `Phase::Idle` arm to exhaustive `match state.phase` (pass through, same as Implement/Done) | AC-001.1 |
| 10 | `crates/ecc-workflow/src/commands/stop_gate.rs` | Modify | Treat Idle same as Done (no warning) — Idle means "completed cycle, ready for next" | AC-001.1 |
| 11 | `crates/ecc-integration-tests/tests/workflow_lifecycle.rs` | Create | E2E lifecycle test covering full forward path, reset, re-init, illegal transition, artifact timestamps | AC-003.1 through AC-003.5 |

## Implementation Phases

### Phase 1: Add Idle Phase to Domain (US-001)

**Layers: Entity**

**Changes to `crates/ecc-domain/src/workflow/phase.rs`:**

1. Add `Idle` variant to the `Phase` enum (before `Plan` to match lifecycle order).
2. Update `Display` impl: `Self::Idle => write!(f, "idle")`.
3. Update `FromStr` impl: add `"idle"` arm returning `Ok(Self::Idle)`.
4. Serde: `#[serde(rename_all = "lowercase")]` handles `Idle` -> `"idle"` automatically.
5. Add `is_gated(&self) -> bool` method:
   ```rust
   impl Phase {
       /// Returns true for phases where file writes are restricted.
       pub fn is_gated(&self) -> bool {
           matches!(self, Self::Plan | Self::Solution)
       }
   }
   ```

**Changes to `crates/ecc-domain/src/workflow/transition.rs`:**

6. Add Idle transition rules to `resolve_transition`:
   ```rust
   // Add to the legal matches:
   (Phase::Done, Phase::Idle)
       | (Phase::Idle, Phase::Plan)
   ```
   The existing `current == target` idempotent check already handles Idle->Idle.

**Test targets:**
- `phase.rs` tests: Idle Display, FromStr("idle"), serde round-trip, is_gated() for all 5 variants
- `transition.rs` tests: Done->Idle Ok, Idle->Plan Ok, Idle->Idle Ok (idempotent), Idle->Solution Err, Idle->Implement Err, Idle->Done Err

**Boy Scout Delta:** Remove the now-redundant `read_state_phase` helper in `init.rs` (will be replaced by shared `archive_state` in Phase 3).

### Phase 2: Typed Phase-Gate Parsing (US-002)

**Layers: Adapter**

**Changes to `crates/ecc-workflow/src/commands/phase_gate.rs`:**

1. Replace `PhaseResult` enum with a simpler approach. The `read_phase` function is replaced entirely:
   ```rust
   enum PhaseResult {
       Missing,
       Parsed(Phase),
       Corrupt(String),
   }

   fn read_phase_typed(project_dir: &Path) -> PhaseResult {
       let state_path = project_dir.join(".claude/workflow/state.json");
       if !state_path.exists() {
           return PhaseResult::Missing;
       }
       let content = match std::fs::read_to_string(&state_path) {
           Ok(c) => c,
           Err(e) => return PhaseResult::Corrupt(e.to_string()),
       };
       match WorkflowState::from_json(&content) {
           Ok(state) => PhaseResult::Parsed(state.phase),
           Err(e) => PhaseResult::Corrupt(e.to_string()),
       }
   }
   ```

2. Update `run()` to handle the new `PhaseResult`:
   - `Missing` -> pass (unchanged)
   - `Corrupt(msg)` -> `WorkflowOutput::warn(format!("Corrupt state.json: {msg}"))` (exit 0, does NOT block)
   - `Parsed(phase)` -> use `phase.is_gated()` instead of string comparison

3. Replace the string comparison `if phase == "implement" || phase == "done"` with `if !phase.is_gated()` — this covers Idle, Implement, and Done in one check.

**Test targets:**
- Valid state with phase "idle" -> pass (AC-001.7)
- State with `{"phase": 123}` -> warn (AC-002.2)
- State missing "phase" key -> warn (AC-002.3)
- Valid state with phase "plan", Write to src/main.rs -> block (AC-002.4)
- State with `{"phase": "banana"}` -> warn (AC-002.5)
- Valid state with typed deserialization used (AC-002.1)

**Boy Scout Delta:** Remove any `use serde_json::Value` import from phase_gate.rs if no longer needed after removing manual JSON parsing.

### Phase 3: Archive State on Reset (US-005)

**Layers: Adapter**

**Changes to `crates/ecc-workflow/src/io.rs`:**

1. Extract `archive_state` as a public function from `init.rs::archive_stale_state`:
   ```rust
   /// Archive state.json to archive/state-YYYYMMDDHHMMSS.json.
   ///
   /// When `include_done` is false, done-phase states are NOT archived (init behavior).
   /// When `include_done` is true, ALL states are archived (reset behavior).
   pub fn archive_state(workflow_dir: &Path, include_done: bool) -> Result<(), anyhow::Error> {
       let state_path = workflow_dir.join("state.json");
       if !state_path.exists() {
           return Ok(());
       }
       let content = std::fs::read_to_string(&state_path)
           .map_err(|e| anyhow::anyhow!("Failed to read state.json: {e}"))?;
       let is_done = WorkflowState::from_json(&content)
           .map(|s| s.phase == Phase::Done)
           .unwrap_or(false); // corrupt state -> archive it

       if is_done && !include_done {
           return Ok(());
       }

       let archive_dir = workflow_dir.join("archive");
       std::fs::create_dir_all(&archive_dir)
           .map_err(|e| anyhow::anyhow!("Failed to create archive directory: {e}"))?;
       let ts = utc_now_iso8601().replace(['T', ':', 'Z'], "");
       let archive_name = format!("state-{ts}.json");
       std::fs::rename(&state_path, archive_dir.join(&archive_name))
           .map_err(|e| anyhow::anyhow!("Failed to archive state.json to {archive_name}: {e}"))?;
       Ok(())
   }
   ```

**Changes to `crates/ecc-workflow/src/commands/init.rs`:**

2. Replace `archive_stale_state` with `crate::io::archive_state(&workflow_dir, false)`.
3. Remove the now-unused `archive_stale_state` and `read_state_phase` private functions.

**Changes to `crates/ecc-workflow/src/commands/reset.rs`:**

4. Full rewrite of `run()`:
   ```rust
   pub fn run(force: bool, project_dir: &Path) -> WorkflowOutput {
       if !force {
           return WorkflowOutput::block(
               "Reset requires --force flag to prevent accidental state loss. \
                Usage: ecc-workflow reset --force",
           );
       }

       let result = with_state_lock(project_dir, || {
           let workflow_dir = project_dir.join(".claude/workflow");
           let state_path = workflow_dir.join("state.json");
           if !state_path.exists() {
               return WorkflowOutput::pass("No active workflow to reset");
           }

           // Archive state (include done states, unlike init)
           if let Err(e) = archive_state(&workflow_dir, true) {
               return WorkflowOutput::block(format!("Failed to archive state: {e}"));
           }

           // Write minimal Idle state
           let idle_state = WorkflowState {
               phase: Phase::Idle,
               concern: String::new(),
               feature: String::new(),
               started_at: utc_now_iso8601(),
               toolchain: Toolchain { test: None, lint: None, build: None },
               artifacts: Artifacts {
                   plan: None, solution: None, implement: None,
                   campaign_path: None, spec_path: None, design_path: None, tasks_path: None,
               },
               completed: vec![],
           };

           match write_state_atomic(project_dir, &idle_state) {
               Ok(()) => WorkflowOutput::pass("Workflow reset - state archived, phase set to idle"),
               Err(e) => WorkflowOutput::block(format!("Failed to write idle state: {e}")),
           }
       });

       match result {
           Ok(output) => output,
           Err(e) => WorkflowOutput::block(format!("Failed to acquire state lock: {e}")),
       }
   }
   ```

**Test targets:**
- Reset with implement state -> archived + idle state written (AC-005.1)
- Reset with done state -> archived (AC-005.2)
- Archive dir auto-created (AC-005.3)
- No state.json -> pass (AC-005.4)
- Archive dir creation failure -> block (AC-005.5)
- Reset acquires state lock (AC-005.6)

**Boy Scout Delta:** Clean up any remaining `use serde_json::Value` in init.rs if no longer needed.

### Phase 4: Memory Write Error Surfacing (US-004)

**Layers: Adapter**

**Changes to `crates/ecc-workflow/src/commands/transition.rs`:**

1. Replace `let _ =` pattern with individual error capture:
   ```rust
   let mut warnings: Vec<String> = Vec::new();

   if let Err(e) = crate::commands::memory_write::write_action(...) {
       warnings.push(format!("Memory write_action failed: {e}"));
   }
   if let Err(e) = crate::commands::memory_write::write_work_item(...) {
       warnings.push(format!("Memory write_work_item failed: {e}"));
   }
   if let Err(e) = crate::commands::memory_write::write_daily(...) {
       warnings.push(format!("Memory write_daily failed: {e}"));
   }
   if let Err(e) = crate::commands::memory_write::write_memory_index(...) {
       warnings.push(format!("Memory write_memory_index failed: {e}"));
   }
   ```

2. If warnings is non-empty, modify the output message to include them but keep Status::Pass:
   ```rust
   if warnings.is_empty() {
       output
   } else {
       let warn_text = warnings.join("; ");
       WorkflowOutput::warn(format!("{} [warnings: {}]", output.message, warn_text))
   }
   ```

   Note: `WorkflowOutput::warn` produces exit 0 per spec (AC-004.1). The state.json is already written before memory writes (AC-004.4 — no rollback).

**Test targets:**
- Successful transition with failing memory writes -> warn output with exit 0 (AC-004.1)
- Each error captured individually in warning message (AC-004.2)
- All memory writes succeed -> no warnings in output (AC-004.3)
- State.json retains new phase after memory write failure (AC-004.4)

**Boy Scout Delta:** If `write_action` et al. return `Result<(), Box<dyn Error>>`, verify the error types are consistent across all four functions. Standardize if not.

### Phase 5: E2E Workflow Lifecycle Test (US-003)

**Layers: Adapter (test-only)**

**New file: `crates/ecc-integration-tests/tests/workflow_lifecycle.rs`**

This test exercises the full workflow lifecycle by calling the workflow command functions directly (not via CLI binary), using a temp directory as the project root.

```rust
// Test structure:
// 1. init(concern="dev", feature="BL-068") -> phase=plan
// 2. transition("solution", artifact="plan") -> phase=solution, artifacts.plan = ISO8601
// 3. transition("implement") -> phase=implement
// 4. transition("done") -> phase=done
// 5. reset(force=true) -> state archived, phase=idle
// 6. init(concern="dev2", feature="BL-069") -> phase=plan (re-init after reset)
// 7. transition("implement") from plan -> Illegal transition (skips solution)
```

Uses `ecc_workflow` crate functions directly:
- `ecc_workflow::commands::init::run`
- `ecc_workflow::commands::transition::run`
- `ecc_workflow::commands::reset::run`
- `ecc_workflow::io::read_state`

**Test targets:**
- Full forward lifecycle init->plan->solution->implement->done (AC-003.1)
- Reset after done -> state archived, phase=idle (AC-003.2)
- Re-init after reset -> phase=plan (AC-003.3)
- Plan->implement rejected as illegal transition (AC-003.4)
- Artifact timestamps are ISO 8601 after transition with --artifact (AC-003.5)

**Boy Scout Delta:** Add a `workspace_root()` helper to `ecc-integration-tests/tests/common/mod.rs` if not already present for reuse across lifecycle tests.

## Pass Conditions Table

| PC | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | Unit | Idle displays as "idle" | AC-001.1 | `cargo test -p ecc-domain idle_displays_as_idle` | PASS |
| PC-002 | Unit | FromStr("idle") returns Ok(Idle) | AC-001.1 | `cargo test -p ecc-domain from_str_idle` | PASS |
| PC-003 | Unit | Idle serde round-trip | AC-001.1 | `cargo test -p ecc-domain round_trips_idle` | PASS |
| PC-004 | Unit | Done->Idle returns Ok | AC-001.2 | `cargo test -p ecc-domain done_to_idle_returns_ok` | PASS |
| PC-005 | Unit | Idle->Plan returns Ok | AC-001.3 | `cargo test -p ecc-domain idle_to_plan_returns_ok` | PASS |
| PC-006 | Unit | Idle->Solution returns Err | AC-001.4 | `cargo test -p ecc-domain idle_to_solution_returns_err` | PASS |
| PC-007 | Unit | Idle->Implement returns Err | AC-001.4 | `cargo test -p ecc-domain idle_to_implement_returns_err` | PASS |
| PC-008 | Unit | Idle->Done returns Err | AC-001.4 | `cargo test -p ecc-domain idle_to_done_returns_err` | PASS |
| PC-009 | Unit | Idle->Idle returns Ok (idempotent) | AC-001.5 | `cargo test -p ecc-domain idle_to_idle_returns_ok` | PASS |
| PC-010 | Unit | Plan.is_gated() returns true | AC-001.6 | `cargo test -p ecc-domain plan_is_gated` | PASS |
| PC-011 | Unit | Solution.is_gated() returns true | AC-001.6 | `cargo test -p ecc-domain solution_is_gated` | PASS |
| PC-012 | Unit | Idle.is_gated() returns false | AC-001.6 | `cargo test -p ecc-domain idle_is_not_gated` | PASS |
| PC-013 | Unit | Implement.is_gated() returns false | AC-001.6 | `cargo test -p ecc-domain implement_is_not_gated` | PASS |
| PC-014 | Unit | Done.is_gated() returns false | AC-001.6 | `cargo test -p ecc-domain done_is_not_gated` | PASS |
| PC-015 | Unit | Phase-gate passes for idle state | AC-001.7, AC-002.1 | `cargo test -p ecc-workflow phase_gate_passes_for_idle` | PASS |
| PC-016 | Unit | Phase-gate: invalid type -> warn | AC-002.2 | `cargo test -p ecc-workflow phase_gate_corrupt_type_warns` | PASS |
| PC-017 | Unit | Phase-gate: missing phase -> warn | AC-002.3 | `cargo test -p ecc-workflow phase_gate_missing_phase_warns` | PASS |
| PC-018 | Unit | Phase-gate: plan + Write -> block | AC-002.4 | `cargo test -p ecc-workflow phase_gate_plan_blocks_write` | PASS |
| PC-019 | Unit | Phase-gate: unknown variant -> warn | AC-002.5 | `cargo test -p ecc-workflow phase_gate_unknown_variant_warns` | PASS |
| PC-020 | Unit | Reset archives + writes idle | AC-005.1 | `cargo test -p ecc-workflow reset_archives_and_writes_idle` | PASS |
| PC-021 | Unit | Reset archives done states | AC-005.2 | `cargo test -p ecc-workflow reset_archives_done_state` | PASS |
| PC-022 | Unit | Reset creates archive dir | AC-005.3 | `cargo test -p ecc-workflow reset_creates_archive_dir` | PASS |
| PC-023 | Unit | Reset with no state -> pass | AC-005.4 | `cargo test -p ecc-workflow reset_no_state_passes` | PASS |
| PC-024 | Unit | Reset acquires state lock | AC-005.6 | `cargo test -p ecc-workflow reset_acquires_state_lock` | PASS |
| PC-025 | Unit | archive_state: include_done=true archives done | AC-005.2 | `cargo test -p ecc-workflow archive_state_includes_done` | PASS |
| PC-026 | Unit | archive_state: include_done=false skips done | AC-005.2 | `cargo test -p ecc-workflow archive_state_skips_done` | PASS |
| PC-027 | Unit | Transition memory fail -> warn exit 0 | AC-004.1 | `cargo test -p ecc-workflow transition_memory_fail_warns` | PASS |
| PC-028 | Unit | Each memory error captured individually | AC-004.2 | `cargo test -p ecc-workflow transition_captures_each_memory_error` | PASS |
| PC-029 | Unit | Transition success -> no warnings | AC-004.3 | `cargo test -p ecc-workflow transition_success_no_warnings` | PASS |
| PC-030 | Unit | State persists after memory failure | AC-004.4 | `cargo test -p ecc-workflow state_persists_after_memory_failure` | PASS |
| PC-031 | Integration | Full forward lifecycle | AC-003.1 | `cargo test -p ecc-integration-tests workflow_lifecycle_forward` | PASS |
| PC-032 | Integration | Reset after done -> idle | AC-003.2 | `cargo test -p ecc-integration-tests workflow_lifecycle_reset` | PASS |
| PC-033 | Integration | Re-init after reset | AC-003.3 | `cargo test -p ecc-integration-tests workflow_lifecycle_reinit` | PASS |
| PC-034 | Integration | Illegal transition rejected | AC-003.4 | `cargo test -p ecc-integration-tests workflow_lifecycle_illegal` | PASS |
| PC-035 | Integration | Artifact timestamps recorded | AC-003.5 | `cargo test -p ecc-integration-tests workflow_lifecycle_artifacts` | PASS |
| PC-036 | Unit | Archive failure blocks reset (fail-safe) | AC-005.5 | `cargo test -p ecc-workflow reset_archive_failure_blocks` | PASS |
| PC-037 | Unit | scope_check handles Idle phase | AC-001.1 | `cargo test -p ecc-workflow scope_check_idle_passes` | PASS |
| PC-038 | Unit | grill_me_gate handles Idle phase | AC-001.1 | `cargo test -p ecc-workflow grill_me_gate_idle_passes` | PASS |
| PC-039 | Unit | stop_gate treats Idle like Done (no warning) | AC-001.1 | `cargo test -p ecc-workflow stop_gate_idle_no_warning` | PASS |
| PC-040 | Build | Clippy zero warnings | All | `cargo clippy -- -D warnings` | exit 0 |
| PC-041 | Build | Full test suite passes | All | `cargo test` | exit 0 |
| PC-042 | Build | Release build succeeds | All | `cargo build --release` | exit 0 |

## TDD Order

Phases and PCs are already ordered by dependency:

1. **Phase 1 (Domain - Idle):** PC-001 through PC-014 (RED), implement, (GREEN), refactor
2. **Phase 2 (Adapter - Phase-Gate):** PC-015 through PC-019 (RED), implement, (GREEN), refactor. Also add Idle arms to scope_check, grill_me_gate, stop_gate: PC-037, PC-038, PC-039
3. **Phase 3 (Adapter - Reset/Archive):** PC-020 through PC-026, PC-036 (RED), implement, (GREEN), refactor
4. **Phase 4 (Adapter - Transition Warnings):** PC-027 through PC-030 (RED), implement, (GREEN), refactor
5. **Phase 5 (Integration - E2E Lifecycle):** PC-031 through PC-035 (RED), implement, (GREEN), refactor
6. **Final Gate:** PC-040, PC-041, PC-042

Within each phase, write all tests first (they will fail), then implement to pass them, then refactor.

## E2E Assessment

- **Touches user-facing flows?** Yes -- reset command, phase-gate hook, transition command
- **Crosses 3+ modules end-to-end?** Yes -- ecc-domain (phase/transition) -> ecc-workflow (commands) -> filesystem (state.json/archive)
- **New E2E tests needed?** Yes
- **E2E scenarios** (Phase 5):
  1. Full forward lifecycle: init -> plan -> solution -> implement -> done
  2. Reset after done -> archive + idle
  3. Re-init after reset -> plan
  4. Illegal transition from plan -> implement rejected
  5. Artifact timestamps recorded as ISO 8601

## Risks and Mitigations

- **Risk:** Adding Idle variant changes serde behavior for existing state.json files.
  - Mitigation: Existing files never contain "idle" -- they only have plan/solution/implement/done. The enum addition is backwards-compatible for deserialization. Verified by examining `#[serde(rename_all = "lowercase")]`.

- **Risk:** Phase-gate warn-on-corrupt-state could mask real errors in CI.
  - Mitigation: The warn output includes the error message for diagnosis. The spec explicitly requires warn (exit 0), not block, for corrupt state -- this prevents pipeline lockout from unrelated state corruption.

- **Risk:** Extracting `archive_state` to `io.rs` could break init if the behavior subtly differs.
  - Mitigation: The `include_done` boolean parameter preserves exact behavioral parity. Unit tests cover both true/false paths.

- **Risk:** Memory write error surfacing could produce noisy output in normal operation.
  - Mitigation: Warnings only appear when writes actually fail. Success path produces zero warnings (AC-004.3). Memory dir is auto-created on first write.

- **Risk:** Reset acquires lock but archive_state does filesystem I/O while holding it.
  - Mitigation: Archive is a rename (atomic on same filesystem), not a copy. Lock hold time is minimal.

## Success Criteria

- [ ] Phase enum has 5 variants: Idle, Plan, Solution, Implement, Done
- [ ] `Phase::is_gated()` returns true for Plan and Solution only
- [ ] Done->Idle and Idle->Plan transitions work; Idle->other blocked
- [ ] Phase-gate uses `WorkflowState::from_json()`, not manual JSON parsing
- [ ] Corrupt state.json produces warn, not block, from phase-gate
- [ ] Reset archives state.json, writes Idle state, acquires lock
- [ ] Memory write errors produce individual warnings, exit 0
- [ ] E2E lifecycle test covers full forward path, reset, re-init, illegal, artifacts
- [ ] `cargo clippy -- -D warnings` passes with zero warnings
- [ ] `cargo test` passes (all existing + new tests)
- [ ] `cargo build --release` succeeds
