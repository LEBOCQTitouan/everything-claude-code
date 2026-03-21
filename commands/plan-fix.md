---
description: Plan a bug fix — investigation, blast radius analysis, grill-me interview, and spec generation. Outputs spec in conversation.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite, Agent, AskUserQuestion]
---

# Plan Fix Command

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Do NOT enter Plan Mode. Do NOT directly edit `.claude/workflow/state.json`.** State transitions happen via hooks only. Plan Mode is reserved for `/implement`.

!`bash .claude/hooks/workflow-init.sh fix "$ARGUMENTS"`

## Phase 0: Project Detection

Detect the project's test, lint, and build commands:

Test command: !`command -v cargo > /dev/null 2>&1 && echo "cargo test" || (test -f package.json && echo "npm test" || (test -f go.mod && echo "go test ./..." || (test -f pyproject.toml && echo "pytest" || echo "echo 'no test runner detected'")))`

Lint command: !`command -v cargo > /dev/null 2>&1 && echo "cargo clippy -- -D warnings" || (test -f package.json && echo "npm run lint" || (test -f go.mod && echo "golangci-lint run" || (test -f pyproject.toml && echo "ruff check ." || echo "echo 'no linter detected'")))`

Build command: !`command -v cargo > /dev/null 2>&1 && echo "cargo build" || (test -f package.json && echo "npm run build" || (test -f go.mod && echo "go build ./..." || echo "echo 'no build command detected'"))`

Store these commands mentally for use in spec constraints.

> **Tracking**: Create a TodoWrite checklist for this command's phases. If TodoWrite is unavailable, proceed without tracking — the workflow executes identically.

TodoWrite items:
- "Phase 0: Project Detection"
- "Phase 1: Investigation"
- "Phase 2: Audit Context"
- "Phase 3: Blast Radius"
- "Phase 4: Grill-Me Interview"
- "Phase 5: Write the Spec"
- "Phase 6: Adversarial Review"
- "Phase 7: Present and STOP"

Mark each item complete as the phase finishes.

## Phase 1: Investigation

Launch a Task with the `code-reviewer` agent in **read-only investigation mode** (allowedTools: [Read, Grep, Glob, Bash]):

- Provide the user's bug description as context
- Agent investigates the codebase to identify:
  - Probable root cause (not just the symptom)
  - Code paths involved
  - Existing test coverage for the affected area
  - Related code that may have the same issue
- Agent does NOT fix anything — investigation only
- Collect the output — you will incorporate it into the spec

## Phase 2: Audit Context

Read `docs/audits/` for any existing audit reports relevant to the bug area:

1. Glob for `docs/audits/*.md`
2. Scan report titles and summaries for overlap with the bug's domain
3. Extract relevant findings that may explain or relate to the bug
4. Check if the bug was already flagged by a prior audit
5. If no audit reports exist or none are relevant, note "No prior audit findings applicable"

## Phase 3: Web Research

Search the web for known issues, fix patterns, and related pitfalls for the bug's domain.

1. Derive up to 3 focused search queries from the bug description + detected tech stack. Examples: "Rust async deadlock patterns", "tokio spawn_blocking best practices", "common causes of [error message]"
2. Run each query using `WebSearch`. If WebSearch is unavailable, try `exa-web-search` MCP (`web_search_exa`). If both are unavailable, emit a warning: "Web research skipped: no search tool available" and proceed — do NOT hard-fail the plan.
3. From the results, produce a **Research Summary** with 3-7 bullet points: known issues, fix patterns, pitfalls to avoid, and relevant documentation.
4. Carry the Research Summary forward into the spec output.

## Phase 4: Blast Radius

Launch a Task with the `architect` agent (allowedTools: [Read, Grep, Glob, Bash]):

- Provide the investigation findings from Phase 1 as context
- Agent assesses:
  - Which modules are affected by the fix
  - Whether the fix crosses hexagonal boundaries
  - Risk of regression in adjacent code
  - Port/adapter impacts
  - Whether the fix requires a migration or data change
- Collect the output — you will incorporate it into the spec

## Phase 5: Grill-Me Interview

**STOP all research. START interviewing the user.**

You have gathered investigation findings, audit context, and blast radius analysis. Now challenge the user's thinking with fix-specific questions. For each question, provide your recommended answer based on codebase research.

### Mandatory Questions

1. **Root cause vs symptom** — "Is this the root cause or a downstream symptom? Evidence: [investigation findings]" (Recommend based on code-reviewer output)
2. **Minimal vs proper fix** — "Should we apply a minimal patch or a proper structural fix?" (Recommend based on blast radius and time constraints)
3. **Missing tests** — "The affected area has [N] tests covering [X]%. Which scenarios are untested?" (Recommend based on investigation)
4. **Regression risk** — "These [N] modules share code paths with the bug. What regressions should we watch for?" (Recommend based on architect output)
5. **Related audit findings** — "Prior audits flagged [findings]. Should we address these in the same fix?" (Recommend based on severity and relatedness)
6. **Reproducibility** — "Can you provide steps to reproduce, or should we derive them from the code?" (Recommend reproduction steps from investigation)
7. **Data impact** — "Does this bug affect persisted data that needs migration or cleanup?" (Recommend based on investigation)

### Rules

- Explore the codebase yourself instead of asking the user when the answer is findable in code
- Provide your recommended answer for each question
- The user can accept recommendations with "yes", override with their own answer, or say "spec it" to accept all remaining recommendations
- Do NOT proceed until the user says **"spec it"** (or equivalent confirmation)

## Phase 6: Write the Spec

Output the full spec in conversation using the exact schema below. Do NOT write `.claude/workflow/plan.md`. Every section is mandatory.

```markdown
# Spec: <title>

## Problem Statement

<One paragraph describing the bug, its impact, and the root cause identified during investigation.>

## Research Summary

<3-7 bullet points from web research: known issues, fix patterns, pitfalls, relevant docs. If web research was skipped, note "Web research skipped: no search tool available.">

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | ... | ... | Yes/No |

## User Stories

### US-001: <title>

**As a** <role>, **I want** <fix description>, **so that** <expected behavior is restored>.

#### Acceptance Criteria

- AC-001.1: Given <bug trigger condition>, when <action>, then <correct outcome>
- AC-001.2: ...

#### Dependencies

- Depends on: <none or US-NNN>

## Affected Modules

<From architect agent output. List modules, their layer (domain/port/adapter/app/CLI), and the nature of the change.>

## Constraints

<From audits, investigation, and interview. List hard constraints — e.g., "must not change public API", "must be backward-compatible".>

## Non-Requirements

<Explicitly out of scope — from grill-me interview. E.g., "not addressing the related audit finding ARCH-003 in this fix".>

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| ... | ... | ... |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| ... | ... | ... | ... |

## Open Questions

<Should be empty after grill-me. If any remain, list them here.>
```

## Phase 7: Adversarial Review

Launch a Task with the `spec-adversary` agent (allowedTools: [Read, Grep, Glob]):

- Pass the full spec from conversation context (Phase 5 output)
- The agent attacks the spec on 7 dimensions: ambiguity, edge cases, scope, dependencies, testability, decisions, rollback
- The agent returns a verdict in conversation (no file writes)

### Verdict Handling (max 3 rounds)

Track the current round number (starting at 1):

- **FAIL**: Present the adversary's findings to the user. Return to **Phase 4 (Grill-Me)** to address the fundamental issues. After the user confirms updates, re-output the spec (Phase 5), then re-run the adversary (Phase 6). Increment round.
- **CONDITIONAL**: The adversary has suggested specific ACs to add. Update the spec in conversation with the suggested ACs. Re-run the adversary. Increment round.
- **PASS**: Note "Adversarial Review: PASS" in conversation output. Run: `!bash .claude/hooks/phase-transition.sh solution plan`. Proceed to Phase 7.

After 3 FAIL rounds, ask the user:
> "The spec has failed adversarial review 3 times. Would you like to override and proceed anyway, or abandon this spec?"
- If override: note "Adversarial Review: PASS (user override)" in conversation, run `!bash .claude/hooks/phase-transition.sh solution plan`, and proceed
- If abandon: delete workflow artifacts and exit

## Phase 8: Present and STOP

Display a summary of the spec:
- Title
- Root cause (one sentence)
- Affected modules (brief)
- Fix approach (minimal vs proper)
- Adversarial review result (PASS, round number)
- Any open questions remaining

> **Note:** If continuing in a new session, copy the spec recap above or re-run `/plan-fix`.

Then STOP. Say:

> **Run `/solution` to continue.**

Do NOT proceed to solution design or implementation.

## Related Agents

This command invokes:
- `code-reviewer` — Read-only investigation, root cause analysis, test coverage assessment
- `architect` — Blast radius analysis, module impact, regression risk assessment
- `spec-adversary` — Adversarial spec review on 7 dimensions before phase transition
