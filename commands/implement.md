---
description: "Implement the solution — Phase 3. Deterministic TDD loop with mandatory doc updates."
allowed-tools: [Bash, Task, Read, Write, Edit, MultiEdit, Grep, Glob, LS, TodoWrite, TodoRead, EnterPlanMode, ExitPlanMode, TaskCreate, TaskUpdate, TaskGet, TaskList]
---

# Implement Command

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Do NOT directly edit `.claude/workflow/state.json`.** State transitions happen via hooks only.
>
> **Narrative**: See `skills/narrative-conventions/SKILL.md` conventions. Before each agent delegation, gate check, and phase transition, tell the user what is happening and why.

## Phase 0: State Validation

### Worktree Isolation

If not already in a worktree (check `git rev-parse --show-toplevel` vs `git rev-parse --git-common-dir`):
1. Read `concern` and `feature` from `state.json`
2. Run: `!ecc-workflow worktree-name <concern> "<feature>"` — capture the output name
3. Call `EnterWorktree` with the generated name as the branch name
4. If `EnterWorktree` fails, proceed without worktree and warn: "Worktree isolation failed. Proceeding on main tree."

If already in a worktree (from a prior `/spec-*` or `/design` call in this session): skip — already isolated.

1. Read `.claude/workflow/state.json`
2. Verify `phase` is `"solution"` or `"implement"` (re-entry allowed). If this gate blocks, explain what failed and provide specific remediation steps. If any other phase → error:
   > "Current phase is `<phase>`. `/implement` requires phase `solution`. Run `/design` first."
3. **Read spec and design from files if available**: If `artifacts.spec_path` exists in state.json, read the spec from that file. If `artifacts.design_path` exists, read the design from that file. If either file's modification time differs from its artifact timestamp, emit a warning: "File was modified since the original phase. Using file version." If a file path is set but the file does not exist on disk, fall back to step 4.
4. If the spec or design is not in conversation context AND not available from file → ask the user:
   > "Spec and/or design not found in conversation context or on disk. Please re-run `/spec-*` then `/design` or paste the outputs here."
5. Extract `concern` and `feature` from `state.json` for the implementation header
6. **Re-entry**: If `phase` is `"implement"`, resume using this priority:
   1. **tasks.md is the authoritative, primary resume source.** Read `artifacts.tasks_path` from state.json. If the file exists, parse it to find the first incomplete (non-done) PC as the resume point. If a PC has status `failed`, treat it as the resume point and report: "PC-NNN previously failed: <error summary>. Re-dispatching." If all PCs are done, resume from the first incomplete Post-TDD phase (E2E, review, docs, Supplemental docs, implement-done).
   2. **Rebuild TodoWrite from tasks.md.** For each entry in tasks.md, create a corresponding TodoWrite item. Mark items with status `done` as complete.
   3. **Regenerate if tasks.md deleted.** If `artifacts.tasks_path` is set but the file does not exist, regenerate tasks.md from the solution's PC table. Infer completion status using `git log --oneline --after=<started_at from state.json> --grep="PC-NNN"` — if a commit message contains the PC ID after the workflow `started_at` timestamp, mark that PC as `done`. Emit warning: "tasks.md regenerated from git history — verify accuracy."
   4. **Handle malformed tasks.md.** If tasks.md exists but cannot be parsed (malformed markdown), regenerate from the solution's PC table using the git-log inference above. Emit warning: "tasks.md was malformed; regenerated from solution."
   5. **TodoRead fallback.** If `artifacts.tasks_path` is null (BL-029 not active), fall back to reading TodoRead for resume state.
   6. **Campaign re-entry orientation.** If `artifacts.campaign_path` exists in state.json and the file exists, read campaign.md for orientation context: toolchain commands, grill-me decisions, and commit trail.
7. Run: `!ecc-workflow transition implement`

## Phase 0.5: Sources Consultation

If `docs/sources.md` exists:
1. Read `docs/sources.md` and find entries matching the module being modified (via module mapping table)
2. If matches found, surface relevant sources as reference context
3. Update `last_checked` date on matched entries
4. Write updated file back (atomic write)

If `docs/sources.md` does not exist, skip this step silently.

## Phase 1: Enter Plan Mode

> **BLOCKING**: You MUST call `EnterPlanMode` before any implementation work. NEVER skip Plan Mode — not to "save context", not for "simple changes", not for any reason. This is non-negotiable.

1. You MUST call `EnterPlanMode`
2. Write the plan file with the following structure, using the spec and solution from conversation context:

```markdown
# Implementation Plan: <title>

## Spec (from /spec)
<full spec from conversation>

## Solution (from /design)
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
- `[ ] Supplemental docs`
- `[ ] Write implement-done.md`

Also create native tasks via `TaskCreate` for each PC in TDD order. Each task should have:
- **subject**: `PC-NNN: <Description>`
- **description**: The PC's full details (type, AC, command, expected)
- **activeForm**: `Implementing PC-NNN`

Use `TaskUpdate` to mark each task `in_progress` when starting and `completed` when the PC passes. This provides spinner UX and persists across context compaction.

### Generate tasks.md

> **Shared**: See `skills/tasks-generation/SKILL.md` for the full tasks.md format and status trail conventions.

Generate tasks.md in the spec directory using the tasks-generation skill's format. Store `artifacts.tasks_path` in state.json via phase-transition.sh. Commit: `docs: write tasks.md for <feature>`.

### Wave Analysis

> **Shared**: See `skills/wave-analysis/SKILL.md` for the full wave grouping algorithm — left-to-right scan, adjacent PCs with no file overlap grouped into waves, max 4 concurrent subagents per wave (cap), and degenerate cases.

After generating tasks.md, analyze the PC dependency graph and display the wave plan to the user before proceeding to Phase 3.

### Team Manifest Configuration

Before dispatching subagents, read team configuration from `teams/implement-team.md`:

1. If `teams/implement-team.md` exists, parse its YAML frontmatter for agent names, roles, allowed-tools, and `max-concurrent`
2. Use the manifest's `agents` list to construct subagent spawn parameters — each entry's `name` identifies the agent and `allowed-tools` restricts tool access
3. Use `max-concurrent` from the manifest as the wave concurrency cap (overrides the default of 4)
4. If `teams/implement-team.md` does NOT exist:
   - If `ECC_LEGACY_DISPATCH=1` is set: fall back to hard-coded agent configuration with a deprecation warning: "DEPRECATED: Using legacy dispatch. Create teams/implement-team.md to use team manifests."
   - Otherwise: fail with error: "Team manifest required: teams/implement-team.md not found. Set ECC_LEGACY_DISPATCH=1 for legacy behavior."

## Phase 3: TDD Loop (Subagent Dispatch)

For each PC in the order specified by Test Strategy, dispatch to an isolated `tdd-executor` subagent. PCs are dispatched in **waves**. Within each wave, independent PCs are dispatched concurrently. Waves are executed sequentially. If all waves contain a single PC, behavior is identical to sequential dispatch (backward compatible). Each subagent gets a fresh context window.

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

| PC ID | Status | Files Changed | test_names |
|-------|--------|---------------|------------|

Include the `test_names` list from prior PCs' results (for awareness — helps identify which tests already exist). If a prior PC did not return `test_names`, show "--".

#### ## Commit Rules

The commit message templates for RED, GREEN, REFACTOR.

#### ## TDD Cycle Rules

The RED-GREEN-REFACTOR instructions from the `tdd-executor` agent.

#### ## User Guidance

If the user provided free-text guidance after a budget-exceeded prompt (see Fix-Round Budget below), include it verbatim here. Otherwise, omit this section from the context brief.

### Fix-Round Budget

Each PC has a per-PC `fix_round_count` starting at 0. The counter is owned by the parent orchestrator, NOT by the tdd-executor. The tdd-executor has no knowledge of the budget — it continues returning `failure` as before.

**Counter rules:**
- Only GREEN phase test failures consume a fix round. RED phase compilation fixes (tdd-executor line 69) do NOT increment the counter.
- Each time a tdd-executor returns `status: failure` for a given PC, the parent increments that PC's `fix_round_count`.
- A tdd-executor crash or timeout (no structured result returned) does NOT consume a fix round. Report the crash to the user immediately without budget logic.
- If a PC succeeds within budget (fix_round_count is 1 or 2), mark it as success with a note "fixed in N rounds".

**Budget exceeded (fix_round_count reaches 2):**

When a PC's `fix_round_count` reaches 2 and the test still fails, STOP retrying and present a diagnostic report followed by AskUserQuestion.

**Diagnostic report format:**

```markdown
### Test Name
<PC-NNN: description>

### Error Output
<last 50 lines of the failing test output>

### Files Modified
- <file1>
- <file2>

### Fix Attempts
1. Round 1: <what was tried and why it failed>
2. Round 2: <what was tried and why it failed>
```

**AskUserQuestion options:**

Present via AskUserQuestion with these options:
- **"Keep trying (+2 rounds)"** — grants 2 more fix rounds and re-dispatches the tdd-executor with the same context brief
- **"Skip this PC"** — marks the PC as `failed` in tasks.md and proceeds to the next PC
- **"Abort implementation"** — preserves the current state and stops the /implement pipeline

The user may also select "Other" to provide free-text guidance. If guidance is provided, include it in the re-dispatched context brief under a `## User Guidance` heading.

**Hard cap:** The user may select "Keep trying" at most 3 times per PC, for a maximum of 8 total fix rounds (2 initial + 3×2 extensions). After 8 rounds, the only remaining options are "Skip this PC" or "Abort implementation".

### Wave Dispatch

> **Shared**: See `skills/wave-dispatch/SKILL.md` for the full wave dispatch logic — pre-wave setup with git tag `wave-N-start`, single-PC wave (backward compatible, no worktree isolation), multi-PC wave (parallel dispatch with `isolation: "worktree"`, prior waves only in context briefs), post-wave merge (sequential in PC-ID order, merge conflict detection), wave regression verification (run all PC commands from waves 1..W), and wave failure handling (let wave finish, merge successful, discard failed PCs' branches, re-derive on re-entry).

Execute wave dispatch per the skill. Accumulate commit SHA hashes for the Phase Summary. For each commit, also append the SHA and message to campaign.md's `## Commit Trail` table (parent orchestrator only, never subagents). Status updates: append `red@<timestamp>` on dispatch, `green@<timestamp>` on success, `done@<timestamp>` after regression passes.

### Progress Tracking (Parent-Owned)

> **Shared**: See `skills/progress-tracking/SKILL.md` for the full progress tracking logic — TodoWrite, TaskUpdate, tasks.md status updates, and loop completion.

### Post-TDD Coverage Measurement

After all PCs pass and before Phase 4, measure test coverage delta:

1. **Determine before-snapshot**: Use the `wave-1-start` git tag if wave dispatch was used. If no wave tag exists (sequential dispatch), use the commit SHA recorded at Phase 3 entry. If neither exists, skip with "No before-snapshot available".
2. **Run coverage**: Execute `cargo llvm-cov --workspace --json` at the before-snapshot (via `git stash && git checkout <before> && cargo llvm-cov --workspace --json > /tmp/before.json && git checkout - && git stash pop`) and at current HEAD (`cargo llvm-cov --workspace --json > /tmp/after.json`).
3. **Coverage data unavailable**: If `cargo llvm-cov` is not installed or fails (including partial failure — before succeeds but after fails), show "Coverage data unavailable — install cargo-llvm-cov" in the Phase Summary and continue. Partial data is discarded. This NEVER blocks the pipeline.
4. **Render Coverage Delta table**: If both snapshots succeed, compute per-crate delta:

| Crate | Before % | After % | Delta |
|-------|----------|---------|-------|
| ecc-domain | 85.2% | 87.1% | +1.9% |

5. Store the coverage data for inclusion in implement-done.md `## Coverage Delta` section.

## Phase 4: E2E Tests

Read the solution's `## E2E Activation Rules`:

- If no E2E tests are activated → record: "No E2E tests required by solution"
- If E2E tests are activated:
  1. Un-ignore each activated E2E test
  2. Run each test and record results
  3. If any E2E test fails, apply the same fix-round budget as Phase 3: maintain a per-test `fix_round_count`, fix and re-run. Only GREEN phase test failures (not compilation fixes) consume a round. After 2 failed fix attempts, present the diagnostic report and AskUserQuestion with the same options (Keep trying, Skip, Abort, or user guidance via Other). The hard cap of at most 3 extensions (maximum of 8 total fix rounds) applies.
  4. Commit: `test(e2e): add <boundary> E2E tests`

After E2E phase completes (whether tests ran or not), update tasks.md: set "E2E tests" entry to `done@<ISO 8601 timestamp>` and mark `[x]`.

## Phase 5: Code Review

Launch a Task with the `code-reviewer` agent:

- Pass the full list of files changed during the TDD loop
- Pass the spec (from the Spec section of the plan file) as the requirement reference
- Pass the solution (from the Solution section of the plan file) as the design reference
- Agent reviews for: quality, security, maintainability, spec compliance
- Collect findings

> After receiving the code review output, summarize what was found conversationally before applying fixes.

If CRITICAL or HIGH findings:
1. Fix each finding
2. Re-run all PCs to verify no regressions
3. Commit each fix: `fix: address review finding — <description>`

Max 2 fix rounds. If CRITICAL/HIGH findings persist after 2 rounds, report to user and proceed.

Record: code review summary (PASS or findings addressed). Also append findings summary to campaign.md's `## Agent Outputs` table (Agent: code-reviewer, Phase: implement).

After code review completes, update tasks.md: set "Code review" entry to `done@<ISO 8601 timestamp>` and mark `[x]`.

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

For each decision marked `ADR Needed? Yes` in the spec's Decisions table, create `docs/adr/NNN-<slug>.md` using the standard Status/Context/Decision/Consequences format. Commit: `docs(adr): add ADR NNN — <decision title>`.

### Other Doc Updates

For each remaining row in the Doc Update Plan:
1. Apply the doc update
2. You MUST commit immediately: `docs: update <target> for <feature>`

For CHANGELOG.md (always required):
1. Add the feature entry
2. You MUST commit immediately: `docs(changelog): add <feature> entry`

After all doc updates complete, update tasks.md: set "Doc updates" entry to `done@<ISO 8601 timestamp>` and mark `[x]`.

## Phase 7.5: Supplemental Doc Generation

Generate context-aware supplemental documentation while session context is fresh. Phase 7.5 is non-blocking — if a subagent fails, commit successful output, record the failure, and proceed to Phase 7.

### Dispatch

Launch two Task subagents in parallel:

1. **module-summary-updater** (allowedTools: [Read, Write, Edit, Grep, Glob]) — updates `docs/MODULE-SUMMARIES.md` with entries for each Rust crate modified during the TDD loop
2. **diagram-updater** (allowedTools: [Read, Write, Edit, Grep, Glob]) — generates Mermaid diagrams for new cross-module flows, state machines, or bounded contexts

Pass each subagent:
- The list of files changed during the TDD loop
- The spec and design file paths from state.json
- The feature name from state.json

Wait for both tasks to reach a terminal status (completed or failed) before proceeding.

### Result Handling

For each subagent:
- **Success**: Commit output. Module summaries: `docs: update MODULE-SUMMARIES for <feature>`. Diagrams: `docs(diagrams): add <feature> diagrams`
- **Failure (partial failure)**: Record the failure in implement-done.md `## Supplemental Docs` section. Proceed — Phase 7.5 failures are non-blocking.

### Cross-Link Fixup Pass

After both subagents complete:

1. If diagram-updater produced diagrams AND module-summary-updater produced entries, patch MODULE-SUMMARIES entries with links to related diagrams (cross-link fixup)
2. If no diagrams were produced, skip (no-op — no diagram links to add)
3. If the fixup pass made changes, commit: `docs: cross-link MODULE-SUMMARIES to diagrams for <feature>`

After Phase 7.5 completes, update tasks.md: set "Supplemental docs" entry to `done@<ISO 8601 timestamp>` and mark `[x]`.

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
| PC ID | RED | GREEN | REFACTOR | Test Names | Notes |
|-------|-----|-------|----------|------------|-------|
| PC-001 | ✅ fails as expected | ✅ passes, 0 regressions | ✅ cleaned | `module::tests::test_name` | — |
| PC-002 | ✅ fails as expected | ✅ passes, 1 previous PC passes | ⏭ no refactor needed | "--" | — |

The Test Names column contains fully qualified test names from the tdd-executor's `test_names` output field. When `test_names` is absent (older tdd-executor invocations or inline execution), show "--" for graceful degradation. The `test_names` field was added in BL-050; type: list of strings, default when absent: "--".

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

## Coverage Delta
| Crate | Before % | After % | Delta |
|-------|----------|---------|-------|
| ecc-domain | 85.2% | 87.1% | +1.9% |
(or "Coverage data unavailable — install cargo-llvm-cov" if tool is missing/failed)
(or "No before-snapshot available" if neither wave tag nor Phase 3 entry SHA exists)

## Supplemental Docs
| Subagent | Status | Output File | Commit SHA | Notes |
|----------|--------|-------------|------------|-------|
| module-summary-updater | success | docs/MODULE-SUMMARIES.md | abc1234 | -- |
| diagram-updater | success | docs/diagrams/feature-x.md | def5678 | -- |
(or "No supplemental docs generated — change scope did not warrant module summary or diagram updates")

## Subagent Execution
| PC ID | Status | Commit Count | Files Changed Count |
|-------|--------|--------------|---------------------|
| PC-001 | success | 3 | 2 |
(or "Inline execution — subagent dispatch not used" for pre-BL-031 implementations)

If wave-based parallel execution was used (any wave had 2+ PCs), add a `Wave` column to TDD Log and Subagent Execution tables showing which wave each PC belonged to. If all waves had only 1 PC (fully sequential), omit the Wave column for backward compatibility.

## Code Review
<summary — PASS or findings addressed>

## Suggested Commit
<type>(<scope>): <description>
```

After writing:
1. Update tasks.md: set "Write implement-done.md" entry to `done@<ISO 8601 timestamp>` and mark `[x]`
2. Commit tasks.md final state: `docs: finalize tasks.md for <feature>`
3. **Release backlog lock**: If `docs/backlog/.locks/` contains a lock file matching the current feature's BL-NNN ID, remove it. This releases the advisory lock so other sessions can claim the item. If no lock file exists (item was not claimed via picker), skip silently.
4. Run: `!ecc-workflow transition done --artifact implement`
5. Commit: `chore: write implement-done.md`
5. **Serialized merge**: If running in a worktree, run: `!ecc-workflow merge`
   - On **pass**: The branch was rebased, verified (build+test+clippy), merged ff-only to main, and the worktree+branch cleaned up. Call `ExitWorktree` to return to main repo. Proceed to Phase 8.
   - On **warn** (rebase conflict): Worktree preserved with rebase aborted. Tell the user: "Rebase conflicts detected. Resolve conflicts in the worktree, then re-run `/implement` to re-trigger merge."
   - On **warn** (verify failure): Worktree preserved. Tell the user: "Fast verify failed. Fix the issue in the worktree, then re-run `/implement` to re-trigger merge."
   - On **block** (timeout): Tell the user: "Another session is merging. Wait and retry."
   - If NOT in a worktree: skip merge (direct main development).

## Phase 8: Final Verification and STOP

Verify stop hook requirements are met:

1. **stop-gate**: `state.json` phase is `"done"` ✅
2. **doc-enforcement**: `## Docs Updated` section has entries (not empty) ✅
3. **pass-condition-check**: `## Pass Condition Results` has all ✅, no ❌, E2E tests have `#[ignore]` ✅
4. **e2e-boundary-check**: if solution had `## E2E Test Plan` entries, `## E2E Tests` section exists ✅
5. **scope-check**: review any warnings about files changed outside the solution's File Changes table. If unexpected files were flagged, verify they are legitimate (test helpers, lock files, etc.) before proceeding.
6. **doc-level-check**: review any warnings about doc size limits (CLAUDE.md < 200 lines, README < 300 lines, ARCHITECTURE.md code blocks < 20 lines). Address if practical.
7. **supplemental-docs-check**: `## Supplemental Docs` section is present in implement-done.md ✅

### Full Artifact Display

Read the full artifact from `artifacts.tasks_path` in state.json using the Read tool. Display the complete file content inline in conversation as the tasks document body — no truncation, no summary. If the path is null or the file does not exist, emit a warning ("Tasks artifact not found at the expected path; skipping inline display") and skip to the summary tables.

Display a comprehensive Phase Summary using these tables:

### Tasks Executed

| PC ID | Description | RED-GREEN Status | Commit Count |
|-------|-------------|------------------|--------------|
| PC-001 | <description> | GREEN | <count> |

### Commits Made

| Hash (short) | Message |
|--------------|---------|
| <SHA> | <commit message> |

### Docs Updated

| Doc File | Level | What Changed |
|----------|-------|--------------|
| CHANGELOG.md | project | Added <feature> entry |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| .claude/workflow/implement-done.md | Full implementation summary |
| docs/specs/.../tasks.md | Tasks with completion status |

### Phase Summary Persistence

Append a `## Phase Summary` section containing all 4 tables above to the tasks.md file. If `## Phase Summary` already exists in the tasks.md file, overwrite it (idempotent).

### Artifact File Path

Display the persisted file paths for future access:

> **Tasks persisted at:** `<tasks_path from state.json>`
> **Implement-done at:** `.claude/workflow/implement-done.md`

Then STOP. The workflow is complete.

## Constraints

- You MUST enter Plan Mode (EnterPlanMode) in Phase 1 — NEVER skip it for any reason
- The TDD loop is the ONLY way code gets written — no code outside the loop
- Every PC Command is run VERBATIM — no paraphrasing, no modification
- You MUST commit immediately after every RED, GREEN, REFACTOR, and doc update step — never defer commits, never batch multiple steps into one commit, never ask the user whether to commit
- Doc updates happen BEFORE writing implement-done.md (they are part of implementation, not an afterthought)
- implement-done.md schema is EXACT — stop hooks parse it
- One PC at a time — never batch multiple PCs (wave dispatch groups independent PCs per `skills/wave-dispatch/SKILL.md`)
- Campaign.md writes by parent orchestrator only, never by subagents

## Related Agents

This command invokes:
- `tdd-executor` — executes each PC's RED → GREEN → REFACTOR cycle in an isolated subagent with fresh context
- `code-reviewer` — reviews all changes against spec and solution after TDD loop completes
- `module-summary-updater` — updates MODULE-SUMMARIES.md with per-crate entries during Phase 7.5
- `diagram-updater` — generates Mermaid diagrams for cross-module flows during Phase 7.5
