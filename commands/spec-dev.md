---
description: Spec a new feature — requirements analysis, architecture review, web research, grill-me interview, doc-first review, and spec generation.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite, Agent, AskUserQuestion, EnterPlanMode, ExitPlanMode]
---

# Spec Dev Command

> **MANDATORY**: Follow every phase exactly. Do NOT edit `state.json` directly — use hooks. Narrate per `skills/narrative-conventions/SKILL.md`.

!`ecc-workflow init dev "$ARGUMENTS"`

### Worktree Isolation

1. Run `!ecc-workflow worktree-name dev "$ARGUMENTS"` — capture output
2. Call `EnterWorktree` with the name. On failure, warn and proceed on main tree.

## Phase 0: Project Detection

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Project Detection section.

Detect test/lint/build commands:

Test: !`command -v cargo > /dev/null 2>&1 && echo "cargo test" || (test -f package.json && echo "npm test" || (test -f go.mod && echo "go test ./..." || (test -f pyproject.toml && echo "pytest" || echo "echo 'no test runner detected'")))`
Lint: !`command -v cargo > /dev/null 2>&1 && echo "cargo clippy -- -D warnings" || (test -f package.json && echo "npm run lint" || (test -f go.mod && echo "golangci-lint run" || (test -f pyproject.toml && echo "ruff check ." || echo "echo 'no linter detected'")))`
Build: !`command -v cargo > /dev/null 2>&1 && echo "cargo build" || (test -f package.json && echo "npm run build" || (test -f go.mod && echo "go build ./..." || echo "echo 'no build command detected'"))`

Persist to state.json. Campaign init: `!ecc-workflow campaign init docs/specs/<spec-dir>`.

> **Tracking**: TodoWrite checklist below. If unavailable, proceed without tracking.

TodoWrite: Phase 0-8 items (Project Detection, Requirements, Architecture, Prior Audit, Backlog, Grill-Me, Write Spec, Adversarial Review, Present).

## Phase 1: Requirements Analysis

> Dispatching `requirements-analyst` — decomposes input into formal user stories.

Launch Task with `requirements-analyst` (allowedTools: [Read, Grep, Glob, Bash]): decompose into US-NNN with AC-NNN.N, challenge assumptions, validate against codebase, produce dependency DAG.

## Phase 2: Architecture Review

> Dispatching `architect` — reviews architecture impact.

Launch Task with `architect` (allowedTools: [Read, Grep, Glob, Bash]): identify affected modules, hexagonal boundaries, DDD violations, E2E consequences.

## Phase 3: Web Research

> Dispatching web research subagent.

Launch Task (allowedTools: [WebSearch]): derive 3 queries from intent + stack, run searches. Produce 3-7 bullet Research Summary. Fallback: exa-web-search MCP. On failure: "Web research skipped" — proceed.

## Phase 3.5: Sources Consultation

If `docs/sources.md` exists, find matching entries (by subject or module mapping), list as "Consulted sources:", update `last_checked`, atomic write back. Skip silently if absent.

## Phase 3.7: Actor Registry Integration

Check `docs/cartography/journeys/` for actor definitions. Extract Actor: fields, present as suggestion list for US actors. Flag new actors. Graceful fallback if absent.

## Phase 4: Prior Audit Check

Glob `docs/audits/*.md`, scan for overlap with feature domain, extract CRITICAL/HIGH findings as constraints. Note if none applicable.

## Phase 5: Backlog Cross-Reference

If `docs/backlog/BACKLOG.md` exists, cross-reference by keywords. Present high/medium matches. If user wants to include, read full prompts. Skip silently if absent.

## Phase 6: Grill-Me Interview

**STOP research. START interviewing.** Challenge user with domain-specific questions, provide recommended answers from research.

### Mandatory Questions

1. **Scope boundaries** — what's out of scope?
2. **Edge cases** — specific edge case from analysis?
3. **Test strategy** — which paths need 100% vs 80%?
4. **Performance** — latency/throughput requirements?
5. **Security** — auth, user data, external APIs?
6. **Breaking changes** — public API/data contract changes?
7. **Domain concepts** — glossary terms needed?
8. **ADR decisions** — which warrant an ADR?

Use AskUserQuestion with `preview` for architecture comparisons (Mermaid/code). Skip preview for textual questions.

### Rules

> **Shared**: Use `grill-me` skill in spec-mode. See `skills/grill-me/SKILL.md`.

After each answer: `!ecc-workflow campaign append-decision --question "<q>" --answer "<a>" --source recommended|user`

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Grill-Me Disk Persistence and Agent Output Tracking.

## Phase 7: Doc-First Review (Plan Mode)

> **BLOCKING**: MUST call `EnterPlanMode`. NEVER skip.

1. `EnterPlanMode`
2. Write plan: understanding summary, full spec draft, doc preview (README/CLAUDE.md/project changes)
3. `ExitPlanMode` — wait for approval

## Phase 8: Write the Spec

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Draft Spec Persistence.

Output full spec in conversation (exact schema). Sections: Problem Statement, Research Summary, Decisions (table), User Stories (US-NNN with ACs), Affected Modules, Constraints, Non-Requirements, E2E Boundaries (table), Doc Impact Assessment (table), Open Questions.

## Phase 9: Adversarial Review

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Adversarial Review + Verdict Handling, Adversary History Tracking.

Launch `spec-adversary` (allowedTools: [Read, Grep, Glob]): attacks on 7 dimensions.

### Verdict Handling (max 3 rounds)

- **FAIL**: Present findings → Grill-Me → re-output spec → re-run. Increment.
- **CONDITIONAL**: Add suggested ACs → re-run. Increment.
- **PASS**: Persist spec. `!ecc-workflow transition solution --artifact plan --path <path>`.

After 3 FAILs: ask override or abandon.

### Persist Spec

Write to `docs/specs/YYYY-MM-DD-<slug>/spec.md`. Slug: lowercase, non-alphanum→hyphens, max 40 chars. Re-entry: append `## Revision`; `!ecc-workflow campaign show`.

## Phase 10: Present and STOP

Read and display full spec from `artifacts.spec_path`. Display Phase Summary: Grill-Me Decisions, User Stories, Acceptance Criteria, Adversary Findings, Artifacts Persisted. Append `## Phase Summary` to spec file.

> **Spec persisted at:** `docs/specs/YYYY-MM-DD-<slug>/spec.md`

Then STOP. **Run `/design` to continue.** Do NOT proceed to design or implementation.

## Related Agents

- `requirements-analyst` — US decomposition, codebase validation
- `architect` — hexagonal analysis, E2E boundaries
- `spec-adversary` — 7-dimension adversarial review
