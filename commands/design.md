---
description: "Design the technical solution from the spec — Phase 2 of the pipeline. Doc-first with architecture preview."
allowed-tools: [Bash, Task, Read, Grep, Glob, LS, Write, TodoWrite, TodoRead, EnterPlanMode, ExitPlanMode, AskUserQuestion]
---

# Design Command

> **MANDATORY**: Follow every phase exactly. Do NOT edit `state.json` directly — use hooks. Narrate per `skills/narrative-conventions/SKILL.md`.

## Phase 0: State Validation

### Worktree Isolation

If not in a worktree:
1. Read `concern`/`feature` from `state.json`
2. Run `!ecc-workflow worktree-name <concern> "<feature>"` — capture output
3. Call `EnterWorktree`. On failure, warn and proceed.

If already in a worktree: skip.

1. Read `.claude/workflow/state.json`
2. Verify `phase` is `"plan"` or `"solution"` (re-entry). Other → error: "Run `/spec-*` first."
3. **Read spec from file**: Use `artifacts.spec_path`. Warn if modified since spec phase. Fall back to step 4.
4. If spec not in context or on disk → ask user to re-run `/spec-*` or paste.
5. Extract `concern`/`feature`
6. **Re-entry**: If `"solution"`, resume via TodoRead.

> **Tracking**: TodoWrite checklist below. If unavailable, proceed without tracking.

TodoWrite: Phase 0-10 items.

## Phase 0.5: Sources Consultation

If `docs/sources.md` exists, find matching architectural sources, reference as "Consulted sources:", update `last_checked`, atomic write. Skip silently if absent.

## Phase 1: Implementation Design

Launch `planner` (allowedTools: [Read, Grep, Glob, Bash]):
- Design file changes in dependency order with rationale
- Map each change to spec ref (US-NNN, AC-NNN.N)
- Define PCs: type, description, AC coverage, literal bash command, expected result
- Order PCs in TDD dependency order
- Final PCs: lint + build checks

> **Optional**: For new ports/adapters/interfaces, consider `interface-designer` for divergent exploration.
> **Preview**: If 2+ viable approaches, use `AskUserQuestion` with `preview` (Mermaid/file-change/code comparison, <15 lines/option).

## Phase 2: SOLID Validation

> Dispatching `uncle-bob` for SOLID and Clean Architecture evaluation.

Launch `uncle-bob` (allowedTools: [Read, Grep, Glob], context: "fork"): SOLID, Clean Architecture, component principles. Returns PASS or findings.

## Phase 3: Professional Conscience

Launch `robert` (allowedTools: [Read, Grep, Glob, Bash], context: "fork"): Programmer's Oath evaluation. Returns CLEAN or warnings.

## Phase 4: Security Quick-Check

Launch `security-reviewer` (allowedTools: [Read, Grep, Glob, Bash], context: "fork"): design-level scan (not full audit). Returns CLEAR or findings.

## Phase 5: E2E Boundary Detection

1. Read spec's `## E2E Boundaries Affected`
2. Scan Phase 1 changes for port/adapter touches
3. Expand into E2E test entries (boundary, adapter, port, description, default=ignored, activation condition)
4. Produce E2E Activation Rules for THIS implementation

## Phase 6: Doc Update Plan

1. Read spec's `## Doc Impact Assessment`
2. Expand into concrete actions: Doc File, Level, Action, Content Summary, Spec Ref
3. MUST include CHANGELOG.md entry
4. MUST include ADR for decisions marked `ADR Needed? Yes`

## Phase 7: AC Coverage Verification

> Summarize AC coverage: covered count, uncovered count, needed actions.

1. Collect all AC-NNN.N from spec
2. Verify each appears in ≥1 PC's "Verifies AC" column
3. Add PCs for uncovered ACs before proceeding

## Phase 8: Architecture Preview (Plan Mode)

> **BLOCKING**: MUST call `EnterPlanMode`. NEVER skip.

1. `EnterPlanMode`
2. Write plan: design summary, architecture preview (ARCHITECTURE.md changes, Mermaid diagrams, bounded contexts, module summaries), PC overview table
3. `ExitPlanMode` — wait for approval

## Phase 9: Output Design

Output full design in conversation (exact schema). Sections: Spec Reference, File Changes table, Pass Conditions table, Coverage Check, E2E Test Plan, E2E Activation Rules, Test Strategy, Doc Update Plan, SOLID Assessment, Robert's Oath Check, Security Notes, Rollback Plan, Bounded Contexts Affected.

The solution is output in conversation only — no file written.

## Phase 10: Adversarial Review

Launch `solution-adversary` (allowedTools: [Read, Bash, Grep, Glob]): 8 dimensions (coverage, order, fragility, rollback, architecture, blast radius, missing PCs, doc plan).

### Verdict Handling (max 3 rounds)

- **FAIL**: Present findings → redesign (Phases 1-8) → re-run. Increment.
- **CONDITIONAL**: Add PCs/doc fixes → re-run. Increment.
- **PASS**: Persist design. `!ecc-workflow transition implement --artifact solution --path <path>`.

After 3 FAILs: override or abandon.

### Persist Design

Write to `docs/specs/YYYY-MM-DD-<slug>/design.md` (same dir as spec). Re-entry: append `## Revision`. Update campaign.md Artifacts table.

## Phase 11: Present and STOP

Read and display full design from `artifacts.design_path`. Display Phase Summary: Design Reviews, Adversary Findings, File Changes, Artifacts. Append `## Phase Summary` to design file.

> **Design persisted at:** `docs/specs/YYYY-MM-DD-<slug>/design.md`

Then STOP. **Run `/implement` to begin.** Do NOT write code.

## Pass Condition Rules

1. Format: `PC-NNN` — three digits, sequential from 001
2. Command: literal bash, runnable verbatim
3. Expected: PASS, exit 0, or specific output
4. Coverage: every AC covered by ≥1 PC
5. Final PCs: lint + build checks
6. Deterministic: verifiable by command + expected output

## Related Agents

- `planner` — file changes, PCs, TDD order
- `uncle-bob` — SOLID/Clean Architecture
- `robert` — Programmer's Oath
- `security-reviewer` — design-level security
- `solution-adversary` — 8-dimension review
