---
description: Spec a bug fix — investigation, blast radius analysis, web research, grill-me interview, doc-first review, and spec generation.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite, Agent, AskUserQuestion, EnterPlanMode, ExitPlanMode]
---

# Spec Fix Command

> **MANDATORY**: Follow every phase exactly. Do NOT edit `state.json` directly — use hooks. Narrate per `skills/narrative-conventions/SKILL.md`.

!`ecc-workflow init fix "$ARGUMENTS"`

### Worktree Isolation

1. Run `!ecc-workflow worktree-name fix "$ARGUMENTS"` — capture output
2. Call `EnterWorktree` with the name. On failure, warn and proceed on main tree.

## Phase 0: Project Detection

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Project Detection section.

Detect test/lint/build commands (same auto-detection as spec-dev). Persist to state.json. Campaign init: `!ecc-workflow campaign init docs/specs/<spec-dir>`.

> **Tracking**: TodoWrite checklist below. If unavailable, proceed without tracking.

TodoWrite: Phase 0-7 (Project Detection, Investigation, Audit Context, Blast Radius, Grill-Me, Write Spec, Adversarial Review, Present).

## Phase 1: Investigation

> Dispatching `code-reviewer` in read-only mode — root cause analysis.

Launch Task with `code-reviewer` (allowedTools: [Read, Grep, Glob, Bash]): investigate root cause, code paths, test coverage, related code with same issue. No fixes.

## Phase 2: Audit Context

Glob `docs/audits/*.md`, scan for overlap with bug domain, extract findings, check if bug was flagged. Note if none applicable.

## Phase 3: Web Research

> Dispatching web research subagent.

Launch Task (allowedTools: [WebSearch]): derive 3 queries from bug + stack. Produce 3-7 bullet Research Summary. On failure: proceed.

## Phase 3.5: Sources Consultation

If `docs/sources.md` exists, find matching entries, list as "Consulted sources:", update `last_checked`, atomic write. Skip silently if absent.

## Phase 4: Blast Radius

Launch Task with `architect` (allowedTools: [Read, Grep, Glob, Bash]): assess affected modules, boundary crossings, regression risk, port/adapter impacts, migration needs.

## Phase 5: Grill-Me Interview

**STOP research. START interviewing.** Challenge with fix-specific questions, provide recommendations.

### Mandatory Questions

1. **Root cause vs symptom** — evidence from investigation
2. **Minimal vs proper fix** — preview both approaches if viable (code diff vs architecture diagram)
3. **Missing tests** — coverage gaps in affected area
4. **Regression risk** — shared code paths to watch
5. **Related audit findings** — address in same fix?
6. **Reproducibility** — steps to reproduce
7. **Data impact** — persisted data migration/cleanup needed?

> **Shared**: Use `grill-me` skill in spec-mode. Persist decisions to campaign.md after each answer.

## Phase 6: Doc-First Review (Plan Mode)

> **BLOCKING**: MUST call `EnterPlanMode`. NEVER skip.

1. `EnterPlanMode`
2. Write plan: understanding, full spec draft, doc preview
3. `ExitPlanMode` — wait for approval

## Phase 7: Write the Spec

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Draft Spec Persistence.

Output full spec (exact schema). Sections: Problem Statement (bug + root cause), Research Summary, Decisions, User Stories (fix-oriented ACs), Affected Modules, Constraints, Non-Requirements, E2E Boundaries, Doc Impact Assessment, Open Questions.

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

Read and display full spec from `artifacts.spec_path`. Display Phase Summary: Grill-Me Decisions (include Root Cause row), User Stories, ACs, Adversary Findings, Artifacts. Append `## Phase Summary` to spec file.

> **Spec persisted at:** `docs/specs/YYYY-MM-DD-<slug>/spec.md`

Then STOP. **Run `/design` to continue.**

## Related Agents

- `code-reviewer` — read-only investigation, root cause
- `architect` — blast radius, regression risk
- `spec-adversary` — 7-dimension adversarial review
