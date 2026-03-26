---
description: Spec a refactoring — current state analysis, smell catalog, web research, grill-me interview, doc-first review, and spec generation.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite, Agent, AskUserQuestion, EnterPlanMode, ExitPlanMode]
---

# Spec Refactor Command

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Do NOT directly edit `.claude/workflow/state.json`.** State transitions happen via hooks only.
>
> **Narrative**: See `skills/narrative-conventions/SKILL.md` conventions. Before each agent delegation, gate check, and phase transition, tell the user what is happening and why.

!`bash .claude/hooks/workflow-init.sh refactor "$ARGUMENTS"`

## Phase 0: Project Detection

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Project Detection section for auto-detection logic.

Detect the project's test, lint, and build commands:

Test command: !`command -v cargo > /dev/null 2>&1 && echo "cargo test" || (test -f package.json && echo "npm test" || (test -f go.mod && echo "go test ./..." || (test -f pyproject.toml && echo "pytest" || echo "echo 'no test runner detected'")))`

Lint command: !`command -v cargo > /dev/null 2>&1 && echo "cargo clippy -- -D warnings" || (test -f package.json && echo "npm run lint" || (test -f go.mod && echo "golangci-lint run" || (test -f pyproject.toml && echo "ruff check ." || echo "echo 'no linter detected'")))`

Build command: !`command -v cargo > /dev/null 2>&1 && echo "cargo build" || (test -f package.json && echo "npm run build" || (test -f go.mod && echo "go build ./..." || echo "echo 'no build command detected'"))`

Persist detected commands to state.json and create campaign manifest:

> **Shared**: See `skills/spec-pipeline-shared/SKILL.md` — Project Detection section for toolchain-persist.sh usage and Campaign Init section for campaign.md creation.

> **Tracking**: Create a TodoWrite checklist for this command's phases. If TodoWrite is unavailable, proceed without tracking — the workflow executes identically.

TodoWrite items:
- "Phase 0: Project Detection"
- "Phase 1: Current State Analysis"
- "Phase 2: Existing Audit Reports"
- "Phase 3: Smell Catalog"
- "Phase 4: Grill-Me Interview"
- "Phase 5: Write the Spec"
- "Phase 6: Adversarial Review"
- "Phase 7: Present and STOP"

Mark each item complete as the phase finishes.

## Phase 1: Current State Analysis

> Before dispatching, tell the user which agents are being launched in parallel (`evolution-analyst`, `arch-reviewer`, `component-auditor`) and what each will analyze.

Launch **three Tasks in parallel**:

### Task 1: Evolution Analysis
Launch with `evolution-analyst` agent (allowedTools: [Read, Grep, Glob, Bash]):
- Analyze git history for the affected area
- Identify change frequency hotspots, co-change coupling, and bus factor
- Flag complexity trends and churn indicators

### Task 2: Architecture Review
Launch with `arch-reviewer` agent (allowedTools: [Read, Grep, Glob, Bash]):
- Audit current structure for layering violations and dependency direction
- Identify coupling, cohesion, and circular dependency issues
- Check DDD/hexagonal compliance

### Task 3: Component Audit
Launch with `component-auditor` agent (allowedTools: [Read, Grep, Glob, Bash]):
- Evaluate package/module design against the 6 component principles (REP, CCP, CRP, ADP, SDP, SAP)
- Compute instability, abstractness, and main sequence distance
- Flag principle violations

Collect all three outputs — you will use them in the smell catalog.

## Phase 2: Existing Audit Reports

Read `docs/audits/` for any existing audit reports relevant to the refactoring area:

1. Glob for `docs/audits/*.md`
2. Scan report titles and summaries for overlap with the refactoring domain
3. Extract relevant findings (any severity) that support or contradict the refactoring
4. Identify findings that the refactoring should resolve
5. If no audit reports exist or none are relevant, note "No prior audit findings applicable"

## Phase 3: Web Research

> Tell the user this phase is starting: web research will be delegated to a Task subagent to keep the main context lean.

Launch a Task subagent (allowedTools: [WebSearch]) to perform web research in an isolated context:

**Subagent prompt**: Search the web for refactoring patterns, migration guides, and best practices relevant to the refactoring scope. Derive up to 3 focused search queries from the refactoring goal and detected tech stack. Examples: "Rust module decomposition patterns", "extract method refactoring hexagonal architecture", "safe migration strategies for [framework]". Run each query using WebSearch. If WebSearch is unavailable, try exa-web-search MCP (`web_search_exa`). If both are unavailable, return: "Web research skipped: no search tool available." From the results, produce a **Research Summary** with 3-7 bullet points: refactoring patterns, migration strategies, pitfalls to avoid, and relevant guides. Return ONLY the Research Summary.

**Subagent input**: The user's raw refactoring goal and the detected tech stack from Phase 0.

**On success**: Carry the returned Research Summary forward into the spec output.
**On failure** (subagent failed or timed out): Record "Web research skipped: subagent failed" and proceed to the next phase — do NOT hard-fail.

## Phase 4: Smell Catalog

Compile a unified catalog of detected smells from all Phase 1 agent findings and Phase 2 audit reports:

| # | Smell | Source | Severity | Evidence |
|---|-------|--------|----------|----------|
| 1 | ... | evolution-analyst / arch-reviewer / component-auditor / audit | CRITICAL/HIGH/MEDIUM/LOW | ... |

Group by severity. This catalog drives the grill-me interview.

## Phase 5: Grill-Me Interview

**STOP all research. START interviewing the user.**

You have gathered evolution analysis, architecture review, component audit, existing audit reports, and a smell catalog. Now challenge the user's thinking with refactoring-specific questions. For each question, provide your recommended answer based on the analysis.

### Mandatory Questions

1. **Smell triage** — "Here are [N] smells detected. Which should we address now vs defer?" (Recommend based on severity and coupling to the refactoring goal)
2. **Target architecture** — "What should the target state look like? Current violations: [list]" (Recommend based on arch-reviewer output)
3. **Step independence** — "Can each refactoring step be shipped independently, or do some require atomic grouping?" (Recommend based on dependency analysis)
4. **Downstream dependencies** — "These [N] modules depend on the code being refactored. How do we ensure they stay green?" (Recommend based on component-auditor output)
5. **Rename vs behavioral change** — "Which changes are pure renames/moves vs behavioral modifications?" (Recommend based on evolution-analyst output)
6. **Performance budget** — "Will any of these changes affect hot paths or latency-sensitive code?" (Recommend based on evolution-analyst hotspot data)
7. **ADR decisions** — "Which architectural decisions warrant an ADR?" (Recommend based on arch-reviewer output)
8. **Test safety net** — "Current test coverage for affected area: [N]%. Is this sufficient to refactor safely?" (Recommend based on coverage data)

### Rules

> **Shared**: Use the `grill-me` skill in spec-mode for the interview. See `skills/grill-me/SKILL.md`.

### Grill-Me Accumulator

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

<1-3 sentences summarizing the refactoring goal after the grill-me interview>

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

<One paragraph describing the structural problem, its symptoms, and why refactoring is needed now.>

## Research Summary

<3-7 bullet points from web research: refactoring patterns, migration strategies, pitfalls, guides. If web research was skipped, note "Web research skipped: no search tool available.">

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | ... | ... | Yes/No |

## User Stories

### US-001: <title>

**As a** <role>, **I want** <refactoring goal>, **so that** <quality/maintainability benefit>.

#### Acceptance Criteria

- AC-001.1: Given <current state>, when <refactoring applied>, then <target state>
- AC-001.2: ...

#### Dependencies

- Depends on: <none or US-NNN>

### US-002: <title>
...

## Affected Modules

<From arch-reviewer and component-auditor output. List modules, their layer (domain/port/adapter/app/CLI), and the nature of the change.>

## Constraints

<From audits, smell catalog, and interview. E.g., "all refactoring steps must be behavior-preserving", "test suite must stay green after each step".>

## Non-Requirements

<Explicitly out of scope — deferred smells from grill-me interview.>

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
- **PASS**: Note "Adversarial Review: PASS" in conversation output. Then persist the spec (see below). Run: `!bash .claude/hooks/phase-transition.sh solution plan <spec_file_path>`. Proceed to Phase 9.

After 3 FAIL rounds, ask the user:
> "The spec has failed adversarial review 3 times. Would you like to override and proceed anyway, or abandon this spec?"
- If override: note "Adversarial Review: PASS (user override)" in conversation, persist the spec, run `!bash .claude/hooks/phase-transition.sh solution plan <spec_file_path>`, and proceed
- If abandon: delete workflow artifacts and exit

### Persist Spec to File

After adversarial PASS (or user override), write the spec to a versioned file:

1. Generate slug from the feature description: lowercase, replace non-alphanumeric with hyphens, collapse multiple hyphens, max 40 characters
2. Create directory `docs/specs/YYYY-MM-DD-<slug>/`
3. Write the full spec to `docs/specs/YYYY-MM-DD-<slug>/spec.md`
4. If the file already exists (re-entry), append a `## Revision` block with timestamp instead of overwriting
5. Pass the file path to the phase-transition command as the 3rd argument

## Phase 9: Present and STOP

Display a comprehensive Phase Summary using these tables:

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | <question text> | <answer text> | Recommended / User |

Variant: Include rows for **Smells addressed** (smells tackled in this refactor) and **Smells deferred** (smells explicitly out of scope).

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | <title> | <count> | <none or US-NNN> |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | <description> | US-001 |

### Adversary Findings

| Dimension | Verdict | Key Rationale |
|-----------|---------|---------------|
| <dimension> | PASS/FAIL/CONDITIONAL | <rationale> |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/YYYY-MM-DD-<slug>/spec.md | Full spec |

### Phase Summary Persistence

Append a `## Phase Summary` section containing all 5 tables above to the persisted spec file (`docs/specs/YYYY-MM-DD-<slug>/spec.md`). If `## Phase Summary` already exists in the file, overwrite it (idempotent).

> **Note:** If continuing in a new session, copy the spec recap above or re-run `/spec-refactor`.

Then STOP. Say:

> **Run `/design` to continue.**

Do NOT proceed to solution design or implementation.

## Related Agents

This command invokes:
- `evolution-analyst` — Git history analysis, change frequency, co-change coupling, complexity trends
- `arch-reviewer` — Architecture quality audit, layering violations, DDD/hexagonal compliance
- `component-auditor` — Component principles evaluation, instability/abstractness metrics
- `architect` — (implicit via arch-reviewer orchestration)
- `spec-adversary` — Adversarial spec review on 7 dimensions before phase transition
