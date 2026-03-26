---
description: "Design the technical solution from the spec — Phase 2 of the pipeline. Doc-first with architecture preview."
allowed-tools: [Bash, Task, Read, Grep, Glob, LS, Write, TodoWrite, TodoRead, EnterPlanMode, ExitPlanMode, AskUserQuestion]
---

# Design Command

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Do NOT directly edit `.claude/workflow/state.json`.** State transitions happen via hooks only.
>
> **Narrative**: See `skills/narrative-conventions/SKILL.md` conventions. Before each agent delegation, gate check, and phase transition, tell the user what is happening and why.

## Phase 0: State Validation

1. Read `.claude/workflow/state.json`
2. Verify `phase` is `"plan"` or `"solution"` (re-entry allowed). If this gate blocks, explain what failed and provide specific remediation steps. If any other phase → error:
   > "Current phase is `<phase>`. `/design` requires phase `plan`/`spec` or `solution`/`design`. Run the appropriate `/spec-*` command first."
3. **Read spec from file if available**: If `artifacts.spec_path` exists in state.json, read the spec from that file path. If the file's modification time differs from the `artifacts.plan` timestamp, emit a warning: "Spec file was modified since the spec phase. Using file version." If the file does not exist, fall back to step 4.
4. If the spec is not in conversation context AND not available from file → ask the user:
   > "Spec not found in conversation context or on disk. Please re-run the `/spec-*` command or paste the spec output here."
5. Extract `concern` and `feature` from `state.json` for the solution header
6. **Re-entry**: If `phase` is `"solution"`, read existing TodoWrite items via TodoRead to resume progress

> **Tracking**: Create a TodoWrite checklist for this command's phases. If TodoWrite is unavailable, proceed without tracking — the workflow executes identically.

TodoWrite items:
- "Phase 0: State Validation"
- "Phase 1: Implementation Design"
- "Phase 2: SOLID Validation"
- "Phase 3: Professional Conscience"
- "Phase 4: Security Quick-Check"
- "Phase 5: E2E Boundary Detection"
- "Phase 6: Doc Update Plan"
- "Phase 7: AC Coverage Verification"
- "Phase 8: Output Solution"
- "Phase 9: Adversarial Review"
- "Phase 10: Present and STOP"

Mark each item complete as the phase finishes.

## Phase 1: Implementation Design

Launch a Task with the `planner` agent (allowedTools: [Read, Grep, Glob, Bash]):

- Pass the full spec content. If not in conversation context, read from `artifacts.spec_path` in state.json (disk fallback).
- Instruct the agent to:
  1. Design file changes in dependency order (what to create, modify, or delete)
  2. Map each file change to its spec reference (US-NNN, AC-NNN.N)
  3. For each change, provide a rationale explaining why the change is needed
  4. Define pass conditions (PC-NNN) — each with:
     - Type: unit, integration, e2e, lint, or build
     - Description of what is verified
     - Which AC(s) it verifies
     - A literal bash command runnable verbatim
     - Expected result (PASS, exit 0, or specific output)
  5. Order PCs in TDD dependency order (what to implement first)
  6. Final PCs must include lint and build checks
- Collect the output: File Changes table + Pass Conditions table + TDD order

> **Optional**: For specs involving new ports, adapters, or public interfaces, consider invoking the `interface-designer` agent (optional) to explore radically different interface shapes before committing to a design. This spawns parallel sub-agents with divergent constraints and produces a comparison matrix. See the `design-an-interface` skill for methodology.

> **Preview for alternatives**: When the planner produces 2+ viable design approaches, use `AskUserQuestion` with the `preview` field to present each approach visually. Each option's preview should contain a Mermaid component diagram, file-change summary, or code structure comparison. If only one viable approach exists, proceed directly without injecting a forced AskUserQuestion.

## Phase 2: SOLID Validation

> Before dispatching, tell the user which validation agent is being launched (`uncle-bob`) and that it will evaluate the design against SOLID and Clean Architecture principles.

Launch a Task with the `uncle-bob` agent (allowedTools: [Read, Grep, Glob]) with `context: "fork"` (summary output sufficient):

- Pass the proposed file changes from Phase 1 as context
- Instruct the agent to evaluate the design against:
  - SOLID principles (SRP, OCP, LSP, ISP, DIP)
  - Clean Architecture dependency rules
  - Component principles (REP, CCP, CRP, ADP, SDP, SAP)
- Collect the output: PASS or findings with file references and severity

## Phase 3: Professional Conscience

Launch a Task with the `robert` agent (allowedTools: [Read, Grep, Glob, Bash]) with `context: "fork"` (summary output sufficient):

- Pass the spec content from conversation AND the proposed design from Phase 1
- Instruct the agent to evaluate the design against the Programmer's Oath
- Focus on: no harmful code, no mess, proof (test coverage planned), small releases
- Collect the output: CLEAN or warnings with oath references

## Phase 4: Security Quick-Check

Launch a Task with the `security-reviewer` agent (allowedTools: [Read, Grep, Glob, Bash]) with `context: "fork"` (summary output sufficient):

- Pass the proposed file changes from Phase 1 as context
- This is a quick design-level scan, NOT a full audit (that happens during `/verify`)
- Focus on: input validation boundaries, auth concerns, secret handling, injection surfaces
- Collect the output: CLEAR or findings with severity

## Phase 5: E2E Boundary Detection

1. Read the spec's `## E2E Boundaries Affected` table. If not in conversation context, read from the spec file on disk via `artifacts.spec_path`
2. Scan Phase 1 file changes for any port or adapter touches (files in `crates/ecc-ports/`, `crates/ecc-infra/`, or adapter-layer paths)
3. Expand each boundary into concrete E2E test entries:

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|

- **Default State**: `ignored` (E2E tests are ignored by default, un-ignored only when relevant)
- **Run When**: condition that activates the test (e.g., "FileSystem adapter modified", "CLI output format changed")

4. Produce E2E Activation Rules — which specific E2E tests to un-ignore for THIS implementation based on the file changes

## Phase 6: Doc Update Plan

1. Read the spec's `## Doc Impact Assessment` table. If not in conversation context, read from the spec file on disk via `artifacts.spec_path`
2. Expand each entry into a concrete doc action:

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|

3. MUST include a `CHANGELOG.md` entry (even if minimal)
4. MUST include an ADR entry for any decision marked `ADR Needed? Yes` in the spec's `## Decisions Made` table
5. Reference the spec's US/AC for each doc action

## Phase 7: AC Coverage Verification

> After computing the coverage result, summarize it conversationally: how many ACs are covered, how many are uncovered, and what action is needed.

This is the critical gate — every acceptance criterion must be testable.

1. Collect ALL `AC-NNN.N` identifiers from the spec. If not in conversation context, read from the spec file on disk via `artifacts.spec_path`
2. Collect ALL `PC-NNN` pass conditions from the Phase 1 design
3. For each AC, verify it appears in at least one PC's "Verifies AC" column
4. List any uncovered ACs with an explanation
5. If uncovered ACs exist, add PCs to cover them before proceeding
6. The result SHOULD be zero uncovered ACs

## Phase 8: Architecture Preview (Plan Mode)

> **BLOCKING**: You MUST call `EnterPlanMode` here. NEVER skip this phase.

1. Call `EnterPlanMode`
2. Write the plan file with the following structure:

```markdown
# Design Preview: <title>

## Design Summary

<Brief summary of the technical design: file changes, key patterns, TDD order>

## Architecture Preview

Draft of implementation-level doc updates:

### ARCHITECTURE.md changes
<if applicable, show specific sections that will be updated with the new design>

### Mermaid diagrams
<outline any new or modified diagrams — show the Mermaid source>

### Bounded context changes
<if applicable, show updates to docs/domain/bounded-contexts.md>

### Module summaries
<if applicable, list new or modified module summary entries>

## Pass Conditions Overview
<table of PC IDs, descriptions, and commands — for user to review before approving>
```

3. Call `ExitPlanMode` — wait for user approval before proceeding

## Phase 9: Output Design

Output the full design in conversation using the exact schema below. Do NOT write `.claude/workflow/solution.md`. Every section is mandatory.

```markdown
# Solution: <title from spec>

## Spec Reference
Concern: <from state.json>, Feature: <from state.json>

## File Changes (dependency order)
| # | File | Action (create/modify/delete) | Rationale | Spec Ref (US/AC) |
|---|------|-------------------------------|-----------|------------------|
| 1 | ... | ... | ... | US-001, AC-001.1 |

## Pass Conditions
| ID | Type (unit/integration/e2e/lint/build) | Description | Verifies AC | Command | Expected |
|----|----------------------------------------|-------------|-------------|---------|----------|
| PC-001 | unit | ... | AC-001.1 | `cargo test ...` | PASS |

### Coverage Check
Every AC-NNN.N from the spec MUST appear in at least one PC's "Verifies AC" column.
<list of ACs and their covering PCs, or "All ACs covered.">
<list any uncovered ACs with explanation — should be zero>

### E2E Test Plan
| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|

### E2E Activation Rules
<which e2e tests to run un-ignored during THIS implementation>

## Test Strategy
TDD order: which PCs to implement first (dependency order).
<ordered list of PC-NNN with rationale for ordering>

## Doc Update Plan
| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
Must include CHANGELOG.md.
Must include ADRs for decisions marked "ADR Needed? Yes".

## SOLID Assessment
<from uncle-bob — PASS or findings with file references>

## Robert's Oath Check
<from robert — CLEAN or warnings>

## Security Notes
<from security-reviewer — CLEAR or findings>

## Rollback Plan
<reverse dependency order of File Changes — if implementation fails, undo in this order>
```

The solution is output in conversation only — no file is written.

## Phase 10: Adversarial Review

Launch a Task with the `solution-adversary` agent (allowedTools: [Read, Bash, Grep, Glob]):

- Pass the full spec AND solution. If not in conversation context, read spec from `artifacts.spec_path` and design from the current output on disk
- The agent attacks the solution on 8 dimensions: coverage, order, fragility, rollback, architecture, blast radius, missing PCs, doc plan
- The agent returns a verdict in conversation (no file writes)

### Verdict Handling (max 3 rounds)

Track the current round number (starting at 1):

- **FAIL**: Present the adversary's findings to the user. Return to **Phase 1 (Implementation Design)** to redesign. Re-run Phases 2-8 with the updated design, then re-run the adversary (Phase 9). Increment round.
- **CONDITIONAL**: The adversary has suggested specific PCs to add or doc plan fixes. Update the solution in conversation. Re-run the adversary. Increment round.
- **PASS**: Note "Adversarial Review: PASS" in conversation output. Then persist the design (see below). Run: `!bash .claude/hooks/phase-transition.sh implement solution <design_file_path>`. Proceed to Phase 11.

After 3 FAIL rounds, ask the user:
> "The solution has failed adversarial review 3 times. Would you like to override and proceed anyway, or abandon?"
- If override: note "Adversarial Review: PASS (user override)" in conversation, persist the design, run `!bash .claude/hooks/phase-transition.sh implement solution <design_file_path>`, and proceed
- If abandon: reset state to `"plan"` phase and exit

### Persist Design to File

After adversarial PASS (or user override), write the design to a versioned file:

1. Read `artifacts.spec_path` from state.json to determine the spec directory (e.g., `docs/specs/2026-03-21-my-feature/`)
2. Write the full design to `docs/specs/YYYY-MM-DD-<slug>/design.md` in the same directory as the spec
3. If the file already exists (re-entry), append a `## Revision` block with timestamp instead of overwriting
4. Pass the file path to the phase-transition command as the 3rd argument
5. Update campaign.md: set Design row in `## Artifacts` table to the design file path with status `passed`.

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Adversary History Tracking section for campaign.md verdict persistence during design adversarial review.

## Phase 11: Present and STOP

Display a comprehensive Phase Summary using these tables:

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | PASS/findings | <count> |
| Robert | CLEAN/warnings | <count> |
| Security | CLEAR/findings | <count> |

### Adversary Findings

| Dimension | Verdict | Key Rationale |
|-----------|---------|---------------|
| <dimension> | PASS/FAIL/CONDITIONAL | <rationale> |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | <file> | create/modify/delete | US-NNN, AC-NNN.N |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/YYYY-MM-DD-<slug>/design.md | Full design |

### Phase Summary Persistence

Append a `## Phase Summary` section containing all 4 tables above to the persisted design file (`docs/specs/YYYY-MM-DD-<slug>/design.md`). If `## Phase Summary` already exists in the design file, overwrite it (idempotent).

> **Note:** If continuing in a new session, copy the spec and solution recaps above or re-run the commands.

Then STOP. Say:

> **Run `/implement` to begin.**

Do NOT proceed to implementation. Do NOT write any code.

## Pass Condition Rules

These rules govern all PCs written in the solution:

1. **Format**: `PC-NNN` — three digits, sequential starting at 001
2. **Command**: every PC has a literal `Command` column — a bash command runnable verbatim (no placeholders, no pseudo-commands)
3. **Expected**: every PC has an `Expected` column — `PASS`, `exit 0`, or specific expected output
4. **Coverage**: every AC from the spec is covered by >= 1 PC (enforced by Phase 7)
5. **Final PCs**: the last PCs must include lint check (`cargo clippy -- -D warnings` or equivalent) and build check (`cargo build` or equivalent)
6. **Deterministic**: PCs must be verifiable by running the command and checking the expected output — no subjective criteria

## Related Agents

This command invokes:
- `planner` — Implementation design, file changes, pass conditions, TDD order
- `uncle-bob` — SOLID and Clean Architecture validation of proposed design
- `robert` — Programmer's Oath evaluation of the design process
- `security-reviewer` — Quick security scan of the design surface
- `solution-adversary` — Adversarial solution review on 8 dimensions before phase transition
