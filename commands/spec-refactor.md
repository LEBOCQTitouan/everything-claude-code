---
description: Spec a refactoring — current state analysis, smell catalog, web research, grill-me interview, doc-first review, and spec generation.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite, Agent, AskUserQuestion, EnterPlanMode, ExitPlanMode]
---

# Spec Refactor Command

> **MANDATORY**: Follow every phase exactly. Do NOT edit `state.json` directly — use hooks. Narrate per `skills/narrative-conventions/SKILL.md`.

!`ecc-workflow init refactor "$ARGUMENTS"`

### Worktree Isolation

1. Run `!ecc-workflow worktree-name refactor "$ARGUMENTS"` — capture output
2. Call `EnterWorktree` with the name. On failure, warn and proceed on main tree.

## Phase 0: Project Detection

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Project Detection section.

Detect test/lint/build commands (same auto-detection as spec-dev). Persist to state.json. Campaign init: `!ecc-workflow campaign init docs/specs/<spec-dir>`.

> **Tracking**: TodoWrite checklist below. If unavailable, proceed without tracking.

TodoWrite: Phase 0-7 (Project Detection, Current State, Existing Audits, Smell Catalog, Grill-Me, Write Spec, Adversarial Review, Present).

## Phase 1: Current State Analysis

> Dispatching in parallel: `evolution-analyst`, `arch-reviewer`, `component-auditor`.

### Task 1: Evolution Analysis
`evolution-analyst` (allowedTools: [Read, Grep, Glob, Bash]): hotspots, coupling, bus factor, complexity trends.

### Task 2: Architecture Review
`arch-reviewer` (allowedTools: [Read, Grep, Glob, Bash]): layering violations, dependency direction, coupling/cohesion, DDD/hexagonal compliance.

### Task 3: Component Audit
`component-auditor` (allowedTools: [Read, Grep, Glob, Bash]): 6 component principles (REP, CCP, CRP, ADP, SDP, SAP), instability/abstractness/distance.

## Phase 2: Existing Audit Reports

Glob `docs/audits/*.md`, scan for overlap, extract findings supporting/contradicting refactoring. Note if none applicable.

## Phase 3: Web Research

> Dispatching web research subagent.

Launch Task (allowedTools: [WebSearch]): derive 3 queries from goal + stack. Produce 3-7 bullet Research Summary. On failure: proceed.

## Phase 3.5: Sources Consultation

If `docs/sources.md` exists, find matching entries, list as "Consulted sources:", update `last_checked`, atomic write. Skip silently if absent.

## Phase 4: Smell Catalog

Compile unified catalog from Phase 1 agents + Phase 2 audits:

| # | Smell | Source | Severity | Evidence |

Group by severity. Drives grill-me interview.

## Phase 5: Grill-Me Interview

**STOP research. START interviewing.** Challenge with refactoring-specific questions.

### Mandatory Questions

1. **Smell triage** — which to address now vs defer?
2. **Target architecture** — target state? (preview before/after if structural change)
3. **Step independence** — ship independently or atomic grouping?
4. **Downstream dependencies** — how to keep dependents green?
5. **Rename vs behavioral** — pure moves vs behavior changes?
6. **Performance budget** — hot paths affected?
7. **ADR decisions** — which warrant an ADR?
8. **Test safety net** — coverage sufficient for safe refactoring?

> **Shared**: Use `grill-me` skill in spec-mode. Persist decisions to campaign.md.

## Phase 6: Doc-First Review (Plan Mode)

> **BLOCKING**: MUST call `EnterPlanMode`. NEVER skip.

1. `EnterPlanMode`
2. Write plan: understanding, full spec draft, doc preview
3. `ExitPlanMode` — wait for approval

## Phase 7: Write the Spec

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Draft Spec Persistence.

Output full spec (exact schema). Sections: Problem Statement (structural problem + symptoms), Research Summary, Decisions, User Stories (refactoring ACs), Affected Modules, Constraints, Non-Requirements (deferred smells), E2E Boundaries, Doc Impact Assessment, Open Questions.

## Phase 8: Adversarial Review

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Adversarial Review + Verdict Handling.

Launch `spec-adversary` (allowedTools: [Read, Grep, Glob]): 7 dimensions.

### Verdict Handling (max 3 rounds)

- **FAIL**: → Grill-Me → re-output → re-run. Increment.
- **CONDITIONAL**: Add ACs → re-run. Increment.
- **PASS**: Persist spec. `!ecc-workflow transition solution --artifact plan --path <path>`.

After 3 FAILs: override or abandon.

### Persist Spec

Write to `docs/specs/YYYY-MM-DD-<slug>/spec.md`. Re-entry: append `## Revision`.

## Phase 9: Present and STOP

Read and display full spec from `artifacts.spec_path`. Display Phase Summary: Grill-Me Decisions (include Smells addressed/deferred rows), User Stories, ACs, Adversary Findings, Artifacts. Append `## Phase Summary` to spec file.

> **Spec persisted at:** `docs/specs/YYYY-MM-DD-<slug>/spec.md`

Then STOP. **Run `/design` to continue.**

## Related Agents

- `evolution-analyst` — git history, hotspots, complexity
- `arch-reviewer` — architecture quality, DDD compliance
- `component-auditor` — component principles, metrics
- `spec-adversary` — 7-dimension adversarial review
