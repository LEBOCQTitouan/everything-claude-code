---
id: BL-143
title: /project-foundation command — PRD + Architecture docs creation with codebase-aware challenge
status: implemented
created: 2026-04-12
promoted_to: ""
tags: [command, prd, architecture, bmad, interactive, codebase-analysis, grill-me, interview-me]
scope: MEDIUM
target_command: /spec-dev
---

## Optimized Prompt

```
/spec-dev

Create a new slash command `/project-foundation` that guides users through
creating foundational project-level PRD and Architecture documents.
Detect stack from Cargo.toml / package.json / pyproject.toml etc.

CONTEXT
-------
ECC already has `write-a-prd` (feature-level PRD skill) and `/spec-dev`
(feature spec command). This command operates one level above both: it
produces project-wide foundational documents, not feature specs.
Inspiration: BMAD-METHOD (bmad-code-org/BMAD-METHOD) — PM + Architect
agent personas, structured doc creation, challenge loops.

COMMAND BEHAVIOUR
-----------------
1. Worktree-enforced via existing write-guard hook (no special handling needed).
2. Uses ECC deterministic workflow: Plan Mode → grill-me interview →
   adversarial review → persist artifacts.
3. For NEW repos: pure interview mode — leverage `grill-me` (backlog-mode
   depth, standard profile) and `interview-me` skills to elicit:
   - Problem statement and target users
   - Core value proposition and success metrics
   - Bounded contexts and major subsystems
   - Tech stack and key architectural constraints
4. For EXISTING repos: add a codebase-analysis phase before the interview.
   Agent runs `ecc analyze` + reads CLAUDE.md + samples directory structure,
   then contributes its own understanding as structured input. The interview
   then challenges and refines that understanding rather than starting blank.
5. Challenges user thinking throughout — every major claim is challenged via
   grill-me (Clarity → Assumptions → Edge Cases stages; max 2 questions per
   stage in the interview context).
6. Structures thinking with scaffolded templates:
   - PRD template: Problem, Users, Goals, Non-Goals, Success Metrics,
     Key Features (by bounded context), Risks
   - Architecture template: System Overview, Bounded Contexts, Data Flow,
     Tech Stack Rationale, ADRs (initial list), Quality Attributes

DELIVERABLES
------------
- `docs/prd.md` — project-level PRD
- `docs/architecture.md` — initial architecture overview
- `docs/adr/ADR-0001-initial-stack.md` — first ADR scaffolded from interview

ACCEPTANCE CRITERIA
-------------------
- [ ] Command file at `commands/project-foundation.md` with correct frontmatter
      (allowed-tools, model: sonnet, effort: high)
- [ ] Plan Mode gate: user reviews PRD+arch outline before agent writes files
- [ ] Codebase-detection branch: existing repo → analysis phase runs first;
      new repo → skips to pure interview
- [ ] grill-me integration: at least Clarity + Assumptions stages fire during
      PRD creation, Clarity + Edge Cases during architecture creation
- [ ] Adversarial review pass before persisting artifacts (re-use spec-adversary
      or equivalent)
- [ ] Artifacts persisted to `docs/` (not `.claude/`)
- [ ] Works in worktree (write-guard hook already enforces this)
- [ ] CLAUDE.md updated with command reference entry

SCOPE BOUNDARIES
----------------
- NOT a feature spec (that is /spec-dev's domain)
- NOT a full architecture design session (that is /design's domain)
- NOT a replacement for write-a-prd skill (that remains feature-level)
- DO NOT reimplement grill-me or interview-me — compose them
- DO NOT add new Rust CLI commands in this spec; pure command/skill work

VERIFICATION
------------
1. `ecc validate commands` passes
2. Run `/project-foundation` on an empty repo → produces prd.md + architecture.md
3. Run `/project-foundation` on this repo → analysis phase runs, agent
   contributes codebase understanding, interview refines it
4. Artifacts pass adversarial review without CRITICAL findings
```

## Original Input

"command that helps create project-level PRD + Architecture docs (BMAD-style), challenges user's thinking, helps structure it, and for existing repos also provides inputs about how the agent understands the project."

User clarification: The deliverable is a command that (a) helps create PRD + Architecture docs, (b) challenges the user's thinking during creation, (c) helps structure the thinking, (d) for already-existing repos, also contributes the agent's own understanding/analysis of the project as input. Work must happen in a worktree. Must use ECC deterministic workflow.

## Challenge Log

Mode: backlog-mode
Depth Profile: standard
Stages completed: 1/3 (user provided sufficient signal, remaining stages skipped)
Questions answered: 1/1

### Stage 1: Clarity

**Q1**: [Clarification] What is the concrete deliverable — a new slash command, a skill, or both — and what documents does it produce?
**A**: A command that (a) helps create PRD + Architecture docs, (b) challenges thinking, (c) structures thinking, (d) for existing repos contributes agent's own understanding. Work in worktree, ECC deterministic workflow.
**Status**: answered — sufficient signal to proceed

## Related Backlog Items

- BL-012 (write-a-prd skill, implemented) — feature-level PRD; this is project-level
- BL-016 (prd-to-plan skill, open) — consumes PRD output; downstream dependency candidate
- BL-013 (interview-me skill, implemented) — compose, do not reimplement
- BL-011 (grill-me skill, implemented) — compose, do not reimplement
- BL-020 (/design command, implemented) — downstream; /project-foundation produces input for /design
