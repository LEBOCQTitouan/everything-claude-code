---
description: "Implement the solution — Phase 3. Deterministic TDD loop with mandatory doc updates."
allowed-tools: [Bash, Task, Read, Write, Edit, MultiEdit, Grep, Glob, LS, TodoWrite, TodoRead, EnterPlanMode, ExitPlanMode, TaskCreate, TaskUpdate, TaskGet, TaskList]
---

# Implement Command

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Do NOT directly edit `.claude/workflow/state.json`.** State transitions happen via hooks only.

## Phase 0: State Validation

1. Read `.claude/workflow/state.json`
2. Verify `phase` is `"solution"` or `"implement"` (re-entry allowed). If any other phase → error:
   > "Current phase is `<phase>`. `/implement` requires phase `solution`. Run `/solution` first."
3. **Read spec and design from files if available**: If `artifacts.spec_path` exists in state.json, read the spec from that file. If `artifacts.design_path` exists, read the design from that file. If either file's modification time differs from its artifact timestamp, emit a warning: "File was modified since the original phase. Using file version." If a file path is set but the file does not exist on disk, fall back to step 4.
4. If the spec or design is not in conversation context AND not available from file → ask the user:
   > "Spec and/or design not found in conversation context or on disk. Please re-run `/spec-*` and `/design` or paste the outputs here."
5. Extract `concern` and `feature` from `state.json` for the implementation header
6. **Re-entry**: If `phase` is `"implement"`, read existing TodoWrite items via TodoRead to resume progress from the last completed PC
7. Run: `!bash .claude/hooks/phase-transition.sh implement`

## Phase 1: Enter Plan Mode

> **BLOCKING**: You MUST call `EnterPlanMode` before any implementation work. NEVER skip Plan Mode — not to "save context", not for "simple changes", not for any reason. This is non-negotiable.

1. You MUST call `EnterPlanMode`
2. Write the plan file with the following structure, using the spec and solution from conversation context:

```markdown
# Implementation Plan: <title>

## Spec (from /plan)
<full spec from conversation>

## Solution (from /solution)
<full solution from conversation>

## Checklist
<PC items from solution's Test Strategy, in TDD order>
- [ ] PC-NNN: <Description> (RED → GREEN → REFACTOR)
- ...
- [ ] E2E tests (if activated)
- [ ] Code review
- [ ] Doc updates
- [ ] Write implement-done.md
```

3. Call `ExitPlanMode`

## Phase 2: Parse Solution

Extract from the Solution section of the plan file:

1. **Pass Conditions table** — all `PC-NNN` rows with: ID, Type, Description, Verifies AC, Command, Expected
2. **Test Strategy** — the TDD ordering of PCs
3. **E2E Test Plan** and **E2E Activation Rules** — which E2E tests to run
4. **Doc Update Plan** — all doc actions to execute
5. **File Changes table** — for reference during implementation

Create a TodoWrite checklist from the PCs in TDD order:
- `[ ] PC-NNN: <Description> (RED → GREEN → REFACTOR)`
- `[ ] E2E tests (if activated)`
- `[ ] Code review`
- `[ ] Doc updates`
- `[ ] Write implement-done.md`

Also create native tasks via `TaskCreate` for each PC in TDD order. Each task should have:
- **subject**: `PC-NNN: <Description>`
- **description**: The PC's full details (type, AC, command, expected)
- **activeForm**: `Implementing PC-NNN`

Use `TaskUpdate` to mark each task `in_progress` when starting and `completed` when the PC passes. This provides spinner UX and persists across context compaction.

## Phase 3: TDD Loop

For each PC in the order specified by Test Strategy, execute the RED → GREEN → REFACTOR cycle:

### RED

1. Write the test. The test function name and assertion MUST match the PC's Description.
2. Run the PC's Command column **VERBATIM** — do not paraphrase or modify the command.
3. The test MUST FAIL.
   - If it passes → the feature already exists or the test is wrong. Investigate before proceeding.
   - If the command errors for a reason unrelated to the assertion (e.g., compile error), fix the compilation issue and re-run.
4. Record: `PC-NNN RED: <test name> fails as expected`
5. You MUST commit immediately: `test: add <PC description> (PC-NNN)` — do not defer, do not batch, do not ask the user

### GREEN

1. Write the **MINIMUM** code to make this PC's test pass.
2. Do NOT write code for other PCs. One PC at a time.
3. Run the PC's Command. It MUST PASS.
4. Run ALL previously passing PCs' Commands. ALL MUST PASS (no regressions).
   - If a previous PC regresses → fix the regression before proceeding.
5. Record: `PC-NNN GREEN: passes, all N previous PCs pass`
6. You MUST commit immediately: `feat: implement <PC description> (PC-NNN)` — do not defer, do not batch, do not ask the user

### REFACTOR

1. If the code can be cleaned (extract, rename, simplify), do it.
2. Run ALL PCs completed so far. ALL MUST PASS.
3. If no refactoring needed, skip with: `PC-NNN REFACTOR: no refactor needed`
4. Otherwise record: `PC-NNN REFACTOR: cleaned, all pass`
5. If refactored, you MUST commit immediately: `refactor: clean <PC description> (PC-NNN)` — do not defer, do not batch, do not ask the user

### Loop Invariant

- If a test cannot go green without breaking a previous test → **STOP**. Report the conflict to the user with both PC IDs and the nature of the conflict. Do not proceed.
- Never modify a test to make it pass — modify the implementation.
- Exception: the test was wrong per the spec → fix the test AND document why in the TDD Log Notes column.

### Loop Completion

After ALL PCs complete:
1. Run every PC's Command one final time. Record results.
2. Run the lint PC (e.g., `cargo clippy -- -D warnings`). Must pass.
3. Run the build PC (e.g., `cargo build`). Must pass.
4. Update the TodoWrite checklist: mark all PC items complete.

## Phase 4: E2E Tests

Read the solution's `## E2E Activation Rules`:

- If no E2E tests are activated → record: "No E2E tests required by solution"
- If E2E tests are activated:
  1. Un-ignore each activated E2E test
  2. Run each test and record results
  3. If any E2E test fails → fix and re-run. All must pass.
  4. Commit: `test(e2e): add <boundary> E2E tests`

## Phase 5: Code Review

Launch a Task with the `code-reviewer` agent:

- Pass the full list of files changed during the TDD loop
- Pass the spec (from the Spec section of the plan file) as the requirement reference
- Pass the solution (from the Solution section of the plan file) as the design reference
- Agent reviews for: quality, security, maintainability, spec compliance
- Collect findings

If CRITICAL or HIGH findings:
1. Fix each finding
2. Re-run all PCs to verify no regressions
3. Commit each fix: `fix: address review finding — <description>`

Max 2 fix rounds. If CRITICAL/HIGH findings persist after 2 rounds, report to user and proceed.

Record: code review summary (PASS or findings addressed)

## Phase 6: Doc Updates

Execute every row from the solution's `## Doc Update Plan`. Doc updates are part of implementation — they happen BEFORE writing implement-done.md.

### Doc Level Rules

Apply these rules based on the doc target:

- **README.md**: keep short, link out to detailed docs
- **CLAUDE.md**: reductive — high-signal, no redundancy with code
- **ARCHITECTURE.md**: intent and contracts only, no implementation details
- **ADRs** (`docs/adr/`): Status / Context / Decision / Consequences format
- **docs/domain/**: update glossary and bounded contexts
- **docs/runbooks/**: update operational procedures
- **CHANGELOG.md**: always required — even if minimal
- **Inline doc-comments**: for volatile details that belong near the code

### ADR Creation

For each decision marked `ADR Needed? Yes` in the spec's Decisions table:
1. Create `docs/adr/NNN-<slug>.md` using the standard ADR format:
   ```
   # NNN. <Decision Title>

   Date: YYYY-MM-DD

   ## Status
   Accepted

   ## Context
   <why this decision was needed>

   ## Decision
   <what was decided>

   ## Consequences
   <positive and negative impacts>
   ```
2. Commit: `docs(adr): add ADR NNN — <decision title>`

### Other Doc Updates

For each remaining row in the Doc Update Plan:
1. Apply the doc update
2. You MUST commit immediately: `docs: update <target> for <feature>`

For CHANGELOG.md (always required):
1. Add the feature entry
2. You MUST commit immediately: `docs(changelog): add <feature> entry`

## Phase 7: Write implement-done.md

Write `.claude/workflow/implement-done.md` using the exact schema below. Every section is mandatory.

```markdown
# Implementation Complete: <title>

## Spec Reference
Concern: <from state.json>, Feature: <from state.json>

## Changes Made
| # | File | Action | Solution Ref | Tests | Status |
|---|------|--------|--------------|-------|--------|
| 1 | ... | create/modify/delete | PC-001 | test_name | done |

## TDD Log
| PC ID | RED | GREEN | REFACTOR | Notes |
|-------|-----|-------|----------|-------|
| PC-001 | ✅ fails as expected | ✅ passes, 0 regressions | ✅ cleaned | — |
| PC-002 | ✅ fails as expected | ✅ passes, 1 previous PC passes | ⏭ no refactor needed | — |

## Pass Condition Results
| PC ID | Command | Expected | Actual | Status |
|-------|---------|----------|--------|--------|
| PC-001 | `cargo test ...` | PASS | PASS | ✅ |

All pass conditions: N/N ✅

## E2E Tests
| # | Test | Boundary | Result | Notes |
|---|------|----------|--------|-------|
(or "No E2E tests required by solution")

## Docs Updated
| # | Doc File | Level | What Changed |
|---|----------|-------|--------------|
| 1 | CHANGELOG.md | project | Added <feature> entry |
(MUST include CHANGELOG.md at minimum)

## ADRs Created
| # | File | Decision |
|---|------|----------|
(or "None required")

## Code Review
<summary — PASS or findings addressed>

## Suggested Commit
<type>(<scope>): <description>
```

After writing, run: `!bash .claude/hooks/phase-transition.sh done implement`

Commit: `chore: write implement-done.md`

## Phase 8: Final Verification and STOP

Verify stop hook requirements are met:

1. **stop-gate**: `state.json` phase is `"done"` ✅
2. **doc-enforcement**: `## Docs Updated` section has entries (not empty) ✅
3. **pass-condition-check**: `## Pass Condition Results` has all ✅, no ❌, E2E tests have `#[ignore]` ✅
4. **e2e-boundary-check**: if solution had `## E2E Test Plan` entries, `## E2E Tests` section exists ✅
5. **scope-check**: review any warnings about files changed outside the solution's File Changes table. If unexpected files were flagged, verify they are legitimate (test helpers, lock files, etc.) before proceeding.
6. **doc-level-check**: review any warnings about doc size limits (CLAUDE.md < 200 lines, README < 300 lines, ARCHITECTURE.md code blocks < 20 lines). Address if practical.

Display a summary:
- **Title**: from the spec
- **PCs passed**: N/N
- **E2E tests**: N passed or "none required"
- **Docs updated**: count
- **ADRs created**: count or "none"
- **Code review**: PASS or findings addressed
- **Suggested commit**: the message from implement-done.md

Then STOP. The workflow is complete.

## Constraints

- You MUST enter Plan Mode (EnterPlanMode) in Phase 1 — NEVER skip it for any reason
- The TDD loop is the ONLY way code gets written — no code outside the loop
- Every PC Command is run VERBATIM — no paraphrasing, no modification
- You MUST commit immediately after every RED, GREEN, REFACTOR, and doc update step — never defer commits, never batch multiple steps into one commit, never ask the user whether to commit
- Doc updates happen BEFORE writing implement-done.md (they are part of implementation, not an afterthought)
- implement-done.md schema is EXACT — stop hooks parse it
- One PC at a time — never batch multiple PCs

## Related Agents

This command invokes:
- `tdd-guide` — assists with complex TDD cycles during the RED → GREEN → REFACTOR loop
- `code-reviewer` — reviews all changes against spec and solution after TDD loop completes
