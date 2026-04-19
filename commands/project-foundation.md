---
description: Bootstrap project-level foundational documents — PRD, architecture overview, initial ADR, and optional CLAUDE.md for new repos.
allowed-tools: [Task, Read, Grep, Glob, Bash, Write, Edit, TodoWrite, TodoRead, AskUserQuestion, EnterPlanMode, ExitPlanMode, Agent]
---

# Project Foundation

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Narrative**: See narrative-conventions skill.

Bootstrap project-level PRD, architecture, and ADR through guided interview with codebase-aware challenge. For existing repos, an analysis phase pre-populates the interview. Outputs to `docs/foundation/`.

> **Tracking**: Create TodoWrite items for each phase. Mark complete as phases finish.

## Phase 0: Detection

1. Invoke the CLI via the Bash tool (do NOT use `!`-prefix shell eval) to initialize the workflow state for this foundation session. Pass the project description via an environment variable and pipe it through stdin to avoid shell-argv interpolation of metacharacters, e.g.: `env FEATURE='<project description>' sh -c 'printf %s "$FEATURE" | ecc-workflow init foundation --feature-stdin'`. Worktree isolation via existing write-guard.
2. Detect **new repo** vs **existing repo**:
   - New repo: no source files beyond README/LICENSE/`.gitignore`. Skip to Phase 2.
   - Existing repo: has CLAUDE.md, `Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`, or source directories. Proceed to Phase 1.
3. Present detection result for user confirmation via AskUserQuestion.

## Phase 1: Codebase Analysis (existing repos only)

Read CLAUDE.md, sample directory structure, detect tech stack from manifests. Extract: bounded contexts, architecture patterns, tech stack. Present structured understanding for user to confirm, challenge, or override. If analysis contradicts user answers later, user takes precedence.

## Phase 2: PRD Interview

1. Invoke `interview-me` (all 8 stages — user can skip irrelevant ones)
2. Invoke `grill-me` in **foundation-mode** (Clarity + Assumptions, maximum 2 questions per stage)
3. For existing repos, pre-populate PRD sections from Phase 1 analysis
4. Generate PRD following template: Problem, Users, Goals, Non-Goals, Success Metrics, Key Features (by bounded context), Risks — all 7 sections with non-empty content

## Phase 3: Architecture Interview

1. Invoke `grill-me` in **foundation-mode** (Clarity + Edge Cases, maximum 2 questions per stage)
2. For existing repos, pre-populate bounded contexts and tech stack from analysis
3. Generate architecture doc following template: System Overview, Bounded Contexts, Data Flow, Tech Stack Rationale, ADRs (initial list), Quality Attributes — all 6 sections

## Phase 4: Plan Mode Gate

1. Call `EnterPlanMode`
2. Display: PRD draft + architecture draft + ADR outline for review
3. Call `ExitPlanMode` — user approves or rejects. On rejection, no files written; user can abort or restart interview.

## Phase 5: Adversarial Review

Invoke `spec-adversary` with modified dimensions (completeness, consistency, feasibility, ambiguity, scope). Max 3 attempts. On FAIL, re-enter interview to address findings. On 3 consecutive FAILs, user can override or abandon. On PASS, proceed to write.

## Phase 6: Write Documents

1. Create `docs/foundation/` directory
2. Write `docs/foundation/prd.md` — project-level PRD
3. Write `docs/foundation/architecture.md` — architecture overview
4. Auto-detect next ADR number (ADR-0001 for new repos). Write ADR with Status/Context/Decision/Consequences
5. For **new repos only**: generate initial `CLAUDE.md` with project overview, tech stack, test/build/lint commands, references to PRD and architecture docs
6. For **existing repos**: update CLAUDE.md with `/project-foundation` command reference entry only
7. If `docs/foundation/` already exists from prior run, append `## Revision` block (do not overwrite)
8. Transition: `!ecc-workflow transition done --artifact implement`

Re-entry supported via ecc-workflow state machine — resume from last completed phase.

## Phase 7: Present and STOP

Display summary of all artifacts written. **STOP.**
