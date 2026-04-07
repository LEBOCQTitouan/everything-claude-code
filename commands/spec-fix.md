---
description: Spec a bug fix — investigation, blast radius analysis, web research, grill-me interview, doc-first review, and spec generation.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite, Agent, AskUserQuestion, EnterPlanMode, ExitPlanMode]
---

# Spec Fix Command

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Do NOT directly edit `.claude/workflow/state.json`.** State transitions happen via hooks only.
>
> **Narrative**: See narrative-conventions skill.

!`ecc-workflow init fix "$ARGUMENTS"`

### Worktree Isolation

Generate a worktree name and isolate this session:
1. Run: `!ecc-workflow worktree-name fix "$ARGUMENTS"` — capture the output name
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

> Before dispatching, tell the user which agent is being launched (`code-reviewer` in read-only mode) and that it will investigate the root cause without making changes.

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

> Tell the user this phase is starting: web research will be delegated to a Task subagent to keep the main context lean.

Launch a Task subagent (allowedTools: [WebSearch]) to perform web research in an isolated context:

**Subagent prompt**: Search the web for known issues, fix patterns, and related pitfalls for the bug's domain. Derive up to 3 focused search queries from the bug description and detected tech stack. Examples: "Rust async deadlock patterns", "tokio spawn_blocking best practices", "common causes of [error message]". Run each query using WebSearch. If WebSearch is unavailable, try exa-web-search MCP (`web_search_exa`). If both are unavailable, return: "Web research skipped: no search tool available." From the results, produce a **Research Summary** with 3-7 bullet points: known issues, fix patterns, pitfalls to avoid, and relevant documentation. Return ONLY the Research Summary.

**Subagent input**: The user's raw bug description and the detected tech stack from Phase 0.

**On success**: Carry the returned Research Summary forward into the spec output.
**On failure** (subagent failed or timed out): Record "Web research skipped: subagent failed" and proceed to the next phase — do NOT hard-fail.

## Phase 3.5: Sources Consultation

If `docs/sources.md` exists:
1. Read `docs/sources.md` and parse entries
2. Find entries matching the bug fix area or affected module (via module mapping table)
3. If matches found, list them as "Consulted sources:" in the output
4. Update `last_checked` date on matched entries
5. Write updated file back (atomic write)

If `docs/sources.md` does not exist, skip this step silently.

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
2. **Minimal vs proper fix** — "Should we apply a minimal patch or a proper structural fix?" (Recommend based on blast radius and time constraints). When both approaches are viable, use AskUserQuestion with `preview` showing each approach — the minimal patch as a code diff snippet and the structural fix as a file-change summary or architecture diagram. If only one approach is viable, skip preview.
3. **Missing tests** — "The affected area has [N] tests covering [X]%. Which scenarios are untested?" (Recommend based on investigation)
4. **Regression risk** — "These [N] modules share code paths with the bug. What regressions should we watch for?" (Recommend based on architect output)
5. **Related audit findings** — "Prior audits flagged [findings]. Should we address these in the same fix?" (Recommend based on severity and relatedness)
6. **Reproducibility** — "Can you provide steps to reproduce, or should we derive them from the code?" (Recommend reproduction steps from investigation)
7. **Data impact** — "Does this bug affect persisted data that needs migration or cleanup?" (Recommend based on investigation)

### Rules

> **Shared**: Use the `grill-me` skill in spec-mode for the interview. See `skills/grill-me/SKILL.md`.

### Grill-Me Accumulator

After each answer, run: `!ecc-workflow campaign append-decision --question "<question>" --answer "<answer>" --source recommended|user`

During each grill-me question, accumulate the question and the user's answer (or accepted recommendation) into a structured list. Track the Source for each answer as either "Recommended" (user accepted) or "User" (user overrode). This accumulated list is used in the Phase Summary.

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Grill-Me Disk Persistence section. After each answer, persist to campaign.md.

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Agent Output Tracking section. After each agent task returns, persist summary to campaign.md.

## Phase 6: Doc-First Review (Plan Mode)

> **BLOCKING**: You MUST call `EnterPlanMode` here. NEVER skip this phase.

1. Call `EnterPlanMode`
2. Write the plan file with the following structure:

```markdown
# Spec Preview: <title>

## How This Spec Was Understood

<1-3 sentences summarizing the bug and fix approach after the grill-me interview>

## Spec Draft

<the full spec you are about to output — all sections from the schema below>

## Doc Preview

Draft of upper-level doc updates:

### README.md changes
<if applicable, or "No changes needed">

### CLAUDE.md changes
<if applicable, or "No changes needed">

### Other doc changes
<CHANGELOG entry preview, any other high-level docs>
```

3. Call `ExitPlanMode` — wait for user approval before proceeding

## Phase 7: Write the Spec

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Draft Spec Persistence section. Write spec-draft.md before adversary dispatch.

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

## Phase 8: Adversarial Review

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Adversarial Review + Verdict Handling section, and Adversary History Tracking section for campaign.md persistence.

Launch a Task with the `spec-adversary` agent (allowedTools: [Read, Grep, Glob]):

- Pass the full spec from conversation context (Phase 5 output)
- The agent attacks the spec on 7 dimensions: ambiguity, edge cases, scope, dependencies, testability, decisions, rollback
- The agent returns a verdict in conversation (no file writes)

### Verdict Handling (max 3 rounds)

> After receiving the adversary verdict, translate the findings into plain language. If this gate blocks, explain what failed and provide specific remediation steps.

Track the current round number (starting at 1):

- **FAIL**: Present the adversary's findings to the user. Return to **Phase 4 (Grill-Me)** to address the fundamental issues. After the user confirms updates, re-output the spec (Phase 5), then re-run the adversary (Phase 6). Increment round.
- **CONDITIONAL**: The adversary has suggested specific ACs to add. Update the spec in conversation with the suggested ACs. Re-run the adversary. Increment round.
- **PASS**: Note "Adversarial Review: PASS" in conversation output. Then persist the spec (see below). Run: `!ecc-workflow transition solution --artifact plan --path <spec_file_path>`. Proceed to Phase 9.

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

## Phase 9: Present and STOP

### Full Artifact Display

Read the full artifact from `artifacts.spec_path` in state.json using the Read tool. Display the complete file content inline in conversation as the spec document body — no truncation, no summary. If the path is null or the file does not exist, emit a warning ("Spec artifact not found at the expected path; skipping inline display") and skip to the summary tables.

Display a comprehensive Phase Summary using these tables:

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | <question text> | <answer text> | Recommended / User |

Variant: Include a **Root cause** row summarizing the identified root cause vs symptom distinction.

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

> **Note:** If continuing in a new session, copy the spec recap above or re-run `/spec-fix`.

### Artifact File Path

Display the persisted file path for future access:

> **Spec persisted at:** `docs/specs/YYYY-MM-DD-<slug>/spec.md`

Then STOP. Say:

> **Run `/design` to continue.**

Do NOT proceed to solution design or implementation.

## Related Agents

This command invokes:
- `code-reviewer` — Read-only investigation, root cause analysis, test coverage assessment
- `architect` — Blast radius analysis, module impact, regression risk assessment
- `spec-adversary` — Adversarial spec review on 7 dimensions before phase transition
