---
description: Spec a new feature — requirements analysis, architecture review, web research, grill-me interview, doc-first review, and spec generation.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite, Agent, AskUserQuestion, EnterPlanMode, ExitPlanMode]
---

# Spec Dev Command

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Do NOT directly edit `.claude/workflow/state.json`.** State transitions happen via hooks only.
>
> **Narrative**: See narrative-conventions skill.

!`ecc-workflow init dev "$ARGUMENTS"`

### Worktree Isolation

Generate a worktree name and isolate this session:
1. Run: `!ecc-workflow worktree-name dev "$ARGUMENTS"` — capture the output name
2. Call `EnterWorktree` with the generated name as the branch name. This isolates all session writes to a dedicated worktree.
3. If `EnterWorktree` fails, proceed without worktree and warn: "Worktree isolation failed. Proceeding on main tree."

## Phase 0: Project Detection

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Project Detection section for auto-detection logic.

Detect the project's test, lint, and build commands:

Test command: !`command -v cargo > /dev/null 2>&1 && echo "cargo test" || (test -f package.json && echo "npm test" || (test -f go.mod && echo "go test ./..." || (test -f pyproject.toml && echo "pytest" || echo "echo 'no test runner detected'")))`

Lint command: !`command -v cargo > /dev/null 2>&1 && echo "cargo clippy -- -D warnings" || (test -f package.json && echo "npm run lint" || (test -f go.mod && echo "golangci-lint run" || (test -f pyproject.toml && echo "ruff check ." || echo "echo 'no linter detected'")))`

Build command: !`command -v cargo > /dev/null 2>&1 && echo "cargo build" || (test -f package.json && echo "npm run build" || (test -f go.mod && echo "go build ./..." || echo "echo 'no build command detected'"))`

Persist detected commands to state.json and create campaign manifest:

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Project Detection section for toolchain-persist.sh usage and Campaign Init section for campaign.md creation.

4. **Campaign init**: Run `!ecc-workflow campaign init docs/specs/<spec-dir>` to create campaign.md for incremental grill-me decision persistence.

> **Tracking**: Create a TodoWrite checklist for this command's phases.

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

> Before dispatching, tell the user which agent is being launched (`requirements-analyst`), that it will decompose their input into formal user stories, and what to expect from its output.

Launch a Task with the `requirements-analyst` agent (allowedTools: [Read, Grep, Glob, Bash]):

- Provide the user's raw input as context
- Agent decomposes input into formal User Stories (US-NNN) with acceptance criteria (AC-NNN.N)
- Agent challenges assumptions and validates against the codebase
- Agent produces a dependency DAG for parallel execution
- Collect the output — you will incorporate it into the spec

## Phase 2: Architecture Review

> Before dispatching, tell the user which agent is being launched (`architect`), that it will review the architecture impact, and what to expect from its output.

Launch a Task with the `architect` agent (allowedTools: [Read, Grep, Glob, Bash]):

- Provide the user stories from Phase 1 as context
- Agent identifies affected modules, hexagonal boundaries, port/adapter impacts
- Agent flags any DDD violations or architectural concerns
- Agent assesses E2E boundary consequences
- Collect the output — you will incorporate it into the spec

## Phase 3: Web Research

> Tell the user this phase is starting: web research will be delegated to a Task subagent to keep the main context lean.

Launch a Task subagent (allowedTools: [WebSearch]) to perform web research in an isolated context:

**Subagent prompt**: Search the web for best practices, relevant libraries, patterns, and prior art related to the feature request. Derive up to 3 focused search queries from the user's stated intent and detected tech stack. Examples: "Rust async error handling best practices 2025", "hexagonal architecture file-based persistence patterns", "Claude Code hook system design patterns". Run each query using WebSearch. If WebSearch is unavailable, try exa-web-search MCP (`web_search_exa`). If both are unavailable, return: "Web research skipped: no search tool available." From the results, produce a **Research Summary** with 3-7 bullet points: relevant libraries, best practices, patterns to follow, pitfalls to avoid, and prior art. Return ONLY the Research Summary.

**Subagent input**: The user's raw input/idea and the detected tech stack from Phase 0.

**On success**: Carry the returned Research Summary forward to subsequent phases — it will be included in the spec output and referenced during the grill-me interview.
**On failure** (subagent failed or timed out): Record "Web research skipped: subagent failed" and proceed to the next phase — do NOT hard-fail.

## Phase 3.5: Sources Consultation

If `docs/sources.md` exists:
1. Read `docs/sources.md` and parse entries
2. Find entries matching the current subject (case-insensitive exact match on `subject` field) OR where the affected module appears in the module mapping table
3. If matches found, list them as "Consulted sources:" in the output
4. Update `last_checked` date on matched entries to today's date
5. Write updated file back (atomic write via temp file + rename)

If `docs/sources.md` does not exist, skip this step silently.

## Phase 3.7: Actor Registry Integration

Before identifying actors for user stories, check `docs/cartography/journeys/` for established actor definitions:

1. If `docs/cartography/journeys/` exists, glob for `*.md` files in that directory
2. For each file found, read its `## Overview` section and extract the **Actor:** field value
3. Collect all unique actor names as a suggestion list
4. When defining actors in user stories (US-NNN), present the suggestion list: "Existing actors from cartography registry: <list>"
5. If the user introduces a new actor not in the list, add a note: "New actor '<name>' introduced. Run cartography to add a journey for this actor."
6. If `docs/cartography/journeys/` does not exist or contains no files, proceed without suggestions (graceful fallback)

This ensures new user stories reference established actors from `docs/cartography/journeys/` consistently.

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

### Preview for Architecture Comparisons

When a mandatory question involves architecture comparisons — especially questions informed by the architect agent output (e.g., breaking changes with multiple mitigation strategies, or alternative module structures) — use AskUserQuestion with `preview` showing each approach's structure (Mermaid diagram, code snippet, or file tree). Purely textual mandatory questions (e.g., scope boundaries, test strategy, domain concepts) MUST NOT force preview. See `skills/grill-me/SKILL.md` § Preview for Visual Alternatives.

### Rules

> **Shared**: Use the `grill-me` skill in spec-mode for the interview. See `skills/grill-me/SKILL.md`.

### Grill-Me Accumulator

After each answer, run: `!ecc-workflow campaign append-decision --question "<question>" --answer "<answer>" --source recommended|user`

During each grill-me question, accumulate the question and the user's answer (or accepted recommendation) into a structured list. Track the Source for each answer as either "Recommended" (user accepted) or "User" (user overrode). This accumulated list is used in the Phase Summary.

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Grill-Me Disk Persistence section. After each answer, persist to campaign.md.

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Agent Output Tracking section. After each agent task returns, persist summary to campaign.md.

## Phase 7: Doc-First Review (Plan Mode)

> **BLOCKING**: You MUST call `EnterPlanMode` here. NEVER skip this phase.

1. Call `EnterPlanMode`
2. Write the plan file with the following structure:

```markdown
# Spec Preview: <title>

## How This Spec Was Understood

<1-3 sentences summarizing the user's intent as you understood it after the grill-me interview>

## Spec Draft

<the full spec you are about to output — all sections from the schema below>

## Doc Preview

Draft of upper-level doc updates that will be made alongside this spec:

### README.md changes
<if applicable, show the specific section that will be updated, or "No changes needed">

### CLAUDE.md changes
<if applicable, show the specific lines/sections that will be updated>

### Project overview changes
<any other high-level docs — CHANGELOG entry preview, glossary additions, etc.>
```

3. Call `ExitPlanMode` — wait for user approval before proceeding

## Phase 8: Write the Spec

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Draft Spec Persistence section. Write spec-draft.md before adversary dispatch.

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

## Phase 9: Adversarial Review

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Adversarial Review + Verdict Handling section, and Adversary History Tracking section for campaign.md persistence.

Launch a Task with the `spec-adversary` agent (allowedTools: [Read, Grep, Glob]):

- Pass the full spec from conversation context (Phase 6 output)
- The agent attacks the spec on 7 dimensions: ambiguity, edge cases, scope, dependencies, testability, decisions, rollback
- The agent returns a verdict in conversation (no file writes)

### Verdict Handling (max 3 rounds)

> After receiving the adversary verdict, translate the findings into plain language for the user. If the gate blocks, explain what failed and provide specific remediation steps so the user understands what to fix.

Track the current round number (starting at 1):

- **FAIL**: Present the adversary's findings to the user. Return to **Phase 5 (Grill-Me)** to address the fundamental issues. After the user confirms updates, re-output the spec (Phase 6), then re-run the adversary (Phase 7). Increment round.
- **CONDITIONAL**: The adversary has suggested specific ACs to add. Update the spec in conversation with the suggested ACs. Re-run the adversary. Increment round.
- **PASS**: Note "Adversarial Review: PASS" in conversation output. Then persist the spec (see below). Run: `!ecc-workflow transition solution --artifact plan --path <spec_file_path>`. Proceed to Phase 10.

After 3 FAIL rounds, ask the user:
> "The spec has failed adversarial review 3 times. Would you like to override and proceed anyway, or abandon this spec?"
- If override: note "Adversarial Review: PASS (user override)" in conversation, persist the spec, run `!ecc-workflow transition solution --artifact plan --path <spec_file_path>`, and proceed
- If abandon: delete workflow artifacts and exit

### Persist Spec to File

After adversarial PASS (or user override), write the spec to a versioned file:

1. Generate slug from the feature description: lowercase, replace non-alphanumeric with hyphens, collapse multiple hyphens, max 40 characters
2. Create directory `docs/specs/YYYY-MM-DD-<slug>/`
3. Write the full spec to `docs/specs/YYYY-MM-DD-<slug>/spec.md`
4. If the file already exists (re-entry), append a `## Revision` block with timestamp instead of overwriting
   On re-entry, run `!ecc-workflow campaign show` to reload prior grill-me decisions from campaign.md.
5. Pass the file path to the phase-transition command as the 3rd argument

## Phase 10: Present and STOP

### Full Artifact Display

Read the full artifact from `artifacts.spec_path` in state.json using the Read tool. Display the complete file content inline in conversation as the spec document body — no truncation, no summary. If the path is null or the file does not exist, emit a warning ("Spec artifact not found at the expected path; skipping inline display") and skip to the summary tables.

Display a comprehensive Phase Summary using these tables:

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | <question text> | <answer text> | Recommended / User |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | <title> | <count> | <none or US-NNN> |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | <description> | US-001 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| <dimension> | <0-100> | PASS/FAIL/CONDITIONAL | <rationale> |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/YYYY-MM-DD-<slug>/spec.md | Full spec |

### Phase Summary Persistence

Append a `## Phase Summary` section containing all 5 tables above to the persisted spec file (`docs/specs/YYYY-MM-DD-<slug>/spec.md`). If `## Phase Summary` already exists in the file, overwrite it (idempotent).

> **Note:** If continuing in a new session, copy the spec recap above or re-run `/spec-dev`.

### Artifact File Path

Display the persisted file path for future access:

> **Spec persisted at:** `docs/specs/YYYY-MM-DD-<slug>/spec.md`

If `continue_to_design` is true AND verdict was PASS: invoke `Skill("design")` to continue. Otherwise: STOP. Say:

> **Run `/design` to continue.**

Do NOT proceed to solution design or implementation.

## Related Agents

This command invokes:
- `requirements-analyst` — User Story decomposition, product challenge, codebase validation, dependency DAG
- `architect` — Hexagonal architecture analysis, module impact, E2E boundary assessment
- `spec-adversary` — Adversarial spec review on 7 dimensions before phase transition
