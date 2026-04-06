---
description: "Implement the solution — Phase 3. Deterministic TDD loop with mandatory doc updates."
allowed-tools: [Bash, Task, Read, Write, Edit, MultiEdit, Grep, Glob, LS, TodoWrite, TodoRead, EnterPlanMode, ExitPlanMode, TaskCreate, TaskUpdate, TaskGet, TaskList]
---

# Implement Command

> **MANDATORY**: Follow every phase exactly. Do NOT edit `state.json` directly — use hooks. Narrate per `skills/narrative-conventions/SKILL.md`.

## Phase 0: State Validation

### Worktree Isolation

If not in a worktree (`git rev-parse --show-toplevel` vs `--git-common-dir`):
1. Read `concern`/`feature` from `state.json`
2. Run `!ecc-workflow worktree-name <concern> "<feature>"` — capture output
3. Call `EnterWorktree` with the name. On failure, warn and proceed on main tree.

If already in a worktree: skip.

1. Read `.claude/workflow/state.json`
2. Verify `phase` is `"solution"` or `"implement"` (re-entry). Other phase → error: "Run `/design` first."
3. **Read spec/design from files**: Use `artifacts.spec_path`/`artifacts.design_path` from state.json. Warn if modification time differs. Fall back to step 4 if file missing.
4. If spec/design not in context or on disk → ask user to re-run `/spec-*` then `/design`.
5. Extract `concern`/`feature` from `state.json`
6. **Re-entry** (phase=`"implement"`): Resume priority:
   1. **tasks.md** (authoritative): Read `artifacts.tasks_path`. Find first incomplete PC. Failed PC = resume point.
   2. **Rebuild TodoWrite** from tasks.md entries.
   3. **Regenerate if deleted**: Infer from `git log --oneline --after=<started_at> --grep="PC-NNN"`. Warn.
   4. **Malformed tasks.md**: Regenerate from solution + git-log inference. Warn.
   5. **TodoRead fallback**: If `tasks_path` null.
   6. **Campaign re-entry**: Read campaign.md if exists.
7. Run `!ecc-workflow transition implement`

## Phase 0.5: Sources Consultation

If `docs/sources.md` exists, find matching entries for modified module, surface as reference, update `last_checked`, atomic write. Skip silently if absent.

## Phase 1: Enter Plan Mode

> **BLOCKING**: MUST call `EnterPlanMode`. NEVER skip.

1. Call `EnterPlanMode`
2. Write plan file with spec, solution, and PC checklist
3. Call `ExitPlanMode`

## Phase 2: Parse Solution

Extract: PC table, Test Strategy, E2E plan/rules, Doc Update Plan, File Changes.

Create TodoWrite checklist and `TaskCreate` for each PC. Use `TaskUpdate` for status.

### Generate tasks.md

> **Shared**: See `skills/tasks-generation/SKILL.md`.

Generate in spec directory. Store `artifacts.tasks_path`. Commit: `docs: write tasks.md for <feature>`.

### Wave Analysis

> **Shared**: See `skills/wave-analysis/SKILL.md`.

Analyze PC dependency graph, display wave plan before Phase 3.

### Team Manifest

Read `teams/implement-team.md` for agents, roles, allowed-tools, `max-concurrent`. If absent: require `ECC_LEGACY_DISPATCH=1` or fail.

## Phase 3: TDD Loop (Subagent Dispatch)

Dispatch each PC to isolated `tdd-executor` in **waves** (concurrent within, sequential across). Each subagent gets fresh context.

### Context Brief (max 500 lines)

MUST NOT include full spec/design or Phase 0-2 context. Sections:
- **## PC Spec**: Verbatim PC fields. Inline if file paths null.
- **## File Paths**: spec_path/design_path from state.json
- **## Files to Modify**: Filtered from File Changes by PC's spec ref
- **## Prior PC Results**: Table (max 5 lines/PC) with test_names ("--" if absent)
- **## Commit Rules**: RED/GREEN/REFACTOR templates
- **## TDD Cycle Rules**: RED-GREEN-REFACTOR instructions
- **## User Guidance**: From budget-exceeded prompt (omit if none)

### Fix-Round Budget

Per-PC `fix_round_count` (starts 0), owned by parent. Only GREEN failures consume rounds. RED fixes/crashes do not.

**Budget exceeded (count=2)**: Diagnostic report + AskUserQuestion:
- "Keep trying (+2 rounds)" | "Skip this PC" | "Abort implementation"
- User may provide guidance via "Other"
- **Hard cap**: 8 total rounds (2 + 3x2)

### Wave Dispatch

> **Shared**: See `skills/wave-dispatch/SKILL.md`.

Accumulate commit SHAs. Append to campaign.md `## Commit Trail` (parent only). Status: `red@ts` → `green@ts` → `done@ts`.

### Progress Tracking

> **Shared**: See `skills/progress-tracking/SKILL.md`.

### Post-TDD Coverage

After all PCs, before Phase 4: run `cargo llvm-cov --workspace --json` at before-snapshot and HEAD. If unavailable, note in summary. NEVER blocks pipeline. Store delta for implement-done.md.

## Phase 4: E2E Tests

Read `## E2E Activation Rules`. None → "No E2E tests required". Activated: un-ignore, run, same fix-round budget (hard cap 8). Commit: `test(e2e): add <boundary> E2E tests`. Update tasks.md.

## Phase 5: Code Review

Launch `code-reviewer` with changed files, spec, solution. CRITICAL/HIGH: fix, re-run PCs, commit. Max 2 rounds. Record + append to campaign.md. Update tasks.md.

## Phase 6: Doc Updates

Execute `## Doc Update Plan`. Docs BEFORE implement-done.md.

**Rules**: README (short) | CLAUDE.md (reductive) | ARCHITECTURE.md (intent) | ADRs (SCDC format) | CHANGELOG (always required) | inline comments (volatile)

ADRs: `docs/adr/NNN-<slug>.md`. Other docs: commit immediately. CHANGELOG: commit immediately. Update tasks.md.

## Phase 7.5: Supplemental Docs

Non-blocking. Two parallel Tasks:
1. **module-summary-updater** → MODULE-SUMMARIES.md
2. **diagram-updater** → Mermaid diagrams

Success: commit. Failure: record, proceed. Cross-link fixup if both produced output. Update tasks.md.

## Phase 7: Write implement-done.md

Write `.claude/workflow/implement-done.md` — exact schema, every section mandatory:

```markdown
# Implementation Complete: <title>
## Spec Reference
## Changes Made (table: File, Action, Solution Ref, Tests, Status)
## TDD Log (table: PC ID, RED, GREEN, REFACTOR, Test Names, Notes)
Test Names from tdd-executor `test_names` field. "--" if absent (BL-050).
## Pass Condition Results (table: PC ID, Command, Expected, Actual, Status)
## E2E Tests (table or "No E2E tests required")
## Docs Updated (table, MUST include CHANGELOG.md)
## ADRs Created (table or "None required")
## Coverage Delta (table or "unavailable"/"no before-snapshot")
## Supplemental Docs (table: Subagent, Status, Output, SHA, Notes)
## Subagent Execution (table: PC ID, Status, Commits, Files)
If wave-based (2+ PCs/wave), add Wave column. Omit if sequential.
## Code Review (PASS or findings addressed)
## Suggested Commit
```

After writing:
1. Update tasks.md → `done@ts`, mark `[x]`
2. Commit tasks.md: `docs: finalize tasks.md for <feature>`
3. Release backlog lock (remove BL-NNN.lock if exists)
4. Run `!ecc-workflow transition done --artifact implement`
5. Commit: `chore: write implement-done.md`
6. **Merge** (if worktree): `!ecc-workflow merge`
   - pass: rebased+verified+merged. `ExitWorktree`.
   - warn (rebase): "Resolve conflicts, re-run `/implement`."
   - warn (verify): "Fix issue, re-run `/implement`."
   - block: "Another session merging. Wait."
   - Not in worktree: skip.

## Phase 8: Final Verification and STOP

Verify: phase=done, docs updated, all PCs pass, E2E section exists if needed, scope check, doc-level check, supplemental docs present.

Read and display full tasks.md. Display Phase Summary (Tasks, Commits, Docs, Artifacts). Append to tasks.md.

> **Tasks persisted at:** `<tasks_path>` | **Implement-done at:** `.claude/workflow/implement-done.md`

Then STOP.

## Constraints

- MUST enter Plan Mode — NEVER skip
- TDD loop is the ONLY way code gets written
- PC Commands run VERBATIM
- Commit immediately after every RED/GREEN/REFACTOR/doc step — never defer/batch
- Doc updates BEFORE implement-done.md
- implement-done.md schema is EXACT — hooks parse it
- One PC at a time (wave dispatch groups independent PCs)
- Campaign.md: parent orchestrator only

## Related Agents

- `tdd-executor` — RED → GREEN → REFACTOR per PC
- `code-reviewer` — post-TDD review
- `module-summary-updater` — MODULE-SUMMARIES.md (Phase 7.5)
- `diagram-updater` — Mermaid diagrams (Phase 7.5)
