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

### Generate tasks.md

Persist a session-independent task tracker alongside the spec and design artifacts in `docs/specs/YYYY-MM-DD-<slug>/tasks.md`:

1. Read `artifacts.spec_path` from `state.json`. If `spec_path` is null or the spec directory is not available, emit a warning: "Spec directory not available. tasks.md generation skipped." and skip this subsection.
2. Derive the spec directory from `spec_path` (e.g., `docs/specs/2026-03-21-my-feature/`)
3. Write `tasks.md` in that directory using this exact format:

```markdown
# Tasks: <feature title>

## Pass Conditions

- [ ] PC-001: <description> | `<command>` | pending@<ISO 8601 timestamp>
- [ ] PC-002: <description> | `<command>` | pending@<ISO 8601 timestamp>
...

## Post-TDD

- [ ] E2E tests | pending@<ISO 8601 timestamp>
- [ ] Code review | pending@<ISO 8601 timestamp>
- [ ] Doc updates | pending@<ISO 8601 timestamp>
- [ ] Write implement-done.md | pending@<ISO 8601 timestamp>
```

4. Store `artifacts.tasks_path` in state.json: run `!bash .claude/hooks/phase-transition.sh implement implement <tasks_path>`
5. Commit: `docs: write tasks.md for <feature>`

Status updates during the TDD loop (Phase 3) append to each line's trail:
- Dispatch: append `| red@<ISO 8601 timestamp>`
- Subagent success: append `| green@<ISO 8601 timestamp>`
- Regression verification passes: append `| done@<ISO 8601 timestamp>` and mark `[x]`
- Failure: append `| failed@<ISO 8601 timestamp> ERROR: <summary>`

## Phase 3: TDD Loop (Subagent Dispatch)

For each PC in the order specified by Test Strategy, dispatch to an isolated `tdd-executor` subagent. PCs are dispatched **sequentially** — one at a time, never in parallel. Each subagent gets a fresh context window.

### Context Brief Construction

For each PC, build a context brief with these exact headings (max 500 lines total). The brief MUST NOT include the full spec content, full design content, prior PC implementation reasoning, or Phase 0-2 context — only the PC-specific slice.

#### ## PC Spec

The PC's verbatim fields: ID, Type, Description, Verifies AC, Command, Expected. When spec/design file paths are null in state.json (BL-029 not active), include the PC's verbatim fields (ID, type, description, command, expected, verifies AC text) inline here instead of referencing files.

#### ## File Paths

- `spec_path`: from `state.json` `artifacts.spec_path` (nullable)
- `design_path`: from `state.json` `artifacts.design_path` (nullable)

#### ## Files to Modify

List of files this PC should modify, filtered from the solution's File Changes table by this PC's spec ref.

#### ## Prior PC Results

Summary table of all previously completed PCs (max 5 lines per PC):

| PC ID | Status | Files Changed |
|-------|--------|---------------|

#### ## Commit Rules

The commit message templates for RED, GREEN, REFACTOR.

#### ## TDD Cycle Rules

The RED-GREEN-REFACTOR instructions from the `tdd-executor` agent.

### Per-PC Subagent Dispatch

For each PC (sequential, in Test Strategy order):

1. Build the context brief using the headings above
2. Launch a Task with the `tdd-executor` agent (allowedTools: [Read, Write, Edit, MultiEdit, Bash, Grep, Glob])
3. The subagent executes RED → GREEN → REFACTOR and commits atomically
4. Receive the subagent's structured result: pc_id, status, red_result, green_result, refactor_result, commits, files_changed, error
5. If the subagent returns `RED_ALREADY_PASSES` → investigate. The feature may already exist or the test is wrong.
6. If the subagent crashes or times out after partial commits → report: subagent exit state, last commit SHA(s) via `git log -3 --oneline`, and the PC in progress. Do NOT auto-revert.
7. If the subagent returns `failure` → **STOP** and report the error to the user. If the failure message mentions a prior PC or a structural conflict, report it as a potential PC conflict (the subagent believes making its test pass necessarily breaks prior code — analogous to the old Loop Invariant). If the failure suggests a test/spec mismatch, the parent should investigate and, if confirmed, fix the test locally (with a TDD Log note) and re-dispatch the PC. Do not proceed to the next PC.

### Parent Regression Verification

After each subagent completes successfully:

1. Run every PC command from PC-001 through the just-completed PC-N (run ALL prior PCs' commands plus PC-N's command)
2. For the first PC (PC-001), skip regression check — there are no prior PCs to verify
3. Regression check MUST pass BEFORE marking PC-N as complete — if any prior PC fails, PC-N is NOT marked complete
4. If a regression is detected → **STOP** and report: the regressing PC ID, the failing command, actual vs expected output, and the PC-N that caused it. Do not proceed.

### Progress Tracking (Parent-Owned)

After regression verification passes:

1. Update TodoWrite to mark PC-NNN as complete
2. Call TaskUpdate to mark PC-N's task as completed
3. If the subagent failed, do NOT mark the PC as complete — TodoWrite and Task remain in-progress
4. On re-entry (implement phase re-entry), read TodoRead and resume from the first incomplete PC

### Loop Completion (Parent-Owned)

After ALL PCs complete successfully:

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

## Subagent Execution
| PC ID | Status | Commit Count | Files Changed Count |
|-------|--------|--------------|---------------------|
| PC-001 | success | 3 | 2 |
(or "Inline execution — subagent dispatch not used" for pre-BL-031 implementations)

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
- `tdd-executor` — executes each PC's RED → GREEN → REFACTOR cycle in an isolated subagent with fresh context
- `code-reviewer` — reviews all changes against spec and solution after TDD loop completes
