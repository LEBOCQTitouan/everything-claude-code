---
description: Plan a new feature — requirements analysis, architecture review, grill-me interview, and spec generation. Outputs spec in conversation.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite, Agent, AskUserQuestion]
---

# Plan Dev Command

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Do NOT enter Plan Mode. Do NOT directly edit `.claude/workflow/state.json`.** State transitions happen via hooks only. Plan Mode is reserved for `/implement`.

!`bash .claude/hooks/workflow-init.sh dev "$ARGUMENTS"`

## Phase 0: Project Detection

Detect the project's test, lint, and build commands:

Test command: !`command -v cargo > /dev/null 2>&1 && echo "cargo test" || (test -f package.json && echo "npm test" || (test -f go.mod && echo "go test ./..." || (test -f pyproject.toml && echo "pytest" || echo "echo 'no test runner detected'")))`

Lint command: !`command -v cargo > /dev/null 2>&1 && echo "cargo clippy -- -D warnings" || (test -f package.json && echo "npm run lint" || (test -f go.mod && echo "golangci-lint run" || (test -f pyproject.toml && echo "ruff check ." || echo "echo 'no linter detected'")))`

Build command: !`command -v cargo > /dev/null 2>&1 && echo "cargo build" || (test -f package.json && echo "npm run build" || (test -f go.mod && echo "go build ./..." || echo "echo 'no build command detected'"))`

Store these commands mentally for use in spec constraints.

> **Tracking**: Create a TodoWrite checklist for this command's phases. If TodoWrite is unavailable, proceed without tracking — the workflow executes identically.

TodoWrite items:
- "Phase 0: Project Detection"
- "Phase 1: Requirements Analysis"
- "Phase 2: Architecture Review"
- "Phase 3: Prior Audit Check"
- "Phase 4: Backlog Cross-Reference"
- "Phase 5: Grill-Me Interview"
- "Phase 6: Write the Spec"
- "Phase 7: Adversarial Review"
- "Phase 8: Present and STOP"

Mark each item complete as the phase finishes.

## Phase 1: Requirements Analysis

Launch a Task with the `requirements-analyst` agent (allowedTools: [Read, Grep, Glob, Bash]):

- Provide the user's raw input as context
- Agent decomposes input into formal User Stories (US-NNN) with acceptance criteria (AC-NNN.N)
- Agent challenges assumptions and validates against the codebase
- Agent produces a dependency DAG for parallel execution
- Collect the output — you will incorporate it into the spec

## Phase 2: Architecture Review

Launch a Task with the `architect` agent (allowedTools: [Read, Grep, Glob, Bash]):

- Provide the user stories from Phase 1 as context
- Agent identifies affected modules, hexagonal boundaries, port/adapter impacts
- Agent flags any DDD violations or architectural concerns
- Agent assesses E2E boundary consequences
- Collect the output — you will incorporate it into the spec

## Phase 3: Web Research

Search the web for best practices, relevant libraries, patterns, and prior art related to the feature request. This grounds the plan in current external knowledge, not just training data and local codebase.

1. Derive up to 3 focused search queries from the user's stated intent + detected tech stack. Examples: "Rust async error handling best practices 2025", "hexagonal architecture file-based persistence patterns", "Claude Code hook system design patterns"
2. Run each query using `WebSearch`. If WebSearch is unavailable, try `exa-web-search` MCP (`web_search_exa`). If both are unavailable, emit a warning: "Web research skipped: no search tool available" and proceed to the next phase — do NOT hard-fail the plan.
3. From the search results, produce a **Research Summary** with 3-7 bullet points: relevant libraries, best practices, patterns to follow, pitfalls to avoid, and prior art.
4. Carry the Research Summary forward — it will be included in the spec output and referenced during the grill-me interview.

## Phase 4: Prior Audit Check

Read `docs/audits/` for any existing audit reports relevant to the feature area:

1. Glob for `docs/audits/*.md`
2. Scan report titles and summaries for overlap with the feature domain
3. Extract relevant findings (CRITICAL/HIGH severity) that should constrain the implementation
4. If no audit reports exist or none are relevant, note "No prior audit findings applicable"

## Phase 5: Backlog Cross-Reference

Check if `docs/backlog/BACKLOG.md` exists:

1. If it exists, read it and cross-reference against the feature request
2. Use keyword matching on titles, tags, and content of open entries
3. If matches found, present them:
   - **High confidence**: "These backlog items are directly related: BL-NNN. Consider bundling."
   - **Medium confidence**: "These items may be related: BL-NNN. Review before proceeding."
4. If user wants to include items, read their full optimized prompts and incorporate into planning context
5. If `docs/backlog/BACKLOG.md` does not exist, skip silently

## Phase 6: Grill-Me Interview

**STOP all research. START interviewing the user.**

You have gathered requirements, architecture analysis, audit findings, and backlog context. Now challenge the user's thinking with domain-specific questions. For each question, provide your recommended answer based on codebase research — the user can accept or override.

### Mandatory Questions

1. **Scope boundaries** — "What is explicitly OUT of scope?" (Recommend based on requirements-analyst output)
2. **Edge cases** — "What happens when [specific edge case from codebase analysis]?" (Recommend based on existing error handling patterns)
3. **Test strategy** — "Which critical paths need 100% coverage vs 80%?" (Recommend based on domain criticality)
4. **Performance constraints** — "Are there latency/throughput requirements?" (Recommend based on existing benchmarks or SLAs if found)
5. **Security implications** — "Does this touch auth, user data, or external APIs?" (Recommend based on architect findings)
6. **Breaking changes** — "Will this change any existing public API or data contract?" (Recommend based on affected modules)
7. **Domain concepts** — "Are there domain terms that need defining in the glossary?" (Recommend based on new concepts found)
8. **ADR decisions** — "Which decisions are significant enough to warrant an ADR?" (Recommend based on architect output)

### Rules

- Explore the codebase yourself instead of asking the user when the answer is findable in code
- Provide your recommended answer for each question
- The user can accept recommendations with "yes", override with their own answer, or say "spec it" to accept all remaining recommendations
- Do NOT proceed until the user says **"spec it"** (or equivalent confirmation)

## Phase 7: Write the Spec

Output the full spec in conversation using the exact schema below. Do NOT write `.claude/workflow/plan.md`. Every section is mandatory.

```markdown
# Spec: <title>

## Problem Statement

<One paragraph describing the problem and why it needs solving.>

## Research Summary

<3-7 bullet points from web research: relevant libraries, best practices, patterns, pitfalls, prior art. If web research was skipped, note "Web research skipped: no search tool available.">

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | ... | ... | Yes/No |

## User Stories

### US-001: <title>

**As a** <role>, **I want** <capability>, **so that** <benefit>.

#### Acceptance Criteria

- AC-001.1: Given <context>, when <action>, then <outcome>
- AC-001.2: ...

#### Dependencies

- Depends on: <none or US-NNN>

### US-002: <title>
...

## Affected Modules

<From architect agent output. List modules, their layer (domain/port/adapter/app/CLI), and the nature of the change.>

## Constraints

<From audits, backlog, and interview. List hard constraints that the solution must respect.>

## Non-Requirements

<Explicitly out of scope — from grill-me interview.>

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

## Phase 8: Adversarial Review

Launch a Task with the `spec-adversary` agent (allowedTools: [Read, Grep, Glob]):

- Pass the full spec from conversation context (Phase 6 output)
- The agent attacks the spec on 7 dimensions: ambiguity, edge cases, scope, dependencies, testability, decisions, rollback
- The agent returns a verdict in conversation (no file writes)

### Verdict Handling (max 3 rounds)

Track the current round number (starting at 1):

- **FAIL**: Present the adversary's findings to the user. Return to **Phase 5 (Grill-Me)** to address the fundamental issues. After the user confirms updates, re-output the spec (Phase 6), then re-run the adversary (Phase 7). Increment round.
- **CONDITIONAL**: The adversary has suggested specific ACs to add. Update the spec in conversation with the suggested ACs. Re-run the adversary. Increment round.
- **PASS**: Note "Adversarial Review: PASS" in conversation output. Run: `!bash .claude/hooks/phase-transition.sh solution plan`. Proceed to Phase 8.

After 3 FAIL rounds, ask the user:
> "The spec has failed adversarial review 3 times. Would you like to override and proceed anyway, or abandon this spec?"
- If override: note "Adversarial Review: PASS (user override)" in conversation, run `!bash .claude/hooks/phase-transition.sh solution plan`, and proceed
- If abandon: delete workflow artifacts and exit

## Phase 9: Present and STOP

Display a summary of the spec:
- Title
- Number of user stories
- Key decisions
- Affected modules (brief)
- Adversarial review result (PASS, round number)
- Any open questions remaining

> **Note:** If continuing in a new session, copy the spec recap above or re-run `/plan-dev`.

Then STOP. Say:

> **Run `/solution` to continue.**

Do NOT proceed to solution design or implementation.

## Related Agents

This command invokes:
- `requirements-analyst` — User Story decomposition, product challenge, codebase validation, dependency DAG
- `architect` — Hexagonal architecture analysis, module impact, E2E boundary assessment
- `spec-adversary` — Adversarial spec review on 7 dimensions before phase transition
